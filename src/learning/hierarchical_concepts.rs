// Hierarchical concept learning - combining patterns into abstract concepts

use std::collections::{HashMap, HashSet};

/// A concept in the hierarchy (can be primitive pattern or abstract combination)
#[derive(Debug, Clone)]
pub struct Concept {
    pub id: String,
    pub name: String,
    pub level: usize,  // 0 = primitive patterns, 1+ = abstract concepts
    pub components: Vec<String>,  // Sub-concepts that compose this concept
    pub observed_count: usize,
    pub confidence: f64,
}

impl Concept {
    pub fn new_primitive(name: String) -> Self {
        Self {
            id: format!("prim_{}", name),
            name,
            level: 0,
            components: Vec::new(),
            observed_count: 0,
            confidence: 1.0,
        }
    }

    pub fn new_composite(name: String, components: Vec<String>, level: usize) -> Self {
        Self {
            id: format!("comp_{}_{}", level, name),
            name,
            level,
            components,
            observed_count: 0,
            confidence: 0.5,  // Start with lower confidence for learned concepts
        }
    }

    pub fn is_primitive(&self) -> bool {
        self.level == 0
    }

    pub fn update_confidence(&mut self, new_observation: bool) {
        // Simple Bayesian update
        if new_observation {
            self.confidence = (self.confidence * 0.9) + 0.1;
        } else {
            self.confidence = self.confidence * 0.9;
        }
        self.confidence = self.confidence.clamp(0.1, 1.0);
    }
}

/// Manages hierarchical concept learning
pub struct ConceptHierarchy {
    concepts: HashMap<String, Concept>,
    primitive_patterns: HashSet<String>,
    max_levels: usize,
}

impl ConceptHierarchy {
    pub fn new(max_levels: usize) -> Self {
        // Initialize with primitive pattern concepts
        let primitive_names = vec![
            "circle", "center", "edges", "clusters", "symmetry",
            "waves", "spiral", "motion", "gradient", "boundaries", "complexity",
        ];

        let mut concepts = HashMap::new();
        let mut primitives = HashSet::new();

        for name in primitive_names {
            let concept = Concept::new_primitive(name.to_string());
            primitives.insert(name.to_string());
            concepts.insert(concept.id.clone(), concept);
        }

        Self {
            concepts,
            primitive_patterns: primitives,
            max_levels,
        }
    }

    /// Observe a set of patterns occurring together
    pub fn observe_pattern_set(&mut self, patterns: &[String]) {
        // Update primitive pattern counts
        for pattern in patterns {
            if let Some(concept_id) = self.find_concept_id(pattern) {
                if let Some(concept) = self.concepts.get_mut(&concept_id) {
                    concept.observed_count += 1;
                }
            }
        }

        // Look for frequently co-occurring patterns to create higher-level concepts
        if patterns.len() >= 2 {
            self.discover_composite_concepts(patterns);
        }
    }

    /// Discover new composite concepts from co-occurring patterns
    fn discover_composite_concepts(&mut self, patterns: &[String]) {
        // Look for pairs and triples that occur together frequently
        if patterns.len() == 2 {
            self.try_create_composite(patterns, 1);
        } else if patterns.len() >= 3 {
            // Try creating concepts from subsets
            for size in 2..=patterns.len().min(3) {
                let subsets = self.generate_subsets(patterns, size);
                for subset in subsets {
                    self.try_create_composite(&subset, 1);
                }
            }
        }
    }

    fn try_create_composite(&mut self, components: &[String], level: usize) {
        if level > self.max_levels {
            return;
        }

        // Generate a name for this composite concept
        let mut sorted_components: Vec<_> = components.iter().cloned().collect();
        sorted_components.sort();
        let name = sorted_components.join("_and_");

        // Check if this composite already exists
        let concept_id = format!("comp_{}_{}", level, name);
        if self.concepts.contains_key(&concept_id) {
            // Already exists, just increment count
            if let Some(concept) = self.concepts.get_mut(&concept_id) {
                concept.observed_count += 1;
                concept.update_confidence(true);
            }
            return;
        }

        // Create new composite concept
        let concept = Concept::new_composite(name, sorted_components, level);
        self.concepts.insert(concept.id.clone(), concept);
    }

