//! Consolidation Engine — HDC → NCA Pattern Learning
//!
//! During "sleep" cycles, scans the HDC store for semantic clusters,
//! extracts common patterns, and encodes them as NCA attractor states.
//! This is the bridge between exact episodic memory (HDC) and
//! compressed semantic memory (NCA grid).
//!
//! ## Architecture
//!
//! The consolidation engine implements Step 6 of the v0.6.0 plan:
//!
//! 1. **Cluster Scan**: Sample centroids from the HDC store, find nearest
//!    neighbors via cosine similarity, identify dense semantic clusters.
//! 2. **Pattern Extraction**: For each cluster, compute the centroid
//!    embedding and extract representative text.
//! 3. **NCA Encoding**: Write cluster patterns into the NCA grid as
//!    knowledge entries, then run consolidation steps to strengthen them
//!    into stable attractor states.
//!
//! This is the "dream" — but not the broken v0.3.3 dream. The old dream
//! cycle tried to move knowledge between two NCA grids. The new dream
//! moves knowledge from HDC → NCA. One direction. Clean.
//!
//! ## Sleep Cycle
//!
//! A full sleep cycle:
//! ```text
//! Load HDC store → Sample centroids → Find clusters →
//! Extract patterns → Encode into NCA → Consolidate → Save
//! ```
//!
//! The NCA grid becomes a compressed model of what the HDC store knows.
//! Clusters become attractors. The grid learns the *structure* of the
//! knowledge, not every fact — that's what the HDC store is for.

use crate::grid::{ConsolidationParams, GRID_SIZE, KNOWLEDGE_ACTIVATION};
use crate::hdc::HdcStore;
use crate::distributed_knowledge::encoder::{encode_text, EncoderConfig, write_knowledge};
use crate::distributed_knowledge::{NCAKnowledge, default_brain_path, KnowledgeStore};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

// ── Cluster Representation ─────────────────────────────────────────────────

/// A semantic cluster discovered in the HDC store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HdcCluster {
    /// Cluster ID (index in discovery order)
    pub id: usize,
    /// Centroid embedding (mean of member embeddings)
    pub centroid: Vec<f32>,
    /// Number of entries in this cluster
    pub size: usize,
    /// Mean cosine similarity within the cluster (coherence)
    pub coherence: f32,
    /// Representative text samples (up to 3)
    pub samples: Vec<String>,
    /// Common keywords extracted from member texts
    pub keywords: Vec<String>,
}

// ── Sleep Report ───────────────────────────────────────────────────────────

/// Results of a single sleep cycle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SleepReport {
    /// Timestamp of the sleep cycle (Unix epoch seconds)
    pub timestamp: u64,
    /// Number of clusters discovered
    pub clusters_found: usize,
    /// Number of clusters encoded into NCA grid
    pub clusters_encoded: usize,
    /// Total HDC entries scanned
    pub entries_scanned: usize,
    /// Mean cluster coherence
    pub mean_coherence: f32,
    /// Duration of the sleep cycle
    pub duration_secs: f64,
    /// NCA grid state after consolidation (knowledge cell count)
    pub nca_knowledge_cells: usize,
    /// HDC store size at time of sleep
    pub hdc_entries: usize,
}

// ── Consolidation Engine ───────────────────────────────────────────────────

/// Configuration for the consolidation engine.
#[derive(Clone, Debug)]
pub struct ConsolidationConfig {
    /// Number of centroids to sample for cluster discovery
    pub num_centroids: usize,
    /// Number of nearest neighbors to check per centroid
    pub neighbors_per_centroid: usize,
    /// Minimum cosine similarity for cluster membership
    pub cluster_threshold: f32,
    /// Minimum cluster size to keep (ignore tiny clusters)
    pub min_cluster_size: usize,
    /// Maximum clusters to encode per sleep cycle
    pub max_clusters: usize,
    /// Number of consolidation steps after encoding
    pub consolidation_steps: usize,
    /// Path to the HDC store
    pub hdc_path: PathBuf,
    /// Path to the NCA brain
    pub brain_path: PathBuf,
    /// Consolidation parameters for the NCA grid
    pub params: ConsolidationParams,
    /// Encoder config for writing to NCA grid
    pub encoder_config: EncoderConfig,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            num_centroids: 50,
            neighbors_per_centroid: 20,
            cluster_threshold: 0.75,
            min_cluster_size: 5,
            max_clusters: 50,
            consolidation_steps: 3,
            hdc_path: PathBuf::from(crate::hdc::default_hdc_path()),
            brain_path: PathBuf::from(default_brain_path()),
            params: ConsolidationParams::default(),
            encoder_config: EncoderConfig::default(),
        }
    }
}

