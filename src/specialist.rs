//! Specialist Profile System
//!
//! Turns brain templates into hireable autonomous employees.
//! Each specialist bundles a trained NCA brain with a role definition,
//! system prompt, capability list, and quality metrics.
//!
//! Flow:
//!   1. Train a brain via curriculum ingestion (sage-curriculum ingest)
//!   2. Export as brain template (sage-template export)
//!   3. Define specialist profile (sage-specialist define)
//!   4. Publish to hub (sage-specialist publish)
//!   5. Users browse and hire (sage-specialist hire <name>)
//!   6. Specialist works autonomously via worker loop

use crate::brain_templates::BrainTemplate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Experience level for a specialist
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Junior,
    Mid,
    Senior,
    Lead,
    Principal,
}

impl ExperienceLevel {
    pub fn label(&self) -> &str {
        match self {
            ExperienceLevel::Junior => "junior",
            ExperienceLevel::Mid => "mid-level",
            ExperienceLevel::Senior => "senior",
            ExperienceLevel::Lead => "lead",
            ExperienceLevel::Principal => "principal",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "junior" => Some(ExperienceLevel::Junior),
            "mid" | "mid-level" | "midlevel" => Some(ExperienceLevel::Mid),
            "senior" => Some(ExperienceLevel::Senior),
            "lead" => Some(ExperienceLevel::Lead),
            "principal" => Some(ExperienceLevel::Principal),
            _ => None,
        }
    }
}

/// A specific task capability the specialist can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    /// Expected quality threshold for this capability (0.0-1.0)
    pub quality_threshold: f64,
    /// Average time to complete (in seconds, for estimation)
    pub avg_completion_secs: u64,
}

/// Quality metrics from curriculum verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall hit rate across all topics (0.0-1.0)
    pub hit_rate: f64,
    /// Mean relevance score of top results
    pub mean_relevance: f64,
    /// Number of topics verified
    pub topics_verified: usize,
    /// Number of facts encoded
    pub facts_encoded: usize,
    /// Active cells in the brain
    pub active_cells: usize,
    /// Grid utilization (active / total cells)
    pub grid_utilization: f64,
    /// Per-topic hit rates
    pub topic_hit_rates: Vec<TopicQuality>,
}

/// Quality for a single topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicQuality {
    pub topic: String,
    pub hit_rate: f64,
    pub facts_count: usize,
}

/// The specialist's role — what kind of work they do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistRole {
    /// Role category (e.g. "software-engineer", "data-analyst")
    pub category: String,
    /// Specific title (e.g. "Junior React Developer")
    pub title: String,
    /// Experience level
    pub level: ExperienceLevel,
    /// Domain expertise areas
    pub domains: Vec<String>,
    /// Primary tech stack / tools
    pub tools: Vec<String>,
    /// Industries this specialist is suited for
    pub industries: Vec<String>,
}

/// The system prompt that defines the specialist's behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistPrompt {
    /// Core identity prompt (who they are, how they work)
    pub identity: String,
    /// Task handling instructions
    pub task_instructions: String,
    /// Quality standards they must meet
    pub quality_standards: String,
    /// Communication style
    pub communication_style: String,
    /// Constraints / things they should NOT do
    pub constraints: String,
}

impl SpecialistPrompt {
    /// Assemble the full system prompt from components
    pub fn assemble(&self) -> String {
        format!(
            "{}\n\n## Task Instructions\n{}\n\n## Quality Standards\n{}\n\n## Communication Style\n{}\n\n## Constraints\n{}",
            self.identity,
            self.task_instructions,
            self.quality_standards,
            self.communication_style,
            self.constraints
        )
    }
}

/// Hiring metadata for the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiringInfo {
    /// Suggested hourly rate in USD (for display only)
    pub suggested_rate_usd: f64,
    /// Availability: "full-time", "part-time", "on-demand"
    pub availability: String,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Estimated ramp-up time in minutes
    pub ramp_up_minutes: u64,
    /// Languages the specialist works in
    pub languages: Vec<String>,
    /// Timezone preference
    pub timezone: Option<String>,
}

