//! Node Identity — Ed25519 keypair generation and management.
//!
//! Each SAGE node has a unique identity derived from a real Ed25519 keypair.
//! The node ID is a short, human-readable hash of the public key: `sage-XXXXXX`.
//! Additionally, each node has a human-readable name like "swift-harbor".
//!
//! The libp2p PeerId is cryptographically derived from the same seed via
//! `to_libp2p_keypair()`, ensuring a stable, authentic network identity across restarts.

use ed25519_dalek::{Signer, SigningKey};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// ── Human-readable node names (adjective-noun pairs) ───────────────────────
// 50 adjectives × 50 nouns = 2500 combinations

const ADJECTIVES: &[&str] = &[
    "swift", "calm", "bold", "bright", "deep", "fair", "glad", "keen", "mild", "pure",
    "rare", "sage", "warm", "wise", "zest", "amber", "azure", "coral", "dusty", "faded",
    "golden", "hazy", "ivory", "jade", "lapis", "maple", "noble", "olive", "pearl", "quiet",
    "rosy", "silken", "tawny", "urban", "velvet", "wild", "young", "agile", "brave", "clear",
    "dusk", "eager", "fleet", "gentle", "humble", "inner", "jolly", "kindred", "lively", "merry",
];

const NOUNS: &[&str] = &[
    "harbor", "ridge", "creek", "grove", "haven", "brook", "cliff", "delta", "forge", "glade",
    "hill", "inlet", "knoll", "lake", "marsh", "nexus", "oasis", "peak", "quay", "river",
    "shore", "trail", "vale", "wharf", "zenith", "anchor", "beacon", "canyon", "drift", "ember",
    "fjord", "glacier", "hollow", "island", "jetty", "kelp", "ledge", "meadow", "north", "orbit",
    "prism", "quest", "reef", "summit", "tundra", "union", "vista", "woods", "yard", "zone",
];

/// Generate a human-readable name from a seed (deterministic).
pub fn generate_human_name(seed: &[u8; 32]) -> String {
    // Use first two bytes to select adjective and noun
    let adj_idx = seed[0] as usize % ADJECTIVES.len();
    let noun_idx = seed[1] as usize % NOUNS.len();
    format!("{}-{}", ADJECTIVES[adj_idx], NOUNS[noun_idx])
}

/// A SAGE node identity backed by an Ed25519 keypair (64-byte seed+public stored on disk).
#[derive(Clone)]
pub struct NodeIdentity {
    /// 32-byte secret seed
    seed: [u8; 32],
    /// 32-byte public key
    pub public_key: [u8; 32],
    /// Short human-readable ID like `sage-7f3a2b`
    pub node_id: String,
    /// Human-readable name like `swift-harbor`
    pub human_name: String,
}

impl fmt::Debug for NodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeIdentity")
            .field("node_id", &self.node_id)
            .field("human_name", &self.human_name)
            .finish()
    }
}

impl fmt::Display for NodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.human_name, self.node_id)
    }
}

impl NodeIdentity {
    /// Derive the short node ID from a public key.
    pub fn derive_node_id(public_key: &[u8; 32]) -> String {
        // Simple hash: take first 3 bytes of a basic hash of the public key
        // Using a simple xor-fold for now (no extra deps needed)
        let mut hash = [0u8; 32];
        // Poor-man's hash: just use the raw public key bytes directly
        // The public key is already unique, so first 3 bytes give us ~16M combinations
        hash.copy_from_slice(public_key);
        format!("sage-{:02x}{:02x}{:02x}", hash[0], hash[1], hash[2])
    }

