//! sage-criticality: Measure and optimize NCA criticality
//!
//! Commands:
//!   measure  — Measure current NCA criticality metrics
//!   train    — Train NCA with criticality regularization
//!   sweep    — Sweep criticality weights to find the sweet spot

use sage::inference::criticality::{measure_criticality, train_nca_critical};
use sage::inference::nca_predictor::{
    default_weights_path, NcaPredictor, NcaWeights, SimpleTokenizer, TrainingConfig,
};
// Reservoir imports available for future integration
// use sage::inference::reservoir::{train_reservoir_readout, ReservoirConfig, FeatureStrategy};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }
    match args[1].as_str() {
        "measure" => cmd_measure(&args[2..]),
        "train" => cmd_train(&args[2..]),
        "sweep" => cmd_sweep(&args[2..]),
        "--help" | "-h" | "help" => print_help(),
        other => {
            eprintln!("Unknown command: {}. Use --help.", other);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("sage-criticality: NCA edge-of-chaos measurement & optimization");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  measure  Measure criticality of trained NCA weights");
    eprintln!("  train    Train NCA with criticality regularization");
    eprintln!("  sweep    Sweep criticality weight and show accuracy vs criticality tradeoff");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --weights <path>       NCA weights file (default: ~/.sage/nca_weights.bin)");
    eprintln!("  --corpus <file>        Text corpus for train/sweep");
    eprintln!("  --grid-size <n>        Grid side length (default: 8)");
    eprintln!("  --samples <n>          Perturbation samples (default: 100)");
    eprintln!("  --perturbation <f>     Perturbation size (default: 0.05)");
    eprintln!("  --critical             Enable criticality regularization (train)");
    eprintln!("  --crit-weight <f>      Criticality weight (default: 0.3)");
    eprintln!("  --epochs <n>           Training epochs (default: 30)");
}

struct Opts {
    weights_path: Option<String>,
    corpus_path: Option<String>,
    grid_size: usize,
    samples: usize,
    perturbation: f64,
    critical: bool,
    crit_weight: f64,
    epochs: usize,
    max_examples: usize,
}

fn parse_opts(args: &[String]) -> Opts {
    let mut o = Opts {
        weights_path: None,
        corpus_path: None,
        grid_size: 8,
        samples: 100,
        perturbation: 0.05,
        critical: false,
        crit_weight: 0.3,
        epochs: 30,
        max_examples: 50,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--weights" => {
                i += 1;
                if i < args.len() {
                    o.weights_path = Some(args[i].clone());
                }
            }
            "--corpus" => {
                i += 1;
                if i < args.len() {
                    o.corpus_path = Some(args[i].clone());
                }
            }
            "--grid-size" => {
                i += 1;
                if i < args.len() {
                    o.grid_size = args[i].parse().unwrap_or(8);
                }
            }
            "--samples" => {
                i += 1;
                if i < args.len() {
                    o.samples = args[i].parse().unwrap_or(100);
                }
            }
            "--perturbation" => {
                i += 1;
                if i < args.len() {
                    o.perturbation = args[i].parse().unwrap_or(0.05);
                }
            }
            "--critical" => {
                o.critical = true;
            }
            "--crit-weight" => {
                i += 1;
                if i < args.len() {
                    o.crit_weight = args[i].parse().unwrap_or(0.3);
                }
            }
            "--epochs" => {
                i += 1;
                if i < args.len() {
                    o.epochs = args[i].parse().unwrap_or(30);
                }
            }
            "--max-examples" => {
                i += 1;
                if i < args.len() {
                    o.max_examples = args[i].parse().unwrap_or(50);
                }
            }
            _ => {}
        }
        i += 1;
    }
    o
}

