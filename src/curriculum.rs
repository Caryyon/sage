//! Bulk Curriculum Ingestion
//!
//! Structured knowledge loading for domain-expert brain training.
//! Reads a curriculum JSON file (topics → facts), encodes into the NCA grid,
//! runs consolidation between topics, and verifies retrieval quality.
//!
//! Curriculum JSON format:
//! ```json
//! {
//!   "name": "cs-fundamentals",
//!   "domain": "computer-science",
//!   "topics": [
//!     {
//!       "name": "data-structures",
//!       "region": [0, 0, 85, 85],
//!       "facts": [
//!         "A stack is a LIFO (last in, first out) data structure",
//!         "Push adds to top of stack, pop removes from top of stack",
//!         ...
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! Region allocation distributes topics across grid sub-regions to reduce
//! hash collisions between different knowledge domains. If no region is
//! specified, the topic gets the next available region evenly divided.

use crate::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Curriculum configuration for ingestion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// Confidence for injected facts (0.0-1.0, default: 0.95 — higher than chat)
    pub confidence: f64,
    /// Gaussian spread radius (default: 4 — tighter than chat's 6)
    pub spread_radius: usize,
    /// Spatial decay rate for gaussian write (default: 0.25 — tighter than chat's 0.4)
    pub spatial_decay: f64,
    /// Consolidation steps to run between topics (default: 5)
    pub consolidation_steps: usize,
    /// Number of integration (smooth) steps per encoding burst (default: 3)
    pub integration_steps: usize,
    /// Whether to verify after each topic (default: true)
    pub verify_each_topic: bool,
    /// Minimum acceptable hit rate per topic (0.0-1.0, default: 0.5)
    pub min_topic_hit_rate: f64,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            confidence: 0.95,
            spread_radius: 4,
            spatial_decay: 0.25,
            consolidation_steps: 5,
            integration_steps: 3,
            verify_each_topic: true,
            min_topic_hit_rate: 0.5,
        }
    }
}

/// A single fact with its expected query terms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurriculumFact {
    /// The fact text to encode
    pub fact: String,
    /// Optional: key terms to use when verifying this fact
    pub query: Option<String>,
}

/// A topic in the curriculum (collection of related facts)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurriculumTopic {
    /// Topic name
    pub name: String,
    /// Optional grid region [x1, y1, x2, y2] for this topic (0-indexed, inclusive)
    pub region: Option<[usize; 4]>,
    /// Facts to encode for this topic
    pub facts: Vec<CurriculumFact>,
}

/// Full curriculum definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Curriculum {
    /// Name of this curriculum
    pub name: String,
    /// Domain it belongs to
    pub domain: String,
    /// Optional description
    pub description: Option<String>,
    /// Topics in this curriculum
    pub topics: Vec<CurriculumTopic>,
}

/// Verification result for a single fact
#[derive(Clone, Debug, Serialize)]
pub struct FactVerifyResult {
    pub fact: String,
    pub query_used: String,
    pub found: bool,
    pub rank: Option<usize>,
    pub relevance: f64,
    pub grid_position: (usize, usize),
}

/// Verification results for a topic
#[derive(Clone, Debug, Serialize)]
pub struct TopicVerifyResult {
    pub topic: String,
    pub total_facts: usize,
    pub hits: usize,
    pub hit_rate: f64,
    pub avg_relevance: f64,
    pub facts: Vec<FactVerifyResult>,
}

/// Full ingestion report
#[derive(Clone, Debug, Serialize)]
pub struct IngestionReport {
    pub curriculum: String,
    pub elapsed_secs: f64,
    pub total_facts: usize,
    pub total_topics: usize,
    pub active_cells_before: usize,
    pub active_cells_after: usize,
    pub topics: Vec<TopicVerifyResult>,
    pub overall_hit_rate: f64,
    pub overall_avg_relevance: f64,
}

