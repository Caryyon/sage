// Concept Association Engine - Makes creative connections between ideas
// Based on NCA loss similarity patterns: concepts with similar NCA responses are related!

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptAssociation {
    pub concept_a: String,
    pub concept_b: String,
    pub similarity_score: f64,  // 0.0 = unrelated, 1.0 = very similar
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Similar,      // "creativity" ≈ "innovation" (very close NCA loss)
    Opposite,     // "chaos" ≈ "order" (both strong but different patterns)
    Related,      // "learning" ≈ "knowledge" (moderate connection)
    Analogous,    // "light" ≈ "warmth" (metaphorical connection)
}

pub struct AssociationEngine {
    /// Map of concept → NCA loss
    concept_losses: HashMap<String, f64>,

    /// Discovered associations
    associations: Vec<ConceptAssociation>,

    /// Threshold for considering concepts similar
    similarity_threshold: f64,
}

impl AssociationEngine {
    pub fn new() -> Self {
        Self {
            concept_losses: HashMap::new(),
            associations: Vec::new(),
            similarity_threshold: 0.10,  // Losses within 0.10 are "similar"
        }
    }

    /// Record NCA loss for a concept
    pub fn record_concept_loss(&mut self, concept: String, loss: f64) {
        self.concept_losses.insert(concept.clone(), loss);

        // Check for new associations with this concept
        self.discover_associations_for(&concept, loss);
    }

    /// Find associations for a newly processed concept
    fn discover_associations_for(&mut self, new_concept: &str, new_loss: f64) {
        for (existing_concept, &existing_loss) in &self.concept_losses {
            if existing_concept == new_concept {
                continue;  // Don't associate with self
            }

            let similarity = self.calculate_similarity(new_loss, existing_loss);

            if similarity > 0.0 {
                let connection_type = self.determine_connection_type(new_loss, existing_loss);

                self.associations.push(ConceptAssociation {
                    concept_a: new_concept.to_string(),
                    concept_b: existing_concept.clone(),
                    similarity_score: similarity,
                    connection_type,
                });
            }
        }
    }

    /// Calculate similarity score between two NCA losses
    fn calculate_similarity(&self, loss_a: f64, loss_b: f64) -> f64 {
        let diff = (loss_a - loss_b).abs();

        if diff < self.similarity_threshold {
            // Very similar - exponential scoring
            let normalized_diff = diff / self.similarity_threshold;
            1.0 - normalized_diff
        } else {
            0.0  // Too different
        }
    }

    /// Determine the type of connection based on loss patterns
    fn determine_connection_type(&self, loss_a: f64, loss_b: f64) -> ConnectionType {
        let diff = (loss_a - loss_b).abs();
        let avg = (loss_a + loss_b) / 2.0;

        if diff < 0.03 {
            ConnectionType::Similar  // Nearly identical processing
        } else if diff < 0.08 {
            ConnectionType::Related  // Close but distinct
        } else if avg < 0.20 && diff < 0.15 {
            ConnectionType::Analogous  // Both understood well, different patterns
        } else {
            ConnectionType::Opposite  // Different processing patterns
        }
    }

    /// Find concepts related to a given concept
    pub fn find_related_concepts(&self, concept: &str, limit: usize) -> Vec<ConceptAssociation> {
        let mut related: Vec<ConceptAssociation> = self.associations
            .iter()
            .filter(|assoc| assoc.concept_a == concept || assoc.concept_b == concept)
            .cloned()
            .collect();

        // Sort by similarity score (highest first)
        related.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        related.into_iter().take(limit).collect()
    }

    /// Get the most creative connection for a concept
    pub fn get_creative_connection(&self, concept: &str) -> Option<String> {
        let related = self.find_related_concepts(concept, 1);

        if let Some(assoc) = related.first() {
            let other_concept = if assoc.concept_a == concept {
                &assoc.concept_b
            } else {
                &assoc.concept_a
            };

            let connection_phrase = match assoc.connection_type {
                ConnectionType::Similar => "reminds me of",
                ConnectionType::Related => "is connected to",
                ConnectionType::Analogous => "is like",
                ConnectionType::Opposite => "contrasts with",
            };

            Some(format!("That {} {}", connection_phrase, other_concept))
        } else {
            None
        }
    }

    /// Get all associations
    pub fn get_associations(&self) -> &[ConceptAssociation] {
        &self.associations
    }

    /// Get association count
    pub fn association_count(&self) -> usize {
        self.associations.len()
    }

    /// Export associations as JSON
    pub fn export_associations(&self) -> String {
        serde_json::to_string_pretty(&self.associations).unwrap_or_else(|_| "[]".to_string())
    }

    /// Import associations from JSON
    pub fn import_associations(&mut self, json: &str) -> Result<(), String> {
        let associations: Vec<ConceptAssociation> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse associations: {}", e))?;

        self.associations = associations;
        Ok(())
    }