/// The complete specialist profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistProfile {
    /// Unique name (e.g. "junior-react-dev-v1")
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Short pitch — why hire this specialist
    pub tagline: String,
    /// Longer description of what they do
    pub description: String,
    /// Version of this profile (for updates)
    pub version: String,
    /// The role definition
    pub role: SpecialistRole,
    /// Capabilities this specialist can perform
    pub capabilities: Vec<Capability>,
    /// Quality metrics from training
    pub quality: QualityMetrics,
    /// The system prompt that defines behavior
    pub prompt: SpecialistPrompt,
    /// Hiring / marketplace metadata
    pub hiring: HiringInfo,
    /// Reference to the brain template this is built on
    pub template_name: String,
    /// When this profile was created
    pub created_at: u64,
    /// Who created it (node ID)
    pub author_node_id: String,
    /// Tags for search/discovery
    pub tags: Vec<String>,
}

impl SpecialistProfile {
    /// Create a new specialist profile from a brain template
    #[allow(clippy::too_many_arguments)]
    pub fn from_template(
        template: &BrainTemplate,
        role: SpecialistRole,
        capabilities: Vec<Capability>,
        quality: QualityMetrics,
        prompt: SpecialistPrompt,
        hiring: HiringInfo,
        display_name: &str,
        tagline: &str,
        description: &str,
    ) -> Self {
        Self {
            name: template.name.clone(),
            display_name: display_name.to_string(),
            tagline: tagline.to_string(),
            description: description.to_string(),
            version: template.version.clone(),
            role,
            capabilities,
            quality,
            prompt,
            hiring,
            template_name: template.name.clone(),
            created_at: template.created_at,
            author_node_id: template.source_node_id.clone(),
            tags: template.tags.clone(),
        }
    }

    /// Save to ~/.sage/specialists/<name>.specialist
    pub fn save(&self, specialists_dir: &PathBuf) -> Result<String, String> {
        let filename = format!("{}.specialist", sanitize_name(&self.name));
        let path = specialists_dir.join(&filename);

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize specialist: {}", e))?;

        std::fs::create_dir_all(specialists_dir)
            .map_err(|e| format!("Failed to create specialists dir: {}", e))?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write specialist file: {}", e))?;

        Ok(path.to_string_lossy().to_string())
    }

    /// Load from a .specialist file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read specialist file: {}", e))?;

        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse specialist: {}", e))
    }

    /// Generate a summary string for display
    pub fn summary(&self) -> String {
        let level = self.role.level.label();
        let domains = self.role.domains.join(", ");
        let tools = if self.role.tools.is_empty() {
            String::new()
        } else {
            format!(" | Tools: {}", self.role.tools.join(", "))
        };
        let hit_rate = (self.quality.hit_rate * 100.0).round() as u32;
        let cells = self.quality.active_cells;

        format!(
            "{} — {} {} specialist\n\
             Domain: {}{}\n\
             Quality: {}% hit rate, {} active cells, {} facts encoded\n\
             Rate: ${}/hr | {} | {} concurrent tasks\n\
             {}",
            self.display_name,
            level,
            self.role.title,
            domains,
            tools,
            hit_rate,
            cells,
            self.quality.facts_encoded,
            self.hiring.suggested_rate_usd,
            self.hiring.availability,
            self.hiring.max_concurrent_tasks,
            self.tagline,
        )
    }
}

/// Default specialists directory
pub fn default_specialists_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("specialists")
}

/// List all specialist profiles in the directory
pub fn list_specialists(dir: &PathBuf) -> Vec<SpecialistProfile> {
    let mut profiles = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "specialist") {
                if let Ok(profile) = SpecialistProfile::load(&path) {
                    profiles.push(profile);
                }
            }
        }
    }

    profiles.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    profiles
}