fn cmd_measure(args: &[String]) {
    let opts = parse_opts(args);

    // Load weights
    let weights_path = opts
        .weights_path
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_weights_path);

    let corpus = if let Some(ref path) = opts.corpus_path {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Failed to read corpus: {}", e);
            std::process::exit(1);
        })
    } else {
        SHAKESPEARE.to_string()
    };

    let tokenizer = SimpleTokenizer::from_corpus(&corpus, opts.grid_size * opts.grid_size);
    let tokens = tokenizer.encode(&corpus);

    let weights = if weights_path.exists() {
        eprintln!("📂 Loading weights from {}", weights_path.display());
        NcaWeights::load(&weights_path).unwrap_or_else(|e| {
            eprintln!("Failed to load weights: {}. Using random.", e);
            NcaWeights::random()
        })
    } else {
        eprintln!(
            "⚠️  No weights found at {}. Using random weights.",
            weights_path.display()
        );
        NcaWeights::random()
    };

    let mut predictor = NcaPredictor::with_grid_size(tokenizer, weights, 3, opts.grid_size);
    let probe = if tokens.len() >= 5 {
        tokens[..5].to_vec()
    } else {
        tokens.clone()
    };

    eprintln!(
        "🔬 Measuring criticality ({} samples, perturbation={})...",
        opts.samples, opts.perturbation
    );
    let metrics = measure_criticality(&mut predictor, &probe, opts.samples, opts.perturbation);

    println!();
    println!("╔══════════════════════════════════════════╗");
    println!("║       NCA Criticality Report             ║");
    println!("╠══════════════════════════════════════════╣");
    println!(
        "║  Branching ratio:    {:>8.4}  (critical=1.0) ║",
        metrics.branching_ratio
    );
    println!(
        "║  Lyapunov estimate:  {:>8.4}  (critical≈0.0) ║",
        metrics.lyapunov_estimate
    );
    println!(
        "║  Power law τ:        {:>8.4}  (critical≈1.5) ║",
        metrics.power_law_exponent
    );
    println!(
        "║  Criticality score:  {:>8.4}  (1.0=perfect)  ║",
        metrics.criticality_score
    );
    println!("╚══════════════════════════════════════════╝");

    let regime = if metrics.branching_ratio < 0.8 {
        "SUBCRITICAL (ordered)"
    } else if metrics.branching_ratio > 1.2 {
        "SUPERCRITICAL (chaotic)"
    } else {
        "NEAR-CRITICAL (edge of chaos) ✨"
    };
    println!("\n  Regime: {}", regime);

    println!("\n  Avalanche Size Distribution:");
    println!("{}", metrics.avalanche_stats.ascii_histogram(30));
    println!();
}

