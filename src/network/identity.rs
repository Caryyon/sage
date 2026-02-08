//! Node Identity — Ed25519 keypair generation and management.
//!
//! Each SAGE node has a unique identity derived from an Ed25519 keypair.
//! The node ID is a short, human-readable hash of the public key: `sage-XXXXXX`.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A SAGE node identity backed by an Ed25519 keypair (64-byte seed+public stored on disk).
#[derive(Clone)]
pub struct NodeIdentity {
    /// 32-byte secret seed
    seed: [u8; 32],
    /// 32-byte public key
    pub public_key: [u8; 32],
    /// Short human-readable ID like `sage-7f3a2b`
    pub node_id: String,
}

impl fmt::Debug for NodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeIdentity")
            .field("node_id", &self.node_id)
            .finish()
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
        format!(
            "sage-{:02x}{:02x}{:02x}",
            hash[0], hash[1], hash[2]
        )
    }

    /// Generate a brand-new random identity.
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);
        Self::from_seed(seed)
    }

    /// Create identity from a 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        // Derive public key: for now we store seed and derive a "public key"
        // using a simple deterministic transform. In production this would use
        // actual Ed25519 scalar multiplication. For the foundation we'll use
        // a hash-based derivation that's deterministic.
        let public_key = simple_pubkey_derive(&seed);
        let node_id = Self::derive_node_id(&public_key);
        Self {
            seed,
            public_key,
            node_id,
        }
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
        Ok(Self {
            seed,
            public_key,
            node_id,
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

    /// Sign a message (placeholder — returns seed-based HMAC-like tag).
    pub fn sign(&self, message: &[u8]) -> [u8; 32] {
        // Placeholder: XOR seed with message hash
        let mut tag = self.seed;
        for (i, &b) in message.iter().enumerate() {
            tag[i % 32] ^= b;
        }
        tag
    }
}

/// Simple deterministic "public key" derivation from seed.
/// NOT cryptographically secure — placeholder until we add ed25519-dalek.
fn simple_pubkey_derive(seed: &[u8; 32]) -> [u8; 32] {
    let mut pk = [0u8; 32];
    for i in 0..32 {
        // Deterministic scramble
        pk[i] = seed[i]
            .wrapping_mul(137)
            .wrapping_add(seed[(i + 13) % 32])
            ^ 0xA5;
    }
    pk
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}