    fn generate_subsets(&self, items: &[String], size: usize) -> Vec<Vec<String>> {
        if size == 0 || size > items.len() {
            return vec![];
        }

        if size == items.len() {
            return vec![items.to_vec()];
        }

        let mut result = Vec::new();
        self.generate_subsets_helper(items, size, 0, &mut vec![], &mut result);
        result
    }

    fn generate_subsets_helper(
        &self,
        items: &[String],
        size: usize,
        start: usize,
        current: &mut Vec<String>,
        result: &mut Vec<Vec<String>>,
    ) {
        if current.len() == size {
            result.push(current.clone());
            return;
        }

        for i in start..items.len() {
            current.push(items[i].clone());
            self.generate_subsets_helper(items, size, i + 1, current, result);
            current.pop();
        }
    }

    /// Find concept ID from pattern name
    fn find_concept_id(&self, pattern: &str) -> Option<String> {
        // First check if it's a primitive
        if self.primitive_patterns.contains(pattern) {
            return Some(format!("prim_{}", pattern));
        }

        // Then search through composites
        for (id, concept) in &self.concepts {
            if concept.name == pattern {
                return Some(id.clone());
            }
        }

        None
    }

    /// Get the most confident concepts at a given level
    pub fn get_concepts_at_level(&self, level: usize) -> Vec<&Concept> {
        let mut concepts: Vec<_> = self.concepts
            .values()
            .filter(|c| c.level == level)
            .collect();

        concepts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        concepts
    }

    /// Get concepts sorted by observation count
    pub fn get_most_observed_concepts(&self, count: usize) -> Vec<&Concept> {
        let mut concepts: Vec<_> = self.concepts.values().collect();
        concepts.sort_by(|a, b| b.observed_count.cmp(&a.observed_count));
        concepts.into_iter().take(count).collect()
    }

    /// Describe a concept in terms of its components
    pub fn describe_concept(&self, concept_id: &str) -> Option<String> {
        let concept = self.concepts.get(concept_id)?;

        if concept.is_primitive() {
            return Some(format!("Primitive: {}", concept.name));
        }

        let component_names: Vec<_> = concept.components
            .iter()
            .map(|comp| comp.as_str())
            .collect();

        Some(format!(
            "Composite (Level {}): {} = [{}]",
            concept.level,
            concept.name,
            component_names.join(" + ")
        ))
    }

    /// Get statistics about the hierarchy
    pub fn get_stats(&self) -> HierarchyStats {
        let mut level_counts = vec![0; self.max_levels + 1];
        let mut total_observations = 0;

        for concept in self.concepts.values() {
            if concept.level <= self.max_levels {
                level_counts[concept.level] += 1;
            }
            total_observations += concept.observed_count;
        }

        HierarchyStats {
            total_concepts: self.concepts.len(),
            level_counts,
            total_observations,
        }
    }