/// The consolidation engine: bridges HDC episodic memory → NCA semantic memory.
pub struct ConsolidationEngine {
    config: ConsolidationConfig,
    /// Accumulated sleep reports for tracking progress
    history: Vec<SleepReport>,
}

impl ConsolidationEngine {
    /// Create a new consolidation engine with the given configuration.
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    /// Create with default config, loading trained params if available.
    pub fn with_trained_params() -> Self {
        let mut config = ConsolidationConfig::default();
        config.params = ConsolidationParams::load_or_default();
        Self::new(config)
    }

    /// Run a full sleep cycle: scan HDC → find clusters → encode into NCA.
    ///
    /// Returns a SleepReport with metrics. The NCA brain is saved to disk
    /// after encoding.
    pub fn sleep_cycle(&mut self, verbose: bool) -> Result<SleepReport, String> {
        let start = Instant::now();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if verbose {
            eprintln!("🌙 Starting sleep cycle...");
            eprintln!("   Loading HDC store from {:?}", self.config.hdc_path);
        }

        // Step 1: Load HDC store
        let hdc = HdcStore::load(&self.config.hdc_path)?;
        let hdc_entries = hdc.len();

        if verbose {
            eprintln!("   HDC store: {} entries, {} dim", hdc_entries, hdc.dim);
        }

        if hdc_entries < self.config.min_cluster_size {
            return Err(format!(
                "HDC store too small for clustering ({} entries, need ≥ {})",
                hdc_entries, self.config.min_cluster_size
            ));
        }

        // Step 2: Sample centroids
        let centroids = self.sample_centroids(&hdc, verbose);

        if verbose {
            eprintln!("   Sampled {} centroids", centroids.len());
        }

        // Step 3: Find clusters around each centroid
        let clusters = self.find_clusters(&hdc, &centroids, verbose);

        if verbose {
            eprintln!("   Found {} clusters (threshold={})",
                clusters.len(), self.config.cluster_threshold);
        }

        // Step 4: Filter and sort clusters by coherence
        let mut filtered: Vec<HdcCluster> = clusters
            .into_iter()
            .filter(|c| c.size >= self.config.min_cluster_size)
            .collect();

        filtered.sort_by(|a, b| b.coherence.partial_cmp(&a.coherence).unwrap_or(std::cmp::Ordering::Equal));
        filtered.truncate(self.config.max_clusters);

        let mean_coherence = if filtered.is_empty() {
            0.0
        } else {
            filtered.iter().map(|c| c.coherence).sum::<f32>() / filtered.len() as f32
        };

        if verbose {
            eprintln!("   Kept {} clusters (min_size={}, max={})",
                filtered.len(), self.config.min_cluster_size, self.config.max_clusters);
            eprintln!("   Mean coherence: {:.3}", mean_coherence);
        }

        // Step 5: Encode clusters into NCA grid
        let clusters_encoded = self.encode_clusters(&filtered, verbose)?;

        // Step 6: Run NCA consolidation steps
        if clusters_encoded > 0 {
            self.consolidate_nca(verbose)?;
        }

        let duration = start.elapsed().as_secs_f64();

        // Count knowledge cells in NCA grid
        let nca_knowledge_cells = self.count_knowledge_cells()?;

        let report = SleepReport {
            timestamp,
            clusters_found: filtered.len(),
            clusters_encoded,
            entries_scanned: hdc_entries,
            mean_coherence,
            duration_secs: duration,
            nca_knowledge_cells,
            hdc_entries,
        };

        self.history.push(report.clone());

        if verbose {
            eprintln!("✅ Sleep cycle complete in {:.1}s", duration);
            eprintln!("   Clusters: {} found, {} encoded", report.clusters_found, report.clusters_encoded);
            eprintln!("   NCA knowledge cells: {}", report.nca_knowledge_cells);
        }

        Ok(report)
    }

    /// Run multiple sleep cycles, stopping when few new clusters are found.
    pub fn sleep_until_stable(
        &mut self,
        max_cycles: usize,
        min_new_clusters: usize,
        verbose: bool,
    ) -> Result<Vec<SleepReport>, String> {
        let mut reports = Vec::new();

        for cycle in 0..max_cycles {
            let report = self.sleep_cycle(verbose)?;
            let encoded = report.clusters_encoded;
            reports.push(report);

            if encoded < min_new_clusters {
                if verbose {
                    eprintln!("   Stable: only {} new clusters, stopping after {} cycles",
                        encoded, cycle + 1);
                }
                break;
            }
        }

        Ok(reports)
    }