    /// Generate a brand-new random identity.
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);
        Self::from_seed(seed)
    }

    /// Create identity from a 32-byte seed using real Ed25519 scalar multiplication.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = *signing_key.verifying_key().as_bytes();
        let node_id = Self::derive_node_id(&public_key);
        let human_name = generate_human_name(&public_key);
        Self {
            seed,
            public_key,
            node_id,
            human_name,
        }
    }

    /// Convert this node identity to a libp2p Keypair.
    ///
    /// The resulting libp2p PeerId is deterministically derived from this node's seed,
    /// so the same `~/.sage/identity.key` always produces the same peer on the network.
    /// This replaces the previous `load_stable_keypair()` workaround in libp2p_transport.rs.
    pub fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair {
        // libp2p's ed25519::Keypair::try_from_bytes expects 64 bytes:
        // first 32 = secret scalar, last 32 = public key (compressed point)
        let mut kp_bytes = [0u8; 64];
        let signing_key = SigningKey::from_bytes(&self.seed);
        kp_bytes[..32].copy_from_slice(signing_key.as_bytes());
        kp_bytes[32..].copy_from_slice(signing_key.verifying_key().as_bytes());

        libp2p::identity::Keypair::from(
            libp2p::identity::ed25519::Keypair::try_from_bytes(&mut kp_bytes)
                .expect("seed is always a valid Ed25519 secret key"),
        )
    }

    /// The stable libp2p PeerId for this node (same across restarts, same seed).
    pub fn peer_id(&self) -> libp2p::PeerId {
        libp2p::PeerId::from_public_key(&self.to_libp2p_keypair().public())
    }

    /// Default path for the identity key file.
    pub fn default_key_path() -> PathBuf {
        let home = dirs::home_dir().expect("could not determine home directory");
        home.join(".sage").join("identity.key")
    }

    /// Load identity from disk, or generate and save a new one.
    pub fn load_or_generate(path: Option<&Path>) -> io::Result<Self> {
        let path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_key_path);

        if path.exists() {
            Self::load(&path)
        } else {
            let identity = Self::generate();
            identity.save(&path)?;
            Ok(identity)
        }
    }

    /// Load from a key file (64 bytes: 32 seed + 32 public).
    pub fn load(path: &Path) -> io::Result<Self> {
        let data = fs::read(path)?;
        if data.len() != 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("identity key file should be 64 bytes, got {}", data.len()),
            ));
        }
        let mut seed = [0u8; 32];
        let mut public_key = [0u8; 32];
        seed.copy_from_slice(&data[..32]);
        public_key.copy_from_slice(&data[32..]);
        let node_id = Self::derive_node_id(&public_key);
        let human_name = load_or_generate_human_name(&public_key);
        Ok(Self {
            seed,
            public_key,
            node_id,
            human_name,
        })
    }

    /// Save identity to disk.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(&self.seed);
        data.extend_from_slice(&self.public_key);
        fs::write(path, &data)?;
        // Restrict permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    /// Sign a message with this node's Ed25519 private key.
    ///
    /// Returns a 64-byte Ed25519 signature. Verify against `self.public_key`.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signing_key = SigningKey::from_bytes(&self.seed);
        signing_key.sign(message).to_bytes()
    }
}


/// Default path for node name file.
fn default_name_path() -> PathBuf {
    let home = dirs::home_dir().expect("could not determine home directory");
    home.join(".sage").join("node_name")
}

