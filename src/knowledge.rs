// Knowledge Discovery Module - AGI-driven insights about the simulated world
//
// This module enables the AGI to discover emergent details about settlements,
// resources, practices, and culture through autonomous probe agents.

use crate::civilization::{CivilizationSimulator, SettlementType};
use rand::Rng;
use std::collections::HashMap;

// Discovered knowledge about what's being extracted/produced
#[derive(Clone, Debug)]
pub struct ResourceDiscovery {
    pub settlement_id: usize,
    pub resource_type: String,  // e.g., "Iron Ore", "Wheat", "Tuna"
    pub quantity: String,       // e.g., "abundant", "scarce", "moderate"
    pub discovered_at_tick: usize,
}

// Discovered cultural practices and rituals
#[derive(Clone, Debug)]
pub struct CulturalPractice {
    pub settlement_id: usize,
    pub practice_name: String,
    pub description: String,
    pub frequency: String,  // e.g., "daily", "seasonal", "annual"
    pub discovered_at_tick: usize,
}

// Discovered trade goods being exchanged
#[derive(Clone, Debug)]
pub struct TradeGood {
    pub route_id: usize,
    pub good_name: String,
    pub from_settlement: usize,
    pub to_settlement: usize,
    pub volume: String,  // e.g., "high", "medium", "low"
    pub discovered_at_tick: usize,
}

// Discovered vocabulary/terminology specific to settlements
#[derive(Clone, Debug)]
pub struct LocalVocabulary {
    pub settlement_id: usize,
    pub word: String,
    pub meaning: String,
    pub usage_count: usize,
    pub discovered_at_tick: usize,
}

// HYPOTHESIS-DRIVEN DISCOVERY: AGI generates theories and tests them
#[derive(Clone, Debug)]
pub struct Hypothesis {
    pub id: usize,
    pub theory: String,                    // e.g., "Mining towns produce ore"
    pub hypothesis_type: HypothesisType,   // What kind of hypothesis
    pub confidence: f64,                   // 0.0-1.0, starts at 0.5 (unknown)
    pub evidence_for: usize,               // Number of confirming observations
    pub evidence_against: usize,           // Number of refuting observations
    pub test_probes_spawned: usize,        // How many probes spawned to test this
    pub generated_at_tick: usize,
    pub last_tested_tick: usize,
    pub is_active: bool,                   // Still testing vs. concluded
}

// CAUSAL REASONING: Understanding WHY, not just WHAT
#[derive(Clone, Debug)]
pub struct CausalLink {
    pub cause: String,                     // What happened first
    pub effect: String,                    // What happened as a result
    pub strength: f64,                     // 0.0-1.0, how strong is the causal relationship
    pub observations: usize,               // How many times observed
    pub temporal_gap: usize,               // Ticks between cause and effect
    pub confidence: f64,                   // How sure we are this is causal (not just correlation)
}

#[derive(Clone, Debug)]
pub struct CausalModel {
    pub links: Vec<CausalLink>,
    pub next_link_id: usize,
}

// EMERGENT ABSTRACTION: AGI discovers its own categories
#[derive(Clone, Debug)]
pub struct DiscoveredCategory {
    pub category_id: usize,
    pub name: String,                      // Auto-generated descriptive name
    pub category_type: CategoryType,       // What kind of thing is this?
    pub members: Vec<usize>,               // IDs of things in this category
    pub centroid: Vec<f64>,                // Average feature vector
    pub defining_features: Vec<String>,    // What makes this category unique
    pub discovered_at_tick: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CategoryType {
    SettlementCluster,   // Discovered types of settlements
    ResourceCluster,     // Discovered types of resources
    PracticeCluster,     // Discovered types of cultural practices
}

#[derive(Clone, Debug)]
pub struct AbstractionEngine {
    pub categories: Vec<DiscoveredCategory>,
    pub next_category_id: usize,
    pub last_clustering_tick: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HypothesisType {
    ResourcePattern,     // "Settlement type X produces resource Y"
    CulturalSpread,      // "Practice X spreads via trade routes"
    TradePattern,        // "Good X flows from biome Y to biome Z"
    PopulationDriver,    // "Resource X drives population growth"
    VocabularyDiffusion, // "Word X spreads between connected settlements"
}

// CURIOSITY-DRIVEN LEARNING: Questions the AGI couldn't answer guide future exploration
#[derive(Clone, Debug)]
pub struct UnansweredQuestion {
    pub question: String,
    pub concepts: Vec<String>,            // What concepts were involved (settlement, trade, etc.)
    pub confidence_when_asked: f64,       // How confident was the answer (if any)
    pub asked_at_tick: usize,
    pub times_asked: usize,               // Track if user keeps asking
    pub exploration_priority: f64,        // Higher = more important to explore (0.0-1.0)
}

// Central knowledge base storing all discoveries
#[derive(Clone, Debug)]
pub struct KnowledgeBase {
    pub resources: Vec<ResourceDiscovery>,
    pub practices: Vec<CulturalPractice>,
    pub trade_goods: Vec<TradeGood>,
    pub vocabulary: Vec<LocalVocabulary>,
    pub hypotheses: Vec<Hypothesis>,
    pub next_hypothesis_id: usize,
    pub causal_model: CausalModel,
    pub abstraction: AbstractionEngine,
    pub unanswered_questions: Vec<UnansweredQuestion>,  // Questions to explore
    pub discovery_count: usize,
    pub current_tick: usize,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        KnowledgeBase {
            resources: Vec::new(),
            practices: Vec::new(),
            trade_goods: Vec::new(),
            vocabulary: Vec::new(),
            hypotheses: Vec::new(),
            next_hypothesis_id: 0,
            causal_model: CausalModel {
                links: Vec::new(),
                next_link_id: 0,
            },
            abstraction: AbstractionEngine {
                categories: Vec::new(),
                next_category_id: 0,
                last_clustering_tick: 0,
            },
            unanswered_questions: Vec::new(),
            discovery_count: 0,
            current_tick: 0,
        }
    }

    pub fn add_resource(&mut self, discovery: ResourceDiscovery) {
        self.resources.push(discovery);
        self.discovery_count += 1;
    }

    pub fn add_practice(&mut self, practice: CulturalPractice) {
        self.practices.push(practice);
        self.discovery_count += 1;
    }

    pub fn add_trade_good(&mut self, good: TradeGood) {
        self.trade_goods.push(good);
        self.discovery_count += 1;
    }

    pub fn add_vocabulary(&mut self, vocab: LocalVocabulary) {
        self.vocabulary.push(vocab);
        self.discovery_count += 1;
    }

