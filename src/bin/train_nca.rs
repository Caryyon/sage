//! train-nca: Train NCA update rule on word associations using CMA-ES
//!
//! Trains on semantic pairs (cat→mammal, dog→mammal, oak→tree, etc.)
//! Saves weights to ~/.sage/nca_weights.bin
//!
//! Usage: cargo run --bin train-nca [--epochs 50] [--verbose] [--quick]
//!                                   [--train-embeddings [--embed-epochs N] [--embed-lr F]]

use sage::distributed_knowledge::encoder::{
    encode_text_hash, EncoderConfig, LinearProjection, PROJECTION_WEIGHTS_PATH,
};
use sage::inference::nca_predictor::{
    default_weights_path, train_nca, NcaPredictor, Optimizer, TrainingConfig,
};
use std::path::PathBuf;

/// Synthetic word-association corpus.
/// Repeated patterns teach the NCA that certain words co-occur.
/// The repetition gives the CMA-ES a clear gradient to follow.
/// Expanded to cover all semantic clusters in SYNONYM_PAIRS.
const WORD_ASSOC_CORPUS: &str = r#"
# Animals cluster
cat is a mammal cat is a mammal cat is a pet animal
dog is a mammal dog is a mammal dog is a pet animal
cat and dog are both mammals cat dog mammal pet
wolf is a mammal wolf is a predator animal
bear is a mammal bear is a predator animal
deer is a mammal deer is a wild animal
rabbit is a mammal rabbit is a pet animal
horse is a mammal horse is a large animal
eagle is a bird eagle is a predator animal
bird is an animal bird flies bird has wings
fish is an animal salmon is a fish shark is a fish
shark is a predator whale is a mammal animal
lion is a mammal lion is a predator tiger is a mammal tiger is a predator
lion and tiger are both predators mammal
elephant is a mammal elephant is a large animal

# Nature and terrain cluster
mountain is terrain mountain has a peak hill is smaller than mountain
river is water river is a stream creek flows into river
ocean is water ocean is like a sea lake is water
forest is terrain forest is like woods trees grow in forest
desert is terrain valley is terrain glacier is ice
volcano is a mountain cliff is terrain canyon is a valley
island is land coast is a shore beach is on the shore
beach and coast are near the ocean

# Weather cluster
rain is weather rain brings water storm has rain
snow is weather snow is frozen ice snow falls in winter
wind is weather storm is weather
thunder comes with lightning storm has thunder
lightning strikes during storm fog is weather mist is like fog
frost is ice frost comes with cold drought is dry weather
flood brings water flood is a disaster
hurricane is a storm tornado is a storm hurricane and tornado are dangerous

# Plants cluster
tree is a plant oak is a tree pine is a tree
maple is a tree birch is a tree
flower is a plant rose is a flower tulip is a flower
grass is a plant moss is a plant fern is a plant
vine is a plant shrub is a plant shrub is like a bush
mushroom is a fungus cactus is a plant
leaf grows on plant root is part of plant seed grows into plant
pollen comes from flower

# Food cluster
bread is food bread is made from grain
fruit is food apple is a fruit orange is a fruit banana is a fruit
meat is food beef is meat pork is meat chicken is meat
rice is food rice is a grain soup is food
cheese is food cheese is dairy milk is dairy
grain is food wheat is a grain
bean is food spice is food herb is a plant herb is a spice
vegetable is food carrot is a vegetable potato is a vegetable

# Colors cluster
red is a color blue is a color green is a color
yellow is a color orange is a color purple is a color
black is a color white is a color brown is a color gray is a color
red is like crimson blue is like azure
green is like emerald purple is like violet
pink is a color pink is similar to red

# Emotions cluster
joy is an emotion joy is happiness happiness is an emotion
fear is an emotion fear is like terror
anger is an emotion anger is like rage
sadness is an emotion sadness is like grief
love is an emotion love is like affection
surprise is an emotion disgust is an emotion
trust is an emotion hope is an emotion hope is like optimism
shame is an emotion guilt is like shame
anxiety is fear anxiety is an emotion
peace is calm calm is an emotion

