//! sage-reservoir: Reservoir computing with linear readout for NCA token prediction
//!
//! Usage:
//!   sage-reservoir train --corpus text.txt [--grid-size 16] [--epochs 50] [--readout-epochs 200]
//!   sage-reservoir eval --corpus text.txt [--weights path/to/readout.bin]
//!   sage-reservoir compare --corpus text.txt  # Side-by-side reservoir vs ES
//!   sage-reservoir bench [--corpus text.txt] [--output results.json]  # Structured benchmark
//!   sage-reservoir --demo  # Quick demo on Shakespeare excerpt

use sage::inference::nca_predictor::{
    self, default_weights_path, NcaPredictor, NcaWeights, SimpleTokenizer, TrainingConfig,
};
use sage::inference::reservoir::{
    default_readout_path, extract_features, train_reservoir_readout, train_standalone_readout,
    FeatureStrategy, ReservoirConfig, ReservoirReadout,
};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "train" => cmd_train(&args[2..]),
        "standalone" => cmd_standalone(&args[2..]),
        "eval" => cmd_eval(&args[2..]),
        "compare" => cmd_compare(&args[2..]),
        "bench" => cmd_bench(&args[2..]),
        "--demo" => cmd_demo(),
        "--help" | "-h" | "help" => print_help(),
        other => {
            eprintln!("Unknown command: {}. Use --help.", other);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("sage-reservoir: Reservoir computing with linear readout for NCA");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  train      Train NCA (ES) then train linear readout");
    eprintln!("  standalone Train linear readout on RANDOM NCA (no ES training)");
    eprintln!("  eval       Evaluate a trained readout on a corpus");
    eprintln!("  compare    Side-by-side comparison: reservoir vs ES-only");
    eprintln!("  bench      Run structured benchmark with JSON output");
    eprintln!("  --demo     Quick demo on Shakespeare excerpt");
    eprintln!();
    eprintln!("Train options:");
    eprintln!("  --corpus <file>          Text corpus");
    eprintln!("  --grid-size <n>          Grid side length (default: 8)");
    eprintln!("  --epochs <n>             ES training epochs for NCA (default: 30)");
    eprintln!("  --readout-epochs <n>     Readout training epochs (default: 200)");
    eprintln!("  --strategy <flat|stats>  Feature extraction (default: flat)");
    eprintln!("  --lr <f>                 Learning rate (default: 0.001)");
    eprintln!("  --max-examples <n>       Max training examples (default: 100)");
}

struct Opts {
    corpus_path: Option<String>,
    grid_size: usize,
    epochs: usize,
    readout_epochs: usize,
    strategy: FeatureStrategy,
    lr: f64,
    max_examples: usize,
    demo: bool,
}

fn parse_opts(args: &[String]) -> Opts {
    let mut o = Opts {
        corpus_path: None,
        grid_size: 8,
        epochs: 30,
        readout_epochs: 200,
        strategy: FeatureStrategy::FlatState,
        lr: 0.001,
        max_examples: 100,
        demo: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
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
            "--epochs" => {
                i += 1;
                if i < args.len() {
                    o.epochs = args[i].parse().unwrap_or(30);
                }
            }
            "--readout-epochs" => {
                i += 1;
                if i < args.len() {
                    o.readout_epochs = args[i].parse().unwrap_or(200);
                }
            }
            "--strategy" => {
                i += 1;
                if i < args.len() {
                    o.strategy = match args[i].as_str() {
                        "stats" | "spatial" => FeatureStrategy::SpatialStats,
                        _ => FeatureStrategy::FlatState,
                    };
                }
            }
            "--lr" => {
                i += 1;
                if i < args.len() {
                    o.lr = args[i].parse().unwrap_or(0.001);
                }
            }
            "--max-examples" => {
                i += 1;
                if i < args.len() {
                    o.max_examples = args[i].parse().unwrap_or(100);
                }
            }
            "--demo" => {
                o.demo = true;
            }
            _ => {}
        }
        i += 1;
    }
    o
}

fn load_corpus(opts: &Opts) -> String {
    if opts.demo {
        return SHAKESPEARE.to_string();
    }
    if let Some(ref path) = opts.corpus_path {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", path, e);
            std::process::exit(1);
        })
    } else {
        eprintln!("Provide --corpus <file> or --demo. See --help.");
        std::process::exit(1);
    }
}