    // Query top N most common resources
    pub fn top_resources(&self, n: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for resource in &self.resources {
            *counts.entry(resource.resource_type.clone()).or_insert(0) += 1;
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    // Query top N most traded words
    pub fn top_traded_words(&self, n: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for vocab in &self.vocabulary {
            *counts.entry(vocab.word.clone()).or_insert(0) += vocab.usage_count;
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    // Get practices for a specific settlement
    pub fn practices_for_settlement(&self, settlement_id: usize) -> Vec<&CulturalPractice> {
        self.practices.iter()
            .filter(|p| p.settlement_id == settlement_id)
            .collect()
    }

    // SELF-DIRECTED EXPLORATION: Analyze knowledge gaps
    // Returns (area, urgency) where urgency is 0-1 (1 = critical gap)
    pub fn analyze_knowledge_gaps(&self, civ: &CivilizationSimulator) -> Vec<(String, f64)> {
        let mut gaps = Vec::new();

        let total_settlements = civ.settlements.len();
        if total_settlements == 0 {
            return gaps;
        }

        // Gap 1: Resource coverage - do we know what each settlement produces?
        let settlements_with_resources: std::collections::HashSet<usize> =
            self.resources.iter().map(|r| r.settlement_id).collect();
        let resource_coverage = settlements_with_resources.len() as f64 / total_settlements as f64;
        if resource_coverage < 0.8 {
            gaps.push(("Resource discovery".to_string(), 1.0 - resource_coverage));
        }

        // Gap 2: Cultural knowledge - do we understand their practices?
        let settlements_with_practices: std::collections::HashSet<usize> =
            self.practices.iter().map(|p| p.settlement_id).collect();
        let culture_coverage = settlements_with_practices.len() as f64 / total_settlements as f64;
        if culture_coverage < 0.7 {
            gaps.push(("Cultural understanding".to_string(), 1.0 - culture_coverage));
        }

        // Gap 3: Trade knowledge - do we know what's being exchanged?
        let total_routes = civ.trade_routes.len();
        let routes_with_goods: std::collections::HashSet<usize> =
            self.trade_goods.iter().map(|g| g.route_id).collect();
        if total_routes > 0 {
            let trade_coverage = routes_with_goods.len() as f64 / total_routes as f64;
            if trade_coverage < 0.6 {
                gaps.push(("Trade good knowledge".to_string(), 1.0 - trade_coverage));
            }
        }

        // Gap 4: Vocabulary richness - do we understand local terminology?
        let settlements_with_vocab: std::collections::HashSet<usize> =
            self.vocabulary.iter().map(|v| v.settlement_id).collect();
        let vocab_coverage = settlements_with_vocab.len() as f64 / total_settlements as f64;
        if vocab_coverage < 0.5 {
            gaps.push(("Vocabulary acquisition".to_string(), 1.0 - vocab_coverage));
        }

        // Gap 5: Depth of knowledge - do we have multiple discoveries per settlement?
        let avg_discoveries_per_settlement =
            (self.resources.len() + self.practices.len() + self.vocabulary.len()) as f64
            / total_settlements as f64;
        if avg_discoveries_per_settlement < 3.0 {
            gaps.push(("Knowledge depth".to_string(), 1.0 - (avg_discoveries_per_settlement / 3.0).min(1.0)));
        }

        // Sort by urgency (highest first)
        gaps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        gaps
    }

    // Get settlements that need more exploration
    pub fn get_underexplored_settlements(&self, civ: &CivilizationSimulator) -> Vec<usize> {
        let mut settlement_discovery_counts: Vec<(usize, usize)> = Vec::new();

        for (sid, _) in civ.settlements.iter().enumerate() {
            let count = self.resources.iter().filter(|r| r.settlement_id == sid).count()
                + self.practices.iter().filter(|p| p.settlement_id == sid).count()
                + self.vocabulary.iter().filter(|v| v.settlement_id == sid).count();
            settlement_discovery_counts.push((sid, count));
        }

        // Sort by discovery count (least explored first)
        settlement_discovery_counts.sort_by_key(|(_, count)| *count);

        // Return bottom 30% (most underexplored)
        let threshold = (settlement_discovery_counts.len() * 3) / 10;
        settlement_discovery_counts.iter()
            .take(threshold.max(1))
            .map(|(sid, _)| *sid)
            .collect()
    }

    // HYPOTHESIS-DRIVEN DISCOVERY: Generate theories based on discovered patterns
    pub fn generate_hypotheses(&mut self, civ: &CivilizationSimulator) -> Vec<Hypothesis> {
        let mut new_hypotheses = Vec::new();
        let mut rng = rand::thread_rng();

        // Only generate hypotheses if we have enough data (at least 10 discoveries)
        if self.discovery_count < 10 {
            return new_hypotheses;
        }

        // Don't generate too many active hypotheses at once (max 10)
        let active_count = self.hypotheses.iter().filter(|h| h.is_active).count();
        if active_count >= 10 {
            return new_hypotheses;
        }

        // H1: Resource Pattern Hypothesis - "Settlement type X tends to produce resource Y"
        if rng.gen::<f64>() < 0.3 && self.resources.len() >= 5 {
            // Find most common resource
            let mut resource_counts: HashMap<String, usize> = HashMap::new();
            for res in &self.resources {
                *resource_counts.entry(res.resource_type.clone()).or_insert(0) += 1;
            }
            if let Some((resource, count)) = resource_counts.iter().max_by_key(|(_, c)| *c) {
                if *count >= 3 {
                    // Find which settlement type produces it most
                    let settlements_with_resource: Vec<_> = self.resources.iter()
                        .filter(|r| &r.resource_type == resource)
                        .map(|r| r.settlement_id)
                        .collect();

                    if !settlements_with_resource.is_empty() {
                        let first_sid = settlements_with_resource[0];
                        let settlement_type = &civ.settlements[first_sid].settlement_type;
                        let type_name = match settlement_type {
                            SettlementType::Village => "villages",
                            SettlementType::MiningTown => "mining towns",
                            SettlementType::FishingPort => "fishing ports",
                            SettlementType::TradeHub => "trade hubs",
                        };

                        let theory = format!("{} tend to produce {}", type_name, resource);
                        new_hypotheses.push(Hypothesis {
                            id: self.next_hypothesis_id,
                            theory,
                            hypothesis_type: HypothesisType::ResourcePattern,
                            confidence: 0.5,
                            evidence_for: 0,
                            evidence_against: 0,
                            test_probes_spawned: 0,
                            generated_at_tick: self.current_tick,
                            last_tested_tick: self.current_tick,
                            is_active: true,
                        });
                        self.next_hypothesis_id += 1;
                    }
                }
            }
        }

        // H2: Cultural Spread Hypothesis - "Practice X spreads via trade routes"
        if rng.gen::<f64>() < 0.25 && self.practices.len() >= 4 && !civ.trade_routes.is_empty() {
            // Find practices that appear in multiple settlements
            let mut practice_locations: HashMap<String, Vec<usize>> = HashMap::new();
            for practice in &self.practices {
                practice_locations.entry(practice.practice_name.clone())
                    .or_insert_with(Vec::new)
                    .push(practice.settlement_id);
            }

            // Find a practice in at least 2 settlements
            if let Some((practice_name, locations)) = practice_locations.iter()
                .find(|(_, locs)| locs.len() >= 2) {

                // Check if those settlements are connected by trade routes
                let connected = civ.trade_routes.iter().any(|route| {
                    (locations.contains(&route.settlement_a) && locations.contains(&route.settlement_b))
                    || (locations.contains(&route.settlement_b) && locations.contains(&route.settlement_a))
                });

                if connected {
                    let theory = format!("'{}' practice spreads via trade routes", practice_name);
                    new_hypotheses.push(Hypothesis {
                        id: self.next_hypothesis_id,
                        theory,
                        hypothesis_type: HypothesisType::CulturalSpread,
                        confidence: 0.5,
                        evidence_for: 0,
                        evidence_against: 0,
                        test_probes_spawned: 0,
                        generated_at_tick: self.current_tick,
                        last_tested_tick: self.current_tick,
                        is_active: true,
                    });
                    self.next_hypothesis_id += 1;
                }
            }
        }

        // H3: Trade Pattern Hypothesis - "Good X flows from settlement type Y to Z"
        if rng.gen::<f64>() < 0.2 && self.trade_goods.len() >= 3 {
            // Find most traded good
            let mut good_counts: HashMap<String, usize> = HashMap::new();
            for good in &self.trade_goods {
                *good_counts.entry(good.good_name.clone()).or_insert(0) += 1;
            }

            if let Some((good, count)) = good_counts.iter().max_by_key(|(_, c)| *c) {
                if *count >= 2 {
                    // Find typical flow direction
                    if let Some(trade) = self.trade_goods.iter().find(|g| &g.good_name == good) {
                        let from_type = &civ.settlements[trade.from_settlement].settlement_type;
                        let to_type = &civ.settlements[trade.to_settlement].settlement_type;
                        let from_name = match from_type {
                            SettlementType::Village => "villages",
                            SettlementType::MiningTown => "mining towns",
                            SettlementType::FishingPort => "fishing ports",
                            SettlementType::TradeHub => "trade hubs",
                        };
                        let to_name = match to_type {
                            SettlementType::Village => "villages",
                            SettlementType::MiningTown => "mining towns",
                            SettlementType::FishingPort => "fishing ports",
                            SettlementType::TradeHub => "trade hubs",
                        };

                        let theory = format!("{} typically flows from {} to {}", good, from_name, to_name);
                        new_hypotheses.push(Hypothesis {
                            id: self.next_hypothesis_id,
                            theory,
                            hypothesis_type: HypothesisType::TradePattern,
                            confidence: 0.5,
                            evidence_for: 0,
                            evidence_against: 0,
                            test_probes_spawned: 0,
                            generated_at_tick: self.current_tick,
                            last_tested_tick: self.current_tick,
                            is_active: true,
                        });
                        self.next_hypothesis_id += 1;
                    }
                }
            }
        }

        // Add generated hypotheses to knowledge base
        self.hypotheses.extend(new_hypotheses.clone());
        new_hypotheses
    }

    // Test active hypotheses by examining new evidence
    pub fn test_hypotheses(&mut self, civ: &CivilizationSimulator) {
        for hypothesis in &mut self.hypotheses {
            if !hypothesis.is_active {
                continue;
            }

            // Test based on hypothesis type
            match hypothesis.hypothesis_type {
                HypothesisType::ResourcePattern => {
                    // Check if recent resource discoveries match the pattern
                    // Theory format: "X tend to produce Y"
                    let recent_resources: Vec<_> = self.resources.iter()
                        .filter(|r| r.discovered_at_tick > hypothesis.last_tested_tick)
                        .collect();

                    for resource in recent_resources {
                        // Safety check: ensure settlement still exists
                        if resource.settlement_id >= civ.settlements.len() {
                            continue;
                        }

                        let settlement_type = &civ.settlements[resource.settlement_id].settlement_type;
                        let type_name = match settlement_type {
                            SettlementType::Village => "villages",
                            SettlementType::MiningTown => "mining towns",
                            SettlementType::FishingPort => "fishing ports",
                            SettlementType::TradeHub => "trade hubs",
                        };

                        // Does this discovery support or refute the hypothesis?
                        if hypothesis.theory.contains(type_name) && hypothesis.theory.contains(&resource.resource_type) {
                            hypothesis.evidence_for += 1;
                        } else if hypothesis.theory.contains(type_name) {
                            // Same settlement type but different resource
                            hypothesis.evidence_against += 1;
                        }
                    }
                },

                HypothesisType::CulturalSpread => {
                    // Check if practices appear in connected settlements
                    let recent_practices: Vec<_> = self.practices.iter()
                        .filter(|p| p.discovered_at_tick > hypothesis.last_tested_tick)
                        .collect();

                    for practice in recent_practices {
                        if hypothesis.theory.contains(&practice.practice_name) {
                            // Check if this settlement is connected to others with same practice
                            let other_settlements_with_practice: Vec<_> = self.practices.iter()
                                .filter(|p| p.practice_name == practice.practice_name && p.settlement_id != practice.settlement_id)
                                .map(|p| p.settlement_id)
                                .collect();

                            let is_connected = other_settlements_with_practice.iter().any(|&other_sid| {
                                civ.trade_routes.iter().any(|route| {
                                    (route.settlement_a == practice.settlement_id && route.settlement_b == other_sid)
                                    || (route.settlement_b == practice.settlement_id && route.settlement_a == other_sid)
                                })
                            });

                            if is_connected {
                                hypothesis.evidence_for += 1;
                            } else {
                                hypothesis.evidence_against += 1;
                            }
                        }
                    }
                },

                HypothesisType::TradePattern => {
                    // Check if trade goods flow in the predicted direction
                    let recent_trades: Vec<_> = self.trade_goods.iter()
                        .filter(|t| t.discovered_at_tick > hypothesis.last_tested_tick)
                        .collect();

                    for trade in recent_trades {
                        // Safety check: ensure settlements still exist
                        if trade.from_settlement >= civ.settlements.len() || trade.to_settlement >= civ.settlements.len() {
                            continue;
                        }

                        let from_type = &civ.settlements[trade.from_settlement].settlement_type;
                        let to_type = &civ.settlements[trade.to_settlement].settlement_type;
                        let from_name = match from_type {
                            SettlementType::Village => "villages",
                            SettlementType::MiningTown => "mining towns",
                            SettlementType::FishingPort => "fishing ports",
                            SettlementType::TradeHub => "trade hubs",
                        };
                        let to_name = match to_type {
                            SettlementType::Village => "villages",
                            SettlementType::MiningTown => "mining towns",
                            SettlementType::FishingPort => "fishing ports",
                            SettlementType::TradeHub => "trade hubs",
                        };

                        if hypothesis.theory.contains(&trade.good_name)
                            && hypothesis.theory.contains(from_name)
                            && hypothesis.theory.contains(to_name) {
                            hypothesis.evidence_for += 1;
                        } else if hypothesis.theory.contains(&trade.good_name) {
                            hypothesis.evidence_against += 1;
                        }
                    }
                },

                _ => {
                    // Other hypothesis types not yet implemented
                }
            }

            // Update confidence based on evidence ratio
            let total_evidence = hypothesis.evidence_for + hypothesis.evidence_against;
            if total_evidence > 0 {
                hypothesis.confidence = hypothesis.evidence_for as f64 / total_evidence as f64;
            }

            // Conclude hypothesis if we have enough evidence (at least 5 data points)
            if total_evidence >= 5 {
                hypothesis.is_active = false;
            }

            hypothesis.last_tested_tick = self.current_tick;
        }
    }

    // Get confirmed hypotheses (high confidence, concluded)
    pub fn get_confirmed_theories(&self) -> Vec<&Hypothesis> {
        self.hypotheses.iter()
            .filter(|h| !h.is_active && h.confidence > 0.7)
            .collect()
    }

    // Get refuted hypotheses (low confidence, concluded)
    pub fn get_refuted_theories(&self) -> Vec<&Hypothesis> {
        self.hypotheses.iter()
            .filter(|h| !h.is_active && h.confidence < 0.3)
            .collect()
    }

    // CAUSAL REASONING: Infer cause-effect relationships from temporal patterns
    pub fn infer_causality(&mut self, civ: &CivilizationSimulator) {
        // Look for temporal patterns: if A happens, then B often happens shortly after
        // Collect links to add to avoid borrow checker issues
        let mut links_to_add: Vec<(String, String, usize)> = Vec::new();

        // Pattern 1: Trade route → Cultural spread
        // Check if trade routes cause cultural practices to spread
        for route in &civ.trade_routes {
            let settlements = [route.settlement_a, route.settlement_b];

            // Find practices that appeared in one settlement, then the other
            for practice in &self.practices {
                let other_sid = if practice.settlement_id == settlements[0] { settlements[1] } else { settlements[0] };

                // Check if the same practice appeared in the other settlement later
                let later_practice = self.practices.iter()
                    .find(|p| p.settlement_id == other_sid
                           && p.practice_name == practice.practice_name
                           && p.discovered_at_tick > practice.discovered_at_tick);

                if let Some(later) = later_practice {
                    let temporal_gap = later.discovered_at_tick - practice.discovered_at_tick;

                    // Infer causal link: trade route → practice spread
                    let cause = format!("Trade route between settlements {} and {}", settlements[0], settlements[1]);
                    let effect = format!("'{}' practice spreads to connected settlement", practice.practice_name);

                    links_to_add.push((cause, effect, temporal_gap));
                }
            }
        }

        // Pattern 2: Resource discovery → Trade
        // When a settlement discovers a resource, it often starts trading it
        for resource in &self.resources {
            // Find trades from this settlement that occurred after the resource discovery
            let later_trades: Vec<_> = self.trade_goods.iter()
                .filter(|t| (t.from_settlement == resource.settlement_id || t.to_settlement == resource.settlement_id)
                         && t.discovered_at_tick > resource.discovered_at_tick
                         && t.good_name.contains(&resource.resource_type))
                .collect();

            for trade in later_trades {
                let temporal_gap = trade.discovered_at_tick - resource.discovered_at_tick;

                let cause = format!("Settlement {} discovers {}", resource.settlement_id, resource.resource_type);
                let effect = format!("{} becomes a traded good", resource.resource_type);

                links_to_add.push((cause, effect, temporal_gap));
            }
        }

        // Now add all the links
        for (cause, effect, temporal_gap) in links_to_add {
            self.add_or_strengthen_causal_link(cause, effect, temporal_gap);
        }

        // Pattern 3: Population growth → Resource scarcity
        // (Would need population tracking over time - simplified version)
        // This is a placeholder for future causal inference patterns
    }

    fn add_or_strengthen_causal_link(&mut self, cause: String, effect: String, temporal_gap: usize) {
        // Check if this link already exists
        if let Some(link) = self.causal_model.links.iter_mut()
            .find(|l| l.cause == cause && l.effect == effect) {
            // Strengthen existing link
            link.observations += 1;
            link.strength = (link.strength * (link.observations - 1) as f64 + 1.0) / link.observations as f64;
            link.temporal_gap = ((link.temporal_gap * (link.observations - 1)) + temporal_gap) / link.observations;
            // Confidence increases with more observations (using logarithmic growth)
            link.confidence = (link.observations as f64 / (link.observations as f64 + 2.0)).min(0.95);
        } else {
            // Create new causal link
            self.causal_model.links.push(CausalLink {
                cause,
                effect,
                strength: 0.5,  // Initial strength
                observations: 1,
                temporal_gap,
                confidence: 0.33,  // Low confidence initially
            });
            self.causal_model.next_link_id += 1;
        }
    }

    // Get strong causal relationships (high confidence)
    pub fn get_causal_explanations(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut explanations = Vec::new();

        // Find causal links related to the query
        for link in &self.causal_model.links {
            if link.confidence > 0.6 &&
               (link.cause.to_lowercase().contains(&query_lower) ||
                link.effect.to_lowercase().contains(&query_lower)) {
                explanations.push(format!(
                    "BECAUSE: {} → {} (confidence: {:.0}%, observed {} times, ~{} ticks apart)",
                    link.cause, link.effect, link.confidence * 100.0, link.observations, link.temporal_gap
                ));
            }
        }

        explanations
    }

    // EMERGENT ABSTRACTION: Discover settlement categories from behavior patterns
    pub fn discover_settlement_categories(&mut self, civ: &CivilizationSimulator) {
        // Only recluster periodically (expensive operation)
        if self.current_tick - self.abstraction.last_clustering_tick < 50 {
            return;
        }

        // Need enough settlements to cluster (at least 6 for 3 clusters)
        if civ.settlements.len() < 6 {
            return;
        }

        // Extract feature vectors for each settlement
        let mut features: Vec<Vec<f64>> = Vec::new();
        for (sid, settlement) in civ.settlements.iter().enumerate() {
            let feature_vec = self.extract_settlement_features(sid, settlement, civ);
            features.push(feature_vec);
        }

        // Perform k-means clustering (3 clusters)
        let num_clusters = 3.min(civ.settlements.len() / 2);
        let clusters = self.kmeans_cluster(&features, num_clusters);

        // Create discovered categories
        let mut new_categories = Vec::new();
        for (_cluster_id, member_indices) in clusters.iter().enumerate() {
            if member_indices.is_empty() {
                continue;
            }

            // Compute centroid
            let centroid = self.compute_centroid(&features, member_indices);

            // Generate descriptive name based on defining features
            let name = self.generate_category_name(&centroid, member_indices, civ);

            // Extract defining features
            let defining_features = self.extract_defining_features(&centroid);

            new_categories.push(DiscoveredCategory {
                category_id: self.abstraction.next_category_id,
                name,
                category_type: CategoryType::SettlementCluster,
                members: member_indices.clone(),
                centroid,
                defining_features,
                discovered_at_tick: self.current_tick,
            });

            self.abstraction.next_category_id += 1;
        }

        // Replace old settlement categories with new ones
        self.abstraction.categories.retain(|c| c.category_type != CategoryType::SettlementCluster);
        self.abstraction.categories.extend(new_categories);
        self.abstraction.last_clustering_tick = self.current_tick;
    }

    // Extract behavioral features from a settlement
    fn extract_settlement_features(&self, sid: usize, settlement: &crate::civilization::Settlement, civ: &CivilizationSimulator) -> Vec<f64> {
        let mut features = Vec::new();

        // Feature 1: Population (normalized 0-1)
        features.push((settlement.population as f64 / 1000.0).min(1.0));

        // Feature 2: Number of resources discovered
        let resource_count = self.resources.iter().filter(|r| r.settlement_id == sid).count();
        features.push((resource_count as f64 / 5.0).min(1.0));

        // Feature 3: Number of cultural practices
        let practice_count = self.practices.iter().filter(|p| p.settlement_id == sid).count();
        features.push((practice_count as f64 / 5.0).min(1.0));

        // Feature 4: Trade connectivity (how many routes)
        let route_count = civ.trade_routes.iter()
            .filter(|r| r.settlement_a == sid || r.settlement_b == sid)
            .count();
        features.push((route_count as f64 / 3.0).min(1.0));

        // Feature 5: Vocabulary diversity
        let vocab_count = self.vocabulary.iter().filter(|v| v.settlement_id == sid).count();
        features.push((vocab_count as f64 / 10.0).min(1.0));

        features
    }

    // Simple k-means clustering
    fn kmeans_cluster(&self, features: &[Vec<f64>], k: usize) -> Vec<Vec<usize>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if features.is_empty() || k == 0 {
            return vec![];
        }

        let feature_dim = features[0].len();
        let n = features.len();

        // Initialize centroids randomly
        let mut centroids: Vec<Vec<f64>> = Vec::new();
        for _ in 0..k {
            let idx = rng.gen_range(0..n);
            centroids.push(features[idx].clone());
        }

        // Run k-means for 10 iterations
        for _ in 0..10 {
            // Assign each point to nearest centroid
            let mut assignments = vec![0; n];
            for (i, feature) in features.iter().enumerate() {
                let mut min_dist = f64::MAX;
                let mut best_cluster = 0;

                for (c, centroid) in centroids.iter().enumerate() {
                    let dist = Self::euclidean_distance(feature, centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        best_cluster = c;
                    }
                }

                assignments[i] = best_cluster;
            }

            // Update centroids
            for c in 0..k {
                let cluster_points: Vec<&Vec<f64>> = assignments.iter()
                    .enumerate()
                    .filter(|(_, &cluster)| cluster == c)
                    .map(|(i, _)| &features[i])
                    .collect();

                if !cluster_points.is_empty() {
                    let mut new_centroid = vec![0.0; feature_dim];
                    for point in &cluster_points {
                        for (j, &val) in point.iter().enumerate() {
                            new_centroid[j] += val;
                        }
                    }
                    for val in &mut new_centroid {
                        *val /= cluster_points.len() as f64;
                    }
                    centroids[c] = new_centroid;
                }
            }
        }

        // Return final cluster assignments
        let mut clusters = vec![Vec::new(); k];
        for (i, feature) in features.iter().enumerate() {
            let mut min_dist = f64::MAX;
            let mut best_cluster = 0;

            for (c, centroid) in centroids.iter().enumerate() {
                let dist = Self::euclidean_distance(feature, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    best_cluster = c;
                }
            }

            clusters[best_cluster].push(i);
        }

        clusters
    }

    fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    fn compute_centroid(&self, features: &[Vec<f64>], indices: &[usize]) -> Vec<f64> {
        if indices.is_empty() || features.is_empty() {
            return vec![];
        }

        let dim = features[0].len();
        let mut centroid = vec![0.0; dim];

        for &idx in indices {
            if idx < features.len() {
                for (j, &val) in features[idx].iter().enumerate() {
                    centroid[j] += val;
                }
            }
        }

        for val in &mut centroid {
            *val /= indices.len() as f64;
        }

        centroid
    }

    // Generate a descriptive name for a discovered category
    fn generate_category_name(&self, centroid: &[f64], _members: &[usize], _civ: &CivilizationSimulator) -> String {
        // Features: [population, resources, practices, trade, vocabulary]
        if centroid.len() < 5 {
            return "Unknown Cluster".to_string();
        }

        let pop_score = centroid[0];
        let resource_score = centroid[1];
        let culture_score = centroid[2];
        let trade_score = centroid[3];
        let _vocab_score = centroid[4];

        // Determine primary characteristic
        let mut descriptors = Vec::new();

        if trade_score > 0.6 {
            descriptors.push("Trade-oriented");
        }
        if resource_score > 0.6 {
            descriptors.push("Resource-rich");
        }
        if culture_score > 0.6 {
            descriptors.push("Culturally diverse");
        }
        if pop_score > 0.7 {
            descriptors.push("Populous");
        } else if pop_score < 0.3 {
            descriptors.push("Small");
        }

        if descriptors.is_empty() {
            descriptors.push("Balanced");
        }

        format!("{} Settlements", descriptors.join(", "))
    }

    fn extract_defining_features(&self, centroid: &[f64]) -> Vec<String> {
        let mut features = Vec::new();
        let labels = ["Population", "Resources", "Culture", "Trade", "Vocabulary"];

        for (i, &val) in centroid.iter().enumerate() {
            if i < labels.len() {
                let level = if val > 0.7 { "High" } else if val > 0.4 { "Moderate" } else { "Low" };
                features.push(format!("{} {}", level, labels[i]));
            }
        }

        features
    }

    // Get discovered category for a settlement
    pub fn get_settlement_category(&self, settlement_id: usize) -> Option<&DiscoveredCategory> {
        self.abstraction.categories.iter()
            .filter(|c| c.category_type == CategoryType::SettlementCluster)
            .find(|c| c.members.contains(&settlement_id))
    }

    // ============ CURIOSITY-DRIVEN LEARNING ============
    // Record a question that couldn't be answered well

    /// Record a question that the AGI couldn't answer confidently
    /// This drives future exploration toward filling knowledge gaps
    pub fn record_unanswered_question(
        &mut self,
        question: String,
        concepts: Vec<String>,
        confidence: f64,
    ) {
        // Check if this question was already asked
        if let Some(existing) = self.unanswered_questions.iter_mut()
            .find(|q| q.question.to_lowercase() == question.to_lowercase()) {
            // Already asked - increase priority and count
            existing.times_asked += 1;
            existing.exploration_priority = (existing.exploration_priority + 0.1).min(1.0);
            return;
        }

        // Calculate initial exploration priority based on confidence
        // Lower confidence = higher priority to explore
        let priority = 1.0 - confidence;

        self.unanswered_questions.push(UnansweredQuestion {
            question: question.clone(),
            concepts,
            confidence_when_asked: confidence,
            asked_at_tick: self.current_tick,
            times_asked: 1,
            exploration_priority: priority,
        });
    }

    /// Get high-priority questions that need exploration
    pub fn get_exploration_priorities(&self) -> Vec<(String, Vec<String>, f64)> {
        let mut questions: Vec<_> = self.unanswered_questions.iter()
            .map(|q| (q.question.clone(), q.concepts.clone(), q.exploration_priority))
            .collect();

        // Sort by priority (highest first)
        questions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        questions
    }

    /// Mark a question as answered (confidence improved)
    pub fn mark_question_answered(&mut self, question: &str, new_confidence: f64) {
        if let Some(q) = self.unanswered_questions.iter_mut()
            .find(|q| q.question.to_lowercase() == question.to_lowercase()) {
            // Lower priority if confidence improved significantly
            if new_confidence > 0.7 {
                q.exploration_priority *= 0.5;
            }
        }

        // Remove questions that have been answered well
        self.unanswered_questions.retain(|q| q.exploration_priority > 0.2);
    }

    /// Get top concepts that need exploration based on unanswered questions
    pub fn get_top_exploration_concepts(&self) -> Vec<String> {
        let mut concept_scores: HashMap<String, f64> = HashMap::new();

        for q in &self.unanswered_questions {
            for concept in &q.concepts {
                *concept_scores.entry(concept.clone()).or_insert(0.0) += q.exploration_priority;
            }
        }

        let mut concepts: Vec<_> = concept_scores.into_iter().collect();
        concepts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        concepts.into_iter().take(5).map(|(c, _)| c).collect()
    }
}

// Probe types that discover knowledge about the world
#[derive(Clone, Copy, Debug)]
pub enum ProbeType {
    Resource,      // Discovers what's being extracted/produced
    Culture,       // Discovers rituals and practices
    Trade,         // Discovers what goods are being traded
    Vocabulary,    // Discovers local terminology
}

// A probe agent that discovers specific knowledge
pub struct DiscoveryProbe {
    pub probe_type: ProbeType,
    pub target_settlement: Option<usize>,
    pub target_route: Option<usize>,
    pub parent_discovery: Option<String>,  // What discovery triggered this probe
    pub recursion_depth: usize,             // How many probes deep in the chain
}

impl DiscoveryProbe {
    // Execute probe to discover knowledge and return follow-up probes
    // Returns: Vec<DiscoveryProbe> - recursive follow-up probes to explore deeper
    pub fn execute(&self, civ: &CivilizationSimulator, kb: &mut KnowledgeBase) -> Vec<DiscoveryProbe> {
        let max_recursion = 3;  // Don't go more than 3 levels deep
        let mut follow_up_probes = Vec::new();
        let mut rng = rand::thread_rng();

        match self.probe_type {
            ProbeType::Resource => {
                if let Some(sid) = self.target_settlement {
                    if let Some(settlement) = civ.settlements.get(sid) {
                        // Generate plausible resources based on settlement type
                        let resource = match settlement.settlement_type {
                            SettlementType::MiningTown => {
                                let ores = ["Iron Ore", "Copper Ore", "Gold Ore", "Silver Ore", "Coal", "Gemstones"];
                                ores[rng.gen_range(0..ores.len())]
                            },
                            SettlementType::FishingPort => {
                                let fish = ["Tuna", "Salmon", "Cod", "Mackerel", "Shellfish", "Seaweed"];
                                fish[rng.gen_range(0..fish.len())]
                            },
                            SettlementType::Village => {
                                let crops = ["Wheat", "Barley", "Corn", "Rice", "Vegetables", "Fruit"];
                                crops[rng.gen_range(0..crops.len())]
                            },
                            SettlementType::TradeHub => {
                                let goods = ["Textiles", "Spices", "Pottery", "Tools", "Furniture", "Jewelry"];
                                goods[rng.gen_range(0..goods.len())]
                            },
                        };

                        let quantities = ["abundant", "moderate", "scarce"];
                        let quantity = quantities[rng.gen_range(0..quantities.len())].to_string();
                        kb.add_resource(ResourceDiscovery {
                            settlement_id: sid,
                            resource_type: resource.to_string(),
                            quantity: quantity.clone(),
                            discovered_at_tick: kb.current_tick,
                        });

                        // RECURSIVE CURIOSITY: Resource discovered → Ask follow-up questions
                        if self.recursion_depth < max_recursion && rng.gen::<f64>() < 0.4 {
                            let discovery_description = format!("Found {} ({}) at settlement {}", resource, quantity, sid);

                            // "What tools/products are made from this resource?"
                            // Spawn a culture probe to discover practices related to this resource
                            follow_up_probes.push(DiscoveryProbe {
                                probe_type: ProbeType::Culture,
                                target_settlement: Some(sid),
                                target_route: None,
                                parent_discovery: Some(discovery_description.clone()),
                                recursion_depth: self.recursion_depth + 1,
                            });

                            // "Is this resource traded?" - Check if settlement has trade routes
                            let has_routes = civ.trade_routes.iter()
                                .any(|r| r.settlement_a == sid || r.settlement_b == sid);
                            if has_routes {
                                if let Some((rid, _)) = civ.trade_routes.iter().enumerate()
                                    .find(|(_, r)| r.settlement_a == sid || r.settlement_b == sid) {
                                    follow_up_probes.push(DiscoveryProbe {
                                        probe_type: ProbeType::Trade,
                                        target_settlement: None,
                                        target_route: Some(rid),
                                        parent_discovery: Some(format!("{} → Check if traded", discovery_description)),
                                        recursion_depth: self.recursion_depth + 1,
                                    });
                                }
                            }
                        }
                    }
                }
            },

            ProbeType::Culture => {
                if let Some(sid) = self.target_settlement {
                    if let Some(settlement) = civ.settlements.get(sid) {
                        // Generate plausible cultural practices
                        let (practice, desc, freq) = match settlement.settlement_type {
                            SettlementType::MiningTown => {
                                let practices = [
                                    ("Ore Blessing Ceremony", "Miners bless new veins before extraction", "weekly"),
                                    ("Forge Festival", "Celebration of metalworking craftsmanship", "annual"),
                                    ("Deep Song", "Traditional mining songs sung underground", "daily"),
                                ];
                                practices[rng.gen_range(0..practices.len())]
                            },
                            SettlementType::FishingPort => {
                                let practices = [
                                    ("Tide Prayer", "Morning prayers to the sea for safe fishing", "daily"),
                                    ("Net Mending Circle", "Community gathering to repair nets", "weekly"),
                                    ("Storm Dance", "Ritual dance to calm rough seas", "seasonal"),
                                ];
                                practices[rng.gen_range(0..practices.len())]
                            },
                            SettlementType::Village => {
                                let practices = [
                                    ("Harvest Festival", "Celebration of the autumn harvest", "annual"),
                                    ("Planting Ritual", "Blessing of seeds before spring planting", "seasonal"),
                                    ("Market Day", "Weekly gathering for trade and socialization", "weekly"),
                                ];
                                practices[rng.gen_range(0..practices.len())]
                            },
                            SettlementType::TradeHub => {
                                let practices = [
                                    ("Merchant's Oath", "Trading codes sworn at market opening", "daily"),
                                    ("Caravan Welcome", "Ceremonial greeting for arriving traders", "frequent"),
                                    ("Price Setting Council", "Weekly meeting to establish fair trade values", "weekly"),
                                ];
                                practices[rng.gen_range(0..practices.len())]
                            },
                        };

                        kb.add_practice(CulturalPractice {
                            settlement_id: sid,
                            practice_name: practice.to_string(),
                            description: desc.to_string(),
                            frequency: freq.to_string(),
                            discovered_at_tick: kb.current_tick,
                        });
                    }
                }
            },

            ProbeType::Trade => {
                if let Some(rid) = self.target_route {
                    if let Some(route) = civ.trade_routes.get(rid) {
                        let from_settlement = &civ.settlements[route.settlement_a];
                        let to_settlement = &civ.settlements[route.settlement_b];

                        // Determine plausible trade goods based on settlement types
                        let good = match (&from_settlement.settlement_type, &to_settlement.settlement_type) {
                            (SettlementType::MiningTown, SettlementType::FishingPort) => {
                                ["Metal Tools", "Iron", "Copper"][rng.gen_range(0..3)]
                            },
                            (SettlementType::FishingPort, SettlementType::Village) => {
                                ["Fresh Fish", "Dried Fish", "Salt"][rng.gen_range(0..3)]
                            },
                            (SettlementType::Village, SettlementType::TradeHub) => {
                                ["Grain", "Vegetables", "Livestock"][rng.gen_range(0..3)]
                            },
                            _ => {
                                ["General Goods", "Crafts", "Food"][rng.gen_range(0..3)]
                            },
                        };

                        let volumes = ["high", "medium", "low"];
                        kb.add_trade_good(TradeGood {
                            route_id: rid,
                            good_name: good.to_string(),
                            from_settlement: route.settlement_a,
                            to_settlement: route.settlement_b,
                            volume: volumes[rng.gen_range(0..volumes.len())].to_string(),
                            discovered_at_tick: kb.current_tick,
                        });
                    }
                }
            },

            ProbeType::Vocabulary => {
                if let Some(sid) = self.target_settlement {
                    if let Some(settlement) = civ.settlements.get(sid) {
                        // Generate settlement-specific vocabulary
                        let (word, meaning) = match settlement.settlement_type {
                            SettlementType::MiningTown => {
                                let terms = [
                                    ("veinseek", "The art of finding ore deposits"),
                                    ("deepmark", "Traditional mining symbols"),
                                    ("stonesong", "Echo patterns that indicate cave stability"),
                                ];
                                terms[rng.gen_range(0..terms.len())]
                            },
                            SettlementType::FishingPort => {
                                let terms = [
                                    ("tidecall", "The timing of optimal fishing"),
                                    ("netweave", "Traditional fishing net patterns"),
                                    ("wavereading", "Skill of predicting weather from waves"),
                                ];
                                terms[rng.gen_range(0..terms.len())]
                            },
                            SettlementType::Village => {
                                let terms = [
                                    ("seedwise", "Knowledge of planting seasons"),
                                    ("fieldmark", "Traditional crop rotation patterns"),
                                    ("harvesthand", "Skilled agricultural worker"),
                                ];
                                terms[rng.gen_range(0..terms.len())]
                            },
                            SettlementType::TradeHub => {
                                let terms = [
                                    ("fairweight", "Honest measurement in trade"),
                                    ("roadtale", "Stories shared by traveling merchants"),
                                    ("pricewise", "Skilled in negotiation"),
                                ];
                                terms[rng.gen_range(0..terms.len())]
                            },
                        };

                        kb.add_vocabulary(LocalVocabulary {
                            settlement_id: sid,
                            word: word.to_string(),
                            meaning: meaning.to_string(),
                            usage_count: rng.gen_range(5..50),
                            discovered_at_tick: kb.current_tick,
                        });
                    }
                }
            },
        }

        // Return any follow-up probes spawned by recursive curiosity
        follow_up_probes
    }
}

// Spawn probes based on AGI curiosity
pub fn spawn_discovery_probes(civ: &CivilizationSimulator, kb: &mut KnowledgeBase, base_spawn_chance: f64) -> Vec<DiscoveryProbe> {
    let mut rng = rand::thread_rng();
    let mut probes = Vec::new();

    // SELF-DIRECTED EXPLORATION: Analyze knowledge gaps and focus curiosity
    let gaps = kb.analyze_knowledge_gaps(civ);
    let underexplored = kb.get_underexplored_settlements(civ);

    // Build urgency map: gap type -> urgency (0.0-1.0)
    let mut urgency_map: std::collections::HashMap<&str, f64> = std::collections::HashMap::new();
    for (gap_name, urgency) in &gaps {
        if gap_name.contains("Resource") {
            urgency_map.insert("resource", *urgency);
        } else if gap_name.contains("Cultural") {
            urgency_map.insert("culture", *urgency);
        } else if gap_name.contains("Trade") {
            urgency_map.insert("trade", *urgency);
        } else if gap_name.contains("Vocabulary") {
            urgency_map.insert("vocab", *urgency);
        }
    }

    // Boost spawn rates based on urgency (0.0 urgency = 0.5x rate, 1.0 urgency = 3.0x rate)
    let resource_rate = base_spawn_chance * (0.5 + 2.5 * urgency_map.get("resource").unwrap_or(&0.0));
    let culture_rate = base_spawn_chance * (0.5 + 2.5 * urgency_map.get("culture").unwrap_or(&0.0));
    let trade_rate = base_spawn_chance * (0.5 + 2.5 * urgency_map.get("trade").unwrap_or(&0.0));
    let vocab_rate = base_spawn_chance * (0.5 + 2.5 * urgency_map.get("vocab").unwrap_or(&0.0));

    // TARGETED EXPLORATION: Focus on underexplored settlements first
    // Resource probes - prioritize underexplored settlements
    for &sid in &underexplored {
        if rng.gen::<f64>() < resource_rate * 2.0 {  // 2x rate for underexplored
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Resource,
                target_settlement: Some(sid),
                target_route: None,
                parent_discovery: Some(format!("Exploring underexplored settlement {}", sid)),
                recursion_depth: 0,
            });
        }
    }

    // Also probe other settlements at normal rate
    for (sid, _) in civ.settlements.iter().enumerate() {
        if !underexplored.contains(&sid) && rng.gen::<f64>() < resource_rate {
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Resource,
                target_settlement: Some(sid),
                target_route: None,
                parent_discovery: None,
                recursion_depth: 0,
            });
        }
    }