fn cmd_train(args: &[String]) {
    let opts = parse_opts(args);

    let corpus = if let Some(ref path) = opts.corpus_path {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Failed to read corpus: {}", e);
            std::process::exit(1);
        })
    } else {
        eprintln!("Using built-in Shakespeare corpus.");
        SHAKESPEARE.to_string()
    };

    let (accuracy_weight, criticality_weight) = if opts.critical {
        (1.0 - opts.crit_weight, opts.crit_weight)
    } else {
        (1.0, 0.0)
    };

    let config = TrainingConfig {
        epochs: opts.epochs,
        grid_size: opts.grid_size,
        max_examples: opts.max_examples,
        ..Default::default()
    };

    match train_nca_critical(&corpus, &config, accuracy_weight, criticality_weight, true) {
        Ok((predictor, accuracy, crit_score)) => {
            let random = 1.0 / predictor.tokenizer.vocab_size() as f64;
            eprintln!("\n✅ Training complete!");
            eprintln!(
                "   Accuracy: {:.2}% ({:.1}× random)",
                accuracy * 100.0,
                accuracy / random
            );
            eprintln!("   Criticality score: {:.4}", crit_score);

            // Save weights
            let path = default_weights_path();
            if let Err(e) = predictor.weights().save(&path) {
                eprintln!("   ❌ Failed to save weights: {}", e);
            } else {
                eprintln!("   💾 Saved to {}", path.display());
            }
        }
        Err(e) => {
            eprintln!("❌ Training failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_sweep(args: &[String]) {
    let opts = parse_opts(args);

    let corpus = if let Some(ref path) = opts.corpus_path {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Failed to read corpus: {}", e);
            std::process::exit(1);
        })
    } else {
        eprintln!("Using built-in Shakespeare corpus.");
        SHAKESPEARE.to_string()
    };

    let crit_weights = [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0];

    eprintln!("🔍 Criticality Weight Sweep");
    eprintln!(
        "   Testing {} weight configurations...\n",
        crit_weights.len()
    );

    println!("┌─────────────┬──────────┬─────────────┬──────────┐");
    println!("│ Crit Weight  │ Accuracy │ Criticality │ Combined │");
    println!("├─────────────┼──────────┼─────────────┼──────────┤");

    let mut results = Vec::new();

    for &cw in &crit_weights {
        let aw = 1.0 - cw;
        let config = TrainingConfig {
            epochs: opts.epochs,
            grid_size: opts.grid_size,
            max_examples: opts.max_examples,
            ..Default::default()
        };

        match train_nca_critical(&corpus, &config, aw, cw, false) {
            Ok((predictor, accuracy, crit_score)) => {
                let combined = aw * accuracy + cw * crit_score;
                println!(
                    "│ {:>11.1} │ {:>7.2}% │ {:>11.4} │ {:>8.4} │",
                    cw,
                    accuracy * 100.0,
                    crit_score,
                    combined
                );
                results.push((cw, accuracy, crit_score, combined));

                // Also measure full criticality for this configuration
                let mut pred = predictor;
                let tokens = pred.tokenizer.encode(&corpus);
                let probe = if tokens.len() >= 5 {
                    tokens[..5].to_vec()
                } else {
                    tokens
                };
                let metrics = measure_criticality(&mut pred, &probe, 50, 0.05);
                eprintln!(
                    "     → BR={:.3}, λ={:.3}, τ={:.3}",
                    metrics.branching_ratio, metrics.lyapunov_estimate, metrics.power_law_exponent
                );
            }
            Err(e) => {
                println!("│ {:>11.1} │  FAILED  │   FAILED    │  FAILED  │", cw);
                eprintln!("     → Error: {}", e);
            }
        }
    }

    println!("└─────────────┴──────────┴─────────────┴──────────┘");

    // Find sweet spot
    if let Some((cw, acc, crit, _)) = results
        .iter()
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
    {
        println!("\n🎯 Sweet spot: criticality_weight={:.1}", cw);
        println!("   Accuracy: {:.2}%, Criticality: {:.4}", acc * 100.0, crit);
    }
}

const SHAKESPEARE: &str = r#"
To be, or not to be, that is the question:
Whether 'tis nobler in the mind to suffer
The slings and arrows of outrageous fortune,
Or to take arms against a sea of troubles,
And by opposing end them. To die, to sleep,
No more; and by a sleep to say we end
The heart-ache and the thousand natural shocks
That flesh is heir to: 'tis a consummation
Devoutly to be wish'd. To die, to sleep;
To sleep, perchance to dream—ay, there's the rub:
For in that sleep of death what dreams may come,
When we have shuffled off this mortal coil,
Must give us pause—there's the respect
That makes calamity of so long life.
Friends, Romans, countrymen, lend me your ears;
I come to bury Caesar, not to praise him.
The evil that men do lives after them;
The good is oft interred with their bones;
So let it be with Caesar. The noble Brutus
Hath told you Caesar was ambitious:
If it were so, it was a grievous fault,
And grievously hath Caesar answer'd it.
All that glitters is not gold;
Often have you heard that told:
Many a man his life hath sold
But my outside to behold:
Gilded tombs do worms enfold.
Now is the winter of our discontent
Made glorious summer by this sun of York;
And all the clouds that lour'd upon our house
In the deep bosom of the ocean buried.
"#;
