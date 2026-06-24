//! SAGE Distributed Knowledge Benchmarks
//!
//! Measures encoding speed, retrieval accuracy, diff sizes,
//! merge quality, grid capacity, and network simulation.

use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::grid::{ConsolidationParams, Grid};
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize, Clone, Debug)]
struct BenchmarkResults {
    encoding_speed: Vec<EncodingSpeedResult>,
    retrieval_accuracy: RetrievalAccuracyResult,
    diff_size: DiffSizeResult,
    merge_quality: MergeQualityResult,
    grid_capacity: GridCapacityResult,
    network_simulation: Vec<NetworkSimResult>,
    consolidation_params_comparison: ConsolidationParamsComparisonResult,
}

#[derive(Serialize, Clone, Debug)]
struct EncodingSpeedResult {
    item_count: usize,
    total_ms: f64,
    per_item_us: f64,
}

#[derive(Serialize, Clone, Debug)]
struct RetrievalAccuracyResult {
    total_facts: usize,
    total_queries: usize,
    hits: usize,
    precision: f64,
    recall: f64,
}

#[derive(Serialize, Clone, Debug)]
struct DiffSizeResult {
    items_encoded: usize,
    avg_changes_per_item: f64,
    avg_bytes_per_delta: f64,
}

#[derive(Serialize, Clone, Debug)]
struct MergeQualityResult {
    node_a_items: usize,
    node_b_items: usize,
    a_retrievable_after_merge: usize,
    b_retrievable_after_merge: usize,
    total_retrievable: usize,
    degradation_pct: f64,
    fill_levels: Vec<FillLevelResult>,
}

#[derive(Serialize, Clone, Debug)]
struct FillLevelResult {
    items_encoded: usize,
    retrievable: usize,
    quality_pct: f64,
}

#[derive(Serialize, Clone, Debug)]
struct GridCapacityResult {
    max_items_before_drop: usize,
    quality_at_max: f64,
    measurements: Vec<(usize, f64)>,
}

#[derive(Serialize, Clone, Debug)]
struct NetworkSimResult {
    node_count: usize,
    items_per_node: usize,
    total_unique_items: usize,
    avg_retrieval_quality: f64,
    total_storage_bytes: usize,
    gossip_rounds: usize,
}

#[derive(Serialize, Clone, Debug)]
struct ConsolidationParamsComparisonResult {
    /// Retrieval accuracy with default params
    default_params: RetrievalAccuracyResult,
    /// Retrieval accuracy with trained params
    trained_params: RetrievalAccuracyResult,
    /// Improvement percentage (positive = trained better)
    improvement_pct: f64,
    /// Whether trained params file was found
    trained_params_loaded: bool,
    /// The trained params used (if loaded)
    trained_params_values: Option<ConsolidationParamsJson>,
}

#[derive(Serialize, Clone, Debug)]
struct ConsolidationParamsJson {
    decay_rate: f64,
    strengthen_rate: f64,
    spread_rate: f64,
    confidence_boost: f64,
    activation_threshold: f64,
}

impl From<ConsolidationParams> for ConsolidationParamsJson {
    fn from(p: ConsolidationParams) -> Self {
        Self {
            decay_rate: p.decay_rate,
            strengthen_rate: p.strengthen_rate,
            spread_rate: p.spread_rate,
            confidence_boost: p.confidence_boost,
            activation_threshold: p.activation_threshold,
        }
    }
}

// --- Facts and queries for retrieval accuracy ---