    // Culture probes - prioritize underexplored
    for &sid in &underexplored {
        if rng.gen::<f64>() < culture_rate * 2.0 {
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Culture,
                target_settlement: Some(sid),
                target_route: None,
                parent_discovery: Some(format!("Cultural exploration of underexplored settlement {}", sid)),
                recursion_depth: 0,
            });
        }
    }

    for (sid, _) in civ.settlements.iter().enumerate() {
        if !underexplored.contains(&sid) && rng.gen::<f64>() < culture_rate {
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Culture,
                target_settlement: Some(sid),
                target_route: None,
                parent_discovery: None,
                recursion_depth: 0,
            });
        }
    }

    // Trade probes - spawn based on trade gap urgency
    for (rid, _) in civ.trade_routes.iter().enumerate() {
        if rng.gen::<f64>() < trade_rate {
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Trade,
                target_settlement: None,
                target_route: Some(rid),
                parent_discovery: None,
                recursion_depth: 0,
            });
        }
    }

    // Vocabulary probes - prioritize underexplored
    for &sid in &underexplored {
        if rng.gen::<f64>() < vocab_rate * 2.0 {
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Vocabulary,
                target_settlement: Some(sid),
                target_route: None,
                parent_discovery: Some(format!("Vocabulary learning at underexplored settlement {}", sid)),
                recursion_depth: 0,
            });
        }
    }

    for (sid, _) in civ.settlements.iter().enumerate() {
        if !underexplored.contains(&sid) && rng.gen::<f64>() < vocab_rate {
            probes.push(DiscoveryProbe {
                probe_type: ProbeType::Vocabulary,
                target_settlement: Some(sid),
                target_route: None,
                parent_discovery: None,
                recursion_depth: 0,
            });
        }
    }

    probes
}

