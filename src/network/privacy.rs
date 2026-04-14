//! Privacy Module — Differential privacy, aggregation thresholds, Sybil resistance,
//! PII filtering, and local-only knowledge support.

use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::diff::KnowledgeDiff;

// ─── Privacy Configuration ──────────────────────────────────────────────────

/// Privacy settings loaded from config.toml `[privacy]` section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Differential privacy epsilon (lower = more private, default 1.0).
    pub epsilon: f64,
    /// Sensitivity of a single update (max L2 norm of one conversation's grid changes).
    pub sensitivity: f64,
    /// Minimum conversations to accumulate before broadcasting a diff.
    pub min_conversations_before_sync: usize,
    /// Whether to filter PII from knowledge before encoding.
    pub filter_pii: bool,
    /// Grid channels reserved for local-only knowledge (excluded from diffs).
    pub local_only_channels: Vec<usize>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            epsilon: 1.0,
            sensitivity: 1.0,
            min_conversations_before_sync: 5,
            filter_pii: true,
            local_only_channels: vec![],
        }
    }
}

// ─── Differential Privacy ───────────────────────────────────────────────────

/// Add calibrated Gaussian noise to a KnowledgeDiff before broadcasting.
/// Uses the Gaussian mechanism: noise ~ N(0, σ²) where σ = sensitivity * √(2 ln(1.25/δ)) / ε.
/// We fix δ = 1e-5.
pub fn apply_differential_privacy(diff: &mut KnowledgeDiff, config: &PrivacyConfig) {
    if config.epsilon <= 0.0 {
        return;
    }
    let delta: f64 = 1e-5;
    let sigma = config.sensitivity * (2.0_f64 * (1.25_f64 / delta).ln()).sqrt() / config.epsilon;
    let mut rng = rand::thread_rng();

    for change in &mut diff.changes {
        let noise: f64 = sample_gaussian(&mut rng, 0.0, sigma);
        change.new_value += noise;
    }
}

/// Sample from a Gaussian distribution using Box-Muller transform.
fn sample_gaussian<R: Rng>(rng: &mut R, mean: f64, std_dev: f64) -> f64 {
    let u1: f64 = rng.gen_range(1e-10..1.0);
    let u2: f64 = rng.gen::<f64>() * std::f64::consts::TAU;
    mean + std_dev * (-2.0 * u1.ln()).sqrt() * u2.cos()
}

// ─── Aggregation Threshold ──────────────────────────────────────────────────

/// Tracks conversation count to enforce minimum aggregation before sync.
#[derive(Debug, Clone)]
pub struct AggregationTracker {
    conversations_since_last_sync: usize,
    threshold: usize,
}

impl AggregationTracker {
    pub fn new(threshold: usize) -> Self {
        Self {
            conversations_since_last_sync: 0,
            threshold,
        }
    }

    /// Record that a conversation happened. Returns true if threshold is met.
    pub fn record_conversation(&mut self) -> bool {
        self.conversations_since_last_sync += 1;
        self.conversations_since_last_sync >= self.threshold
    }

    /// Reset the counter after a sync.
    pub fn reset(&mut self) {
        self.conversations_since_last_sync = 0;
    }

    pub fn conversations_pending(&self) -> usize {
        self.conversations_since_last_sync
    }
}

// ─── Sybil Resistance — Proof of Knowledge ─────────────────────────────────

/// A knowledge challenge issued to new peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChallenge {
    /// Unique challenge ID.
    pub challenge_id: String,
    /// The query the peer must answer.
    pub query: String,
    /// Timestamp when issued.
    pub issued_at_ms: u64,
    /// Expected minimum response length (bytes).
    pub min_response_len: usize,
}

/// Response to a knowledge challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub node_id: String,
    pub response: String,
    pub response_hash: [u8; 32],
}

/// Result of verifying a challenge response.
#[derive(Debug, Clone, PartialEq)]
pub enum ChallengeResult {
    /// Peer demonstrated non-trivial knowledge.
    Passed,
    /// Response was too short or trivial.
    Failed(String),
}

/// Predefined challenge queries for Sybil resistance.
const CHALLENGE_QUERIES: &[&str] = &[
    "What do you know about machine learning?",
    "Explain the concept of neural networks.",
    "What is knowledge representation?",
    "Describe how distributed systems work.",
    "What are cellular automata?",
];

