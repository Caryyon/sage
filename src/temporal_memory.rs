// Temporal Memory System - Short-term and long-term memory consolidation
//
// This mimics how human memory works:
// - Short-term: Recent experiences, limited capacity, fast access
// - Long-term: Consolidated memories, unlimited capacity, slower access
// - Consolidation: Important short-term memories move to long-term
// - Forgetting: Less important memories fade over time
//
// Examples:
// - Recent conversation → Short-term
// - Significant emotional event → Gets consolidated to long-term
// - Repeated pattern → Strengthened in long-term
// - Unused memory → Gradually fades

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::emotional_gradients::EmotionalState;

/// A memory trace - represents a single remembered experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrace {
    /// Unique identifier
    pub id: u64,

    /// The content of the memory
    pub content: String,

    /// Associated concepts
    pub concepts: Vec<String>,

    /// Emotional state during encoding
    pub emotion: Option<EmotionalState>,

    /// When this memory was created
    pub timestamp: u64,

    /// How many times this memory has been recalled
    pub recall_count: u64,

    /// Last time this memory was accessed
    pub last_accessed: u64,

    /// Strength/importance of this memory (0.0 to 1.0)
    pub strength: f64,

    /// Whether this memory is emotionally significant
    pub emotional_significance: f64,
}

impl MemoryTrace {
    pub fn new(
        id: u64,
        content: String,
        concepts: Vec<String>,
        emotion: Option<EmotionalState>,
        timestamp: u64,
    ) -> Self {
        let emotional_significance = emotion
            .as_ref()
            .map(|e| e.intensity * e.valence.abs())
            .unwrap_or(0.0);

        Self {
            id,
            content,
            concepts,
            emotion,
            timestamp,
            recall_count: 0,
            last_accessed: timestamp,
            strength: 1.0,  // Start at full strength
            emotional_significance,
        }
    }

    /// Calculate importance score for consolidation decision
    pub fn importance(&self) -> f64 {
        let recency_factor = 0.3;
        let recall_factor = 0.4;
        let emotion_factor = 0.3;

        let recall_score = (self.recall_count as f64 / 10.0).min(1.0);
        let emotion_score = self.emotional_significance;

        // Recency is handled externally based on position in queue
        recall_score * recall_factor + emotion_score * emotion_factor
    }

    /// Access this memory (increments recall count)
    pub fn access(&mut self, current_time: u64) {
        self.recall_count += 1;
        self.last_accessed = current_time;

        // Strengthen memory through recall (spaced repetition effect)
        self.strength = (self.strength + 0.1).min(1.0);
    }

    /// Apply memory decay
    pub fn decay(&mut self, decay_rate: f64) {
        self.strength = (self.strength - decay_rate).max(0.0);
    }

    /// Check if memory has faded completely
    pub fn is_forgotten(&self) -> bool {
        self.strength < 0.1
    }
}

/// Short-term memory - recent experiences with limited capacity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermMemory {
    /// FIFO queue of recent memories
    memories: VecDeque<MemoryTrace>,

    /// Maximum capacity (like human working memory ~7 items)
    capacity: usize,

    /// Next memory ID
    next_id: u64,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            memories: VecDeque::with_capacity(capacity),
            capacity,
            next_id: 0,
        }
    }

    /// Add a new memory to short-term storage
    pub fn encode(
        &mut self,
        content: String,
        concepts: Vec<String>,
        emotion: Option<EmotionalState>,
        timestamp: u64,
    ) -> &MemoryTrace {
        let id = self.next_id;
        self.next_id += 1;

        let memory = MemoryTrace::new(id, content, concepts, emotion, timestamp);

        // If at capacity, remove oldest (pushed out to consolidation)
        if self.memories.len() >= self.capacity {
            self.memories.pop_front();
        }

        self.memories.push_back(memory);
        self.memories.back().unwrap()
    }

    /// Get most recent memories
    pub fn get_recent(&self, count: usize) -> Vec<&MemoryTrace> {
        self.memories
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Get all memories (for consolidation)
    pub fn get_all(&self) -> Vec<&MemoryTrace> {
        self.memories.iter().collect()
    }

    /// Clear short-term memory
    pub fn clear(&mut self) {
        self.memories.clear();
    }

    /// Get memory count
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }
}