fn get_facts() -> Vec<(&'static str, &'static str)> {
    // (fact, topic_tag)
    vec![
        (
            "The speed of light is approximately 299792458 meters per second",
            "physics",
        ),
        (
            "Water freezes at zero degrees celsius at standard pressure",
            "chemistry",
        ),
        ("The mitochondria is the powerhouse of the cell", "biology"),
        (
            "Rust programming language focuses on memory safety",
            "programming",
        ),
        (
            "Python was created by Guido van Rossum in 1991",
            "programming",
        ),
        (
            "The earth orbits the sun in approximately 365 days",
            "astronomy",
        ),
        (
            "DNA stores genetic information using four nucleotide bases",
            "biology",
        ),
        (
            "Machine learning uses statistical models to find patterns",
            "ai",
        ),
        (
            "TCP provides reliable ordered delivery of data packets",
            "networking",
        ),
        (
            "Photosynthesis converts light energy into chemical energy",
            "biology",
        ),
        ("The Fibonacci sequence starts with zero and one", "math"),
        (
            "Neural networks are inspired by biological brain neurons",
            "ai",
        ),
        (
            "HTTP is the protocol used for web communication",
            "networking",
        ),
        (
            "Gravity acceleration on earth is about 9.8 meters per second squared",
            "physics",
        ),
        (
            "The linux kernel was created by Linus Torvalds in 1991",
            "programming",
        ),
        (
            "Oxygen makes up about 21 percent of earths atmosphere",
            "chemistry",
        ),
        (
            "Binary search has logarithmic time complexity",
            "algorithms",
        ),
        (
            "The moon orbits earth approximately every 27 days",
            "astronomy",
        ),
        (
            "Quantum computers use qubits instead of classical bits",
            "computing",
        ),
        (
            "The human genome contains approximately 3 billion base pairs",
            "biology",
        ),
        (
            "SHA256 produces a 256 bit cryptographic hash",
            "cryptography",
        ),
        ("Mars is the fourth planet from the sun", "astronomy"),
        (
            "Transistors are the building blocks of modern processors",
            "computing",
        ),
        (
            "RNA acts as a messenger between DNA and protein synthesis",
            "biology",
        ),
        ("Graph databases store data as nodes and edges", "databases"),
        (
            "The speed of sound in air is approximately 343 meters per second",
            "physics",
        ),
        (
            "JavaScript was created in 10 days by Brendan Eich",
            "programming",
        ),
        (
            "Sodium chloride is the chemical name for table salt",
            "chemistry",
        ),
        ("Backpropagation is used to train neural networks", "ai"),
        (
            "The great wall of china is over 13000 miles long",
            "geography",
        ),
        ("Pi is approximately 3.14159265358979", "math"),
        (
            "Kubernetes orchestrates containerized applications",
            "devops",
        ),
        (
            "The amazon river is the largest river by volume",
            "geography",
        ),
        ("Euler's number e is approximately 2.71828", "math"),
        (
            "Git was created by Linus Torvalds for linux development",
            "programming",
        ),
        ("Helium is the second lightest element", "chemistry"),
        ("Redis is an in-memory key value data store", "databases"),
        (
            "The pacific ocean is the largest ocean on earth",
            "geography",
        ),
        ("Gradient descent optimizes neural network parameters", "ai"),
        (
            "Assembly language provides direct hardware control",
            "programming",
        ),
        (
            "Calcium is essential for bone formation in humans",
            "biology",
        ),
        (
            "Docker containers share the host operating system kernel",
            "devops",
        ),
        (
            "Venus is the hottest planet in our solar system",
            "astronomy",
        ),
        ("Merge sort has a time complexity of n log n", "algorithms"),
        (
            "The boiling point of water is 100 degrees celsius",
            "chemistry",
        ),
        (
            "Blockchain is a distributed immutable ledger technology",
            "cryptography",
        ),
        (
            "The sahara is the largest hot desert in the world",
            "geography",
        ),
        (
            "Transformers use self attention mechanisms for sequences",
            "ai",
        ),
        (
            "PostgreSQL is an advanced open source relational database",
            "databases",
        ),
        (
            "The human brain has approximately 86 billion neurons",
            "biology",
        ),
        // Additional facts to reach 100
        (
            "Iron is the most abundant element in earths core",
            "chemistry",
        ),
        (
            "Jupiter is the largest planet in our solar system",
            "astronomy",
        ),
        (
            "Dijkstra algorithm finds shortest paths in weighted graphs",
            "algorithms",
        ),
        ("The nile is the longest river in the world", "geography"),
        (
            "Convolutional neural networks excel at image recognition",
            "ai",
        ),
        ("MongoDB is a document oriented NoSQL database", "databases"),
        (
            "The human heart beats approximately 100000 times per day",
            "biology",
        ),
        (
            "Rust uses ownership and borrowing for memory management",
            "programming",
        ),
        (
            "The deepest point in the ocean is the mariana trench",
            "geography",
        ),
        (
            "Quick sort has average case n log n time complexity",
            "algorithms",
        ),
        (
            "Saturn has the most prominent ring system of any planet",
            "astronomy",
        ),
        ("Recurrent neural networks process sequential data", "ai"),
        (
            "TLS encrypts data transmitted over computer networks",
            "cryptography",
        ),
        (
            "The amazon rainforest produces 20 percent of world oxygen",
            "geography",
        ),
        (
            "Haskell is a purely functional programming language",
            "programming",
        ),
        ("The periodic table has 118 confirmed elements", "chemistry"),
        (
            "B trees are commonly used in database indexing",
            "databases",
        ),
        (
            "Neptune is the farthest known planet from the sun",
            "astronomy",
        ),
        ("Reinforcement learning uses rewards to train agents", "ai"),
        (
            "The great barrier reef is the largest coral reef system",
            "geography",
        ),
        (
            "Go programming language was designed at Google",
            "programming",
        ),
        (
            "Carbon dioxide is a greenhouse gas in earths atmosphere",
            "chemistry",
        ),
        (
            "Hash tables provide average constant time lookup",
            "algorithms",
        ),
        (
            "Mercury is the smallest planet in our solar system",
            "astronomy",
        ),
        (
            "Generative adversarial networks use two competing networks",
            "ai",
        ),
        (
            "Mount everest is the tallest mountain above sea level",
            "geography",
        ),
        ("Elixir runs on the erlang virtual machine", "programming"),
        (
            "Hydrogen is the most abundant element in the universe",
            "chemistry",
        ),
        (
            "Cassandra is a distributed wide column NoSQL database",
            "databases",
        ),
        (
            "The andromeda galaxy is the nearest large galaxy to us",
            "astronomy",
        ),
        (
            "Natural language processing enables computers to understand text",
            "ai",
        ),
        (
            "The dead sea is the lowest point on earths surface",
            "geography",
        ),
        (
            "C programming language was created by Dennis Ritchie",
            "programming",
        ),
        (
            "Ozone in the stratosphere protects earth from UV radiation",
            "chemistry",
        ),
        (
            "Depth first search explores as far as possible along branches",
            "algorithms",
        ),
        (
            "Uranus rotates on its side with an axial tilt of 98 degrees",
            "astronomy",
        ),
        (
            "Attention mechanisms let models focus on relevant input parts",
            "ai",
        ),
        (
            "Lake baikal is the deepest freshwater lake in the world",
            "geography",
        ),
        ("TypeScript adds static types to JavaScript", "programming"),
        (
            "Noble gases are chemically inert and rarely form compounds",
            "chemistry",
        ),
        (
            "A star search uses heuristics to find optimal paths",
            "algorithms",
        ),
        (
            "Pluto was reclassified as a dwarf planet in 2006",
            "astronomy",
        ),
        (
            "Word embeddings represent words as dense numeric vectors",
            "ai",
        ),
        (
            "The sahel is a semiarid region south of the sahara",
            "geography",
        ),
        (
            "Scala combines object oriented and functional programming",
            "programming",
        ),
        ("Acids have a pH less than 7 on the pH scale", "chemistry"),
        (
            "Breadth first search explores all neighbors at current depth first",
            "algorithms",
        ),
        (
            "Proxima centauri is the closest star to our sun",
            "astronomy",
        ),
        (
            "Batch normalization stabilizes and accelerates neural network training",
            "ai",
        ),
        (
            "Antarctica contains about 70 percent of earths fresh water",
            "geography",
        ),
        (
            "Swift was created by Apple for iOS development",
            "programming",
        ),
    ]
}

