//! sage-train: Train the NCA token predictor
//!
//! Usage:
//!   sage-train --corpus path/to/text.txt [--epochs 100] [--cell-type mlp|kan]
//!   sage-train --demo   # Train on built-in Shakespeare excerpt

use sage::inference::nca_predictor::{self, TrainingConfig, Optimizer, CellType, default_weights_path};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut corpus_path: Option<String> = None;
    let mut epochs = 100;
    let mut demo = false;
    let mut grid_size: Option<usize> = None;
    let mut max_examples: Option<usize> = None;
    let mut optimizer: Option<Optimizer> = None;
    let mut sigma: Option<f64> = None;
    let mut population_size: Option<usize> = None;
    let mut cell_type = CellType::Mlp;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" => { i += 1; corpus_path = Some(args[i].clone()); }
            "--epochs" => { i += 1; epochs = args[i].parse().unwrap_or(100); }
            "--grid-size" => { i += 1; grid_size = Some(args[i].parse().unwrap_or(8)); }
            "--max-examples" => { i += 1; max_examples = Some(args[i].parse().unwrap_or(30)); }
            "--optimizer" => {
                i += 1;
                optimizer = Some(match args[i].as_str() {
                    "cma-es" | "cmaes" | "cma" => Optimizer::CmaEs,
                    "es" => Optimizer::Es,
                    other => { eprintln!("Unknown optimizer '{}', using 'es'", other); Optimizer::Es }
                });
            }
            "--sigma" => { i += 1; sigma = Some(args[i].parse().unwrap_or(0.3)); }
            "--population-size" => { i += 1; population_size = Some(args[i].parse().unwrap_or(10)); }
            "--cell-type" => {
                i += 1;
                cell_type = match args[i].as_str() {
                    "kan" => CellType::Kan,
                    "mlp" => CellType::Mlp,
                    other => { eprintln!("Unknown cell type '{}', using 'mlp'", other); CellType::Mlp }
                };
            }
            "--demo" => { demo = true; }
            "--help" | "-h" => {
                eprintln!("sage-train: NCA token prediction trainer");
                eprintln!("  --corpus <file>     Text corpus to train on");
                eprintln!("  --epochs <n>        Training epochs (default: 100)");
                eprintln!("  --grid-size <n>     NCA grid side length (default: 8)");
                eprintln!("  --max-examples <n>  Max training examples (default: 30)");
                eprintln!("  --optimizer <es|cma-es>  Optimizer (default: es)");
                eprintln!("  --sigma <f>         Initial step size (default: 0.02 for es, 0.3 for cma-es)");
                eprintln!("  --population-size <n>  Population size (cma-es auto-selects if not set)");
                eprintln!("  --cell-type <mlp|kan>  Cell brain type (default: mlp)");
                eprintln!("  --demo              Train on built-in Shakespeare excerpt");
                return;
            }
            _ => { eprintln!("Unknown arg: {}", args[i]); }
        }
        i += 1;
    }

    let corpus = if demo {
        SHAKESPEARE.to_string()
    } else if let Some(path) = corpus_path {
        fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", path, e);
            std::process::exit(1);
        })
    } else {
        eprintln!("Provide --corpus <file> or --demo. See --help.");
        std::process::exit(1);
    };

    eprintln!("🧬 NCA Token Prediction Training (cell type: {})", cell_type);
    eprintln!("   Corpus: {} chars, {} words", corpus.len(), corpus.split_whitespace().count());

    let mut config = if demo && epochs == 100 && grid_size.is_none() && max_examples.is_none() && optimizer.is_none() {
        TrainingConfig::default()
    } else {
        TrainingConfig {
            epochs,
            ..Default::default()
        }
    };
    if let Some(gs) = grid_size {
        config.grid_size = gs;
    }
    if let Some(me) = max_examples {
        config.max_examples = me;
    }
    if let Some(opt) = optimizer {
        config.optimizer = opt;
    }
    if let Some(s) = sigma {
        config.sigma = s;
    }
    if let Some(ps) = population_size {
        config.population_size = ps;
    }

    match cell_type {
        CellType::Mlp => {
            match nca_predictor::train_nca(&corpus, &config, true) {
                Ok((predictor, accuracy, random_baseline)) => {
                    print_results(accuracy, random_baseline);
                    let path = default_weights_path();
                    match predictor.weights().save(&path) {
                        Ok(()) => eprintln!("   💾 Weights saved to {}", path.display()),
                        Err(e) => eprintln!("   ❌ Failed to save weights: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("❌ Training failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        CellType::Kan => {
            match nca_predictor::train_nca_kan(&corpus, &config, true) {
                Ok((_predictor, accuracy, random_baseline)) => {
                    print_results(accuracy, random_baseline);
                    // KAN weights are saved internally by train_nca_kan
                    // The predictor holds MLP placeholder weights
                    eprintln!("   💾 KAN training complete (weights in memory)");
                }
                Err(e) => {
                    eprintln!("❌ Training failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_results(accuracy: f64, random_baseline: f64) {
    eprintln!("\n✅ Training complete!");
    eprintln!("   Final top-5 accuracy: {:.2}%", accuracy * 100.0);
    eprintln!("   Random baseline:      {:.4}%", random_baseline * 100.0);
    let ratio = accuracy / random_baseline;
    eprintln!("   Signal ratio:         {:.1}x random", ratio);

    if ratio > 1.5 {
        eprintln!("   🎉 SIGNAL DETECTED! NCA predicts better than random!");
    } else {
        eprintln!("   ⚠️  Weak/no signal yet. Try more epochs or different corpus.");
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
For who would bear the whips and scorns of time,
Th' oppressor's wrong, the proud man's contumely,
The pangs of despised love, the law's delay,
The insolence of office, and the spurns
That patient merit of th' unworthy takes,
When he himself might his quietus make
With a bare bodkin? Who would fardels bear,
To grunt and sweat under a weary life,
But that the dread of something after death,
The undiscovered country, from whose bourn
No traveller returns, puzzles the will,
And makes us rather bear those ills we have
Than fly to others that we know not of?
Thus conscience does make cowards of us all,
And thus the native hue of resolution
Is sicklied o'er with the pale cast of thought,
And enterprises of great pith and moment
With this regard their currents turn awry
And lose the name of action.
Friends, Romans, countrymen, lend me your ears;
I come to bury Caesar, not to praise him.
The evil that men do lives after them;
The good is oft interred with their bones;
So let it be with Caesar. The noble Brutus
Hath told you Caesar was ambitious:
If it were so, it was a grievous fault,
And grievously hath Caesar answer'd it.
Here, under leave of Brutus and the rest,
For Brutus is an honourable man;
So are they all, all honourable men,
Come I to speak in Caesar's funeral.
He was my friend, faithful and just to me:
But Brutus says he was ambitious;
And Brutus is an honourable man.
All that glitters is not gold;
Often have you heard that told:
Many a man his life hath sold
But my outside to behold:
Gilded tombs do worms enfold.
Had you been as wise as bold,
Young in limbs, in judgment old,
Your answer had not been inscroll'd:
Fare you well; your suit is cold.
Now is the winter of our discontent
Made glorious summer by this sun of York;
And all the clouds that lour'd upon our house
In the deep bosom of the ocean buried.
"#;