# Actions and movement cluster
run is movement run is like sprint walk is movement
run and walk are both movement jump is movement jump is like leap
leap is movement swim is movement fly is movement
climb is movement crawl is movement dive is movement dive is like swim
sprint is movement sprint is like dash
dance is movement skip is movement skip is like hop

# Science cluster
atom is a particle atom is matter molecule is made of atoms
molecule is matter cell is biology
energy is physics energy is power
force is physics gravity is a force
light is energy light is made of photons
heat is energy heat is temperature
pressure is a force mass is matter mass is like weight
electron is a particle proton is a particle neutron is a particle
electron and proton are particles wave is physics frequency describes wave

# Technology cluster
code is software code is a program
data is information network is like internet
server is a computer memory is in a computer memory is storage
processor is a computer chip processor is in a computer
algorithm is code software is a program
hardware is part of computer database stores data database is storage
rust is a language python is a language
javascript is a language java is a language
compiler is software browser is software api is software

# Geography cluster
city is a place city is like a town town is a place
country is a nation country is a place
continent is land island is land peninsula is land
coast is land border is a boundary
capital is a city region is an area
province is a region state is a region
village is a small town metropolis is a large city

# Time cluster
second is time minute is time hour is time
day is time week is time month is time
year is time decade is time century is time era is time
moment is time instant is a moment
dawn is time dusk is time dawn and dusk are opposite
morning is time evening is time morning and evening are opposite

# Materials cluster
wood is a material metal is a material
stone is a material stone is like rock
glass is a material plastic is a material
fabric is a material fabric is like cloth
paper is a material rubber is a material
clay is a material sand is a material
steel is a metal iron is a metal
gold is a metal silver is a metal gold and silver are precious
copper is a metal bronze is a metal
leather is a material cotton is a fabric silk is a fabric

# Relationships cluster
friend is a companion friend is an ally
family includes relatives parent is family
parent is mother or father mother and father are parents
child is family child is offspring
partner is a companion spouse is a partner
teacher is a mentor student is a learner teacher and student interact
colleague is a coworker neighbor is a person stranger is a person
sibling is family brother is a sibling sister is a sibling
"#;

/// Quick mode corpus — minimal 8 word-pairs for fast pipeline testing
/// Should complete in under 30 seconds
const QUICK_CORPUS: &str = r#"
cat is a mammal cat mammal
dog is a mammal dog mammal
oak is a tree oak tree
pine is a tree pine tree
salmon is a fish salmon fish
eagle is a bird eagle bird
rust is a language rust language
python is a language python language
"#;