/// Load human name from disk or generate one from public key.
/// The name is persisted to ~/.sage/node_name for consistency across restarts.
fn load_or_generate_human_name(public_key: &[u8; 32]) -> String {
    let path = default_name_path();

    // Try to load existing name
    if path.exists() {
        if let Ok(name) = fs::read_to_string(&path) {
            let name = name.trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }

    // Generate new name from public key
    let name = generate_human_name(public_key);

    // Persist the name
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, &name);

    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_deterministic_from_seed() {
        let seed = [42u8; 32];
        let a = NodeIdentity::from_seed(seed);
        let b = NodeIdentity::from_seed(seed);
        assert_eq!(a.node_id, b.node_id);
        assert_eq!(a.public_key, b.public_key);
    }

    #[test]
    fn test_node_id_format() {
        let id = NodeIdentity::generate();
        assert!(id.node_id.starts_with("sage-"));
        assert_eq!(id.node_id.len(), 11); // "sage-" + 6 hex chars
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("sage_test_identity");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("test.key");

        let original = NodeIdentity::generate();
        original.save(&path).unwrap();
        let loaded = NodeIdentity::load(&path).unwrap();

        assert_eq!(original.node_id, loaded.node_id);
        assert_eq!(original.public_key, loaded.public_key);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_different_seeds_different_ids() {
        let a = NodeIdentity::from_seed([1u8; 32]);
        let b = NodeIdentity::from_seed([2u8; 32]);
        assert_ne!(a.node_id, b.node_id);
    }

    #[test]
    fn test_human_name_format() {
        let seed = [42u8; 32];
        let name = generate_human_name(&seed);
        // Should be adjective-noun format
        assert!(name.contains('-'), "human name should contain hyphen: {}", name);
        let parts: Vec<_> = name.split('-').collect();
        assert_eq!(parts.len(), 2, "human name should have exactly 2 parts");
        assert!(!parts[0].is_empty(), "adjective should not be empty");
        assert!(!parts[1].is_empty(), "noun should not be empty");
    }

    #[test]
    fn test_human_name_deterministic() {
        let seed = [123u8; 32];
        let a = generate_human_name(&seed);
        let b = generate_human_name(&seed);
        assert_eq!(a, b, "same seed should produce same name");
    }

    #[test]
    fn test_identity_has_human_name() {
        let id = NodeIdentity::generate();
        assert!(!id.human_name.is_empty(), "identity should have a human name");
        assert!(id.human_name.contains('-'), "human name should be adjective-noun: {}", id.human_name);
    }

    /// Public key must be the real Ed25519 verifying key for the given seed.
    #[test]
    fn test_real_ed25519_public_key() {
        let seed = [42u8; 32];
        let id = NodeIdentity::from_seed(seed);
        let expected_pk = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        assert_eq!(id.public_key, expected_pk, "public_key must be real Ed25519 verifying key");
    }

    /// `to_libp2p_keypair()` must produce the same PeerId every time for the same seed.
    #[test]
    fn test_to_libp2p_keypair_deterministic() {
        let id = NodeIdentity::from_seed([7u8; 32]);
        let kp1 = id.to_libp2p_keypair();
        let kp2 = id.to_libp2p_keypair();
        assert_eq!(
            kp1.public().to_peer_id(),
            kp2.public().to_peer_id(),
            "same seed → same PeerId always",
        );
    }

    /// `peer_id()` must be stable across calls.
    #[test]
    fn test_peer_id_stable() {
        let id = NodeIdentity::from_seed([99u8; 32]);
        assert_eq!(id.peer_id(), id.peer_id(), "peer_id() must be idempotent");
    }

    /// Different seeds → different PeerIds.
    #[test]
    fn test_different_seeds_different_peer_ids() {
        let id_a = NodeIdentity::from_seed([1u8; 32]);
        let id_b = NodeIdentity::from_seed([2u8; 32]);
        assert_ne!(id_a.peer_id(), id_b.peer_id());
    }

    /// `sign()` must produce a valid Ed25519 signature verifiable with the public key.
    #[test]
    fn test_sign_verifies() {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        let id = NodeIdentity::from_seed([55u8; 32]);
        let message = b"hello sage network";
        let sig_bytes = id.sign(message);

        let sig = Signature::from_bytes(&sig_bytes);
        let vk = VerifyingKey::from_bytes(&id.public_key).expect("valid Ed25519 public key");
        assert!(vk.verify(message, &sig).is_ok(), "signature must verify");
    }

    /// `sign()` must produce different signatures for different messages.
    #[test]
    fn test_sign_is_deterministic_and_unique() {
        let id = NodeIdentity::from_seed([33u8; 32]);
        let sig1 = id.sign(b"message one");
        let sig2 = id.sign(b"message two");
        assert_ne!(sig1, sig2, "different messages → different signatures");
    }
}