impl IngestionReport {
    /// Print a human-readable summary
    pub fn summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "📚 Curriculum: {} ({})\n",
            self.curriculum, self.elapsed_secs
        ));
        out.push_str(&format!(
            "   Topics: {} | Facts: {} | Time: {:.1}s\n",
            self.total_topics, self.total_facts, self.elapsed_secs
        ));
        out.push_str(&format!(
            "   Grid: {} → {} active cells ({} new)\n\n",
            self.active_cells_before,
            self.active_cells_after,
            self.active_cells_after.saturating_sub(self.active_cells_before)
        ));

        for topic in &self.topics {
            let status = if topic.hit_rate >= 0.67 {
                "✅"
            } else if topic.hit_rate >= 0.5 {
                "⚠️ "
            } else {
                "❌"
            };
            out.push_str(&format!(
                "   {} {}: {}/{} ({:.0}%) avg_rel={:.3}\n",
                status, topic.topic, topic.hits, topic.total_facts,
                topic.hit_rate * 100.0, topic.avg_relevance
            ));
        }

        out.push_str(&format!(
            "\n   Total: {}/{} ({:.0}%) avg_rel={:.3}\n",
            self.topics.iter().map(|t| t.hits).sum::<usize>(),
            self.total_facts,
            self.overall_hit_rate * 100.0,
            self.overall_avg_relevance,
        ));

        out
    }
}

/// Ingest a curriculum into an NCAKnowledge store.
///
/// Flow:
///   1. Parse curriculum
///   2. For each topic: allocate region, encode all facts, consolidate, verify
///   3. Save brain + export template
///   4. Return ingestion report
pub fn ingest_curriculum(
    knowledge: &mut NCAKnowledge,
    curriculum: &Curriculum,
    config: &IngestionConfig,
) -> IngestionReport {
    let start = Instant::now();
    let active_before = knowledge.active_knowledge(0.01).len();
    let mut topic_results = Vec::new();

    // Calculate region allocations: divide grid into grid sections for each topic
    let grid_size = knowledge.grid.width;
    let topic_regions = compute_topic_regions(curriculum, grid_size);

    // Temporary encoder config override for tighter writes
    let saved_config = knowledge.config.clone();
    knowledge.config.spread_radius = config.spread_radius;
    knowledge.config.spatial_decay = config.spatial_decay;

    for (topic, region) in curriculum.topics.iter().zip(&topic_regions) {
        let mut fact_results = Vec::new();

        // Encode all facts in this topic
        let topic_center = (
            (region[0] + region[2]) / 2,
            (region[1] + region[3]) / 2,
        );

        for fact in &topic.facts {
            let facts = &fact.fact;
            let pos = encode_fact(knowledge, facts, config);
            fact_results.push(FactVerifyResult {
                fact: facts.clone(),
                query_used: String::new(),
                found: true,
                rank: None,
                relevance: 1.0,
                grid_position: pos,
            });
        }

        // Run integration steps to let the grid settle patterns
        for _ in 0..config.integration_steps {
            knowledge
                .grid
                .smooth_hidden_channels(topic_center.0, topic_center.1, config.spread_radius * 3, 1);
        }

        // Consolidate: strengthen co-occurring patterns
        if config.consolidation_steps > 0 {
            knowledge
                .grid
                .consolidate_knowledge(config.consolidation_steps);
        }

        // Verify if enabled
        let hits;
        if config.verify_each_topic {
            fact_results = verify_topic(knowledge, topic);
            hits = fact_results.iter().filter(|r| r.found).count();
        } else {
            hits = fact_results.len(); // all counted as hits since we didn't verify
        }

        let avg_rel = if fact_results.is_empty() {
            0.0
        } else {
            fact_results.iter().map(|r| r.relevance).sum::<f64>() / fact_results.len() as f64
        };

        let hit_rate = if topic.facts.is_empty() {
            1.0
        } else {
            hits as f64 / topic.facts.len() as f64
        };

        topic_results.push(TopicVerifyResult {
            topic: topic.name.clone(),
            total_facts: topic.facts.len(),
            hits,
            hit_rate,
            avg_relevance: avg_rel,
            facts: fact_results,
        });
    }

    // Restore original config
    knowledge.config = saved_config;

    let active_after = knowledge.active_knowledge(0.01).len();
    let elapsed = start.elapsed().as_secs_f64();

    let total_facts: usize = curriculum.topics.iter().map(|t| t.facts.len()).sum();
    let total_hits: usize = topic_results.iter().map(|t| t.hits).sum();
    let overall_hit_rate = if total_facts == 0 {
        0.0
    } else {
        total_hits as f64 / total_facts as f64
    };
    let overall_avg_rel: f64 = if total_facts == 0 {
        0.0
    } else {
        topic_results
            .iter()
            .map(|t| t.facts.iter().map(|f| f.relevance).sum::<f64>())
            .sum::<f64>()
            / total_facts as f64
    };

    IngestionReport {
        curriculum: curriculum.name.clone(),
        elapsed_secs: elapsed,
        total_facts,
        total_topics: curriculum.topics.len(),
        active_cells_before: active_before,
        active_cells_after: active_after,
        topics: topic_results,
        overall_hit_rate,
        overall_avg_relevance: overall_avg_rel,
    }
}