fn get_queries() -> Vec<(&'static str, &'static str)> {
    // (query, expected_topic)
    vec![
        ("how fast does light travel", "physics"),
        ("at what temperature does water freeze", "chemistry"),
        ("what generates energy in cells", "biology"),
        ("which language focuses on memory safety", "programming"),
        ("who created python programming language", "programming"),
        ("how long does earth take to orbit sun", "astronomy"),
        ("how does DNA store information", "biology"),
        ("what does machine learning do", "ai"),
        ("how does TCP deliver data", "networking"),
        ("how do plants convert sunlight", "biology"),
        ("what are the first fibonacci numbers", "math"),
        ("what inspires neural network design", "ai"),
        ("what protocol do websites use", "networking"),
        ("how fast do objects fall on earth", "physics"),
        ("who made the linux kernel", "programming"),
        ("what percentage of air is oxygen", "chemistry"),
        ("what is binary search complexity", "algorithms"),
        ("how long does the moon take to orbit", "astronomy"),
        ("what do quantum computers use", "computing"),
        ("how many base pairs in human genome", "biology"),
    ]
}

// --- Benchmark functions ---

fn bench_encoding_speed() -> Vec<EncodingSpeedResult> {
    let mut results = Vec::new();
    for &count in &[100, 1000, 10000] {
        let mut store = NCAKnowledge::new();
        // Disable ollama for benchmarks (use hash-based encoding)
        store.config.ollama_url = None;

        let start = Instant::now();
        for i in 0..count {
            let text = format!("knowledge item number {} about topic {}", i, i % 50);
            store.encode(&text, 0.8);
        }
        let elapsed = start.elapsed();
        let total_ms = elapsed.as_secs_f64() * 1000.0;
        results.push(EncodingSpeedResult {
            item_count: count,
            total_ms,
            per_item_us: (total_ms * 1000.0) / count as f64,
        });
    }
    results
}