    /// Get the sleep history.
    pub fn history(&self) -> &[SleepReport] {
        &self.history
    }

    // ── Private Methods ────────────────────────────────────────────────────

    /// Sample centroid candidates from the HDC store.
    ///
    /// Strategy: stratified random sampling to ensure coverage.
    /// - 60% random uniform sample
    /// - 40% sampled from high-confidence entries (likely important)
    fn sample_centroids(&self, hdc: &HdcStore, verbose: bool) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let n = hdc.len();
        let num = self.config.num_centroids.min(n);

        let mut indices: Vec<usize> = Vec::with_capacity(num);

        // 60% uniform random
        let uniform_count = (num as f64 * 0.6) as usize;
        let mut all_indices: Vec<usize> = (0..n).collect();
        all_indices.shuffle(&mut rng);
        indices.extend_from_slice(&all_indices[..uniform_count.min(n)]);

        // 40% from high-confidence entries
        let conf_count = num - indices.len();
        if conf_count > 0 {
            let mut conf_indices: Vec<(usize, f32)> = hdc.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.confidence > 0.7)
                .map(|(i, e)| (i, e.confidence))
                .collect();

            conf_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            conf_indices.truncate(conf_count * 3); // oversample then shuffle
            conf_indices.shuffle(&mut rng);
            conf_indices.truncate(conf_count);

