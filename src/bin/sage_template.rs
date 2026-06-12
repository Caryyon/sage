//! sage-template — Brain template management CLI
//!
//! Commands:
//!   sage-template list                         — list all templates
//!   sage-template export <name> [options]      — export current brain to named template
//!   sage-template import <template> [options]  — import template into current brain
//!   sage-template info <name>                  — show template metadata
//!   sage-template inspect <name>               — show grid activation heatmap

use clap::{Parser, Subcommand};
use sage::brain_templates::{
    default_templates_dir, export_brain_to_template, find_template, import_template_to_knowledge,
    list_templates, BrainTemplateBundle,
};
use sage::distributed_knowledge::KnowledgeStore;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sage-template", about = "Brain template manager for SAGE")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Templates directory override
    #[arg(long, env = "SAGE_TEMPLATES_DIR")]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available brain templates
    List,

    /// Export current brain to a named template
    Export {
        /// Template name (e.g. "junior-dev")
        name: String,

        /// Human-readable description
        #[arg(short, long)]
        description: Option<String>,

        /// Comma-separated tags
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Domain this template specializes in
        #[arg(long)]
        domain: Option<String>,

        /// Source brain.bin path (default: ~/.sage/brain.bin)
        #[arg(short, long)]
        brain: Option<String>,
    },

    /// Import a template into current brain (or new brain)
    Import {
        /// Template name or path to .template file
        template: String,

        /// Destination brain.bin path (default: ~/.sage/brain.bin)
        #[arg(short, long)]
        brain: Option<String>,

        /// Overwrite existing brain without prompting
        #[arg(long)]
        force: bool,
    },

    /// Show metadata for a template
    Info {
        /// Template name
        name: String,
    },

    /// Show grid activation heatmap for a template
    Inspect {
        /// Template name
        name: String,

        /// Resolution: show every Nth row/col (default: 4, for 256→64 cells)
        #[arg(short, long, default_value_t = 4)]
        step: usize,
    },

    /// Pull a template from the SAGE hub
    Pull {
        /// Template name to pull
        name: String,

        /// Hub URL (default: https://api.whatssage.ai)
        #[arg(long, default_value = "https://api.whatssage.ai")]
        hub: String,

        /// Force overwrite if already exists locally
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let templates_dir = cli.dir.unwrap_or_else(default_templates_dir);

    match cli.command {
        Commands::List => cmd_list(&templates_dir),
        Commands::Export {
            name,
            description,
            tags,
            domain,
            brain,
        } => {
            let brain_path = brain.unwrap_or_else(default_brain_path);
            let desc = description.unwrap_or_else(|| format!("Template exported from {}", brain_path));
            match export_brain_to_template(
                &brain_path,
                &name,
                &desc,
                tags,
                domain,
                &templates_dir,
            ) {
                Ok(path) => println!("✅ Exported '{}' → {}", name, path),
                Err(e) => {
                    eprintln!("❌ Export failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Import {
            template,
            brain,
            force,
        } => {
            let brain_path = brain.unwrap_or_else(default_brain_path);

            // Resolve template: name in dir, or direct path
            let template_path = if std::path::Path::new(&template).exists() {
                PathBuf::from(template)
            } else {
                match find_template(&template, &templates_dir) {
                    Ok(b) => {
                        // Reconstruct path from name
                        templates_dir.join(format!("{}.template", sanitize_name(&b.meta.name)))
                    }
                    Err(e) => {
                        eprintln!("❌ {}", e);
                        std::process::exit(1);
                    }
                }
            };

            // Safety check: don't overwrite brain without consent
            if std::path::Path::new(&brain_path).exists() && !force {
                eprintln!(
                    "⚠️  Brain already exists at {}. Use --force to overwrite.",
                    brain_path
                );
                std::process::exit(1);
            }

            match import_template_to_knowledge(&template_path) {
                Ok(knowledge) => {
                    if let Err(e) = knowledge.save(&brain_path) {
                        eprintln!("❌ Failed to save brain: {}", e);
                        std::process::exit(1);
                    }
                    println!(
                        "✅ Imported template → {} ({} active cells)",
                        brain_path,
                        knowledge.active_knowledge(0.01).len()
                    );
                }
                Err(e) => {
                    eprintln!("❌ Import failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Info { name } => {
            match find_template(&name, &templates_dir) {
                Ok(bundle) => println!("{}", bundle.info()),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Inspect { name, step } => {
            match find_template(&name, &templates_dir) {
                Ok(bundle) => print_heatmap(&bundle, step),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Pull { name, hub, force } => cmd_pull(&templates_dir, &name, &hub, force),
    }
}

fn cmd_list(templates_dir: &PathBuf) {
    let templates = list_templates(templates_dir);

    if templates.is_empty() {
        println!("No templates found in {}", templates_dir.display());
        println!("  Export one: sage-template export <name>");
        return;
    }

    println!("📦 {} template(s) in {}", templates.len(), templates_dir.display());
    println!();
    for t in templates {
        let tags = if t.meta.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", t.meta.tags.join(", "))
        };
        println!(
            "  • {}{}{}\n    {} | {} active cells | {}",
            t.meta.name,
            tags,
            t.meta.domain.as_ref().map(|d| format!(" ({})", d)).unwrap_or_default(),
            t.meta.description,
            t.meta.active_cells,
            chrono::DateTime::from_timestamp(t.meta.created_at as i64, 0)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        );
    }
}

fn print_heatmap(bundle: &BrainTemplateBundle, step: usize) {
    use sage::grid::KNOWLEDGE_ACTIVATION;

    let grid = &bundle.grid;
    println!(
        "🔥 Activation heatmap for '{}' ({}×{}, showing every {}th cell):\n",
        bundle.meta.name, grid.width, grid.height, step
    );

    let chars = [' ', '·', '░', '▒', '▓', '█'];

    for y in (0..grid.height).step_by(step) {
        let mut line = String::new();
        for x in (0..grid.width).step_by(step) {
            let val = grid.cells[y][x][KNOWLEDGE_ACTIVATION];
            let idx = (val * (chars.len() as f64 - 1.0))
                .round()
                .clamp(0.0, chars.len() as f64 - 1.0) as usize;
            line.push(chars[idx]);
        }
        println!("{}", line);
    }

    // Stats
    let total: f64 = grid
        .cells
        .iter()
        .flatten()
        .map(|c| c[KNOWLEDGE_ACTIVATION])
        .sum();
    let avg = total / (grid.width * grid.height) as f64;
    let max = grid
        .cells
        .iter()
        .flatten()
        .map(|c| c[KNOWLEDGE_ACTIVATION])
        .fold(0.0f64, |a, b| a.max(b));
    let active = grid
        .cells
        .iter()
        .flatten()
        .filter(|c| c[KNOWLEDGE_ACTIVATION] > 0.01)
        .count();

    println!();
    println!("   Stats: avg={:.4}, max={:.4}, active_cells={}", avg, max, active);
}

fn default_brain_path() -> String {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".sage")
        .join("brain.bin")
        .to_string_lossy()
        .to_string()
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "")
}

fn cmd_pull(templates_dir: &PathBuf, name: &str, hub: &str, force: bool) {
    let local_path = templates_dir.join(format!("{}.template", sanitize_name(name)));

    if local_path.exists() && !force {
        eprintln!("⚠️  Template '{}' already exists locally.", name);
        eprintln!("   Use --force to overwrite.");
        std::process::exit(1);
    }

    println!("📡 Pulling template '{}' from {}...", name, hub);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = reqwest::Client::new();
        client
            .get(format!("{}/api/v1/templates/{}", hub, name))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
    });

    match result {
        Ok(resp) if resp.status().is_success() => {
            let bytes = rt.block_on(async { resp.bytes().await });
            match bytes {
                Ok(data) => {
                    // Verify the template deserializes correctly
                    let bundle: BrainTemplateBundle = match bincode::deserialize(&data) {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("❌ Invalid template data from hub: {}", e);
                            std::process::exit(1);
                        }
                    };

                    // Save to local templates dir
                    std::fs::create_dir_all(templates_dir).ok();
                    if let Err(e) = std::fs::write(&local_path, &data) {
                        eprintln!("❌ Failed to save template: {}", e);
                        std::process::exit(1);
                    }

                    println!("✅ Pulled template: {}", bundle.meta.name);
                    println!("   Saved to: {}", local_path.display());
                    println!("   {} active cells | {} tags | {}",
                        bundle.meta.active_cells,
                        bundle.meta.tags.join(", "),
                        bundle.meta.description,
                    );
                    println!();
                    println!("Next: sage-template import {}", name);
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
