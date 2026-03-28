//! train-nca: Train NCA update rule on word associations using CMA-ES
//!
//! Trains on semantic pairs (cat→mammal, dog→mammal, oak→tree, etc.)
//! Saves weights to ~/.sage/nca_weights.bin
//!
//! Usage: cargo run --bin train-nca [--epochs 50] [--verbose]

use sage::inference::nca_predictor::{
    default_weights_path, train_nca, NcaPredictor, NcaWeights, Optimizer, SimpleTokenizer,
    TrainingConfig,
};

/// Synthetic word-association corpus.
/// Repeated patterns teach the NCA that certain words co-occur.
/// The repetition gives the CMA-ES a clear gradient to follow.
const WORD_ASSOC_CORPUS: &str = r#"
cat is a mammal cat is a mammal cat is a mammal
dog is a mammal dog is a mammal dog is a mammal
cat and dog are both mammals cat dog mammal
mammal cat mammal dog mammal cat mammal dog
oak is a tree oak is a tree oak is a tree
pine is a tree pine is a tree pine is a tree
oak and pine are both trees oak pine tree
tree oak tree pine tree oak tree pine
cat mammal dog mammal cat mammal dog mammal
oak tree pine tree oak tree pine tree
mammal includes cat mammal includes dog
tree includes oak tree includes pine
animal mammal cat animal mammal dog
plant tree oak plant tree pine
cat meows cat purrs cat sleeps cat hunts
dog barks dog runs dog sleeps dog plays
oak grows oak tall oak leaves oak acorn
pine grows pine tall pine needles pine cone
mammal warm blood mammal breathe air mammal have fur
tree has bark tree has roots tree has leaves tree grows tall
cat whiskers cat tail cat paws cat fur mammal
dog paws dog tail dog fur dog teeth mammal
oak wood oak bark oak ring oak acorn tree
pine resin pine bark pine cone pine needle tree
"#;