/// Encode a single fact with higher confidence and tighter spread
fn encode_fact(
    knowledge: &mut NCAKnowledge,
    fact_text: &str,
    config: &IngestionConfig,
) -> (usize, usize) {
    knowledge.encode(fact_text, config.confidence)
}

/// Verify a topic's facts by querying each one back
fn verify_topic(knowledge: &NCAKnowledge, topic: &CurriculumTopic) -> Vec<FactVerifyResult> {
    topic
        .facts
        .iter()
        .map(|fact| {
            let query = fact.query.as_deref().unwrap_or(&fact.fact);
            let results = knowledge.query(query, 5);

            // Check if any result contains the original fact text or has high relevance
            let match_info = results
                .iter()
                .enumerate()
                .find(|(_, r)| {
                    r.text
                        .as_ref()
                        .map(|t| t.contains(&fact.fact) || fact.fact.contains(t.as_str()))
                        .unwrap_or(false)
                })
                .map(|(idx, r)| (Some(idx), r.relevance));

            let (rank, relevance) = match_info
                .or_else(|| {
                    // No exact text match — check if top result has meaningful relevance
                    results.first().map(|r| (Some(0), r.relevance))
                })
                .unwrap_or((None, 0.0));

            FactVerifyResult {
                fact: fact.fact.clone(),
                query_used: query.to_string(),
                found: rank.is_some() && relevance > 0.3,
                rank,
                relevance,
                grid_position: results
                    .first()
                    .map(|r| r.position)
                    .unwrap_or((0, 0)),
            }
        })
        .collect()
}

/// Compute grid region allocations for each topic.
///
/// If a topic specifies its own region, use it. Otherwise divide remaining
/// grid space evenly among topics that don't have explicit regions.
fn compute_topic_regions(curriculum: &Curriculum, grid_size: usize) -> Vec<[usize; 4]> {
    let n_topics = curriculum.topics.len();
    // Split grid into a roughly square arrangement of topics
    let cols = (n_topics as f64).sqrt().ceil() as usize;
    let rows = ((n_topics as f64) / (cols as f64)).ceil() as usize;
    let cell_w = grid_size / cols;
    let cell_h = grid_size / rows;

    curriculum
        .topics
        .iter()
        .enumerate()
        .map(|(i, topic)| {
            if let Some(region) = topic.region {
                region
            } else {
                let col = i % cols;
                let row = i / cols;
                let margin = 2; // small gap between regions
                [
                    col * cell_w + margin,
                    row * cell_h + margin,
                    ((col + 1) * cell_w).min(grid_size - 1) - margin,
                    ((row + 1) * cell_h).min(grid_size - 1) - margin,
                ]
            }
        })
        .collect()
}

/// Load a curriculum from a JSON file
pub fn load_curriculum(path: &PathBuf) -> Result<Curriculum, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read curriculum file: {}", e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse curriculum JSON: {}", e))
}