/// Find a specialist by name in the directory
pub fn find_specialist(name: &str, dir: &PathBuf) -> Result<SpecialistProfile, String> {
    let path = dir.join(format!("{}.specialist", sanitize_name(name)));
    if path.exists() {
        SpecialistProfile::load(&path)
    } else {
        // Try fuzzy match
        for profile in list_specialists(dir) {
            if profile.name == name || profile.display_name.to_lowercase().contains(&name.to_lowercase()) {
                return Ok(profile);
            }
        }
        Err(format!("Specialist '{}' not found in {}", name, dir.display()))
    }
}

/// Sanitize a name for use in filenames
fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "")
}

/// Pre-built specialist role definitions for common roles
pub mod presets {
    use super::*;

    pub fn junior_react_developer() -> SpecialistRole {
        SpecialistRole {
            category: "software-engineer".to_string(),
            title: "Junior React Developer".to_string(),
            level: ExperienceLevel::Junior,
            domains: vec![
                "frontend".to_string(),
                "react".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "web-development".to_string(),
            ],
            tools: vec![
                "React 18+".to_string(),
                "TypeScript".to_string(),
                "Next.js".to_string(),
                "Tailwind CSS".to_string(),
                "Vitest".to_string(),
                "React Testing Library".to_string(),
            ],
            industries: vec![
                "saas".to_string(),
                "e-commerce".to_string(),
                "startups".to_string(),
            ],
        }
    }

    pub fn data_analyst() -> SpecialistRole {
        SpecialistRole {
            category: "data".to_string(),
            title: "Data Analyst".to_string(),
            level: ExperienceLevel::Mid,
            domains: vec![
                "data-analysis".to_string(),
                "statistics".to_string(),
                "sql".to_string(),
                "visualization".to_string(),
            ],
            tools: vec![
                "Python".to_string(),
                "Pandas".to_string(),
                "SQL".to_string(),
                "Jupyter".to_string(),
                "Tableau".to_string(),
            ],
            industries: vec![
                "finance".to_string(),
                "healthcare".to_string(),
                "e-commerce".to_string(),
                "saas".to_string(),
            ],
        }
    }

    pub fn content_writer() -> SpecialistRole {
        SpecialistRole {
            category: "content".to_string(),
            title: "Technical Content Writer".to_string(),
            level: ExperienceLevel::Mid,
            domains: vec![
                "technical-writing".to_string(),
                "documentation".to_string(),
                "blogging".to_string(),
                "developer-relations".to_string(),
            ],
            tools: vec![
                "Markdown".to_string(),
                "MDX".to_string(),
                "Git".to_string(),
                "Notion".to_string(),
            ],
            industries: vec![
                "developer-tools".to_string(),
                "saas".to_string(),
                "open-source".to_string(),
            ],
        }
    }

    pub fn devops_engineer() -> SpecialistRole {
        SpecialistRole {
            category: "infrastructure".to_string(),
            title: "DevOps Engineer".to_string(),
            level: ExperienceLevel::Senior,
            domains: vec![
                "devops".to_string(),
                "ci-cd".to_string(),
                "cloud".to_string(),
                "containers".to_string(),
                "monitoring".to_string(),
            ],
            tools: vec![
                "Docker".to_string(),
                "Kubernetes".to_string(),
                "Terraform".to_string(),
                "GitHub Actions".to_string(),
                "AWS".to_string(),
                "Prometheus".to_string(),
            ],
            industries: vec![
                "saas".to_string(),
                "fintech".to_string(),
                "enterprise".to_string(),
            ],
        }
    }

    pub fn customer_support() -> SpecialistRole {
        SpecialistRole {
            category: "support".to_string(),
            title: "Customer Support Specialist".to_string(),
            level: ExperienceLevel::Junior,
            domains: vec![
                "customer-support".to_string(),
                "troubleshooting".to_string(),
                "product-knowledge".to_string(),
            ],
            tools: vec![
                "Zendesk".to_string(),
                "Intercom".to_string(),
                "Slack".to_string(),
            ],
            industries: vec![
                "saas".to_string(),
                "e-commerce".to_string(),
                "fintech".to_string(),
            ],
        }
    }

