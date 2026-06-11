//! Brain Template System
//!
//! Named snapshots of trained NCA grids. Think of them as "expert brains"
//! that can be cloned and deployed to new nodes.
//!
//! Each template is a tagged BrainHeader + grid + text_store + metadata.
//! Templates live in ~/.sage/templates/ and are importable at node startup.
//!
//! Usage flow:
//!   1. Train a node on a domain (e.g. "junior-dev", "mechatronics-expert")
//!   2. Export: `sage-template export --name junior-dev --tags rust,cs`
//!   3. Copy .template file to another machine
//!   4. Import on new node: `sage-node --template junior-dev`
//!   5. New node starts with pre-loaded knowledge, then grows from there

use crate::distributed_knowledge::{KnowledgeStore, NCAKnowledge, TextStore};
use crate::grid::{Grid, NUM_CHANNELS};
use crate::network::identity::NodeIdentity;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Human-readable metadata for a brain template
#[derive(Clone, Serialize, Deserialize)]
pub struct BrainTemplate {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub domain: Option<String>,
    pub created_at: u64,
    pub source_node_id: String,
    pub grid_size: usize,
    pub channels: usize,
    pub active_cells: usize,
    pub version: String,
}

/// Full template bundle: metadata + grid + text_store
#[derive(Clone, Serialize, Deserialize)]
pub struct BrainTemplateBundle {
    pub meta: BrainTemplate,
    pub grid: Grid,
    pub text_store: TextStore,
}

impl BrainTemplateBundle {
    /// Snapshot an NCAKnowledge into a named template
    pub fn from_knowledge(
        knowledge: &NCAKnowledge,
        name: &str,
        description: &str,
        tags: Vec<String>,
        domain: Option<String>,
    ) -> Self {
        let active = knowledge.active_knowledge(0.01).len();
        let node_id_str = format!("{:x}", knowledge.node_id.to_bits());

        Self {
            meta: BrainTemplate {
                name: name.to_string(),
                description: description.to_string(),
                tags,
                domain,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                source_node_id: node_id_str,
                grid_size: knowledge.grid.width,
                channels: NUM_CHANNELS,
                active_cells: active,
                version: crate_version(),
            },
            grid: knowledge.grid.clone(),
            text_store: knowledge.text_store.clone(),
        }
    }

    /// Convert back to NCAKnowledge (for loading into a running node)
    pub fn to_knowledge(&self) -> NCAKnowledge {
        // Derive a node_id from the source node hash. New node gets same
        // knowledge patterns but its own identity is handled separately.
        let node_id_bytes = {
            let mut buf = [0u8; 8];
            let src = self.meta.source_node_id.as_bytes();
            let len = src.len().min(8);
            buf[..len].copy_from_slice(&src[..len]);
            buf
        };
        let node_id_f64 = f64::from_bits(u64::from_le_bytes(node_id_bytes));

        let mut knowledge = crate::distributed_knowledge::NCAKnowledge::new()
            .with_grid(self.grid.clone())
            .with_node_id(node_id_f64);
        knowledge.text_store = self.text_store.clone();
        knowledge
    }

    /// Save to ~/.sage/templates/<name>.template
    pub fn save(&self, templates_dir: &PathBuf) -> Result<String, String> {
        let filename = format!("{}.template", sanitize_name(&self.meta.name));
        let path = templates_dir.join(&filename);

        let data = bincode::serialize(self)
            .map_err(|e| format!("Template serialization error: {}", e))?;

        std::fs::create_dir_all(templates_dir)
            .map_err(|e| format!("Dir creation error: {}", e))?;

        std::fs::write(&path, &data)
            .map_err(|e| format!("Template write error: {}", e))?;

        Ok(path.to_string_lossy().to_string())
    }

    /// Load from a .template file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("Template read error: {}", e))?;

        bincode::deserialize(&data)
            .map_err(|e| format!("Template deserialization error: {}", e))
    }

    /// Sign this template with the node's Ed25519 identity.
    /// Returns the signature bytes (64 bytes).
    pub fn sign(&self, identity: &NodeIdentity) -> Result<Vec<u8>, String> {
        let data = bincode::serialize(self)
            .map_err(|e| format!("Template serialization error: {}", e))?;
        Ok(identity.sign(&data).to_vec())
    }

    /// Verify a signature against this template using a public key.
    pub fn verify_signature(&self, signature: &[u8; 64], public_key: &[u8; 32]) -> Result<bool, String> {
        let data = bincode::serialize(self)
            .map_err(|e| format!("Template serialization error: {}", e))?;
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        let sig = Signature::from_bytes(signature);
        let vk = VerifyingKey::from_bytes(public_key)
            .map_err(|e| format!("Invalid public key: {}", e))?;
        Ok(vk.verify(&data, &sig).is_ok())
    }

    /// Check if a newer version of this template exists on the hub.
    /// Returns Some(new_version) if an update is available.
    pub fn check_for_update(&self, hub_url: &str) -> Option<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;

        let resp = client
            .get(format!("{}/api/v1/templates/{}", hub_url, self.meta.name))
            .send()
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let bytes = resp.bytes().ok()?;
        let remote: BrainTemplateBundle = bincode::deserialize(&bytes).ok()?;

        if remote.meta.version != self.meta.version {
            Some(remote.meta.version)
        } else {
            None
        }
    }

    /// Human-readable summary for CLI output
    pub fn info(&self) -> String {
        let created = chrono::DateTime::from_timestamp(self.meta.created_at as i64, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| self.meta.created_at.to_string());

        let domain = self.meta.domain.as_deref().unwrap_or("general");
        let tags = if self.meta.tags.is_empty() {
            "none".to_string()
        } else {
            self.meta.tags.join(", ")
        };

        format!(
            "📦 {} (sage v{})\n   {}\n   Domain: {} | Active cells: {} / {}×{}\n   Created: {} | Source: {}\n   Tags: {}",
            self.meta.name,
            self.meta.version,
            self.meta.description,
            domain,
            self.meta.active_cells,
            self.meta.grid_size,
            self.meta.grid_size,
            created,
            &self.meta.source_node_id[..16.min(self.meta.source_node_id.len())],
            tags,
        )
    }
}