impl KnowledgeChallenge {
    /// Create a new random challenge.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..CHALLENGE_QUERIES.len());
        let challenge_id = format!("challenge-{}", rng.gen::<u64>());
        Self {
            challenge_id,
            query: CHALLENGE_QUERIES[idx].to_string(),
            issued_at_ms: now_ms(),
            min_response_len: 50,
        }
    }

    /// Verify a challenge response.
    pub fn verify(&self, response: &ChallengeResponse) -> ChallengeResult {
        if response.challenge_id != self.challenge_id {
            return ChallengeResult::Failed("Challenge ID mismatch".into());
        }

        // Check minimum length
        if response.response.len() < self.min_response_len {
            return ChallengeResult::Failed(format!(
                "Response too short: {} bytes (min {})",
                response.response.len(),
                self.min_response_len
            ));
        }

        // Check for non-trivial content: must have reasonable word diversity
        let words: HashSet<&str> = response.response.split_whitespace().collect();
        if words.len() < 5 {
            return ChallengeResult::Failed("Response lacks word diversity".into());
        }

        // Check that it's not just repeated characters
        let unique_chars: HashSet<char> = response.response.chars().collect();
        if unique_chars.len() < 10 {
            return ChallengeResult::Failed("Response lacks character diversity".into());
        }

        ChallengeResult::Passed
    }
}

// ─── PII Filtering ──────────────────────────────────────────────────────────

/// Filter PII patterns from text before encoding into the knowledge grid.
pub struct PiiFilter {
    email_re: Regex,
    phone_re: Regex,
    ssn_re: Regex,
    blocklist_hashes: HashSet<u64>,
}

impl PiiFilter {
    pub fn new() -> Self {
        Self {
            email_re: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            phone_re: Regex::new(r"\b(\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b").unwrap(),
            ssn_re: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
            blocklist_hashes: HashSet::new(),
        }
    }

    /// Add a blocked content hash.
    pub fn add_blocklist_hash(&mut self, hash: u64) {
        self.blocklist_hashes.insert(hash);
    }

    /// Check if text contains PII patterns. Returns the filtered text.
    pub fn filter(&self, text: &str) -> String {
        let mut result = self.email_re.replace_all(text, "[EMAIL]").to_string();
        result = self.phone_re.replace_all(&result, "[PHONE]").to_string();
        result = self.ssn_re.replace_all(&result, "[SSN]").to_string();
        result
    }

    /// Check if text contains any PII.
    pub fn contains_pii(&self, text: &str) -> bool {
        self.email_re.is_match(text) || self.phone_re.is_match(text) || self.ssn_re.is_match(text)
    }

    /// Check text against blocklist (simple hash).
    pub fn is_blocked(&self, text: &str) -> bool {
        let hash = simple_hash(text);
        self.blocklist_hashes.contains(&hash)
    }
}