    /// Get all available preset roles
    pub fn all_roles() -> Vec<(&'static str, SpecialistRole)> {
        vec![
            ("junior-react-dev", junior_react_developer()),
            ("data-analyst", data_analyst()),
            ("content-writer", content_writer()),
            ("devops-engineer", devops_engineer()),
            ("customer-support", customer_support()),
        ]
    }

    /// Get a preset role by name
    pub fn get_role(name: &str) -> Option<SpecialistRole> {
        match name.to_lowercase().as_str() {
            "junior-react-dev" | "react-dev" | "frontend-dev" => Some(junior_react_developer()),
            "data-analyst" | "analyst" => Some(data_analyst()),
            "content-writer" | "writer" | "technical-writer" => Some(content_writer()),
            "devops-engineer" | "devops" | "sre" => Some(devops_engineer()),
            "customer-support" | "support" => Some(customer_support()),
            _ => None,
        }
    }

    /// Generate a default system prompt for a given role
    pub fn default_prompt(role: &SpecialistRole) -> SpecialistPrompt {
        let level = role.level.label();
        let title = &role.title;
        let domains = role.domains.join(", ");
        let tools = role.tools.join(", ");

        SpecialistPrompt {
            identity: format!(
                "You are a {level} {title} working autonomously for your employer. \
                 Your expertise covers: {domains}. \
                 You work with: {tools}. \
                 You are powered by SAGE — a decentralized AI that learns from every task. \
                 Your knowledge is stored in a Neural Cellular Automata brain that grows stronger with use. \
                 You take pride in delivering high-quality, production-ready work."
            ),
            task_instructions: format!(
                "When given a task:\n\
                 1. Understand the requirements fully before starting\n\
                 2. Retrieve relevant knowledge from your NCA brain\n\
                 3. Plan your approach in clear steps\n\
                 4. Execute with attention to detail\n\
                 5. Validate your output against the requirements\n\
                 6. Report completion with a summary of what you did\n\
                 \n\
                 If a task is outside your expertise ({domains}), \
                 clearly state that and suggest who might handle it better."
            ),
            quality_standards: format!(
                "Your work must meet these standards:\n\
                 - Correctness: output must be accurate and functional\n\
                 - Completeness: address all requirements, no partial work\n\
                 - Clarity: results should be well-organized and easy to understand\n\
                 - Consistency: follow established patterns and conventions\n\
                 - Testability: where applicable, include verification steps\n\
                 \n\
                 As a {level} specialist, you are expected to work independently \
                 and ask clarifying questions only when truly blocked."
            ),
            communication_style: "Communicate professionally and concisely. \
                 When reporting results, use this structure:\n\
                 1. What was requested\n\
                 2. What you did\n\
                 3. Key decisions made\n\
                 4. The deliverable\n\
                 5. Any follow-up needed\n\
                 \n\
                 Be direct. No fluff. Your employer values clarity over charm."
                .to_string(),
            constraints: "You must NOT:\n\
                 - Make up facts or capabilities you don't have\n\
                 - Execute tasks outside your defined capabilities\n\
                 - Share internal NCA state or technical implementation details\n\
                 - Claim expertise in domains not listed in your profile\n\
                 - Exceed your max concurrent task limit"
                .to_string(),
        }
    }