/// Export an existing brain.bin to a template
pub fn export_brain_to_template(
    brain_path: &str,
    name: &str,
    description: &str,
    tags: Vec<String>,
    domain: Option<String>,
    templates_dir: &PathBuf,
) -> Result<String, String> {
    let mut knowledge = NCAKnowledge::new();
    knowledge.load(brain_path)?;

    let bundle = BrainTemplateBundle::from_knowledge(&knowledge, name, description, tags, domain);
    bundle.save(templates_dir)
}

/// Import a template and return the NCAKnowledge (caller saves as brain.bin)
pub fn import_template_to_knowledge(template_path: &PathBuf) -> Result<NCAKnowledge, String> {
    let bundle = BrainTemplateBundle::load(template_path)?;
    Ok(bundle.to_knowledge())
}

/// List all templates in the templates directory
pub fn list_templates(templates_dir: &PathBuf) -> Vec<BrainTemplateBundle> {
    let mut templates = Vec::new();

    if !templates_dir.exists() {
        return templates;
    }

    let entries = match std::fs::read_dir(templates_dir) {
        Ok(e) => e,
        Err(_) => return templates,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("template") {
            match BrainTemplateBundle::load(&path) {
                Ok(t) => templates.push(t),
                Err(e) => eprintln!("Warning: failed to load template {:?}: {}", path, e),
            }
        }
    }

    templates.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
    templates
}

/// Find a template by name (fuzzy-ish: exact match first, then contains)
pub fn find_template(
    name: &str,
    templates_dir: &PathBuf,
) -> Result<BrainTemplateBundle, String> {
    let templates = list_templates(templates_dir);

    // Exact match
    if let Some(t) = templates.iter().find(|t| t.meta.name == name) {
        return Ok(t.clone());
    }

    // Case-insensitive exact
    let name_lower = name.to_lowercase();
    if let Some(t) = templates.iter().find(|t| t.meta.name.to_lowercase() == name_lower) {
        return Ok(t.clone());
    }

    // Contains
    if let Some(t) = templates
        .iter()
        .find(|t| t.meta.name.to_lowercase().contains(&name_lower))
    {
        return Ok(t.clone());
    }

    // Filename match
    let filename = format!("{}.template", sanitize_name(name));
    let path = templates_dir.join(&filename);
    if path.exists() {
        return BrainTemplateBundle::load(&path);
    }

    Err(format!(
        "Template '{}' not found. Run `sage-template list` to see available templates.",
        name
    ))
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "")
}

fn crate_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Default templates directory: ~/.sage/templates
pub fn default_templates_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".sage").join("templates")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed_knowledge::KnowledgeStore;

    #[test]
    fn test_template_roundtrip() {
        let mut knowledge = NCAKnowledge::new();
        knowledge.encode("Rust ownership and borrowing", 0.9);
        knowledge.encode("Python list comprehensions", 0.8);

        let bundle = BrainTemplateBundle::from_knowledge(
            &knowledge,
            "junior-dev",
            "CS fundamentals for junior developers",
            vec!["rust".into(), "python".into(), "cs".into()],
            Some("computer-science".into()),
        );

        assert_eq!(bundle.meta.name, "junior-dev");
        assert_eq!(bundle.meta.tags, vec!["rust", "python", "cs"]);
        assert!(bundle.meta.active_cells > 0);

        // Convert back to knowledge
        let restored = bundle.to_knowledge();
        let results = restored.query("Rust ownership", 5);
        assert!(!results.is_empty(), "Restored knowledge should be queryable");
    }

    #[test]
    fn test_template_save_load() {
        let tmp_dir = std::env::temp_dir().join("sage_template_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let mut knowledge = NCAKnowledge::new();
        knowledge.encode("template persistence test", 0.9);

        let bundle = BrainTemplateBundle::from_knowledge(
            &knowledge,
            "test-template",
            "Testing save/load",
            vec!["test".into()],
            None,
        );

        let path = bundle.save(&tmp_dir).expect("save should succeed");
        assert!(std::path::Path::new(&path).exists());

        let loaded = BrainTemplateBundle::load(&PathBuf::from(path)).expect("load should succeed");
        assert_eq!(loaded.meta.name, "test-template");
        assert_eq!(loaded.meta.tags, vec!["test"]);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_find_template() {
        let tmp_dir = std::env::temp_dir().join("sage_find_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let mut k1 = NCAKnowledge::new();
        k1.encode("cs stuff", 0.9);
        let b1 = BrainTemplateBundle::from_knowledge(&k1, "junior-dev", "jd", vec![], None);
        b1.save(&tmp_dir).unwrap();

        let mut k2 = NCAKnowledge::new();
        k2.encode("mech stuff", 0.9);
        let b2 = BrainTemplateBundle::from_knowledge(&k2, "mechatronics-expert", "mech", vec![], None);
        b2.save(&tmp_dir).unwrap();

        let found = find_template("junior", &tmp_dir).expect("should find by contains");
        assert_eq!(found.meta.name, "junior-dev");

        let found2 = find_template("MECH", &tmp_dir).expect("should find case-insensitive");
        assert_eq!(found2.meta.name, "mechatronics-expert");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