/// Verify retrieval after training: query "cat" and check if related words score high
fn verify_retrieval(predictor: &mut NcaPredictor) {
    let tokenizer = predictor.tokenizer.clone();
    let cat_ids = tokenizer.encode("cat");
    let mammal_ids = tokenizer.encode("mammal");
    let dog_ids = tokenizer.encode("dog");
    let tree_ids = tokenizer.encode("tree");
    let oak_ids = tokenizer.encode("oak");

    if cat_ids.is_empty() || mammal_ids.is_empty() {
        eprintln!("⚠  'cat' or 'mammal' not in vocabulary — corpus too small");
        return;
    }

    let activations = predictor.run_and_read(&cat_ids);

    // Rank all tokens by activation
    let mut indexed: Vec<(usize, f64)> = activations
        .iter()
        .enumerate()
        .map(|(i, &v)| (i, v))
        .collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Find ranks of target tokens
    let mammal_rank = mammal_ids
        .first()
        .and_then(|id| indexed.iter().position(|(i, _)| i == id));
    let dog_rank = dog_ids
        .first()
        .and_then(|id| indexed.iter().position(|(i, _)| i == id));
    let tree_rank = tree_ids
        .first()
        .and_then(|id| indexed.iter().position(|(i, _)| i == id));
    let oak_rank = oak_ids
        .first()
        .and_then(|id| indexed.iter().position(|(i, _)| i == id));

    eprintln!("\n🔍 Retrieval verification for query 'cat':");
    eprintln!("   Vocab size: {}", tokenizer.vocab_size());
    eprintln!(
        "   'mammal' rank: {} ({})",
        mammal_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if mammal_rank.map_or(false, |r| r < 10) {
            "✅ TOP 10!"
        } else if mammal_rank.map_or(false, |r| r < 20) {
            "👍 top 20"
        } else {
            "❌ low rank"
        }
    );
    eprintln!(
        "   'dog' rank:    {} ({})",
        dog_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if dog_rank.map_or(false, |r| r < 10) {
            "✅ TOP 10!"
        } else if dog_rank.map_or(false, |r| r < 20) {
            "👍 top 20"
        } else {
            "❌ low rank"
        }
    );
    eprintln!(
        "   'tree' rank:   {} ({})",
        tree_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if tree_rank.map_or(false, |r| r < 20) {
            "🔴 unexpectedly high (cross-cat)"
        } else {
            "✅ lower rank (expected)"
        }
    );
    eprintln!(
        "   'oak' rank:    {} ({})",
        oak_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if oak_rank.map_or(false, |r| r < 20) {
            "🔴 unexpectedly high (cross-cat)"
        } else {
            "✅ lower rank (expected)"
        }
    );

    // Show top-10 tokens
    eprintln!("\n   Top-10 activations for 'cat' query:");
    for (rank, (id, val)) in indexed.iter().take(10).enumerate() {
        let tok = if *id < tokenizer.id_to_token.len() {
            tokenizer.id_to_token[*id].as_str()
        } else {
            "<oob>"
        };
        eprintln!("   {:2}. {:15} {:.6}", rank + 1, tok, val);
    }

    // Now verify oak query → tree/pine
    if !oak_ids.is_empty() && !tree_ids.is_empty() {
        let oak_activations = predictor.run_and_read(&oak_ids);
        let mut oak_indexed: Vec<(usize, f64)> = oak_activations
            .iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
        oak_indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let tree_rank2 = tree_ids
            .first()
            .and_then(|id| oak_indexed.iter().position(|(i, _)| i == id));
        eprintln!("\n🔍 Retrieval verification for query 'oak':");
        eprintln!(
            "   'tree' rank: {}",
            tree_rank2.map_or("not found".to_string(), |r| format!("#{}", r + 1))
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut epochs = 50;
    let mut verbose = false;
    let mut population_size = 12;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--epochs" => {
                i += 1;
                epochs = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(50);
            }
            "--population" | "--pop" => {
                i += 1;
                population_size = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(12);
            }
            "--verbose" | "-v" => {
                verbose = true;
            }
            "--help" | "-h" => {
                eprintln!("train-nca: Train NCA on word associations using CMA-ES");
                eprintln!("  --epochs <n>      Number of training epochs (default: 50)");
                eprintln!("  --population <n>  CMA-ES population size (default: 12)");
                eprintln!("  --verbose/-v      Show per-epoch progress");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    eprintln!("🧠 SAGE NCA Word-Association Trainer");
    eprintln!("   Corpus: built-in word associations (cat/dog/mammal, oak/pine/tree)");
    eprintln!("   Optimizer: CMA-ES (separable diagonal)");
    eprintln!("   Epochs: {}", epochs);
    eprintln!("   Grid size: 8×8 (64 cells, training mode)");
    eprintln!();

    let config = TrainingConfig {
        population_size,
        sigma: 0.3,
        learning_rate: 0.001,
        epochs,
        context_window: 3,
        grid_size: 8, // small grid for fast training
        nca_steps: 5,
        max_examples: 50,
        optimizer: Optimizer::CmaEs,
    };

    match train_nca(WORD_ASSOC_CORPUS, &config, verbose) {
        Ok((mut predictor, accuracy, random_baseline)) => {
            let ratio = if random_baseline > 0.0 {
                accuracy / random_baseline
            } else {
                0.0
            };

            eprintln!("\n✅ Training complete!");
            eprintln!("   Final top-5 accuracy: {:.2}%", accuracy * 100.0);
            eprintln!("   Random baseline:      {:.4}%", random_baseline * 100.0);
            eprintln!("   Signal ratio:         {:.1}x random", ratio);

            if ratio > 1.5 {
                eprintln!("   🎉 Signal detected! NCA predicts better than random!");
            } else if ratio > 1.0 {
                eprintln!("   📈 Weak signal. Try more epochs for better performance.");
            } else {
                eprintln!("   ⚠  No clear signal yet. Weights saved anyway for future reference.");
            }

            // Save weights
            let path = default_weights_path();
            match predictor.weights().save(&path) {
                Ok(()) => {
                    let size_kb = path
                        .metadata()
                        .map(|m| m.len() as f64 / 1024.0)
                        .unwrap_or(0.0);
                    eprintln!("\n💾 Weights saved to: {}", path.display());
                    eprintln!("   Size: {:.1} KB", size_kb);
                }
                Err(e) => {
                    eprintln!("\n❌ Failed to save weights: {}", e);
                    std::process::exit(1);
                }
            }

            // Verify retrieval quality
            verify_retrieval(&mut predictor);
        }
        Err(e) => {
            eprintln!("❌ Training failed: {}", e);
            std::process::exit(1);
        }
    }
}
