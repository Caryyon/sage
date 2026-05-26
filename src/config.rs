//! SAGE configuration loading from ~/.sage/config.toml

use serde::Deserialize;

/// Top-level SAGE configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SageConfig {
    pub node: NodeConfig,
    pub chat: ChatConfig,
    pub network: NetworkSettings,
    pub grid: GridConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub name: String,
    pub port: u16,
    pub sync_interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ChatConfig {
    pub ollama_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    pub enable_mdns: bool,
    pub bootstrap_peers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GridConfig {
    /// Grid size (width and height). Must be a power of 2, 64–1024.
    pub size: usize,
}

#[allow(clippy::derivable_impls)]
impl Default for SageConfig {
    fn default() -> Self {
        Self {
            node: NodeConfig::default(),
            chat: ChatConfig::default(),
            network: NetworkSettings::default(),
            grid: GridConfig::default(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            port: 7433,
            sync_interval: 300,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            model: "qwen2.5:14b".into(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            bootstrap_peers: Vec::new(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for GridConfig {
    fn default() -> Self {
        Self { size: 256 }
    }
}

impl GridConfig {
    /// Validate grid size: must be power of 2, 64–1024.
    pub fn validate(&self) -> Result<(), String> {
        if self.size < 64 || self.size > 1024 {
            return Err(format!("grid.size must be 64–1024, got {}", self.size));
        }
        if !self.size.is_power_of_two() {
            return Err(format!("grid.size must be a power of 2, got {}", self.size));
        }
        Ok(())
    }
}

/// Load config from ~/.sage/config.toml, falling back to defaults.
pub fn load_config() -> SageConfig {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let path = format!("{home}/.sage/config.toml");
    load_config_from(&path)
}

/// Load config from a specific path.
pub fn load_config_from(path: &str) -> SageConfig {
    match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<SageConfig>(&contents) {
            Ok(mut cfg) => {
                if let Err(e) = cfg.grid.validate() {
                    eprintln!("Warning: {e}, using default grid size 256");
                    cfg.grid.size = 256;
                }
                cfg
            }
            Err(e) => {
                eprintln!("Warning: failed to parse config: {e}");
                SageConfig::default()
            }
        },
        Err(_) => SageConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = SageConfig::default();
        assert_eq!(cfg.grid.size, 256);
        assert!(cfg.grid.validate().is_ok());
    }

    #[test]
    fn test_grid_validation() {
        let mut g = GridConfig { size: 64 };
        assert!(g.validate().is_ok());
        g.size = 1024;
        assert!(g.validate().is_ok());
        g.size = 100; // not power of 2
        assert!(g.validate().is_err());
        g.size = 32; // too small
        assert!(g.validate().is_err());
        g.size = 2048; // too large
        assert!(g.validate().is_err());
    }

    #[test]
    fn test_parse_config_with_grid() {
        let toml_str = r#"
[grid]
size = 512
"#;
        let cfg: SageConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.grid.size, 512);
        assert!(cfg.grid.validate().is_ok());
    }
}