            for (idx, _) in conf_indices {
                if !indices.contains(&idx) {
                    indices.push(idx);
                }
            }
        }

        // Deduplicate
        indices.sort();
        indices.dedup();

        if verbose {
            eprintln!("   Centroids: {} uniform + {} high-confidence = {} total",
                uniform_count, indices.len() - uniform_count.min(indices.len()), indices.len());
        }

        indices
    }

    /// Find clusters around each centroid.
    ///
    /// For each centroid, query the HDC store for nearest neighbors.
    /// If enough neighbors are above the similarity threshold, form a cluster.
    fn find_clusters(
        &self,
        hdc: &HdcStore,
        centroid_indices: &[usize],
        verbose: bool,
    ) -> Vec<HdcCluster> {
        let mut clusters: Vec<HdcCluster> = Vec::new();
        let mut assigned: Vec<bool> = vec![false; hdc.len()];

        for (cluster_id, &centroid_idx) in centroid_indices.iter().enumerate() {
            if assigned[centroid_idx] {
                continue; // Already part of another cluster
            }

            let centroid_entry = &hdc.entries[centroid_idx];
            let centroid_emb = &centroid_entry.embedding;

            // Query for nearest neighbors (with indices)
            let neighbors = hdc.query_detailed(centroid_emb, self.config.neighbors_per_centroid);

            // Collect members above threshold
            let mut member_indices: Vec<usize> = Vec::new();
            let mut member_sims: Vec<f32> = Vec::new();

            for result in &neighbors {
                if result.relevance >= self.config.cluster_threshold && !assigned[result.index] {
                    member_indices.push(result.index);
                    member_sims.push(result.relevance);
                    assigned[result.index] = true;
                }
            }

            // Include the centroid itself
            if !assigned[centroid_idx] {
                member_indices.push(centroid_idx);
                member_sims.push(1.0); // self-similarity
                assigned[centroid_idx] = true;
            }

            if member_indices.len() >= self.config.min_cluster_size {
                // Compute centroid as mean of member embeddings
                let dim = centroid_emb.len();
                let mut mean_emb = vec![0.0f32; dim];
                for &idx in &member_indices {
                    for (j, &val) in hdc.entries[idx].embedding.iter().enumerate() {
                        mean_emb[j] += val;
                    }
                }
                for val in &mut mean_emb {
                    *val /= member_indices.len() as f32;
                }

                // Compute coherence (mean pairwise similarity to centroid)
                let coherence = member_sims.iter().sum::<f32>() / member_sims.len() as f32;

                // Extract representative samples (up to 3)
                let samples: Vec<String> = member_indices
                    .iter()
                    .take(3)
                    .map(|&idx| {
                        let text = &hdc.entries[idx].text;
                        // Truncate to ~200 chars, respecting UTF-8 boundaries
                        if text.chars().count() > 200 {
                            let truncated: String = text.chars().take(200).collect();
                            format!("{}...", truncated)
                        } else {
                            text.clone()
                        }
                    })
                    .collect();

                // Extract common keywords (simple: words appearing in multiple members)
                let member_texts: Vec<&str> = member_indices.iter().map(|&i| hdc.entries[i].text.as_str()).collect();
                let keywords = extract_common_keywords(&member_texts);

                clusters.push(HdcCluster {
                    id: cluster_id,
                    centroid: mean_emb,
                    size: member_indices.len(),
                    coherence,
                    samples,
                    keywords,
                });
            }
        }

        clusters
    }

    /// Encode discovered clusters into the NCA grid.
    ///
    /// For each cluster, we encode the centroid pattern as a knowledge entry.
    /// The representative text is written to the text store, and the centroid
    /// embedding is written to the grid via write_knowledge().
    fn encode_clusters(
        &self,
        clusters: &[HdcCluster],
        verbose: bool,
    ) -> Result<usize, String> {
        if clusters.is_empty() {
            return Ok(0);
        }

        // Load or create NCA knowledge store
        let mut knowledge = if self.config.brain_path.exists() {
            if verbose {
                eprintln!("   Loading existing NCA brain from {:?}", self.config.brain_path);
            }
            let mut k = NCAKnowledge::new();
            k.load(&self.config.brain_path.to_string_lossy())?;
            k
        } else {
            if verbose {
                eprintln!("   Creating new NCA brain");
            }
            NCAKnowledge::new()
        };

        let mut encoded = 0usize;

        for cluster in clusters {
            // Build a descriptive text for the cluster
            let label = format!(
                "[CLUSTER:{}] size={} coherence={:.3} keywords={} | samples: {}",
                cluster.id,
                cluster.size,
                cluster.coherence,
                cluster.keywords.join(", "),
                cluster.samples.first().map(|s| s.as_str()).unwrap_or("")
            );

            // Encode the cluster label as knowledge
            // This writes to the NCA grid using the encoder
            let features = encode_text(&label, &self.config.encoder_config);
            let confidence = (cluster.coherence as f64).clamp(0.3, 1.0);

            // Write to grid — this places the pattern in the knowledge channels
            let (cx, cy) = write_knowledge(
                &mut knowledge.grid,
                &features,
                confidence,
                0.5, // timestamp
                &self.config.encoder_config,
            );

            // Also store the label in the text store for retrieval
            knowledge.text_store.insert(cx, cy, label);

            encoded += 1;

            if verbose && encoded % 10 == 0 {
                eprintln!("   Encoded {}/{} clusters", encoded, clusters.len());
            }
        }

        // Save the brain
        knowledge.save(&self.config.brain_path.to_string_lossy())?;

        if verbose {
            eprintln!("   Saved NCA brain: {} entries in text store", knowledge.text_store.len());
        }

        Ok(encoded)
    }

    /// Run consolidation steps on the NCA grid to strengthen encoded patterns.
    fn consolidate_nca(&self, verbose: bool) -> Result<(), String> {
        let mut knowledge = NCAKnowledge::new();
        knowledge.load(&self.config.brain_path.to_string_lossy())?;

        if verbose {
            eprintln!("   Running {} consolidation steps...", self.config.consolidation_steps);
        }

        knowledge.grid.consolidate_knowledge_with_params(
            self.config.consolidation_steps,
            &self.config.params,
        );

        knowledge.save(&self.config.brain_path.to_string_lossy())?;

        if verbose {
            eprintln!("   Consolidation complete, brain saved");
        }

        Ok(())
    }

    /// Count cells with active knowledge in the NCA grid.
    fn count_knowledge_cells(&self) -> Result<usize, String> {
        if !self.config.brain_path.exists() {
            return Ok(0);
        }

        let mut knowledge = NCAKnowledge::new();
        knowledge.load(&self.config.brain_path.to_string_lossy())?;
        let mut count = 0;

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                if knowledge.grid.cells[y][x][KNOWLEDGE_ACTIVATION] > 0.01 {
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

// ── Utility Functions ──────────────────────────────────────────────────────

/// Compute cosine similarity between two f32 vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a < 1e-10 || mag_b < 1e-10 {
        return 0.0;
    }
    (dot / (mag_a * mag_b)).clamp(-1.0, 1.0)
}