// FEATURE INTEGRATION: Spawn probes guided by analogy discoveries
// When analogy system finds similar patterns, explore those settlements deeper
pub fn spawn_analogy_guided_probes(
    civ: &CivilizationSimulator,
    _kb: &mut KnowledgeBase,
    analogies: &[crate::agi::Analogy],
    _feature_vectors: &std::collections::HashMap<String, Vec<f64>>,
) -> Vec<DiscoveryProbe> {
    let mut probes = Vec::new();
    let mut rng = rand::thread_rng();

    // Safety check: need settlements to explore
    if civ.settlements.is_empty() {
        return probes;
    }

    // For each strong analogy (similarity > 0.7), spawn targeted probes
    for analogy in analogies.iter().filter(|a| a.similarity_score > 0.7) {
        // Analogy detected between patterns (e.g., "mountains" and "hills" are similar)
        // This suggests settlements in similar terrains might have similar cultures/resources
        // Spawn culture probes to test this hypothesis

        // Find settlements that might match the pattern
        // For now, sample some random settlements to explore
        // (In future: could use terrain data to find actual matching settlements)

        for _ in 0..2 {  // Spawn 2 probes per strong analogy
            let settlement_id = rng.gen_range(0..civ.settlements.len());

            // 50% chance culture probe, 50% chance resource probe
            let probe_type = if rng.gen_bool(0.5) {
                ProbeType::Culture
            } else {
                ProbeType::Resource
            };

            probes.push(DiscoveryProbe {
                probe_type,
                target_settlement: Some(settlement_id),
                target_route: None,
                parent_discovery: Some(format!(
                    "Analogy-guided: exploring similarity between {} and {} (score: {:.2})",
                    analogy.source_pattern, analogy.target_pattern, analogy.similarity_score
                )),
                recursion_depth: 0,
            });
        }
    }

    probes
}