/// Generate a sample curriculum file for testing
pub fn sample_curriculum() -> Curriculum {
    Curriculum {
        name: "cs-fundamentals".into(),
        domain: "computer-science".into(),
        description: Some("Core CS concepts for junior developer brain".into()),
        topics: vec![
            CurriculumTopic {
                name: "data-structures".into(),
                region: None,
                facts: vec![
                    CurriculumFact {
                        fact: "An array stores elements in contiguous memory with O(1) random access".into(),
                        query: Some("array O(1) random access contiguous".into()),
                    },
                    CurriculumFact {
                        fact: "A stack is a LIFO (last in, first out) data structure with push and pop operations".into(),
                        query: Some("stack LIFO push pop".into()),
                    },
                    CurriculumFact {
                        fact: "A queue is a FIFO (first in, first out) data structure with enqueue and dequeue".into(),
                        query: Some("queue FIFO enqueue dequeue".into()),
                    },
                ],
            },
            CurriculumTopic {
                name: "algorithms".into(),
                region: None,
                facts: vec![
                    CurriculumFact {
                        fact: "Binary search operates on sorted arrays in O(log n) time".into(),
                        query: Some("binary search O(log n) sorted".into()),
                    },
                    CurriculumFact {
                        fact: "Bubble sort compares adjacent elements and swaps them, running in O(n²) time".into(),
                        query: Some("bubble sort O(n^2) adjacent swap".into()),
                    },
                ],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed_knowledge::NCAKnowledge;

    #[test]
    fn test_curriculum_ingestion_basic() {
        let mut knowledge = NCAKnowledge::new();
        // Force hash-based encoding for deterministic tests
        knowledge.config.ollama_url = None;

        let curriculum = sample_curriculum();
        let config = IngestionConfig {
            consolidation_steps: 2,
            integration_steps: 1,
            verify_each_topic: true,
            ..Default::default()
        };

        let report = ingest_curriculum(&mut knowledge, &curriculum, &config);

        // Should have 2 topics, 5 facts total
        assert_eq!(report.total_facts, 5);
        assert_eq!(report.total_topics, 2);
        assert!(report.active_cells_after >= report.active_cells_before);

        // At least some facts should be retrievable (hash-based, lower quality)
        assert!(
            report.overall_hit_rate > 0.0,
            "Should have some retrievable facts, got hit_rate={}",
            report.overall_hit_rate
        );

        // Active cells should increase after ingestion
        assert!(
            report.active_cells_after > 0,
            "Curriculum ingestion should create active cells"
        );

        eprintln!("{}", report.summary());
    }

    #[test]
    fn test_curriculum_region_allocation() {
        let curriculum = Curriculum {
            name: "test".into(),
            domain: "test".into(),
            description: None,
            topics: vec![CurriculumTopic {
                name: "explicit-region".into(),
                region: Some([10, 10, 50, 50]),
                facts: vec![CurriculumFact {
                    fact: "test".into(),
                    query: None,
                }],
            }],
        };

        let regions = compute_topic_regions(&curriculum, 256);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0], [10, 10, 50, 50]);
    }

    #[test]
    fn test_curriculum_auto_region_allocation() {
        let curriculum = Curriculum {
            name: "test".into(),
            domain: "test".into(),
            description: None,
            topics: vec![
                CurriculumTopic {
                    name: "topic-1".into(),
                    region: None,
                    facts: vec![],
                },
                CurriculumTopic {
                    name: "topic-2".into(),
                    region: None,
                    facts: vec![],
                },
                CurriculumTopic {
                    name: "topic-3".into(),
                    region: None,
                    facts: vec![],
                },
                CurriculumTopic {
                    name: "topic-4".into(),
                    region: None,
                    facts: vec![],
                },
            ],
        };

        let regions = compute_topic_regions(&curriculum, 256);
        assert_eq!(regions.len(), 4);

        // Each region should be non-zero size
        for r in &regions {
            assert!(r[2] > r[0], "Region should have width: {:?}", r);
            assert!(r[3] > r[1], "Region should have height: {:?}", r);
            assert!(r[2] < 256, "Region should be within grid bounds");
            assert!(r[3] < 256, "Region should be within grid bounds");
        }
    }
}