/// Extract common keywords from a set of texts.
///
/// Simple approach: find words that appear in at least 40% of the texts,
/// excluding very common English stop words.
fn extract_common_keywords(texts: &[&str]) -> Vec<String> {
    if texts.is_empty() {
        return Vec::new();
    }

    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "from", "by", "is", "are", "was", "were", "be", "been",
        "being", "have", "has", "had", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "can", "shall", "you", "your",
        "he", "she", "it", "we", "they", "this", "that", "these", "those",
        "i", "me", "my", "mine", "myself", "we", "us", "our", "ours",
        "not", "no", "nor", "so", "as", "if", "then", "than", "too", "very",
        "just", "about", "above", "after", "again", "all", "also", "any",
        "here", "there", "when", "where", "why", "how", "which", "who",
        "whom", "what", "into", "over", "under", "up", "down", "out",
        "some", "such", "each", "every", "both", "few", "more", "most",
        "other", "own", "same", "one", "two", "first", "new", "now",
        "its", "his", "her", "their", "them", "him", "has", "had",
        "said", "like", "much", "many", "well", "back", "still", "even",
        "only", "through", "before", "between", "long", "great", "little",
        "part", "place", "made", "make", "man", "men", "life", "time",
        "way", "day", "world", "house", "year", "hand", "head", "face",
        "thing", "thought", "came", "come", "went", "go", "see", "know",
        "get", "got", "take", "took", "look", "find", "found", "give",
        "tell", "work", "call", "try", "ask", "need", "feel", "left",
        "right", "old", "young", "good", "bad", "big", "small", "high",
        "low", "far", "near", "last", "next", "always", "never", "often",
        "perhaps", "though", "while", "since", "until", "without",
        "within", "along", "among", "upon", "toward", "around",
        "shall", "must", "cannot", "could", "would", "should",
        "chapter", "part", "section", "page", "volume", "book",
        "illustration", "illustrated", "project", "gutenberg",
        "ebook", "electronic", "edition", "copyright", "license",
        "http", "www", "org", "html", "file", "format", "archive",
        "title", "author", "release", "date", "language", "character",
        "encoding", "start", "end", "contents", "table", "preface",
        "introduction", "footnote", "footnotes", "appendix", "index",
    ].iter().cloned().collect();

    let min_texts = (texts.len() as f64 * 0.4).ceil() as usize;

    // Count word occurrences across texts
    let mut word_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for text in texts {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for word in text.split_whitespace() {
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_alphabetic())
                .map(|c| c.to_ascii_lowercase())
                .collect();

            if cleaned.len() < 3 || stop_words.contains(cleaned.as_str()) {
                continue;
            }

            if seen.insert(cleaned.clone()) {
                *word_counts.entry(cleaned).or_insert(0) += 1;
            }
        }
    }

    // Filter to words appearing in enough texts
    let mut keywords: Vec<(String, usize)> = word_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_texts)
        .collect();

    keywords.sort_by(|a, b| b.1.cmp(&a.1));
    keywords.truncate(10);

    keywords.into_iter().map(|(word, _)| word).collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_common_keywords() {
        let texts = vec![
            "the cat sat on the mat with a hat",
            "the cat wore a hat on the mat",
            "a dog sat on the log in the fog",
        ];

        let keywords = extract_common_keywords(&texts);
        // "cat", "sat", "mat", "hat" should appear in 2/3 texts
        assert!(keywords.contains(&"cat".to_string()));
        assert!(keywords.contains(&"sat".to_string()));
        assert!(keywords.contains(&"mat".to_string()));
    }

    #[test]
    fn test_extract_common_keywords_stop_words_filtered() {
        let texts = vec![
            "the the the the the",
            "the the the the the",
            "the the the the the",
        ];

        let keywords = extract_common_keywords(&texts);
        // "the" is a stop word, should be filtered
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_consolidation_config_defaults() {
        let config = ConsolidationConfig::default();
        assert_eq!(config.num_centroids, 50);
        assert_eq!(config.cluster_threshold, 0.75);
        assert_eq!(config.min_cluster_size, 5);
        assert_eq!(config.max_clusters, 50);
    }

    #[test]
    fn test_sleep_report_serialization() {
        let report = SleepReport {
            timestamp: 1234567890,
            clusters_found: 42,
            clusters_encoded: 30,
            entries_scanned: 83794,
            mean_coherence: 0.85,
            duration_secs: 12.5,
            nca_knowledge_cells: 1500,
            hdc_entries: 83794,
        };

        let json = serde_json::to_string(&report).unwrap();
        let restored: SleepReport = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.clusters_found, 42);
        assert_eq!(restored.clusters_encoded, 30);
    }
}