/// Get the path for quick mode weights (separate from full weights)
fn quick_weights_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{}/.sage/nca_weights_quick.bin", home))
}

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
        if mammal_rank.is_some_and(|r| r < 10) {
            "✅ TOP 10!"
        } else if mammal_rank.is_some_and(|r| r < 20) {
            "👍 top 20"
        } else {
            "❌ low rank"
        }
    );
    eprintln!(
        "   'dog' rank:    {} ({})",
        dog_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if dog_rank.is_some_and(|r| r < 10) {
            "✅ TOP 10!"
        } else if dog_rank.is_some_and(|r| r < 20) {
            "👍 top 20"
        } else {
            "❌ low rank"
        }
    );
    eprintln!(
        "   'tree' rank:   {} ({})",
        tree_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if tree_rank.is_some_and(|r| r < 20) {
            "🔴 unexpectedly high (cross-cat)"
        } else {
            "✅ lower rank (expected)"
        }
    );
    eprintln!(
        "   'oak' rank:    {} ({})",
        oak_rank.map_or("not found".to_string(), |r| format!("#{}", r + 1)),
        if oak_rank.is_some_and(|r| r < 20) {
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

// ══════════════════════════════════════════════════════════════════════════════
// Linear projection (embedding) training via contrastive learning
// ══════════════════════════════════════════════════════════════════════════════

/// Synonym / similar-word pairs for contrastive embedding training.
/// Used to train the linear projection to pull similar words closer together.
/// Organized by semantic cluster with dense intra-cluster pairings.
const SYNONYM_PAIRS: &[(&str, &str)] = &[
    // ═══════════════════════════════════════════════════════════════════════
    // Animals (mammals, pets, wildlife)
    // ═══════════════════════════════════════════════════════════════════════
    ("cat", "dog"),
    ("cat", "mammal"),
    ("cat", "animal"),
    ("cat", "pet"),
    ("dog", "mammal"),
    ("dog", "animal"),
    ("dog", "pet"),
    ("dog", "wolf"),
    ("wolf", "mammal"),
    ("wolf", "animal"),
    ("wolf", "predator"),
    ("bear", "mammal"),
    ("bear", "animal"),
    ("bear", "predator"),
    ("deer", "mammal"),
    ("deer", "animal"),
    ("rabbit", "mammal"),
    ("rabbit", "animal"),
    ("rabbit", "pet"),
    ("horse", "mammal"),
    ("horse", "animal"),
    ("eagle", "bird"),
    ("eagle", "animal"),
    ("eagle", "predator"),
    ("bird", "animal"),
    ("fish", "animal"),
    ("salmon", "fish"),
    ("shark", "fish"),
    ("shark", "predator"),
    ("whale", "mammal"),
    ("whale", "animal"),
    ("lion", "mammal"),
    ("lion", "predator"),
    ("tiger", "mammal"),
    ("tiger", "predator"),
    ("lion", "tiger"),
    ("elephant", "mammal"),
    ("elephant", "animal"),
    // ═══════════════════════════════════════════════════════════════════════
    // Nature / Terrain
    // ═══════════════════════════════════════════════════════════════════════
    ("mountain", "terrain"),
    ("mountain", "peak"),
    ("mountain", "hill"),
    ("river", "water"),
    ("river", "stream"),
    ("river", "creek"),
    ("ocean", "water"),
    ("ocean", "sea"),
    ("lake", "water"),
    ("forest", "terrain"),
    ("forest", "woods"),
    ("desert", "terrain"),
    ("valley", "terrain"),
    ("glacier", "ice"),
    ("volcano", "mountain"),
    ("cliff", "terrain"),
    ("canyon", "valley"),
    ("canyon", "terrain"),
    ("island", "land"),
    ("coast", "shore"),
    ("beach", "shore"),
    ("beach", "coast"),
    // ═══════════════════════════════════════════════════════════════════════
    // Weather / Climate
    // ═══════════════════════════════════════════════════════════════════════
    ("rain", "weather"),
    ("rain", "water"),
    ("rain", "storm"),
    ("snow", "weather"),
    ("snow", "ice"),
    ("snow", "winter"),
    ("wind", "weather"),
    ("storm", "weather"),
    ("storm", "thunder"),
    ("thunder", "lightning"),
    ("thunder", "storm"),
    ("lightning", "storm"),
    ("fog", "weather"),
    ("fog", "mist"),
    ("frost", "ice"),
    ("frost", "cold"),
    ("drought", "weather"),
    ("flood", "water"),
    ("flood", "disaster"),
    ("hurricane", "storm"),
    ("tornado", "storm"),
    ("hurricane", "tornado"),
    // ═══════════════════════════════════════════════════════════════════════
    // Plants / Vegetation
    // ═══════════════════════════════════════════════════════════════════════
    ("tree", "plant"),
    ("oak", "tree"),
    ("pine", "tree"),
    ("oak", "pine"),
    ("maple", "tree"),
    ("birch", "tree"),
    ("flower", "plant"),
    ("rose", "flower"),
    ("tulip", "flower"),
    ("rose", "tulip"),
    ("grass", "plant"),
    ("moss", "plant"),
    ("fern", "plant"),
    ("vine", "plant"),
    ("shrub", "plant"),
    ("shrub", "bush"),
    ("mushroom", "fungus"),
    ("cactus", "plant"),
    ("leaf", "plant"),
    ("root", "plant"),
    ("seed", "plant"),
    ("pollen", "flower"),
    // ═══════════════════════════════════════════════════════════════════════
    // Food / Cuisine
    // ═══════════════════════════════════════════════════════════════════════
    ("bread", "food"),
    ("bread", "grain"),
    ("fruit", "food"),
    ("apple", "fruit"),
    ("orange", "fruit"),
    ("banana", "fruit"),
    ("apple", "orange"),
    ("meat", "food"),
    ("beef", "meat"),
    ("pork", "meat"),
    ("chicken", "meat"),
    ("rice", "food"),
    ("rice", "grain"),
    ("soup", "food"),
    ("cheese", "food"),
    ("cheese", "dairy"),
    ("milk", "dairy"),
    ("grain", "food"),
    ("wheat", "grain"),
    ("bean", "food"),
    ("spice", "food"),
    ("herb", "plant"),
    ("herb", "spice"),
    ("vegetable", "food"),
    ("carrot", "vegetable"),
    ("potato", "vegetable"),
    // ═══════════════════════════════════════════════════════════════════════
    // Colors
    // ═══════════════════════════════════════════════════════════════════════
    ("red", "color"),
    ("blue", "color"),
    ("green", "color"),
    ("yellow", "color"),
    ("orange", "color"),
    ("purple", "color"),
    ("black", "color"),
    ("white", "color"),
    ("brown", "color"),
    ("gray", "color"),
    ("red", "crimson"),
    ("blue", "azure"),
    ("green", "emerald"),
    ("purple", "violet"),
    ("pink", "color"),
    ("pink", "red"),
    // ═══════════════════════════════════════════════════════════════════════
    // Emotions / Feelings
    // ═══════════════════════════════════════════════════════════════════════
    ("joy", "emotion"),
    ("joy", "happiness"),
    ("happiness", "emotion"),
    ("fear", "emotion"),
    ("fear", "terror"),
    ("anger", "emotion"),
    ("anger", "rage"),
    ("sadness", "emotion"),
    ("sadness", "grief"),
    ("love", "emotion"),
    ("love", "affection"),
    ("surprise", "emotion"),
    ("disgust", "emotion"),
    ("trust", "emotion"),
    ("hope", "emotion"),
    ("hope", "optimism"),
    ("shame", "emotion"),
    ("guilt", "shame"),
    ("anxiety", "fear"),
    ("anxiety", "emotion"),
    ("peace", "calm"),
    ("calm", "emotion"),
    // ═══════════════════════════════════════════════════════════════════════
    // Actions / Verbs (movement)
    // ═══════════════════════════════════════════════════════════════════════
    ("run", "movement"),
    ("run", "sprint"),
    ("walk", "movement"),
    ("run", "walk"),
    ("jump", "movement"),
    ("jump", "leap"),
    ("leap", "movement"),
    ("swim", "movement"),
    ("fly", "movement"),
    ("climb", "movement"),
    ("crawl", "movement"),
    ("dive", "movement"),
    ("dive", "swim"),
    ("sprint", "movement"),
    ("sprint", "dash"),
    ("dance", "movement"),
    ("skip", "movement"),
    ("skip", "hop"),
    // ═══════════════════════════════════════════════════════════════════════
    // Science / Physics
    // ═══════════════════════════════════════════════════════════════════════
    ("atom", "particle"),
    ("atom", "matter"),
    ("molecule", "atom"),
    ("molecule", "matter"),
    ("cell", "biology"),
    ("energy", "physics"),
    ("energy", "power"),
    ("force", "physics"),
    ("gravity", "force"),
    ("light", "energy"),
    ("light", "photon"),
    ("heat", "energy"),
    ("heat", "temperature"),
    ("pressure", "force"),
    ("mass", "matter"),
    ("mass", "weight"),
    ("electron", "particle"),
    ("proton", "particle"),
    ("electron", "proton"),
    ("neutron", "particle"),
    ("wave", "physics"),
    ("frequency", "wave"),
    // ═══════════════════════════════════════════════════════════════════════
    // Technology / Computing
    // ═══════════════════════════════════════════════════════════════════════
    ("code", "software"),
    ("code", "program"),
    ("data", "information"),
    ("network", "internet"),
    ("server", "computer"),
    ("memory", "computer"),
    ("memory", "storage"),
    ("processor", "computer"),
    ("processor", "chip"),
    ("algorithm", "code"),
    ("software", "program"),
    ("hardware", "computer"),
    ("database", "storage"),
    ("database", "data"),
    ("rust", "language"),
    ("python", "language"),
    ("rust", "python"),
    ("javascript", "language"),
    ("java", "language"),
    ("compiler", "software"),
    ("browser", "software"),
    ("api", "software"),
    // ═══════════════════════════════════════════════════════════════════════
    // Geography / Places
    // ═══════════════════════════════════════════════════════════════════════
    ("city", "place"),
    ("city", "town"),
    ("town", "place"),
    ("country", "nation"),
    ("country", "place"),
    ("continent", "land"),
    ("island", "land"),
    ("peninsula", "land"),
    ("coast", "land"),
    ("border", "boundary"),
    ("capital", "city"),
    ("region", "area"),
    ("province", "region"),
    ("state", "region"),
    ("village", "town"),
    ("metropolis", "city"),
    // ═══════════════════════════════════════════════════════════════════════
    // Time / Duration
    // ═══════════════════════════════════════════════════════════════════════
    ("second", "time"),
    ("minute", "time"),
    ("hour", "time"),
    ("day", "time"),
    ("week", "time"),
    ("month", "time"),
    ("year", "time"),
    ("decade", "time"),
    ("century", "time"),
    ("era", "time"),
    ("moment", "time"),
    ("instant", "moment"),
    ("dawn", "time"),
    ("dusk", "time"),
    ("dawn", "dusk"),
    ("morning", "time"),
    ("evening", "time"),
    ("morning", "evening"),
    // ═══════════════════════════════════════════════════════════════════════
    // Materials / Substances
    // ═══════════════════════════════════════════════════════════════════════
    ("wood", "material"),
    ("metal", "material"),
    ("stone", "material"),
    ("stone", "rock"),
    ("glass", "material"),
    ("plastic", "material"),
    ("fabric", "material"),
    ("fabric", "cloth"),
    ("paper", "material"),
    ("rubber", "material"),
    ("clay", "material"),
    ("sand", "material"),
    ("steel", "metal"),
    ("iron", "metal"),
    ("gold", "metal"),
    ("silver", "metal"),
    ("gold", "silver"),
    ("copper", "metal"),
    ("bronze", "metal"),
    ("leather", "material"),
    ("cotton", "fabric"),
    ("silk", "fabric"),
    ("cotton", "silk"),
    // ═══════════════════════════════════════════════════════════════════════
    // Relationships / People
    // ═══════════════════════════════════════════════════════════════════════
    ("friend", "companion"),
    ("friend", "ally"),
    ("family", "relative"),
    ("parent", "family"),
    ("parent", "mother"),
    ("parent", "father"),
    ("mother", "father"),
    ("child", "family"),
    ("child", "offspring"),
    ("partner", "companion"),
    ("spouse", "partner"),
    ("teacher", "mentor"),
    ("student", "learner"),
    ("teacher", "student"),
    ("colleague", "coworker"),
    ("neighbor", "person"),
    ("stranger", "person"),
    ("sibling", "family"),
    ("brother", "sibling"),
    ("sister", "sibling"),
    ("brother", "sister"),
];

/// Contrastive training for the linear projection W.
///
/// For each positive pair (a, b): push W·h(a) closer to W·h(b).
/// Negatives are formed by pairing a with a random unrelated word.
///
/// Loss = Σ_{pos} ||W·h(a) - W·h(b)||² - λ · Σ_{neg} ||W·h(a) - W·h(c)||²
/// Gradient computed analytically; weights updated via SGD.
fn train_embedding_projection(epochs: usize, lr: f64, verbose: bool) -> LinearProjection {
    let config = EncoderConfig::default();
    let dim = config.num_features; // 64

    // Start from identity (safe, no-op initially)
    let mut proj = LinearProjection::identity(dim);

    // Precompute hash vectors for all unique words
    let mut all_words: Vec<&str> = SYNONYM_PAIRS.iter().flat_map(|(a, b)| [*a, *b]).collect();
    all_words.sort_unstable();
    all_words.dedup();

    let hash_vecs: std::collections::HashMap<&str, Vec<f64>> = all_words
        .iter()
        .map(|&w| {
            let fv = encode_text_hash(w, &config);
            (w, fv.values)
        })
        .collect();

    // Negative words: anything not in the positive pair
    let neg_words: Vec<&str> = all_words.clone();

    let lambda_neg = 0.3_f64; // negative margin weight

    for epoch in 0..epochs {
        let mut total_pos_loss = 0.0_f64;
        let mut total_neg_loss = 0.0_f64;
        let mut total_grad = vec![0.0_f64; dim * dim];

        for &(wa, wb) in SYNONYM_PAIRS {
            let ha = &hash_vecs[wa];
            let hb = &hash_vecs[wb];

            // Projected embeddings
            let pa = proj.forward(ha);
            let pb = proj.forward(hb);

            // Positive gradient: d/dW ||W·ha - W·hb||² = 2(W·ha - W·hb)·ha^T - 2(W·ha - W·hb)·hb^T
            let diff_pos: Vec<f64> = pa.iter().zip(pb.iter()).map(|(a, b)| a - b).collect();
            let pos_loss: f64 = diff_pos.iter().map(|d| d * d).sum();
            total_pos_loss += pos_loss;

            // Add positive gradient: grad += 2 * diff * ha^T - 2 * diff * hb^T
            for row in 0..dim {
                for col in 0..dim {
                    total_grad[row * dim + col] += 2.0 * diff_pos[row] * (ha[col] - hb[col]);
                }
            }

            // Pick a negative: word with low similarity to wa
            // Simple deterministic selection: use index hash
            let neg_idx = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                (wa, wb, epoch).hash(&mut hasher);
                (hasher.finish() as usize) % neg_words.len()
            };
            let wc = neg_words[neg_idx];
            if wc == wa || wc == wb {
                continue; // skip same-word negatives
            }
            let hc = &hash_vecs[wc];
            let pc = proj.forward(hc);

            // Negative gradient: push apart — subtract contribution
            let diff_neg: Vec<f64> = pa.iter().zip(pc.iter()).map(|(a, c)| a - c).collect();
            let neg_loss: f64 = diff_neg.iter().map(|d| d * d).sum();
            total_neg_loss += neg_loss;

            // grad -= 2 * lambda * diff_neg * (ha - hc)^T
            for row in 0..dim {
                for col in 0..dim {
                    total_grad[row * dim + col] -=
                        2.0 * lambda_neg * diff_neg[row] * (ha[col] - hc[col]);
                }
            }
        }

        // Clip gradient norm to avoid explosion
        let grad_norm: f64 = total_grad.iter().map(|g| g * g).sum::<f64>().sqrt();
        let clip = 10.0_f64;
        let scale = if grad_norm > clip {
            clip / grad_norm
        } else {
            1.0
        };
        for g in &mut total_grad {
            *g *= scale;
        }

        proj.apply_gradient(&total_grad, lr);

        let combined_loss = total_pos_loss - lambda_neg * total_neg_loss;
        if verbose || epoch % (epochs / 10).max(1) == 0 {
            eprintln!(
                "   epoch {:>4}/{} | pos_loss={:.4} neg_loss={:.4} combined={:.4} grad_norm={:.4}",
                epoch + 1,
                epochs,
                total_pos_loss,
                total_neg_loss,
                combined_loss,
                grad_norm,
            );
        }
    }

    // Verify: cat and mammal should be closer than cat and pine
    let cat = proj.forward(&hash_vecs["cat"]);
    let mammal = proj.forward(&hash_vecs["mammal"]);
    let pine = proj.forward(&hash_vecs["pine"]);
    let sim_cat_mammal: f64 = cat.iter().zip(mammal.iter()).map(|(a, b)| a * b).sum();
    let sim_cat_pine: f64 = cat.iter().zip(pine.iter()).map(|(a, b)| a * b).sum();
    eprintln!(
        "\n📊 Verification: sim(cat,mammal)={:.4}  sim(cat,pine)={:.4}  {}",
        sim_cat_mammal,
        sim_cat_pine,
        if sim_cat_mammal > sim_cat_pine {
            "✅ pulled similar closer"
        } else {
            "⚠ not improving yet"
        }
    );

    proj
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut epochs = 50;
    let mut verbose = false;
    let mut population_size = 12;
    let mut quick_mode = false;
    let mut train_embeddings = false;
    let mut embed_epochs = 500_usize;
    let mut embed_lr = 0.001_f64;

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
            "--quick" | "-q" => {
                quick_mode = true;
            }
            "--train-embeddings" => {
                train_embeddings = true;
            }
            "--embed-epochs" => {
                i += 1;
                embed_epochs = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(500);
            }
            "--embed-lr" => {
                i += 1;
                embed_lr = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(0.001);
            }
            "--help" | "-h" => {
                eprintln!("train-nca: Train NCA on word associations using CMA-ES");
                eprintln!("  --epochs <n>         Number of NCA training epochs (default: 50)");
                eprintln!("  --population <n>     CMA-ES population size (default: 12)");
                eprintln!("  --verbose/-v         Show per-epoch progress");
                eprintln!(
                    "  --quick/-q           Quick mode: tiny grid, 8 word-pairs, ~30 seconds"
                );
                eprintln!(
                    "  --train-embeddings   Also train linear projection W for hash→semantic"
                );
                eprintln!("  --embed-epochs <n>   Projection training epochs (default: 500)");
                eprintln!("  --embed-lr <f>       Projection learning rate (default: 0.001)");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // ── Embedding projection training (fast, runs standalone or combined) ───
    if train_embeddings {
        eprintln!("🔢 Training linear embedding projection W ∈ ℝ^{{64×64}}");
        eprintln!(
            "   Contrastive pairs: {} synonym pairs",
            SYNONYM_PAIRS.len()
        );
        eprintln!("   Epochs: {}   LR: {}", embed_epochs, embed_lr);
        eprintln!();

        let proj = train_embedding_projection(embed_epochs, embed_lr, verbose);

        let save_path = PROJECTION_WEIGHTS_PATH;
        match proj.save(save_path) {
            Ok(()) => {
                eprintln!("\n💾 Projection saved → {}", save_path);
            }
            Err(e) => {
                eprintln!("\n❌ Failed to save projection: {}", e);
                std::process::exit(1);
            }
        }

        if !args
            .iter()
            .any(|a| a.starts_with("--epochs") || a == "--quick")
        {
            // Only ran embedding training — done.
            return;
        }
    }

    // Quick mode overrides: 8×8 grid, max 30 epochs, minimal corpus
    let (corpus, epochs, grid_size, max_examples, weights_path) = if quick_mode {
        eprintln!("⚡ Quick mode: testing training pipeline with tiny grid");
        eprintln!("   This should complete in under 30 seconds.");
        eprintln!();
        (
            QUICK_CORPUS,
            epochs.min(30), // max 30 generations
            8,              // 8×8 grid (64 cells)
            16,             // few examples
            quick_weights_path(),
        )
    } else {
        (WORD_ASSOC_CORPUS, epochs, 8, 50, default_weights_path())
    };

    eprintln!("🧠 SAGE NCA Word-Association Trainer");
    if quick_mode {
        eprintln!("   Mode: QUICK (pipeline test)");
        eprintln!("   Corpus: 8 word-pairs (cat/dog/oak/pine/salmon/eagle/rust/python)");
    } else {
        eprintln!("   Corpus: built-in word associations (cat/dog/mammal, oak/pine/tree)");
    }
    eprintln!("   Optimizer: CMA-ES (separable diagonal)");
    eprintln!("   Epochs: {}", epochs);
    eprintln!(
        "   Grid size: {}×{} ({} cells)",
        grid_size,
        grid_size,
        grid_size * grid_size
    );
    eprintln!();

    let config = TrainingConfig {
        population_size,
        sigma: 0.3,
        learning_rate: 0.001,
        epochs,
        context_window: 3,
        grid_size,
        nca_steps: 5,
        max_examples,
        optimizer: Optimizer::CmaEs,
    };

    match train_nca(corpus, &config, verbose) {
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
            let path = weights_path;
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