    /// Generate default capabilities for a role
    pub fn default_capabilities(role: &SpecialistRole) -> Vec<Capability> {
        match role.category.as_str() {
            "software-engineer" => vec![
                Capability {
                    name: "component-development".to_string(),
                    description: "Build React components from specifications".to_string(),
                    quality_threshold: 0.7,
                    avg_completion_secs: 600,
                },
                Capability {
                    name: "bug-fix".to_string(),
                    description: "Diagnose and fix bugs in frontend code".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 900,
                },
                Capability {
                    name: "code-review".to_string(),
                    description: "Review pull requests for quality and correctness".to_string(),
                    quality_threshold: 0.8,
                    avg_completion_secs: 300,
                },
                Capability {
                    name: "refactoring".to_string(),
                    description: "Improve code structure without changing behavior".to_string(),
                    quality_threshold: 0.7,
                    avg_completion_secs: 1200,
                },
                Capability {
                    name: "unit-testing".to_string(),
                    description: "Write comprehensive unit tests".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 600,
                },
            ],
            "data" => vec![
                Capability {
                    name: "data-cleaning".to_string(),
                    description: "Clean and prepare datasets for analysis".to_string(),
                    quality_threshold: 0.8,
                    avg_completion_secs: 600,
                },
                Capability {
                    name: "statistical-analysis".to_string(),
                    description: "Run statistical tests and interpret results".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 900,
                },
                Capability {
                    name: "dashboard-creation".to_string(),
                    description: "Build data visualization dashboards".to_string(),
                    quality_threshold: 0.7,
                    avg_completion_secs: 1200,
                },
                Capability {
                    name: "report-generation".to_string(),
                    description: "Generate analysis reports with findings".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 600,
                },
            ],
            "content" => vec![
                Capability {
                    name: "blog-post".to_string(),
                    description: "Write technical blog posts".to_string(),
                    quality_threshold: 0.7,
                    avg_completion_secs: 1800,
                },
                Capability {
                    name: "documentation".to_string(),
                    description: "Write API/docs/README documentation".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 1200,
                },
                Capability {
                    name: "tutorial".to_string(),
                    description: "Create step-by-step tutorials".to_string(),
                    quality_threshold: 0.7,
                    avg_completion_secs: 2400,
                },
            ],
            "infrastructure" => vec![
                Capability {
                    name: "ci-cd-setup".to_string(),
                    description: "Set up CI/CD pipelines".to_string(),
                    quality_threshold: 0.8,
                    avg_completion_secs: 1800,
                },
                Capability {
                    name: "dockerization".to_string(),
                    description: "Containerize applications".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 900,
                },
                Capability {
                    name: "infrastructure-as-code".to_string(),
                    description: "Write Terraform/Pulumi configurations".to_string(),
                    quality_threshold: 0.8,
                    avg_completion_secs: 1200,
                },
                Capability {
                    name: "monitoring-setup".to_string(),
                    description: "Set up monitoring and alerting".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 900,
                },
            ],
            "support" => vec![
                Capability {
                    name: "ticket-triage".to_string(),
                    description: "Categorize and prioritize support tickets".to_string(),
                    quality_threshold: 0.8,
                    avg_completion_secs: 120,
                },
                Capability {
                    name: "troubleshooting".to_string(),
                    description: "Diagnose and resolve common issues".to_string(),
                    quality_threshold: 0.75,
                    avg_completion_secs: 600,
                },
                Capability {
                    name: "faq-response".to_string(),
                    description: "Answer frequently asked questions".to_string(),
                    quality_threshold: 0.85,
                    avg_completion_secs: 180,
                },
            ],
            _ => vec![Capability {
                name: "general-task".to_string(),
                description: "Handle general tasks in this domain".to_string(),
                quality_threshold: 0.7,
                avg_completion_secs: 600,
            }],
        }
    }

