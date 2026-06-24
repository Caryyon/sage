//! NCA Intuition generation — wired into the chat pipeline.
//!
//! This module implements the Step 7 generation engine as a standalone function
//! that can be called from KnowledgeLoop::chat().

use crate::distributed_knowledge::decoder::scan_active_knowledge;
use crate::distributed_knowledge::encoder::{encode_text, feature_to_position, EncoderConfig};
use crate::distributed_knowledge::text_store::TextStore;
use crate::grid::{Grid, KNOWLEDGE_ACTIVATION};
use std::collections::{HashMap, HashSet};

/// Stop words for keyword extraction
fn is_stop_word(w: &str) -> bool {
    const STOP: &[&str] = &[
        "that", "this", "with", "from", "have", "they", "were", "been", "which",
        "their", "there", "what", "about", "would", "could", "should", "between",
        "into", "through", "during", "before", "after", "above", "below",
        "more", "most", "some", "such", "only", "very", "than", "then",
        "also", "will", "just", "does", "made", "used", "each", "every",
        "both", "few", "other", "same", "like", "make", "well",
    ];
    STOP.contains(&w) || w.len() < 4
}

/// Generate NCA intuition for a query.
///
/// Given a query, this function:
/// 1. Finds cells matching the query's hash position on the grid
/// 2. Boosts their activation (on a scratch copy)
/// 3. Runs Hebbian-like activation spread to neighbors
/// 4. Scans newly-activated cells for associated knowledge
/// 5. Extracts keywords and topic clusters from the activated region
///
/// Returns a formatted string with keyword associations and topic clusters,
/// or None if the grid has insufficient activation.
pub fn generate_intuition(
    grid: &Grid,
    text_store: &TextStore,
    config: &EncoderConfig,
    query: &str,
) -> Option<String> {
    // Clone the grid for scratch operations (don't pollute main grid)
    let mut scratch = grid.clone();

    // 1. Find cells matching the query's hash position
    let query_features = encode_text(query, config);
    let (qx, qy) = feature_to_position(&query_features, scratch.width, scratch.height);

    let radius = 15;
    let mut activated: HashSet<(usize, usize)> = HashSet::new();
    let mut newly_activated: Vec<(usize, usize)> = Vec::new();

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = ((qx as i32 + dx).rem_euclid(scratch.width as i32)) as usize;
            let ny = ((qy as i32 + dy).rem_euclid(scratch.height as i32)) as usize;
            let act = scratch.cells[ny][nx].get(KNOWLEDGE_ACTIVATION).copied().unwrap_or(0.0);
            if act >= 0.05 {
                scratch.cells[ny][nx][KNOWLEDGE_ACTIVATION] = (act + 0.15).min(1.0);
                activated.insert((nx, ny));
                newly_activated.push((nx, ny));
            }
        }
    }

    if activated.is_empty() {
        return None;
    }

    // 2. Run activation spread (Hebbian-like propagation)
    // Only spread from newly-activated cells each step, not the entire set.
    // This prevents O(n²) blowup and simulates wave propagation more naturally.
    let spread_steps = 3;
    let spread_radius = 3;
    let spread_strength = 0.12;
    let activation_threshold = 0.12;

    for _step in 0..spread_steps {
        let mut updates: Vec<(usize, usize, f64)> = Vec::new();
        let mut next_newly: Vec<(usize, usize)> = Vec::new();

        for &(cx, cy) in &newly_activated {
            for dy in -spread_radius..=spread_radius {
                for dx in -spread_radius..=spread_radius {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = ((cx as i32 + dx).rem_euclid(scratch.width as i32)) as usize;
                    let ny = ((cy as i32 + dy).rem_euclid(scratch.height as i32)) as usize;
                    let source_act = scratch.cells[cy][cx]
                        .get(KNOWLEDGE_ACTIVATION)
                        .copied()
                        .unwrap_or(0.0);
                    let dist = ((dx * dx + dy * dy) as f64).sqrt();
                    let boost = source_act * spread_strength / (1.0 + dist);
                    updates.push((nx, ny, boost));
                }
            }
        }

        for (nx, ny, boost) in &updates {
            scratch.cells[*ny][*nx][KNOWLEDGE_ACTIVATION] =
                (scratch.cells[*ny][*nx][KNOWLEDGE_ACTIVATION] + boost).min(1.0);
            // Track newly activated cells (not already in the active set)
            if scratch.cells[*ny][*nx].get(KNOWLEDGE_ACTIVATION).copied().unwrap_or(0.0) >= activation_threshold
                && !activated.contains(&(*nx, *ny))
            {
                activated.insert((*nx, *ny));
                next_newly.push((*nx, *ny));
            }
        }

        newly_activated = next_newly;
        if newly_activated.is_empty() {
            break; // No new activations — propagation has converged
        }
    }

    // 3. Read out activated cells
    let min_activation = 0.08;
    let generated = scan_active_knowledge(&scratch, min_activation);

    let mut keyword_freq: HashMap<String, usize> = HashMap::new();
    let mut cluster_texts: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for cell in &generated {
        if let Some(text) = text_store.peek(cell.position.0, cell.position.1) {
            if seen.contains(text) { continue; }
            seen.insert(text.to_string());

            // Truncate for display — find a safe char boundary
            let snippet = if text.len() > 80 {
                let mut cutoff = 80;
                while cutoff > 0 && !text.is_char_boundary(cutoff) { cutoff -= 1; }
                if cutoff > 0 {
                    format!("{}…", &text[..cutoff])
                } else {
                    text.to_string()
                }
            } else {
                text.to_string()
            };
            cluster_texts.push(snippet);

            // Extract keywords (simple tokenization, filter stop words)
            for word in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
                if word.len() >= 4 && !is_stop_word(word) {
                    *keyword_freq.entry(word.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    if cluster_texts.is_empty() && keyword_freq.is_empty() {
        return None;
    }

    // 4. Build the intuition summary
    let mut summary = String::from("## NCA Intuition\n");
    summary.push_str("(Associative patterns from neural grid propagation):\n\n");

    // Top keywords
    let mut top_keywords: Vec<_> = keyword_freq.into_iter().collect();
    top_keywords.sort_by(|a, b| b.1.cmp(&a.1));
    top_keywords.truncate(10);

    if !top_keywords.is_empty() {
        summary.push_str("**Associated concepts:** ");
        let kw_str = top_keywords
            .iter()
            .map(|(w, c)| format!("{} ({})", w, c))
            .collect::<Vec<_>>()
            .join(", ");
        summary.push_str(&kw_str);
        summary.push_str("\n\n");
    }

    // Top cluster texts
    cluster_texts.truncate(5);
    if !cluster_texts.is_empty() {
        summary.push_str("**Activated knowledge clusters:**\n");
        for (i, text) in cluster_texts.iter().enumerate() {
            summary.push_str(&format!("{}. {}\n", i + 1, text));
        }
    }

    if top_keywords.is_empty() && cluster_texts.is_empty() {
        return None;
    }

    Some(summary)
}