impl Default for PiiFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn simple_hash(text: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in text.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

// ─── Local-only Knowledge ───────────────────────────────────────────────────

/// Filter diff changes to exclude local-only channels.
pub fn filter_local_only_channels(diff: &mut KnowledgeDiff, local_channels: &[usize]) {
    if local_channels.is_empty() {
        return;
    }
    let local_set: HashSet<usize> = local_channels.iter().copied().collect();
    diff.changes.retain(|c| !local_set.contains(&c.channel));
}

/// Default local-only channel index (channel 7 reserved for private knowledge).
pub const PRIVATE_CHANNEL: usize = 7;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::diff::CellChange;

    fn make_diff(changes: Vec<CellChange>) -> KnowledgeDiff {
        KnowledgeDiff {
            source_node: "test-node".into(),
            sequence: 0,
            width: 4,
            height: 4,
            num_channels: 8,
            changes,
            state_hash: [0u8; 32],
            confidence: 1.0,
            timestamp_ms: 0,
            signer_public_key: None,
            signature: None,
        }
    }

    #[test]
    fn test_differential_privacy_adds_noise() {
        let changes = vec![
            CellChange::new(0, 0, 0, 0.0, 1.0),
            CellChange::new(1, 1, 1, 0.0, 2.0),
        ];
        let mut diff = make_diff(changes);
        let config = PrivacyConfig {
            epsilon: 1.0,
            sensitivity: 1.0,
            ..Default::default()
        };

        let original_values: Vec<f64> = diff.changes.iter().map(|c| c.new_value).collect();
        apply_differential_privacy(&mut diff, &config);

        // Values should be changed (with overwhelming probability)
        let noised_values: Vec<f64> = diff.changes.iter().map(|c| c.new_value).collect();
        let any_changed = original_values
            .iter()
            .zip(noised_values.iter())
            .any(|(o, n)| (o - n).abs() > 1e-15);
        assert!(any_changed, "Differential privacy should add noise");
    }

    #[test]
    fn test_dp_higher_epsilon_less_noise() {
        // With very high epsilon, noise should be small
        let changes = vec![CellChange::new(0, 0, 0, 0.0, 100.0)];
        let mut diff = make_diff(changes);
        let config = PrivacyConfig {
            epsilon: 1000.0,
            sensitivity: 0.001,
            ..Default::default()
        };
        apply_differential_privacy(&mut diff, &config);
        // With huge epsilon and tiny sensitivity, noise is negligible
        assert!((diff.changes[0].new_value - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_aggregation_tracker() {
        let mut tracker = AggregationTracker::new(3);
        assert!(!tracker.record_conversation());
        assert!(!tracker.record_conversation());
        assert!(tracker.record_conversation()); // 3rd conversation — threshold met
        tracker.reset();
        assert_eq!(tracker.conversations_pending(), 0);
        assert!(!tracker.record_conversation());
    }

    #[test]
    fn test_pii_filter_email() {
        let filter = PiiFilter::new();
        assert!(filter.contains_pii("Contact me at user@example.com"));
        let filtered = filter.filter("Email: user@example.com please");
        assert!(filtered.contains("[EMAIL]"));
        assert!(!filtered.contains("user@example.com"));
    }

    #[test]
    fn test_pii_filter_phone() {
        let filter = PiiFilter::new();
        assert!(filter.contains_pii("Call 555-123-4567"));
        let filtered = filter.filter("My number is (555) 123-4567");
        assert!(filtered.contains("[PHONE]"));
    }

    #[test]
    fn test_pii_filter_ssn() {
        let filter = PiiFilter::new();
        assert!(filter.contains_pii("SSN: 123-45-6789"));
        let filtered = filter.filter("My SSN is 123-45-6789 ok");
        assert!(filtered.contains("[SSN]"));
        assert!(!filtered.contains("123-45-6789"));
    }

    #[test]
    fn test_pii_filter_no_pii() {
        let filter = PiiFilter::new();
        assert!(!filter.contains_pii("Hello world, this is just text."));
    }

    #[test]
    fn test_challenge_verify_pass() {
        let challenge = KnowledgeChallenge {
            challenge_id: "test-1".into(),
            query: "What do you know about AI?".into(),
            issued_at_ms: 0,
            min_response_len: 50,
        };
        let response = ChallengeResponse {
            challenge_id: "test-1".into(),
            node_id: "peer-a".into(),
            response: "Artificial intelligence encompasses machine learning, neural networks, \
                        natural language processing, and many other subfields of computer science."
                .into(),
            response_hash: [0u8; 32],
        };
        assert_eq!(challenge.verify(&response), ChallengeResult::Passed);
    }

    #[test]
    fn test_challenge_verify_too_short() {
        let challenge = KnowledgeChallenge {
            challenge_id: "test-2".into(),
            query: "Explain neural nets.".into(),
            issued_at_ms: 0,
            min_response_len: 50,
        };
        let response = ChallengeResponse {
            challenge_id: "test-2".into(),
            node_id: "peer-b".into(),
            response: "I dunno".into(),
            response_hash: [0u8; 32],
        };
        assert!(matches!(
            challenge.verify(&response),
            ChallengeResult::Failed(_)
        ));
    }

    #[test]
    fn test_challenge_id_mismatch() {
        let challenge = KnowledgeChallenge {
            challenge_id: "test-3".into(),
            query: "test".into(),
            issued_at_ms: 0,
            min_response_len: 10,
        };
        let response = ChallengeResponse {
            challenge_id: "wrong-id".into(),
            node_id: "peer".into(),
            response:
                "some long response with enough words and characters to pass other checks easily"
                    .into(),
            response_hash: [0u8; 32],
        };
        assert!(matches!(
            challenge.verify(&response),
            ChallengeResult::Failed(_)
        ));
    }

    #[test]
    fn test_local_only_channel_filtering() {
        let changes = vec![
            CellChange::new(0, 0, 0, 0.0, 1.0),
            CellChange::new(0, 0, PRIVATE_CHANNEL, 0.0, 2.0),
            CellChange::new(1, 1, 3, 0.0, 3.0),
        ];
        let mut diff = make_diff(changes);
        filter_local_only_channels(&mut diff, &[PRIVATE_CHANNEL]);
        assert_eq!(diff.changes.len(), 2);
        assert!(diff.changes.iter().all(|c| c.channel != PRIVATE_CHANNEL));
    }

    #[test]
    fn test_blocklist() {
        let mut filter = PiiFilter::new();
        filter.add_blocklist_hash(simple_hash("banned content"));
        assert!(filter.is_blocked("banned content"));
        assert!(!filter.is_blocked("normal content"));
    }
}
