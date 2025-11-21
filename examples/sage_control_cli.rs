// SAGE Control CLI - Manage SAGE instances from the command line
//
// Usage:
//   cargo run --release --example sage_control_cli list
//   cargo run --release --example sage_control_cli stop irc
//   cargo run --release --example sage_control_cli start discord
//   cargo run --release --example sage_control_cli restart irc

use sage::sage_control::{InstanceRegistry, InstanceType};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "list" | "ls" | "status" => {
            list_instances();
        }
        "stop" | "kill" => {
            if args.len() < 3 {
                eprintln!("Error: Missing instance type");
                eprintln!("Usage: sage_control_cli stop <irc|discord>");
                return;
            }
            stop_instance(&args[2]);
        }
        "start" => {
            if args.len() < 3 {
                eprintln!("Error: Missing instance type");
                eprintln!("Usage: sage_control_cli start <irc|discord>");
                return;
            }
            start_instance(&args[2]);
        }
        "restart" => {
            if args.len() < 3 {
                eprintln!("Error: Missing instance type");
                eprintln!("Usage: sage_control_cli restart <irc|discord>");
                return;
            }
            restart_instance(&args[2]);
        }
        "cleanup" => {
            cleanup_dead();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("SAGE Control CLI - Manage SAGE instances");
    println!();
    println!("Usage:");
    println!("  cargo run --release --example sage_control_cli <command> [args]");
    println!();
    println!("Commands:");
    println!("  list, ls, status         List all running instances");
    println!("  stop <irc|discord>       Stop an instance");
    println!("  start <irc|discord>      Start an instance");
    println!("  restart <irc|discord>    Restart an instance");
    println!("  cleanup                  Remove dead instances from registry");
}

fn list_instances() {
    let registry = InstanceRegistry::load();
    let instances = registry.get_all();

    if instances.is_empty() {
        println!("No SAGE instances detected");
        println!();
        println!("Start instances with:");
        println!("  cargo run --release --example sage_irc_autonomous");
        println!("  cargo run --release --example sage_discord_autonomous");
        return;
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("                    SAGE Instance Registry                      ");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    for instance in instances {
        let alive = instance.is_alive();
        let status_emoji = if alive {
            instance.status.indicator()
        } else {
            "💀"
        };

        println!("{} {} {}",
            instance.instance_type.emoji(),
            instance.instance_type.name(),
            status_emoji);
        println!("  PID:     {}", instance.pid);
        println!("  Status:  {}", instance.status.label());
        println!("  Uptime:  {}", instance.uptime_string());
        println!("  Alive:   {}", if alive { "Yes" } else { "Dead - no heartbeat" });
        println!("  Log:     {}", instance.log_path);
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════");
}

fn parse_instance_type(type_str: &str) -> Option<InstanceType> {
    match type_str.to_lowercase().as_str() {
        "irc" | "ircbot" => Some(InstanceType::IrcBot),
        "discord" | "discordbot" => Some(InstanceType::DiscordBot),
        "tui" | "main" => Some(InstanceType::MainTui),
        "learner" | "background" => Some(InstanceType::BackgroundLearner),
        _ => None,
    }
}

fn stop_instance(type_str: &str) {
    let instance_type = match parse_instance_type(type_str) {
        Some(t) => t,
        None => {
            eprintln!("Error: Unknown instance type '{}'", type_str);
            eprintln!("Valid types: irc, discord, tui, learner");
            return;
        }
    };

    println!("Stopping {} instance...", instance_type.name());

    let mut registry = InstanceRegistry::load();
    match registry.stop_instance(&instance_type) {
        Ok(msg) => {
            println!("✅ {}", msg);
            println!();
            println!("The process will exit gracefully and be removed from the registry.");
        }
        Err(err) => {
            eprintln!("❌ Failed to stop instance: {}", err);
        }
    }
}

fn start_instance(type_str: &str) {
    let instance_type = match parse_instance_type(type_str) {
        Some(t) => t,
        None => {
            eprintln!("Error: Unknown instance type '{}'", type_str);
            eprintln!("Valid types: irc, discord");
            return;
        }
    };

    println!("Starting {} instance...", instance_type.name());

    let mut registry = InstanceRegistry::load();
    match registry.start_instance(&instance_type) {
        Ok(msg) => {
            println!("✅ {}", msg);
            println!();
            println!("The instance will register itself within a few seconds.");
            println!("Check status with: cargo run --release --example sage_control_cli list");
        }
        Err(err) => {
            eprintln!("❌ Failed to start instance: {}", err);
        }
    }
}

fn restart_instance(type_str: &str) {
    let instance_type = match parse_instance_type(type_str) {
        Some(t) => t,
        None => {
            eprintln!("Error: Unknown instance type '{}'", type_str);
            eprintln!("Valid types: irc, discord");
            return;
        }
    };

    println!("Restarting {} instance...", instance_type.name());

    let mut registry = InstanceRegistry::load();
    match registry.restart_instance(&instance_type) {
        Ok(msg) => {
            println!("✅ {}", msg);
            println!();
            println!("The new instance will register itself within a few seconds.");
            println!("Check status with: cargo run --release --example sage_control_cli list");
        }
        Err(err) => {
            eprintln!("❌ Failed to restart instance: {}", err);
        }
    }
}

fn cleanup_dead() {
    println!("Cleaning up dead instances...");

    let mut registry = InstanceRegistry::load();
    match registry.cleanup_dead() {
        Ok(_) => {
            println!("✅ Registry cleaned");
            println!();
            list_instances();
        }
        Err(err) => {
            eprintln!("❌ Failed to cleanup: {}", err);
        }
    }
}