fn bench_retrieval_accuracy() -> RetrievalAccuracyResult {
    let facts = get_facts();
    let queries = get_queries();

    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;

    // Build a map: topic -> list of fact indices
    let mut topic_facts: std::collections::HashMap<&str, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, &(fact, topic)) in facts.iter().enumerate() {
        store.encode(fact, 0.9);
        topic_facts.entry(topic).or_default().push(i);
    }

    let mut hits = 0;
    let total_queries = queries.len();

    for &(query, expected_topic) in &queries {
        let results = store.query(query, 5);
        // A hit: any of the top-5 results has text matching a fact with the expected topic
        let expected_facts: Vec<&str> = topic_facts
            .get(expected_topic)
            .map(|indices| indices.iter().map(|&i| facts[i].0).collect())
            .unwrap_or_default();

        let got_hit = results.iter().any(|r| {
            if let Some(ref text) = r.text {
                expected_facts.iter().any(|f| text == f)
            } else {
                false
            }
        });

        if got_hit {
            hits += 1;
        }
    }

    let precision = hits as f64 / total_queries as f64;
    let recall = precision; // In this setup, precision == recall (one expected per query)

    RetrievalAccuracyResult {
        total_facts: facts.len(),
        total_queries,
        hits,
        precision,
        recall,
    }
}

fn bench_diff_size() -> DiffSizeResult {
    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;
    let _empty = Grid::new(sage::grid::GRID_SIZE, sage::grid::GRID_SIZE);

    let mut total_changes = 0usize;
    let mut total_bytes = 0usize;
    let items = 100;

    for i in 0..items {
        let before_grid = store.grid.clone();
        store.encode(&format!("diff test item {}", i), 0.8);
        let delta = store.diff(&before_grid);
        total_changes += delta.changes.len();
        let serialized = bincode::serialize(&delta).unwrap_or_default();
        total_bytes += serialized.len();
    }

    DiffSizeResult {
        items_encoded: items,
        avg_changes_per_item: total_changes as f64 / items as f64,
        avg_bytes_per_delta: total_bytes as f64 / items as f64,
    }
}