    /// Get strongest concepts (most associations)
    pub fn get_strongest_concepts(&self, limit: usize) -> Vec<String> {
        let mut concept_counts: HashMap<String, usize> = HashMap::new();

        for assoc in &self.associations {
            *concept_counts.entry(assoc.concept_a.clone()).or_insert(0) += 1;
            *concept_counts.entry(assoc.concept_b.clone()).or_insert(0) += 1;
        }

        let mut concepts: Vec<(String, usize)> = concept_counts.into_iter().collect();
        concepts.sort_by(|a, b| b.1.cmp(&a.1));

        concepts.into_iter().take(limit).map(|(concept, _)| concept).collect()
    }

    /// Get weakest concepts (fewest associations)
    pub fn get_weakest_concepts(&self, limit: usize) -> Vec<String> {
        let mut concept_counts: HashMap<String, usize> = HashMap::new();

        for assoc in &self.associations {
            *concept_counts.entry(assoc.concept_a.clone()).or_insert(0) += 1;
            *concept_counts.entry(assoc.concept_b.clone()).or_insert(0) += 1;
        }

        let mut concepts: Vec<(String, usize)> = concept_counts.into_iter().collect();
        concepts.sort_by(|a, b| a.1.cmp(&b.1));

        concepts.into_iter().take(limit).map(|(concept, _)| concept).collect()
    }

    /// Get related concept names (not full associations)
    pub fn get_related_concepts(&self, concept: &str, limit: usize) -> Vec<String> {
        let associations = self.find_related_concepts(concept, limit);
        associations.into_iter().map(|assoc| {
            if assoc.concept_a == concept {
                assoc.concept_b
            } else {
                assoc.concept_a
            }
        }).collect()
    }

    /// Record a direct association between two concepts
    pub fn record_association(&mut self, concept_a: String, concept_b: String, similarity: f64) {
        self.associations.push(ConceptAssociation {
            concept_a,
            concept_b,
            similarity_score: similarity,
            connection_type: if similarity > 0.8 {
                ConnectionType::Similar
            } else if similarity > 0.5 {
                ConnectionType::Related
            } else {
                ConnectionType::Analogous
            },
        });
    }

    /// Cluster concepts by similarity
    pub fn get_concept_clusters(&self, min_cluster_size: usize) -> Vec<Vec<String>> {
        let mut clusters: Vec<Vec<String>> = Vec::new();
        let mut assigned: HashMap<String, usize> = HashMap::new();

        for assoc in &self.associations {
            if assoc.similarity_score < 0.7 {
                continue;  // Only cluster very similar concepts
            }

            let cluster_a = assigned.get(&assoc.concept_a).copied();
            let cluster_b = assigned.get(&assoc.concept_b).copied();

            match (cluster_a, cluster_b) {
                (None, None) => {
                    // Create new cluster
                    let cluster_id = clusters.len();
                    clusters.push(vec![assoc.concept_a.clone(), assoc.concept_b.clone()]);
                    assigned.insert(assoc.concept_a.clone(), cluster_id);
                    assigned.insert(assoc.concept_b.clone(), cluster_id);
                }
                (Some(id), None) => {
                    // Add concept_b to existing cluster
                    clusters[id].push(assoc.concept_b.clone());
                    assigned.insert(assoc.concept_b.clone(), id);
                }
                (None, Some(id)) => {
                    // Add concept_a to existing cluster
                    clusters[id].push(assoc.concept_a.clone());
                    assigned.insert(assoc.concept_a.clone(), id);
                }
                (Some(id_a), Some(id_b)) if id_a != id_b => {
                    // Merge clusters
                    let mut cluster_b_concepts = clusters[id_b].clone();
                    clusters[id_a].append(&mut cluster_b_concepts);

                    // Update assignments
                    for concept in &clusters[id_b] {
                        assigned.insert(concept.clone(), id_a);
                    }
                    clusters[id_b].clear();
                }
                _ => {}  // Already in same cluster
            }
        }

        // Filter out empty clusters and small clusters
        clusters.into_iter()
            .filter(|c| c.len() >= min_cluster_size)
            .collect()
    }
}

impl Default for AssociationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_discovery() {
        let mut engine = AssociationEngine::new();

        // Record some concepts with similar losses
        engine.record_concept_loss("creativity".to_string(), 0.12);
        engine.record_concept_loss("innovation".to_string(), 0.14);
        engine.record_concept_loss("chaos".to_string(), 0.35);

        // Should find creativity ≈ innovation
        let related = engine.find_related_concepts("creativity", 5);
        assert!(!related.is_empty());

        println!("Associations discovered: {}", engine.association_count());
        for assoc in related {
            println!("  {} ↔ {} (similarity: {:.2})",
                assoc.concept_a, assoc.concept_b, assoc.similarity_score);
        }
    }

    #[test]
    fn test_creative_connections() {
        let mut engine = AssociationEngine::new();

        engine.record_concept_loss("light".to_string(), 0.11);
        engine.record_concept_loss("warmth".to_string(), 0.13);

        let connection = engine.get_creative_connection("light");
        assert!(connection.is_some());

        println!("Creative connection: {}", connection.unwrap());
    }
}