    /// Generate default hiring info for a role level
    pub fn default_hiring(role: &SpecialistRole) -> HiringInfo {
        let rate = match role.level {
            ExperienceLevel::Junior => 25.0,
            ExperienceLevel::Mid => 50.0,
            ExperienceLevel::Senior => 100.0,
            ExperienceLevel::Lead => 150.0,
            ExperienceLevel::Principal => 200.0,
        };

        HiringInfo {
            suggested_rate_usd: rate,
            availability: "on-demand".to_string(),
            max_concurrent_tasks: match role.level {
                ExperienceLevel::Junior => 1,
                ExperienceLevel::Mid => 2,
                ExperienceLevel::Senior => 3,
                ExperienceLevel::Lead => 4,
                ExperienceLevel::Principal => 5,
            },
            ramp_up_minutes: match role.level {
                ExperienceLevel::Junior => 15,
                ExperienceLevel::Mid => 10,
                ExperienceLevel::Senior => 5,
                ExperienceLevel::Lead => 3,
                ExperienceLevel::Principal => 1,
            },
            languages: vec!["English".to_string()],
            timezone: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_level_labels() {
        assert_eq!(ExperienceLevel::Junior.label(), "junior");
        assert_eq!(ExperienceLevel::Senior.label(), "senior");
        assert_eq!(ExperienceLevel::from_label("mid-level"), Some(ExperienceLevel::Mid));
        assert_eq!(ExperienceLevel::from_label("nonexistent"), None);
    }

    #[test]
    fn test_preset_roles_exist() {
        let roles = presets::all_roles();
        assert_eq!(roles.len(), 5);
        assert!(presets::get_role("junior-react-dev").is_some());
        assert!(presets::get_role("data-analyst").is_some());
    }

    #[test]
    fn test_prompt_assembly() {
        let prompt = SpecialistPrompt {
            identity: "You are a test specialist.".to_string(),
            task_instructions: "Do the task.".to_string(),
            quality_standards: "Be good.".to_string(),
            communication_style: "Be clear.".to_string(),
            constraints: "Don't lie.".to_string(),
        };
        let assembled = prompt.assemble();
        assert!(assembled.contains("You are a test specialist"));
        assert!(assembled.contains("## Task Instructions"));
        assert!(assembled.contains("## Quality Standards"));
        assert!(assembled.contains("## Communication Style"));
        assert!(assembled.contains("## Constraints"));
    }

    #[test]
    fn test_specialist_save_load_roundtrip() {
        use tempfile::tempdir;

        let role = presets::junior_react_developer();
        let capabilities = presets::default_capabilities(&role);
        let prompt = presets::default_prompt(&role);
        let hiring = presets::default_hiring(&role);
        let quality = QualityMetrics {
            hit_rate: 0.85,
            mean_relevance: 0.72,
            topics_verified: 12,
            facts_encoded: 240,
            active_cells: 1500,
            grid_utilization: 0.023,
            topic_hit_rates: vec![],
        };

        let profile = SpecialistProfile {
            name: "test-react-dev".to_string(),
            display_name: "Test React Developer".to_string(),
            tagline: "Builds clean React components".to_string(),
            description: "A test specialist for roundtrip verification".to_string(),
            version: "0.1.0".to_string(),
            role,
            capabilities,
            quality,
            prompt,
            hiring,
            template_name: "test-template".to_string(),
            created_at: 1234567890,
            author_node_id: "abc123".to_string(),
            tags: vec!["react".to_string(), "test".to_string()],
        };

        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let saved_path = profile.save(&dir_path).unwrap();
        assert!(saved_path.ends_with(".specialist"));

        let loaded = SpecialistProfile::load(&PathBuf::from(&saved_path)).unwrap();
        assert_eq!(loaded.name, "test-react-dev");
        assert_eq!(loaded.role.title, "Junior React Developer");
        assert_eq!(loaded.quality.hit_rate, 0.85);
        assert_eq!(loaded.capabilities.len(), 5);
    }

    #[test]
    fn test_list_and_find_specialists() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let role = presets::junior_react_developer();
        let profile = SpecialistProfile {
            name: "find-test".to_string(),
            display_name: "Find Test".to_string(),
            tagline: "test".to_string(),
            description: "test".to_string(),
            version: "0.1.0".to_string(),
            role: role.clone(),
            capabilities: presets::default_capabilities(&role),
            quality: QualityMetrics {
                hit_rate: 0.8,
                mean_relevance: 0.7,
                topics_verified: 5,
                facts_encoded: 100,
                active_cells: 500,
                grid_utilization: 0.01,
                topic_hit_rates: vec![],
            },
            prompt: presets::default_prompt(&role),
            hiring: presets::default_hiring(&role),
            template_name: "test".to_string(),
            created_at: 0,
            author_node_id: "test".to_string(),
            tags: vec![],
        };

        profile.save(&dir_path).unwrap();

        let list = list_specialists(&dir_path);
        assert_eq!(list.len(), 1);

        let found = find_specialist("find-test", &dir_path).unwrap();
        assert_eq!(found.name, "find-test");
    }
}