    /// Export learned concepts for analysis
    pub fn export_learned_concepts(&self) -> Vec<LearnedConceptInfo> {
        let mut concepts: Vec<_> = self.concepts
            .values()
            .filter(|c| c.level > 0 && c.observed_count > 5)  // Only well-observed composites
            .collect();

        concepts.sort_by(|a, b| b.observed_count.cmp(&a.observed_count));

        concepts
            .into_iter()
            .map(|c| LearnedConceptInfo {
                name: c.name.clone(),
                level: c.level,
                components: c.components.clone(),
                observations: c.observed_count,
                confidence: c.confidence,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct HierarchyStats {
    pub total_concepts: usize,
    pub level_counts: Vec<usize>,
    pub total_observations: usize,
}

#[derive(Debug, Clone)]
pub struct LearnedConceptInfo {
    pub name: String,
    pub level: usize,
    pub components: Vec<String>,
    pub observations: usize,
    pub confidence: f64,
}

/// Hierarchical concept learner with abstraction capabilities
pub struct HierarchicalLearner {
    hierarchy: ConceptHierarchy,
    pattern_cache: Vec<Vec<String>>,  // Recent pattern observations
    abstraction_threshold: usize,  // Minimum observations to abstract
}

impl HierarchicalLearner {
    pub fn new(max_levels: usize, abstraction_threshold: usize) -> Self {
        Self {
            hierarchy: ConceptHierarchy::new(max_levels),
            pattern_cache: Vec::new(),
            abstraction_threshold,
        }
    }

    /// Observe a pattern set and potentially learn new abstractions
    pub fn observe(&mut self, patterns: Vec<String>) {
        self.hierarchy.observe_pattern_set(&patterns);
        self.pattern_cache.push(patterns);

        // Periodically try to discover higher-level abstractions
        if self.pattern_cache.len() >= self.abstraction_threshold {
            self.abstract_concepts();
            self.pattern_cache.clear();
        }
    }

    /// Try to discover higher-level abstractions from observed patterns
    fn abstract_concepts(&mut self) {
        // Find frequently co-occurring level-1 concepts to create level-2 concepts
        let level1_concepts = self.hierarchy.get_concepts_at_level(1);

        if level1_concepts.len() < 2 {
            return;
        }

        // Look for concepts that frequently appear together
        let mut co_occurrence: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for patterns in &self.pattern_cache {
            // Find which level-1 concepts are present
            let mut present_concepts = Vec::new();
            for concept in &level1_concepts {
                let all_components_present = concept.components
                    .iter()
                    .all(|comp| patterns.contains(comp));

                if all_components_present {
                    present_concepts.push(concept.name.clone());
                }
            }

            // Track co-occurrences
            for i in 0..present_concepts.len() {
                for j in (i + 1)..present_concepts.len() {
                    let c1 = &present_concepts[i];
                    let c2 = &present_concepts[j];

                    *co_occurrence.entry(c1.clone())
                        .or_insert_with(HashMap::new)
                        .entry(c2.clone())
                        .or_insert(0) += 1;
                }
            }
        }

        // Create level-2 concepts from frequently co-occurring level-1 concepts
        for (c1, co_occurs) in co_occurrence {
            for (c2, count) in co_occurs {
                if count >= self.abstraction_threshold / 2 {
                    // Create a level-2 concept
                    let components = vec![c1.clone(), c2.clone()];
                    let mut sorted_comps = components.clone();
                    sorted_comps.sort();
                    let name = sorted_comps.join("_with_");

                    let concept = Concept::new_composite(name, components, 2);
                    self.hierarchy.concepts.insert(concept.id.clone(), concept);
                }
            }
        }
    }

    pub fn get_hierarchy(&self) -> &ConceptHierarchy {
        &self.hierarchy
    }

    pub fn get_stats(&self) -> HierarchyStats {
        self.hierarchy.get_stats()
    }

    pub fn get_learned_concepts(&self) -> Vec<LearnedConceptInfo> {
        self.hierarchy.export_learned_concepts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_concepts() {
        let hierarchy = ConceptHierarchy::new(3);
        assert!(hierarchy.concepts.len() >= 11);  // At least 11 primitives

        let primitives = hierarchy.get_concepts_at_level(0);
        assert_eq!(primitives.len(), 11);
    }

    #[test]
    fn test_composite_creation() {
        let mut hierarchy = ConceptHierarchy::new(3);

        hierarchy.observe_pattern_set(&["circle".to_string(), "center".to_string()]);
        hierarchy.observe_pattern_set(&["circle".to_string(), "center".to_string()]);

        let composites = hierarchy.get_concepts_at_level(1);
        assert!(!composites.is_empty(), "Should have created composite concepts");
    }
}