fn bench_merge_quality() -> MergeQualityResult {
    let items_per_node = 50;

    let mut store_a = NCAKnowledge::new().with_node_id(1.0);
    store_a.config.ollama_url = None;
    let mut store_b = NCAKnowledge::new().with_node_id(2.0);
    store_b.config.ollama_url = None;

    let mut a_facts = Vec::new();
    let mut b_facts = Vec::new();

    for i in 0..items_per_node {
        let fact_a = format!("alpha knowledge item {}", i);
        let fact_b = format!("beta knowledge item {}", i);
        store_a.encode(&fact_a, 0.9);
        store_b.encode(&fact_b, 0.9);
        a_facts.push(fact_a);
        b_facts.push(fact_b);
    }

    // Merge B into A (with text store!)
    store_a.merge_with_text(&store_b, 0.8);

    // Check retrievability
    let a_retrievable = a_facts
        .iter()
        .filter(|f| {
            let r = store_a.query(f, 3);
            r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
        })
        .count();

    let b_retrievable = b_facts
        .iter()
        .filter(|f| {
            let r = store_a.query(f, 3);
            r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
        })
        .count();

    // Measure degradation at different fill levels
    let mut fill_levels = Vec::new();
    for &fill in &[50, 100, 200, 500] {
        let mut s = NCAKnowledge::new();
        s.config.ollama_url = None;
        let mut encoded_facts = Vec::new();
        for i in 0..fill {
            let f = format!("fill test item number {}", i);
            s.encode(&f, 0.8);
            encoded_facts.push(f);
        }
        // Sample up to 50 items for quality check
        let sample_size = fill.min(50);
        let retrievable = encoded_facts
            .iter()
            .take(sample_size)
            .filter(|f| {
                let r = s.query(f, 3);
                r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
            })
            .count();
        fill_levels.push(FillLevelResult {
            items_encoded: fill,
            retrievable,
            quality_pct: (retrievable as f64 / sample_size as f64) * 100.0,
        });
    }

    let total = a_retrievable + b_retrievable;
    let ideal = items_per_node * 2;
    let degradation = ((ideal - total) as f64 / ideal as f64) * 100.0;

    MergeQualityResult {
        node_a_items: items_per_node,
        node_b_items: items_per_node,
        a_retrievable_after_merge: a_retrievable,
        b_retrievable_after_merge: b_retrievable,
        total_retrievable: total,
        degradation_pct: degradation,
        fill_levels,
    }
}

fn bench_grid_capacity() -> GridCapacityResult {
    let mut measurements = Vec::new();
    let mut max_items = 0;
    let mut quality_at_max = 0.0;

    for &count in &[10, 25, 50, 100, 200, 500, 1000, 2000] {
        let mut store = NCAKnowledge::new();
        store.config.ollama_url = None;
        let mut facts = Vec::new();
        for i in 0..count {
            let f = format!("capacity test distinct item {}", i);
            store.encode(&f, 0.8);
            facts.push(f);
        }
        let sample = count.min(50);
        let found = facts
            .iter()
            .take(sample)
            .filter(|f| {
                let r = store.query(f, 3);
                r.iter().any(|res| res.text.as_deref() == Some(f.as_str()))
            })
            .count();
        let quality = found as f64 / sample as f64;
        measurements.push((count, quality));

        if quality >= 0.5 {
            max_items = count;
            quality_at_max = quality;
        }
    }

    GridCapacityResult {
        max_items_before_drop: max_items,
        quality_at_max,
        measurements,
    }
}

