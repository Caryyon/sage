//! sage-specialist — Specialist profile management CLI
//!
//! Commands:
//!   sage-specialist list                          — list all local specialists
//!   sage-specialist define <name> [options]       — create a specialist from a brain template
//!   sage-specialist info <name>                   — show specialist details
//!   sage-specialist hire <name>                   — deploy a specialist for autonomous work
//!   sage-specialist publish <name>                — publish specialist to the hub
//!   sage-specialist presets                       — list available preset roles

use clap::{Parser, Subcommand};
use sage::brain_templates::{default_templates_dir, find_template};
use sage::inference;
use sage::specialist::{
    default_specialists_dir, find_specialist, list_specialists, presets, Capability,
    HiringInfo, QualityMetrics, SpecialistProfile, SpecialistRole,
};
use sage::worker::{SpecialistWorker, TaskPriority, WorkerConfig};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "sage-specialist",
    about = "Specialist profile manager for SAGE — hireable autonomous AI employees"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Specialists directory override
    #[arg(long, env = "SAGE_SPECIALISTS_DIR")]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all local specialist profiles
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Define a new specialist from a brain template
    Define {
        /// Specialist name (e.g. "junior-react-dev-v1")
        name: String,

        /// Display name (e.g. "Junior React Developer")
        #[arg(long)]
        display_name: Option<String>,

        /// Short tagline / pitch
        #[arg(long)]
        tagline: Option<String>,

        /// Longer description
        #[arg(long)]
        description: Option<String>,

        /// Preset role to use (junior-react-dev, data-analyst, content-writer, devops-engineer, customer-support)
        #[arg(long)]
        role: Option<String>,

        /// Custom role category
        #[arg(long)]
        category: Option<String>,

        /// Custom role title
        #[arg(long)]
        title: Option<String>,

        /// Experience level (junior, mid, senior, lead, principal)
        #[arg(long, default_value = "junior")]
        level: String,

        /// Comma-separated domains
        #[arg(long, value_delimiter = ',')]
        domains: Vec<String>,

        /// Comma-separated tools
        #[arg(long, value_delimiter = ',')]
        tools: Vec<String>,

        /// Comma-separated industries
        #[arg(long, value_delimiter = ',')]
        industries: Vec<String>,

        /// Brain template to base this specialist on
        #[arg(long)]
        template: Option<String>,

        /// Suggested hourly rate in USD
        #[arg(long, default_value_t = 25.0)]
        rate: f64,

        /// Availability (full-time, part-time, on-demand)
        #[arg(long, default_value = "on-demand")]
        availability: String,

        /// Max concurrent tasks
        #[arg(long, default_value_t = 1)]
        max_tasks: usize,

        /// Comma-separated tags
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Quality: hit rate (0.0-1.0)
        #[arg(long, default_value_t = 0.0)]
        hit_rate: f64,

        /// Quality: facts encoded
        #[arg(long, default_value_t = 0)]
        facts_encoded: usize,

        /// Quality: active cells
        #[arg(long, default_value_t = 0)]
        active_cells: usize,
    },

    /// Show detailed info about a specialist
    Info {
        /// Specialist name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Deploy a specialist for autonomous work
    Hire {
        /// Specialist name to hire
        name: String,

        /// Task description to assign immediately
        #[arg(short, long)]
        task: Option<String>,

        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Publish a specialist to the SAGE hub
    Publish {
        /// Specialist name to publish
        name: String,

        /// Hub URL (default: https://api.whatssage.ai)
        #[arg(long, default_value = "https://api.whatssage.ai")]
        hub: String,
    },

    /// Pull a specialist from the SAGE hub
    Pull {
        /// Specialist name to pull
        name: String,

        /// Hub URL (default: https://api.whatssage.ai)
        #[arg(long, default_value = "https://api.whatssage.ai")]
        hub: String,

        /// Force overwrite if already exists locally
        #[arg(long)]
        force: bool,
    },

    /// List available preset roles
    Presets {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let specialists_dir = cli.dir.unwrap_or_else(default_specialists_dir);

    match cli.command {
        Commands::List { json } => cmd_list(&specialists_dir, json),
        Commands::Define {
            name,
            display_name,
            tagline,
            description,
            role,
            category,
            title,
            level,
            domains,
            tools,
            industries,
            template,
            rate,
            availability,
            max_tasks,
            tags,
            hit_rate,
            facts_encoded,
            active_cells,
        } => cmd_define(
            &specialists_dir,
            &name,
            display_name,
            tagline,
            description,
            role,
            category,
            title,
            &level,
            domains,
            tools,
            industries,
            template,
            rate,
            availability,
            max_tasks,
            tags,
            hit_rate,
            facts_encoded,
            active_cells,
        ),
        Commands::Info { name, json } => cmd_info(&specialists_dir, &name, json),
        Commands::Hire {
            name,
            task,
            foreground,
        } => cmd_hire(&specialists_dir, &name, task, foreground),
        Commands::Publish { name, hub } => cmd_publish(&specialists_dir, &name, &hub),
        Commands::Pull { name, hub, force } => cmd_pull(&specialists_dir, &name, &hub, force),
        Commands::Presets { json } => cmd_presets(json),
    }
}

fn cmd_list(dir: &PathBuf, json: bool) {
    let profiles = list_specialists(dir);

    if profiles.is_empty() {
        println!("No specialists found in {}", dir.display());
        println!("  Define one: sage-specialist define <name> --role junior-react-dev");
        return;
    }

    if json {
        let json_str = serde_json::to_string_pretty(&profiles).unwrap_or_default();
        println!("{}", json_str);
        return;
    }

    println!("👥 {} specialist(s) in {}", profiles.len(), dir.display());
    println!();
    for p in &profiles {
        let level = p.role.level.label();
        let hit_rate = (p.quality.hit_rate * 100.0).round() as u32;
        let tags = if p.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", p.tags.join(", "))
        };
        println!(
            "  • {}{} — {} {}\n    {}% hit rate | {} cells | ${}/hr | {}",
            p.display_name,
            tags,
            level,
            p.role.title,
            hit_rate,
            p.quality.active_cells,
            p.hiring.suggested_rate_usd,
            p.hiring.availability,
        );
    }
}

fn cmd_define(
    dir: &PathBuf,
    name: &str,
    display_name: Option<String>,
    tagline: Option<String>,
    description: Option<String>,
    role_preset: Option<String>,
    category: Option<String>,
    title: Option<String>,
    level: &str,
    domains: Vec<String>,
    tools: Vec<String>,
    industries: Vec<String>,
    template_name: Option<String>,
    rate: f64,
    availability: String,
    max_tasks: usize,
    tags: Vec<String>,
    hit_rate: f64,
    facts_encoded: usize,
    active_cells: usize,
) {
    // Resolve role: preset or custom
    let role = if let Some(ref preset_name) = role_preset {
        match presets::get_role(preset_name) {
            Some(r) => r,
            None => {
                eprintln!("❌ Unknown preset role: {}", preset_name);
                eprintln!("   Available: junior-react-dev, data-analyst, content-writer, devops-engineer, customer-support");
                std::process::exit(1);
            }
        }
    } else {
        let exp_level = sage::specialist::ExperienceLevel::from_label(level)
            .unwrap_or(sage::specialist::ExperienceLevel::Junior);
        SpecialistRole {
            category: category.unwrap_or_else(|| "general".to_string()),
            title: title.unwrap_or_else(|| "General Specialist".to_string()),
            level: exp_level,
            domains: if domains.is_empty() {
                vec!["general".to_string()]
            } else {
                domains
            },
            tools,
            industries,
        }
    };

    let capabilities = presets::default_capabilities(&role);
    let prompt = presets::default_prompt(&role);
    let hiring = HiringInfo {
        suggested_rate_usd: rate,
        availability,
        max_concurrent_tasks: max_tasks,
        ramp_up_minutes: presets::default_hiring(&role).ramp_up_minutes,
        languages: vec!["English".to_string()],
        timezone: None,
    };

    let quality = QualityMetrics {
        hit_rate,
        mean_relevance: 0.0,
        topics_verified: 0,
        facts_encoded,
        active_cells,
        grid_utilization: 0.0,
        topic_hit_rates: vec![],
    };

    // If a template is specified, try to load its metadata
    let template_meta = if let Some(ref tmpl) = template_name {
        let templates_dir = default_templates_dir();
        match find_template(tmpl, &templates_dir) {
            Ok(bundle) => {
                let q = QualityMetrics {
                    hit_rate,
                    mean_relevance: 0.0,
                    topics_verified: 0,
                    facts_encoded: if facts_encoded == 0 {
                        bundle.meta.active_cells
                    } else {
                        facts_encoded
                    },
                    active_cells: if active_cells == 0 {
                        bundle.meta.active_cells
                    } else {
                        active_cells
                    },
                    grid_utilization: 0.0,
                    topic_hit_rates: vec![],
                };
                Some((bundle.meta, q))
            }
            Err(e) => {
                eprintln!("⚠️  Template '{}' not found: {}", tmpl, e);
                eprintln!("   Continuing without template reference...");
                None
            }
        }
    } else {
        None
    };

    let (final_quality, final_template_name) = if let Some((meta, q)) = template_meta {
        (q, meta.name)
    } else {
        (quality, "none".to_string())
    };

    let profile = SpecialistProfile {
        name: name.to_string(),
        display_name: display_name.unwrap_or_else(|| name.to_string()),
        tagline: tagline.unwrap_or_else(|| format!("{} {} specialist", role.level.label(), role.title)),
        description: description.unwrap_or_else(|| {
            format!(
                "A {} {} specialist covering {}. Works with {}.",
                role.level.label(),
                role.title,
                role.domains.join(", "),
                role.tools.join(", ")
            )
        }),
        version: "0.1.0".to_string(),
        role,
        capabilities,
        quality: final_quality,
        prompt,
        hiring,
        template_name: final_template_name,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        author_node_id: "local".to_string(),
        tags,
    };

    match profile.save(dir) {
        Ok(path) => {
            println!("✅ Specialist defined: {}", profile.display_name);
            println!("   Saved to: {}", path);
            println!();
            println!("{}", profile.summary());
            println!();
            println!("Next steps:");
            println!("  sage-specialist info {}     — view details", name);
            println!("  sage-specialist hire {}     — deploy for work", name);
            println!("  sage-specialist publish {}  — share to hub", name);
        }
        Err(e) => {
            eprintln!("❌ Failed to save specialist: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_info(dir: &PathBuf, name: &str, json: bool) {
    match find_specialist(name, dir) {
        Ok(profile) => {
            if json {
                let json_str = serde_json::to_string_pretty(&profile).unwrap_or_default();
                println!("{}", json_str);
                return;
            }

            println!("{}", profile.summary());
            println!();
            println!("Capabilities:");
            for cap in &profile.capabilities {
                println!(
                    "  • {} — {} (quality threshold: {}%, ~{}s)",
                    cap.name,
                    cap.description,
                    (cap.quality_threshold * 100.0).round() as u32,
                    cap.avg_completion_secs
                );
            }
            println!();
            println!("System Prompt Preview:");
            let preview: String = profile.prompt.assemble().chars().take(500).collect();
            println!("{}...", preview);
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_hire(dir: &PathBuf, name: &str, task: Option<String>, foreground: bool) {
    match find_specialist(name, dir) {
        Ok(profile) => {
            println!("🧠 Hiring: {} ({} {})", profile.display_name, profile.role.level.label(), profile.role.title);
            println!("   Rate: ${}/hr | Max tasks: {} | Availability: {}",
                profile.hiring.suggested_rate_usd,
                profile.hiring.max_concurrent_tasks,
                profile.hiring.availability,
            );
            println!();

            if foreground {
                println!("⚙️  Starting specialist worker...");
                println!("   Loading inference engine...");

                let engine = inference::default_engine();
                let engine_name = engine.name().to_string();
                println!("   Engine: {}", engine_name);

                let worker = SpecialistWorker::new(
                    profile.clone(),
                    Arc::from(engine),
                    None,
                    Some(WorkerConfig::default()),
                );

                println!("   {} capabilities registered", profile.capabilities.len());
                println!("   System prompt: {} chars", profile.prompt.assemble().len());
                println!();

                if let Some(ref t) = task {
                    let task_id = worker.submit_task(t, None, TaskPriority::Normal);
                    println!("📋 Task submitted: {} ({})", t, task_id);
                }

                println!("🟢 Worker running. Submit tasks via:");
                println!("   sage-specialist hire {} --task \"your task\"", name);
                println!("   POST /api/v1/specialists/{}/task", name);
                println!();
                println!("   Press Ctrl+C to stop.");
                println!();

                // Run the worker loop (blocks until Ctrl+C)
                let worker_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let ws = worker_stop.clone();

                // Handle Ctrl+C
                ctrlc_handler(ws);

                worker.run();

                // Show final stats
                let stats = worker.current_stats();
                println!();
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📊 Session Summary:");
                println!("   Tasks completed: {}", stats.tasks_completed);
                println!("   Tasks failed: {}", stats.tasks_failed);
                println!("   Avg quality: {:.2}", stats.avg_quality);
                println!("   Avg completion: {:.1}s", stats.avg_completion_secs);
                println!("   Tokens used: {}", stats.total_tokens_used);
                println!("   Brain saves: {}", stats.brain_saves);
                println!("   Active cells: {}", stats.active_cells);
                println!("   Uptime: {}s", stats.uptime_secs);
            } else {
                println!("💡 To start the specialist as a background worker:");
                println!("   sage-specialist hire {} --foreground", name);
                println!();
                println!("   Or with an initial task:");
                println!("   sage-specialist hire {} --foreground --task \"Build a login form\"", name);
            }
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

/// Set up Ctrl+C handler to signal worker stop
fn ctrlc_handler(stop: Arc<std::sync::atomic::AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        eprintln!();
        eprintln!("🛑 Shutting down...");
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
    });
}

fn cmd_publish(dir: &PathBuf, name: &str, hub: &str) {
    match find_specialist(name, dir) {
        Ok(profile) => {
            println!("📡 Publishing '{}' to {}...", profile.display_name, hub);

            let body = serde_json::to_string_pretty(&profile).unwrap_or_default();

            // Try to publish via HTTP
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = reqwest::Client::new();
                client
                    .post(format!("{}/api/v1/specialists", hub))
                    .header("Content-Type", "application/json")
                    .body(body)
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await
            });

            match result {
                Ok(resp) if resp.status().is_success() => {
                    println!("✅ Published successfully!");
                    println!("   View at: {}/specialists/{}", hub, profile.name);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = rt.block_on(async { resp.text().await.unwrap_or_default() });
                    eprintln!("❌ Hub returned {}: {}", status, body);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("❌ Failed to reach hub: {}", e);
                    eprintln!("   Hub URL: {}", hub);
                    eprintln!("   Make sure the hub is running and accessible.");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_presets(json: bool) {
    let roles = presets::all_roles();

    if json {
        let role_list: Vec<serde_json::Value> = roles
            .iter()
            .map(|(name, role)| {
                serde_json::json!({
                    "name": name,
                    "category": role.category,
                    "title": role.title,
                    "level": role.level.label(),
                    "domains": role.domains,
                    "tools": role.tools,
                    "industries": role.industries,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&role_list).unwrap_or_default());
        return;
    }

    println!("📋 Available preset specialist roles:");
    println!();
    for (name, role) in &roles {
        let level = role.level.label();
        println!(
            "  {} — {} {}",
            name, level, role.title
        );
        println!("    Category: {}", role.category);
        println!("    Domains: {}", role.domains.join(", "));
        println!("    Tools: {}", role.tools.join(", "));
        println!();
    }
    println!("Use: sage-specialist define <name> --role <preset>");
}

fn cmd_pull(dir: &PathBuf, name: &str, hub: &str, force: bool) {
    let local_path = dir.join(format!("{}.specialist", name.to_lowercase().replace(' ', "_")));

    if local_path.exists() && !force {
        eprintln!("⚠️  Specialist '{}' already exists locally.", name);
        eprintln!("   Use --force to overwrite.");
        std::process::exit(1);
    }

    println!("📡 Pulling specialist '{}' from {}...", name, hub);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = reqwest::Client::new();
        client
            .get(format!("{}/api/v1/specialists/{}", hub, name))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
    });

    match result {
        Ok(resp) if resp.status().is_success() => {
            let body = rt.block_on(async { resp.text().await });
            match body {
                Ok(json_str) => {
                    match serde_json::from_str::<SpecialistProfile>(&json_str) {
                        Ok(profile) => {
                            match profile.save(dir) {
                                Ok(path) => {
                                    println!("✅ Pulled specialist: {}", profile.display_name);
                                    println!("   Saved to: {}", path);
                                    println!();
                                    println!("{}", profile.summary());
                                    println!();
                                    println!("Next: sage-specialist hire {} --foreground", name);
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to save: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Invalid specialist data from hub: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to read response: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Ok(resp) => {
            eprintln!("❌ Hub returned {}: {:?}", resp.status(), rt.block_on(async { resp.text().await }));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Failed to reach hub: {}", e);
            eprintln!("   Hub URL: {}", hub);
            std::process::exit(1);
        }
    }
}