// FEATURE INTEGRATION: Spawn probes driven by curiosity scores AND unanswered questions
// When curiosity system identifies interesting areas OR user asks unanswered questions, explore them
pub fn spawn_curiosity_driven_probes(
    civ: &CivilizationSimulator,
    kb: &mut KnowledgeBase,
    curiosity_interests: &[(String, f64)],  // (topic, interest_score)
) -> Vec<DiscoveryProbe> {
    let mut probes = Vec::new();
    let mut rng = rand::thread_rng();

    // Safety check: need settlements to explore
    if civ.settlements.is_empty() {
        return probes;
    }

    // FIRST: Spawn probes for unanswered questions (high priority!)
    let exploration_concepts = kb.get_top_exploration_concepts();
    for concept in exploration_concepts.iter() {
        let concept_lower = concept.to_lowercase();

        // Map concept to probe type
        let probe_type = if concept_lower.contains("resource") {
            ProbeType::Resource
        } else if concept_lower.contains("culture") || concept_lower.contains("practice") {
            ProbeType::Culture
        } else if concept_lower.contains("trade") {
            ProbeType::Trade
        } else if concept_lower.contains("language") || concept_lower.contains("vocabulary") {
            ProbeType::Vocabulary
        } else {
            continue;  // Skip concepts we can't explore
        };

        // Spawn 2-4 probes for unanswered question concepts (higher priority)
        for _ in 0..3 {
            match probe_type {
                ProbeType::Trade => {
                    if !civ.trade_routes.is_empty() {
                        let route_id = rng.gen_range(0..civ.trade_routes.len());
                        probes.push(DiscoveryProbe {
                            probe_type: ProbeType::Trade,
                            target_settlement: None,
                            target_route: Some(route_id),
                            parent_discovery: Some(format!(
                                "Question-driven: exploring {} (user asked)",
                                concept
                            )),
                            recursion_depth: 0,
                        });
                    }
                },
                _ => {
                    let settlement_id = rng.gen_range(0..civ.settlements.len());
                    probes.push(DiscoveryProbe {
                        probe_type,
                        target_settlement: Some(settlement_id),
                        target_route: None,
                        parent_discovery: Some(format!(
                            "Question-driven: exploring {} (user asked)",
                            concept
                        )),
                        recursion_depth: 0,
                    });
                }
            }
        }
    }

    // SECOND: For each high-interest curiosity topic, spawn probes to learn more
    for (topic, interest) in curiosity_interests.iter().filter(|(_, score)| *score > 0.6) {
        let topic_lower = topic.to_lowercase();

        // Determine probe type based on topic
        let probe_type = if topic_lower.contains("resource") || topic_lower.contains("mine") {
            ProbeType::Resource
        } else if topic_lower.contains("culture") || topic_lower.contains("ritual") {
            ProbeType::Culture
        } else if topic_lower.contains("trade") {
            ProbeType::Trade
        } else {
            ProbeType::Vocabulary
        };

        // Spawn 1-3 probes depending on interest level
        let probe_count = (interest * 3.0).ceil() as usize;
        for _ in 0..probe_count.min(3) {
            match probe_type {
                ProbeType::Trade => {
                    if !civ.trade_routes.is_empty() {
                        let route_id = rng.gen_range(0..civ.trade_routes.len());
                        probes.push(DiscoveryProbe {
                            probe_type: ProbeType::Trade,
                            target_settlement: None,
                            target_route: Some(route_id),
                            parent_discovery: Some(format!(
                                "Curiosity-driven: exploring {} (interest: {:.2})",
                                topic, interest
                            )),
                            recursion_depth: 0,
                        });
                    }
                },
                _ => {
                    let settlement_id = rng.gen_range(0..civ.settlements.len());
                    probes.push(DiscoveryProbe {
                        probe_type,
                        target_settlement: Some(settlement_id),
                        target_route: None,
                        parent_discovery: Some(format!(
                            "Curiosity-driven: exploring {} (interest: {:.2})",
                            topic, interest
                        )),
                        recursion_depth: 0,
                    });
                }
            }
        }
    }

    probes
}