fn bench_network_simulation() -> Vec<NetworkSimResult> {
    let mut results = Vec::new();

    for &node_count in &[10, 50, 100] {
        let items_per_node = 10;
        let mut nodes: Vec<NCAKnowledge> = (0..node_count)
            .map(|i| {
                let mut n = NCAKnowledge::new().with_node_id(i as f64);
                n.config.ollama_url = None;
                n
            })
            .collect();

        // Each node encodes unique knowledge
        let mut all_facts: Vec<Vec<String>> = Vec::new();
        for (i, node) in nodes.iter_mut().enumerate() {
            let mut node_facts = Vec::new();
            for j in 0..items_per_node {
                let fact = format!("node {} knowledge item {}", i, j);
                node.encode(&fact, 0.8);
                node_facts.push(fact);
            }
            all_facts.push(node_facts);
        }

        // Gossip sync: each node merges with every other (simplified full sync)
        let mut gossip_rounds = 0;
        // Do log2(n) rounds of pairwise merging
        let rounds = ((node_count as f64).log2().ceil() as usize).max(1);
        for _ in 0..rounds {
            gossip_rounds += 1;
            let grids: Vec<Grid> = nodes.iter().map(|n| n.grid.clone()).collect();
            for (i, node) in nodes.iter_mut().enumerate() {
                for (j, grid) in grids.iter().enumerate() {
                    if i != j {
                        node.merge(grid, 0.8);
                    }
                }
            }
        }

        // Measure: pick node 0 and check how much total knowledge it has
        let total_unique = node_count * items_per_node;

        // Sample retrieval quality on node 0
        let sample_size = total_unique.min(100);
        let mut found = 0;
        let mut checked = 0;
        'outer: for node_facts in &all_facts {
            for fact in node_facts {
                if checked >= sample_size {
                    break 'outer;
                }
                let r = nodes[0].query(fact, 5);
                if !r.is_empty() {
                    found += 1;
                }
                checked += 1;
            }
        }

        let storage_bytes = bincode::serialize(&nodes[0].grid)
            .map(|b| b.len())
            .unwrap_or(0);

        results.push(NetworkSimResult {
            node_count,
            items_per_node,
            total_unique_items: total_unique,
            avg_retrieval_quality: found as f64 / checked.max(1) as f64,
            total_storage_bytes: storage_bytes,
            gossip_rounds,
        });
    }

    results
}

/// Compare retrieval accuracy with default vs trained consolidation params.
/// This validates the ml-engineer hypothesis that ES-trained params improve retrieval.
fn bench_consolidation_params_comparison() -> ConsolidationParamsComparisonResult {
    let facts = get_facts();
    let queries = get_queries();

    // Build topic map once
    let mut topic_facts: std::collections::HashMap<&str, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, &(_fact, topic)) in facts.iter().enumerate() {
        topic_facts.entry(topic).or_default().push(i);
    }

    // --- Test with default params ---
    let mut store_default = NCAKnowledge::new();
    store_default.config.ollama_url = None;
    
    for &(fact, _topic) in &facts {
        store_default.encode(fact, 0.9);
    }
    
    // Apply default consolidation (2 steps)
    store_default.grid.consolidate_knowledge(2);
    
    let default_result = evaluate_retrieval(&store_default, &queries, &topic_facts, &facts);

    // --- Test with trained params ---
    let trained_params = ConsolidationParams::load_or_default();
    let trained_params_loaded = dirs::home_dir()
        .map(|h| h.join(".sage").join("consolidation_params.json").exists())
        .unwrap_or(false);
    
    let mut store_trained = NCAKnowledge::new();
    store_trained.config.ollama_url = None;
    
    for &(fact, _topic) in &facts {
        store_trained.encode(fact, 0.9);
    }
    
    // Apply consolidation with trained params
    store_trained.grid.consolidate_knowledge_with_params(2, &trained_params);
    
    let trained_result = evaluate_retrieval(&store_trained, &queries, &topic_facts, &facts);
    
    // Calculate improvement
    let improvement = if default_result.precision > 0.0 {
        ((trained_result.precision - default_result.precision) / default_result.precision) * 100.0
    } else {
        0.0
    };

    ConsolidationParamsComparisonResult {
        default_params: default_result,
        trained_params: trained_result,
        improvement_pct: improvement,
        trained_params_loaded,
        trained_params_values: if trained_params_loaded {
            Some(ConsolidationParamsJson::from(trained_params))
        } else {
            None
        },
    }
}