fn train_or_load_nca(corpus: &str, opts: &Opts) -> NcaPredictor {
    // Try loading pre-trained weights
    let weights_path = default_weights_path();
    if weights_path.exists() {
        eprintln!(
            "📂 Loading pre-trained NCA weights from {}",
            weights_path.display()
        );
        if let Ok(weights) = NcaWeights::load(&weights_path) {
            let tokenizer = SimpleTokenizer::from_corpus(corpus, opts.grid_size * opts.grid_size);
            return NcaPredictor::with_grid_size(tokenizer, weights, 3, opts.grid_size);
        }
        eprintln!("   ⚠️ Failed to load, training fresh...");
    }

    eprintln!("🧬 Training NCA with ES ({} epochs)...", opts.epochs);
    let config = TrainingConfig {
        epochs: opts.epochs,
        grid_size: opts.grid_size,
        max_examples: opts.max_examples.min(30),
        ..Default::default()
    };
    match nca_predictor::train_nca(corpus, &config, true) {
        Ok((predictor, acc, baseline)) => {
            eprintln!(
                "   ES accuracy: {:.2}% ({:.1}× random)",
                acc * 100.0,
                acc / baseline
            );
            // Save NCA weights
            if let Err(e) = predictor.weights().save(&weights_path) {
                eprintln!("   ⚠️ Failed to save NCA weights: {}", e);
            }
            predictor
        }
        Err(e) => {
            eprintln!("❌ NCA training failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_train(args: &[String]) {
    let opts = parse_opts(args);
    let corpus = load_corpus(&opts);

    eprintln!("🔬 Reservoir Computing Training Pipeline");
    eprintln!(
        "   Corpus: {} chars, {} words",
        corpus.len(),
        corpus.split_whitespace().count()
    );

    // Phase 1: Train/load NCA
    let mut predictor = train_or_load_nca(&corpus, &opts);

    // Phase 2: Train linear readout (NCA frozen)
    eprintln!(
        "\n🧪 Phase 2: Training linear readout ({} epochs)...",
        opts.readout_epochs
    );
    let rc = ReservoirConfig {
        readout_epochs: opts.readout_epochs,
        learning_rate: opts.lr,
        feature_strategy: opts.strategy,
        max_examples: opts.max_examples,
        context_window: 5,
        ..Default::default()
    };

    match train_reservoir_readout(&mut predictor, &corpus, &rc, true) {
        Ok((readout, top1, top5)) => {
            let random = 1.0 / predictor.tokenizer.vocab_size() as f64;
            eprintln!("\n✅ Reservoir readout training complete!");
            eprintln!(
                "   Top-1 accuracy: {:.2}% ({:.1}× random)",
                top1 * 100.0,
                top1 / random
            );
            eprintln!(
                "   Top-5 accuracy: {:.2}% ({:.1}× random)",
                top5 * 100.0,
                top5 / random
            );
            eprintln!("   Random baseline: {:.4}%", random * 100.0);

            let path = default_readout_path();
            match readout.save(&path) {
                Ok(()) => eprintln!("   💾 Readout saved to {}", path.display()),
                Err(e) => eprintln!("   ❌ Failed to save readout: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Readout training failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_eval(args: &[String]) {
    let opts = parse_opts(args);
    let corpus = load_corpus(&opts);

    let readout_path = default_readout_path();
    let readout = match ReservoirReadout::load(&readout_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "❌ Failed to load readout from {}: {}",
                readout_path.display(),
                e
            );
            eprintln!("   Run 'sage-reservoir train' first.");
            std::process::exit(1);
        }
    };

    let mut predictor = train_or_load_nca(&corpus, &opts);
    let tokens = predictor.tokenizer.encode(&corpus);
    let ctx_window = 5;

    let max_ex = opts
        .max_examples
        .min(tokens.len().saturating_sub(ctx_window));
    let step = ((tokens.len() - ctx_window) / max_ex).max(1);

    let mut correct1 = 0;
    let mut correct5 = 0;
    let mut total = 0;

    for i in (0..tokens.len() - ctx_window).step_by(step).take(max_ex) {
        let ctx = &tokens[i..i + ctx_window];
        let target = tokens[i + ctx_window];
        let grid = predictor.run_and_get_state(ctx);
        let feats = extract_features(&grid, opts.strategy);
        if feats.len() != readout.feature_dim {
            eprintln!(
                "⚠️ Feature dim mismatch: {} vs {}",
                feats.len(),
                readout.feature_dim
            );
            std::process::exit(1);
        }
        let logits = readout.predict(&feats);
        let mut indexed: Vec<(usize, f64)> =
            logits.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if indexed[0].0 == target {
            correct1 += 1;
        }
        if indexed.iter().take(5).any(|(id, _)| *id == target) {
            correct5 += 1;
        }
        total += 1;
    }

    let random = 1.0 / predictor.tokenizer.vocab_size() as f64;
    let top1 = correct1 as f64 / total as f64;
    let top5 = correct5 as f64 / total as f64;
    eprintln!("📊 Evaluation Results:");
    eprintln!("   Examples: {}", total);
    eprintln!(
        "   Top-1: {:.2}% ({:.1}× random)",
        top1 * 100.0,
        top1 / random
    );
    eprintln!(
        "   Top-5: {:.2}% ({:.1}× random)",
        top5 * 100.0,
        top5 / random
    );
}

fn cmd_compare(args: &[String]) {
    let opts = parse_opts(args);
    let corpus = load_corpus(&opts);

    eprintln!("⚔️  Reservoir vs ES-Only Comparison");
    eprintln!("   Corpus: {} chars", corpus.len());
    eprintln!();

    // --- ES-only ---
    eprintln!("━━━ ES-Only (current approach) ━━━");
    let es_config = TrainingConfig {
        epochs: opts.epochs,
        grid_size: opts.grid_size,
        max_examples: opts.max_examples.min(30),
        ..Default::default()
    };
    let (es_acc, es_random) = match nca_predictor::train_nca(&corpus, &es_config, true) {
        Ok((_, acc, rnd)) => (acc, rnd),
        Err(e) => {
            eprintln!("ES failed: {}", e);
            return;
        }
    };

    // --- Reservoir (both strategies) ---
    for strategy in [FeatureStrategy::FlatState, FeatureStrategy::SpatialStats] {
        eprintln!("\n━━━ Reservoir ({:?}) ━━━", strategy);
        // Train fresh NCA for fair comparison
        let nca_config = TrainingConfig {
            epochs: opts.epochs,
            grid_size: opts.grid_size,
            max_examples: opts.max_examples.min(30),
            ..Default::default()
        };
        let mut predictor = match nca_predictor::train_nca(&corpus, &nca_config, false) {
            Ok((p, _, _)) => p,
            Err(e) => {
                eprintln!("NCA training failed: {}", e);
                continue;
            }
        };

        let rc = ReservoirConfig {
            readout_epochs: opts.readout_epochs,
            learning_rate: opts.lr,
            feature_strategy: strategy,
            max_examples: opts.max_examples,
            context_window: 5,
            ..Default::default()
        };

        match train_reservoir_readout(&mut predictor, &corpus, &rc, true) {
            Ok((_, top1, top5)) => {
                eprintln!(
                    "   Top-1: {:.2}% ({:.1}× random)",
                    top1 * 100.0,
                    top1 / es_random
                );
                eprintln!(
                    "   Top-5: {:.2}% ({:.1}× random)",
                    top5 * 100.0,
                    top5 / es_random
                );
            }
            Err(e) => eprintln!("   Readout failed: {}", e),
        }
    }

    // Summary
    eprintln!("\n━━━ Summary ━━━");
    eprintln!(
        "   ES-only top-5:  {:.2}% ({:.1}× random)",
        es_acc * 100.0,
        es_acc / es_random
    );
    eprintln!("   Random baseline: {:.4}%", es_random * 100.0);
}

fn cmd_standalone(args: &[String]) {
    let opts = parse_opts(args);
    let corpus = load_corpus(&opts);

    eprintln!("🧪 Standalone Linear Readout (Random NCA Reservoir)");
    eprintln!("   This tests whether NCA grid dynamics encode language structure");
    eprintln!("   WITHOUT any NCA weight training. Pure reservoir computing.");
    eprintln!(
        "   Corpus: {} chars, {} words",
        corpus.len(),
        corpus.split_whitespace().count()
    );

    let rc = ReservoirConfig {
        readout_epochs: opts.readout_epochs,
        learning_rate: opts.lr,
        feature_strategy: opts.strategy,
        max_examples: opts.max_examples,
        context_window: 5,
        ..Default::default()
    };

    match train_standalone_readout(&corpus, opts.grid_size, 3, &rc, true) {
        Ok((readout, top1, top5, random)) => {
            eprintln!("\n✅ Standalone readout training complete!");
            eprintln!(
                "   Top-1 accuracy: {:.2}% ({:.1}× random)",
                top1 * 100.0,
                top1 / random
            );
            eprintln!(
                "   Top-5 accuracy: {:.2}% ({:.1}× random)",
                top5 * 100.0,
                top5 / random
            );
            eprintln!("   Random baseline: {:.4}%", random * 100.0);

            if top1 / random > 1.5 {
                eprintln!("   🎉 SIGNAL DETECTED! Random NCA dynamics encode structure!");
            }

            let path = default_readout_path();
            match readout.save(&path) {
                Ok(()) => eprintln!("   💾 Readout saved to {}", path.display()),
                Err(e) => eprintln!("   ❌ Failed to save readout: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Standalone readout training failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_bench(args: &[String]) {
    use std::path::PathBuf;

    let opts = parse_opts(args);
    let mut output_path: Option<PathBuf> = None;

    // Parse output path
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(PathBuf::from(args[i].clone()));
                }
            }
            _ => {}
        }
        i += 1;
    }

    let corpus = load_corpus(&opts);

    eprintln!("📊 Reservoir Computing Benchmark");
    eprintln!(
        "   Corpus: {} chars, {} words",
        corpus.len(),
        corpus.split_whitespace().count()
    );
    eprintln!(
        "   Grid: {}×{}, Max examples: {}",
        opts.grid_size, opts.grid_size, opts.max_examples
    );
    eprintln!();

    // Test 1: Standalone reservoir (random NCA) - FlatState features
    eprintln!("━━━ Test 1: Random NCA + FlatState readout ━━━");
    let rc_flat = ReservoirConfig {
        readout_epochs: opts.readout_epochs,
        learning_rate: opts.lr,
        feature_strategy: FeatureStrategy::FlatState,
        max_examples: opts.max_examples,
        context_window: 5,
        ..Default::default()
    };

    let standalone_flat = train_standalone_readout(&corpus, opts.grid_size, 3, &rc_flat, false);
    let (random_flat_top1, random_flat_top5, random_baseline) = match &standalone_flat {
        Ok((_, top1, top5, baseline)) => {
            eprintln!(
                "   Top-1: {:.2}% ({:.2}× random)",
                top1 * 100.0,
                top1 / baseline
            );
            eprintln!(
                "   Top-5: {:.2}% ({:.2}× random)",
                top5 * 100.0,
                top5 / baseline
            );
            (*top1, *top5, *baseline)
        }
        Err(e) => {
            eprintln!("   ❌ Failed: {}", e);
            (0.0, 0.0, 0.0)
        }
    };

    // Test 2: Standalone reservoir (random NCA) - SpatialStats features
    eprintln!("\n━━━ Test 2: Random NCA + SpatialStats readout ━━━");
    let rc_stats = ReservoirConfig {
        readout_epochs: opts.readout_epochs,
        learning_rate: opts.lr,
        feature_strategy: FeatureStrategy::SpatialStats,
        max_examples: opts.max_examples,
        context_window: 5,
        ..Default::default()
    };

    let standalone_stats = train_standalone_readout(&corpus, opts.grid_size, 3, &rc_stats, false);
    let (random_stats_top1, random_stats_top5, _) = match &standalone_stats {
        Ok((_, top1, top5, baseline)) => {
            eprintln!(
                "   Top-1: {:.2}% ({:.2}× random)",
                top1 * 100.0,
                top1 / baseline
            );
            eprintln!(
                "   Top-5: {:.2}% ({:.2}× random)",
                top5 * 100.0,
                top5 / baseline
            );
            (*top1, *top5, *baseline)
        }
        Err(e) => {
            eprintln!("   ❌ Failed: {}", e);
            (0.0, 0.0, random_baseline)
        }
    };

    // Test 3: Trained NCA + Linear readout
    eprintln!("\n━━━ Test 3: Trained NCA (ES) + Linear readout ━━━");
    let nca_config = TrainingConfig {
        epochs: opts.epochs,
        grid_size: opts.grid_size,
        max_examples: opts.max_examples.min(30),
        ..Default::default()
    };

    let (trained_nca_top1, trained_nca_top5) =
        match nca_predictor::train_nca(&corpus, &nca_config, false) {
            Ok((mut predictor, nca_acc, baseline)) => {
                eprintln!(
                    "   ES-only accuracy: {:.2}% ({:.2}× random)",
                    nca_acc * 100.0,
                    nca_acc / baseline
                );

                // Now train linear readout on trained NCA
                let rc = ReservoirConfig {
                    readout_epochs: opts.readout_epochs,
                    learning_rate: opts.lr,
                    feature_strategy: FeatureStrategy::FlatState,
                    max_examples: opts.max_examples,
                    context_window: 5,
                    ..Default::default()
                };
                match train_reservoir_readout(&mut predictor, &corpus, &rc, false) {
                    Ok((_, top1, top5)) => {
                        eprintln!("   Readout on trained NCA:");
                        eprintln!(
                            "     Top-1: {:.2}% ({:.2}× random)",
                            top1 * 100.0,
                            top1 / baseline
                        );
                        eprintln!(
                            "     Top-5: {:.2}% ({:.2}× random)",
                            top5 * 100.0,
                            top5 / baseline
                        );
                        (top1, top5)
                    }
                    Err(e) => {
                        eprintln!("   ❌ Readout failed: {}", e);
                        (0.0, 0.0)
                    }
                }
            }
            Err(e) => {
                eprintln!("   ❌ NCA training failed: {}", e);
                (0.0, 0.0)
            }
        };

    // Analysis
    eprintln!("\n━━━ Analysis ━━━");
    eprintln!("   Random baseline: {:.4}%", random_baseline * 100.0);

    let random_signal_flat = random_flat_top1 / random_baseline;
    let random_signal_stats = random_stats_top1 / random_baseline;
    let trained_signal = trained_nca_top1 / random_baseline;

    eprintln!("\n   Signal detection (higher = better):");
    eprintln!(
        "   Random NCA + FlatState:   {:.2}× baseline",
        random_signal_flat
    );
    eprintln!(
        "   Random NCA + SpatialStats: {:.2}× baseline",
        random_signal_stats
    );
    eprintln!(
        "   Trained NCA + Readout:    {:.2}× baseline",
        trained_signal
    );

    // Verdict
    eprintln!("\n━━━ Verdict ━━━");
    if random_signal_flat > 1.5 || random_signal_stats > 1.5 {
        eprintln!("   ✅ NCA topology provides signal for token prediction");
        eprintln!("      Random NCA + linear readout beats baseline.");
        eprintln!("      Recommendation: Investigate NCA dynamics further.");
    } else if trained_signal > 2.0 {
        eprintln!("   ⚠️  Trained NCA provides signal, but random doesn't.");
        eprintln!("      NCA must be trained; topology alone isn't enough.");
    } else {
        eprintln!("   ❌ NCA provides limited signal for token prediction.");
        eprintln!("      Reservoir computing approach may not be viable.");
    }

    // Write JSON results
    let output = output_path.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sage")
            .join("reservoir_bench.json")
    });

    if let Err(e) = std::fs::create_dir_all(output.parent().unwrap_or(&PathBuf::from("."))) {
        eprintln!("   ⚠️ Failed to create output directory: {}", e);
    }

    let results_json = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "corpus_size": corpus.len(),
        "word_count": corpus.split_whitespace().count(),
        "grid_size": opts.grid_size,
        "readout_epochs": opts.readout_epochs,
        "max_examples": opts.max_examples,
        "random_baseline": random_baseline,
        "tests": {
            "random_nca_flat": {
                "top1_accuracy": random_flat_top1,
                "top5_accuracy": random_flat_top5,
                "signal_ratio": random_signal_flat,
            },
            "random_nca_spatial": {
                "top1_accuracy": random_stats_top1,
                "top5_accuracy": random_stats_top5,
                "signal_ratio": random_signal_stats,
            },
            "trained_nca_readout": {
                "top1_accuracy": trained_nca_top1,
                "top5_accuracy": trained_nca_top5,
                "signal_ratio": trained_signal,
            },
        },
        "verdict": if random_signal_flat > 1.5 || random_signal_stats > 1.5 {
            "nca_viable"
        } else if trained_signal > 2.0 {
            "nca_requires_training"
        } else {
            "nca_not_viable"
        }
    });

    match std::fs::write(
        &output,
        serde_json::to_string_pretty(&results_json).unwrap(),
    ) {
        Ok(()) => eprintln!("\n💾 Results saved to {}", output.display()),
        Err(e) => eprintln!("   ❌ Failed to save results: {}", e),
    }
}

fn cmd_demo() {
    let args = vec!["--demo".to_string()];
    eprintln!("🎭 Running reservoir demo on Shakespeare...\n");
    cmd_compare(&args);
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