/// Long-term memory - consolidated memories with unlimited capacity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermMemory {
    /// All consolidated memories
    memories: Vec<MemoryTrace>,

    /// Index: concept -> memory IDs
    concept_index: HashMap<String, Vec<u64>>,

    /// Index: emotional significance threshold -> memory IDs
    significant_memories: Vec<u64>,

    /// Decay rate for unused memories
    decay_rate: f64,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            concept_index: HashMap::new(),
            significant_memories: Vec::new(),
            decay_rate: 0.001,  // Very slow decay
        }
    }

    /// Consolidate a memory from short-term to long-term
    pub fn consolidate(&mut self, memory: MemoryTrace) {
        let id = memory.id;

        // Update concept index
        for concept in &memory.concepts {
            self.concept_index
                .entry(concept.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }

        // Track emotionally significant memories
        if memory.emotional_significance > 0.6 {
            self.significant_memories.push(id);
        }

        self.memories.push(memory);
    }

    /// Recall memories by concept
    pub fn recall_by_concept(&mut self, concept: &str, current_time: u64) -> Vec<String> {
        if let Some(ids) = self.concept_index.get(concept) {
            let mut results = Vec::new();

            // Update access counts and collect contents
            for id in ids {
                if let Some(memory) = self.memories.iter_mut().find(|m| m.id == *id) {
                    memory.access(current_time);
                    results.push(memory.content.clone());
                }
            }

            results
        } else {
            Vec::new()
        }
    }

    /// Get most significant memories
    pub fn get_significant_memories(&self, count: usize) -> Vec<&MemoryTrace> {
        let mut memories: Vec<&MemoryTrace> = self.significant_memories
            .iter()
            .filter_map(|id| self.memories.iter().find(|m| m.id == *id))
            .collect();

        memories.sort_by(|a, b| {
            b.emotional_significance.partial_cmp(&a.emotional_significance).unwrap()
        });

        memories.into_iter().take(count).collect()
    }

    /// Apply decay to all long-term memories
    pub fn apply_decay(&mut self) {
        for memory in &mut self.memories {
            memory.decay(self.decay_rate);
        }

        // Remove completely forgotten memories
        self.memories.retain(|m| !m.is_forgotten());

        // Rebuild indices after forgetting
        self.rebuild_indices();
    }

    /// Rebuild all indices
    fn rebuild_indices(&mut self) {
        self.concept_index.clear();
        self.significant_memories.clear();

        for memory in &self.memories {
            // Rebuild concept index
            for concept in &memory.concepts {
                self.concept_index
                    .entry(concept.clone())
                    .or_insert_with(Vec::new)
                    .push(memory.id);
            }

            // Rebuild significant memories
            if memory.emotional_significance > 0.6 {
                self.significant_memories.push(memory.id);
            }
        }
    }

    /// Get total number of long-term memories
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete temporal memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMemory {
    short_term: ShortTermMemory,
    long_term: LongTermMemory,

    /// Consolidation threshold - importance score needed to move to long-term
    consolidation_threshold: f64,

    /// How often to trigger consolidation (in time steps)
    consolidation_interval: u64,

    /// Last consolidation time
    last_consolidation: u64,
}

impl TemporalMemory {
    pub fn new() -> Self {
        Self {
            short_term: ShortTermMemory::new(7),  // Miller's magic number
            long_term: LongTermMemory::new(),
            consolidation_threshold: 0.5,
            consolidation_interval: 100,  // Consolidate every 100 time steps
            last_consolidation: 0,
        }
    }

    /// Encode a new experience
    pub fn encode_experience(
        &mut self,
        content: String,
        concepts: Vec<String>,
        emotion: Option<EmotionalState>,
        timestamp: u64,
    ) {
        self.short_term.encode(content, concepts, emotion, timestamp);
    }

    /// Consolidate memories from short-term to long-term
    pub fn consolidate(&mut self, current_time: u64) {
        // Check if it's time to consolidate
        if current_time - self.last_consolidation < self.consolidation_interval {
            return;
        }

        let mut to_consolidate = Vec::new();

        // Evaluate each short-term memory for consolidation
        for (idx, memory) in self.short_term.get_all().iter().enumerate() {
            let importance = memory.importance();

            // High-importance or emotionally significant memories get consolidated
            if importance >= self.consolidation_threshold
                || memory.emotional_significance > 0.7
            {
                to_consolidate.push(idx);
            }
        }

        // Move selected memories to long-term (in reverse to maintain indices)
        for idx in to_consolidate.into_iter().rev() {
            if let Some(memory) = self.short_term.memories.remove(idx) {
                self.long_term.consolidate(memory);
            }
        }

        self.last_consolidation = current_time;
    }

    /// Recall from both short-term and long-term
    pub fn recall(&mut self, concept: &str, current_time: u64) -> Vec<String> {
        let mut results = Vec::new();

        // Check short-term first (faster, more recent)
        for memory in self.short_term.get_all() {
            if memory.concepts.contains(&concept.to_string()) {
                results.push(format!("[Recent] {}", memory.content));
            }
        }

        // Then check long-term
        for content in self.long_term.recall_by_concept(concept, current_time) {
            results.push(format!("[Past] {}", content));
        }

        results
    }

    /// Get a summary of memory state
    pub fn get_summary(&self) -> String {
        format!(
            "Memory: {} recent experiences, {} consolidated memories ({}% capacity)",
            self.short_term.len(),
            self.long_term.len(),
            (self.short_term.len() * 100) / self.short_term.capacity
        )
    }

    /// Apply memory decay (call periodically during idle periods)
    pub fn decay_memories(&mut self) {
        self.long_term.apply_decay();
    }

    /// Get significant memories across both stores
    pub fn get_significant_memories(&self, count: usize) -> Vec<String> {
        let mut results = Vec::new();

        // Get from long-term
        for memory in self.long_term.get_significant_memories(count) {
            results.push(memory.content.clone());
        }

        // Add emotionally significant recent memories
        for memory in self.short_term.get_recent(count) {
            if memory.emotional_significance > 0.6 {
                results.push(memory.content.clone());
            }
        }

        results.into_iter().take(count).collect()
    }
}

impl Default for TemporalMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_term_capacity() {
        let mut stm = ShortTermMemory::new(3);

        for i in 0..5 {
            stm.encode(
                format!("Memory {}", i),
                vec![format!("concept{}", i)],
                None,
                i,
            );
        }

        // Should only hold last 3
        assert_eq!(stm.len(), 3);
        let recent = stm.get_recent(1);
        assert_eq!(recent[0].content, "Memory 4");
    }

    #[test]
    fn test_consolidation() {
        let mut temporal = TemporalMemory::new();

        // Add emotionally significant memory
        let emotion = EmotionalState {
            valence: 0.8,
            arousal: 0.9,
            dominance: 0.7,
            intensity: 0.9,
        };

        temporal.encode_experience(
            "Important event".to_string(),
            vec!["milestone".to_string()],
            Some(emotion),
            100,
        );

        // Trigger consolidation
        temporal.consolidate(200);

        // Should be in long-term now
        assert!(temporal.long_term.len() > 0);
    }

    #[test]
    fn test_memory_recall() {
        let mut temporal = TemporalMemory::new();

        temporal.encode_experience(
            "Learned about rust".to_string(),
            vec!["rust".to_string(), "programming".to_string()],
            None,
            1,
        );

        let results = temporal.recall("rust", 10);
        assert!(results.len() > 0);
        assert!(results[0].contains("rust"));
    }
}