fn evaluate_retrieval(
    store: &NCAKnowledge,
    queries: &[(&'static str, &'static str)],
    topic_facts: &std::collections::HashMap<&str, Vec<usize>>,
    facts: &[(&'static str, &'static str)],
) -> RetrievalAccuracyResult {
    let mut hits = 0;
    let total_queries = queries.len();

    for &(query, expected_topic) in queries {
        let results = store.query(query, 5);
        let expected_facts: Vec<&str> = topic_facts
            .get(expected_topic)
            .map(|indices| indices.iter().map(|&i| facts[i].0).collect())
            .unwrap_or_default();

        let got_hit = results.iter().any(|r| {
            if let Some(ref text) = r.text {
                expected_facts.iter().any(|f| text == f)
            } else {
                false
            }
        });

        if got_hit {
            hits += 1;
        }
    }

    let precision = hits as f64 / total_queries as f64;
    RetrievalAccuracyResult {
        total_facts: facts.len(),
        total_queries,
        hits,
        precision,
        recall: precision,
    }
}

fn print_table(results: &BenchmarkResults) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║              SAGE Distributed Knowledge Benchmarks             ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    // Encoding speed
    println!("║                                                                ║");
    println!("║  📝 Encoding Speed                                             ║");
    println!("╟──────────────┬──────────────┬──────────────────────────────────╢");
    println!("║  Items       │  Total (ms)  │  Per Item (µs)                   ║");
    println!("╟──────────────┼──────────────┼──────────────────────────────────╢");
    for r in &results.encoding_speed {
        println!(
            "║  {:>10}  │  {:>10.1} │  {:>10.1}                         ║",
            r.item_count, r.total_ms, r.per_item_us
        );
    }

    // Retrieval accuracy
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  🔍 Retrieval Accuracy                                         ║");
    println!(
        "║  Facts: {:>4}  Queries: {:>4}  Hits: {:>4}                       ║",
        results.retrieval_accuracy.total_facts,
        results.retrieval_accuracy.total_queries,
        results.retrieval_accuracy.hits
    );
    println!(
        "║  Precision: {:.1}%  Recall: {:.1}%                                ║",
        results.retrieval_accuracy.precision * 100.0,
        results.retrieval_accuracy.recall * 100.0
    );

    // Diff size
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  📦 Diff Size (per knowledge item)                             ║");
    println!(
        "║  Avg changes: {:.1}  Avg bytes: {:.0}                            ║",
        results.diff_size.avg_changes_per_item, results.diff_size.avg_bytes_per_delta
    );

    // Merge quality
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  🔀 Merge Quality                                              ║");
    println!(
        "║  Node A retrievable: {:>3}/{}  Node B retrievable: {:>3}/{}       ║",
        results.merge_quality.a_retrievable_after_merge,
        results.merge_quality.node_a_items,
        results.merge_quality.b_retrievable_after_merge,
        results.merge_quality.node_b_items
    );
    println!(
        "║  Degradation: {:.1}%                                             ║",
        results.merge_quality.degradation_pct
    );
    println!("║  Fill levels:                                                  ║");
    for fl in &results.merge_quality.fill_levels {
        println!(
            "║    {:>5} items → {:.1}% retrievable                            ║",
            fl.items_encoded, fl.quality_pct
        );
    }

    // Grid capacity
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  📊 Grid Capacity                                              ║");
    println!(
        "║  Max items (≥50% quality): {}  Quality: {:.1}%                  ║",
        results.grid_capacity.max_items_before_drop,
        results.grid_capacity.quality_at_max * 100.0
    );
    for (count, quality) in &results.grid_capacity.measurements {
        println!(
            "║    {:>5} items → {:.1}% quality                                ║",
            count,
            quality * 100.0
        );
    }

    // Network simulation
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  🌐 Network Simulation                                         ║");
    println!("╟──────────┬───────┬──────────┬──────────┬────────────────────────╢");
    println!("║  Nodes   │ Items │ Quality  │ Storage  │ Gossip Rounds          ║");
    println!("╟──────────┼───────┼──────────┼──────────┼────────────────────────╢");
    for r in &results.network_simulation {
        println!(
            "║  {:>6}  │ {:>5} │  {:.1}%   │ {:>6}B │ {:>5}                  ║",
            r.node_count,
            r.total_unique_items,
            r.avg_retrieval_quality * 100.0,
            r.total_storage_bytes,
            r.gossip_rounds
        );
    }

    // Consolidation params comparison
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  🎯 Consolidation Params Comparison                             ║");
    println!("║  Tests whether ES-trained params improve retrieval accuracy     ║");
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!(
        "║  Trained params loaded: {}                                     ║",
        if results.consolidation_params_comparison.trained_params_loaded {
            "YES"
        } else {
            "NO (using defaults)"
        }
    );
    
    if let Some(ref params) = results.consolidation_params_comparison.trained_params_values {
        println!(
            "║  decay={:.4} strengthen={:.4} spread={:.4}       ║",
            params.decay_rate, params.strengthen_rate, params.spread_rate
        );
        println!(
            "║  conf_boost={:.4} activation_thresh={:.4}                  ║",
            params.confidence_boost, params.activation_threshold
        );
    }
    
    println!("╟──────────────────────────────────────────────────────────────────╢");
    println!("║  Default params:                                                ║");
    println!(
        "║    Hits: {:>3}/{:>3}  Precision: {:.1}%                              ║",
        results.consolidation_params_comparison.default_params.hits,
        results.consolidation_params_comparison.default_params.total_queries,
        results.consolidation_params_comparison.default_params.precision * 100.0
    );
    println!("║  Trained params:                                                ║");
    println!(
        "║    Hits: {:>3}/{:>3}  Precision: {:.1}%                              ║",
        results.consolidation_params_comparison.trained_params.hits,
        results.consolidation_params_comparison.trained_params.total_queries,
        results.consolidation_params_comparison.trained_params.precision * 100.0
    );
    println!("╟──────────────────────────────────────────────────────────────────╢");
    
    let improvement_sign = if results.consolidation_params_comparison.improvement_pct >= 0.0 {
        "+"
    } else {
        ""
    };
    println!(
        "║  Improvement: {}{:.2}%                                            ║",
        improvement_sign,
        results.consolidation_params_comparison.improvement_pct
    );
    
    if results.consolidation_params_comparison.improvement_pct > 5.0 {
        println!("║  ✅ Trained params significantly better!                        ║");
    } else if results.consolidation_params_comparison.improvement_pct > 0.0 {
        println!("║  ✓ Trained params slightly better                               ║");
    } else if results.consolidation_params_comparison.improvement_pct > -5.0 {
        println!("║  ≈ No significant difference                                     ║");
    } else {
        println!("║  ⚠️  Default params performed better                             ║");
    }

    println!("╚══════════════════════════════════════════════════════════════════╝");
}

fn main() {
    println!("🧠 SAGE Distributed Knowledge Benchmarks");
    println!("=========================================\n");

    println!("Running encoding speed benchmark...");
    let encoding_speed = bench_encoding_speed();
    println!("  ✓ Done");

    println!("Running retrieval accuracy benchmark...");
    let retrieval_accuracy = bench_retrieval_accuracy();
    println!("  ✓ Done");

    println!("Running diff size benchmark...");
    let diff_size = bench_diff_size();
    println!("  ✓ Done");

    println!("Running merge quality benchmark...");
    let merge_quality = bench_merge_quality();
    println!("  ✓ Done");

    println!("Running grid capacity benchmark...");
    let grid_capacity = bench_grid_capacity();
    println!("  ✓ Done");

    println!("Running network simulation benchmark...");
    let network_simulation = bench_network_simulation();
    println!("  ✓ Done");

    println!("Running consolidation params comparison benchmark...");
    let consolidation_params_comparison = bench_consolidation_params_comparison();
    println!("  ✓ Done");

    let results = BenchmarkResults {
        encoding_speed,
        retrieval_accuracy,
        diff_size,
        merge_quality,
        grid_capacity,
        network_simulation,
        consolidation_params_comparison,
    };

    // Print pretty table
    print_table(&results);

    // Output JSON
    let json = serde_json::to_string_pretty(&results).unwrap();
    println!("\n📄 JSON Results:\n{}", json);

    // Save JSON to docs/
    let _ = std::fs::create_dir_all("docs");
    std::fs::write("docs/benchmark_results.json", &json).expect("Failed to write JSON");
    println!("\n✅ Results saved to docs/benchmark_results.json");
}
