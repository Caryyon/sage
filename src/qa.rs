// Question-Answering module - Enables SAGE to answer questions about learned patterns
//
// This module uses AGI features (curiosity, world model, analogy) to generate
// answers about civilization dynamics, cultural evolution, and learned patterns.
//
// NEW: Semantic intent-based Q&A system that understands concepts and dynamically gathers data

use crate::civilization::CivilizationSimulator;
use crate::agi::AGISystem;
use crate::knowledge::KnowledgeBase;
use std::collections::HashMap;

// ============ SEMANTIC Q&A SYSTEM ============
// Understands concepts and dynamically gathers relevant data

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Concept {
    Settlement,      // Settlements, villages, towns, cities
    Trade,           // Trade routes, trading, commerce
    Resource,        // Resources, mining, materials
    Population,      // People, population, inhabitants
    Culture,         // Cultural traits, practices, rituals, customs
    Language,        // Languages, words, vocabulary
    Theory,          // Hypotheses, theories, discoveries
    Category,        // Categories, clusters, types, groups
    Causal,          // Causality, relationships, why things happen
    Comparison,      // Which, most, least, best, worst
    Count,           // How many, total, number
    Location,        // Where, geographical, terrain
}

#[derive(Debug, Clone)]
pub enum QueryIntent {
    Count,           // "How many settlements?"
    List,            // "What resources exist?"
    Comparison,      // "Which settlement has most population?"
    Explanation,     // "Why do trade routes form?"
    Description,     // "What are the categories?"
    Relationship,    // "How do settlements trade?"
    Statistics,      // "What is the average population?"
}

#[derive(Debug, Clone)]
pub struct SemanticQuery {
    pub concepts: Vec<Concept>,
    pub intent: QueryIntent,
    pub modifiers: Vec<String>,  // "top 5", "most", "average", etc.
    pub raw_text: String,
}

// Gathered data from various sources based on detected concepts
#[derive(Debug, Clone, Default)]
pub struct GatheredData {
    pub settlement_data: Option<SettlementData>,
    pub trade_data: Option<TradeData>,
    pub resource_data: Option<ResourceData>,
    pub population_data: Option<PopulationData>,
    pub culture_data: Option<CultureData>,
    pub language_data: Option<LanguageData>,
    pub theory_data: Option<TheoryData>,
    pub category_data: Option<CategoryData>,
}

#[derive(Debug, Clone)]
pub struct SettlementData {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub largest: Option<(usize, usize)>,  // (id, population)
}

#[derive(Debug, Clone)]
pub struct TradeData {
    pub total_routes: usize,
    pub avg_distance: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceData {
    pub total: usize,
    pub top_resources: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct PopulationData {
    pub total: usize,
    pub average: f64,
    pub max: usize,
}

#[derive(Debug, Clone)]
pub struct CultureData {
    pub total_traits: usize,
    pub practices: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LanguageData {
    pub total_languages: usize,
    pub top_words: Vec<(String, usize)>,
    pub total_vocabulary: usize,
}

#[derive(Debug, Clone)]
pub struct TheoryData {
    pub total_hypotheses: usize,
    pub confirmed: usize,
    pub active: usize,
    pub refuted: usize,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CategoryData {
    pub total_categories: usize,
    pub descriptions: Vec<String>,
}

// ============ LEGACY Q&A SYSTEM (for backward compatibility) ============

#[derive(Debug, Clone)]
pub enum QuestionType {
    Why { subject: String },              // "Why did X happen?"
    What { scenario: String },            // "What happens if X?"
    How { mechanism: String },            // "How does X work?"
    Which { comparison: String },         // "Which X has the most Y?"
}

#[derive(Debug, Clone)]
pub struct Question {
    pub question_type: QuestionType,
    pub raw_text: String,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub text: String,
    pub confidence: f64,      // 0-1, how confident SAGE is
    pub reasoning: Vec<String>, // Step-by-step reasoning
    pub evidence: Vec<String>,  // Supporting evidence
}

impl Answer {
    pub fn new(text: String, confidence: f64) -> Self {
        Answer {
            text,
            confidence,
            reasoning: Vec::new(),
            evidence: Vec::new(),
        }
    }

    pub fn with_reasoning(mut self, reasoning: Vec<String>) -> Self {
        self.reasoning = reasoning;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }
}

pub struct QuestionAnsweringSystem {
    // Pre-defined questions SAGE can answer
    pub sample_questions: Vec<String>,
}

impl QuestionAnsweringSystem {
    pub fn new() -> Self {
        QuestionAnsweringSystem {
            sample_questions: vec![
                "What categories of settlements has the AGI discovered?".to_string(),
                "What theories has the AGI developed?".to_string(),
                "What are the top 5 traded words?".to_string(),
                "What is being mined?".to_string(),
                "What rituals do settlements practice?".to_string(),
                "How many villages were there?".to_string(),
                "Which settlement has the most population?".to_string(),
                "Why do settlements form trade routes?".to_string(),
            ],
        }
    }

    // Parse raw text into structured question
    pub fn parse_question(&self, text: &str) -> Option<Question> {
        let text_lower = text.to_lowercase();

        let question_type = if text_lower.starts_with("why") {
            let subject = text[3..].trim().trim_end_matches('?').to_string();
            QuestionType::Why { subject }
        } else if text_lower.starts_with("what happens") {
            let scenario = text[12..].trim().trim_end_matches('?').to_string();
            QuestionType::What { scenario }
        } else if text_lower.starts_with("what") {
            // Accept general "what" questions too
            let scenario = text[4..].trim().trim_end_matches('?').to_string();
            QuestionType::What { scenario }
        } else if text_lower.starts_with("how") {
            let mechanism = text[3..].trim().trim_end_matches('?').to_string();
            QuestionType::How { mechanism }
        } else if text_lower.starts_with("which") {
            let comparison = text[5..].trim().trim_end_matches('?').to_string();
            QuestionType::Which { comparison }
        } else {
            return None;
        };

        Some(Question {
            question_type,
            raw_text: text.to_string(),
        })
    }

    // Answer question using civilization simulation data and discovered knowledge
    pub fn answer_about_civilization(
        &self,
        question: &Question,
        civ: &CivilizationSimulator,
        agi: &AGISystem,
        kb: &KnowledgeBase,
    ) -> Answer {
        match &question.question_type {
            QuestionType::Why { subject } => self.answer_why(subject, civ, agi, kb),
            QuestionType::What { scenario } => self.answer_what(scenario, civ, agi, kb),
            QuestionType::How { mechanism } => self.answer_how(mechanism, civ, agi, kb),
            QuestionType::Which { comparison } => self.answer_which(comparison, civ, agi, kb),
        }
    }

    fn answer_why(&self, subject: &str, civ: &CivilizationSimulator, _agi: &AGISystem, kb: &KnowledgeBase) -> Answer {
        let subject_lower = subject.to_lowercase();

        // "Why do settlements form trade routes?"
        if subject_lower.contains("trade route") {
            let total_routes = civ.trade_routes.len();
            let avg_distance = self.calculate_avg_trade_distance(civ);

            let reasoning = vec![
                "Trade routes form between settlements that are:".to_string(),
                "1. Geographically close (within 15 grid units)".to_string(),
                "2. Both prosperous (high population × biome favorability)".to_string(),
                "3. Established enough to support trade".to_string(),
            ];

            let evidence = vec![
                format!("Currently {} active trade routes exist", total_routes),
                format!("Average route distance: {:.1} units", avg_distance),
                format!("Trade volume increases with time and population"),
            ];

            Answer::new(
                format!("Trade routes emerge between nearby prosperous settlements. Currently {} routes connect settlements within an average distance of {:.1} units. Routes strengthen as populations grow and time passes.",
                    total_routes, avg_distance),
                0.9, // High confidence - directly observable
            )
            .with_reasoning(reasoning)
            .with_evidence(evidence)
        }
        // "Why do villages form in valleys?"
        else if subject_lower.contains("village") && subject_lower.contains("valley") {
            let village_count = civ.settlements.iter()
                .filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::Village))
                .count();

            Answer::new(
                format!("Villages prefer flat, low-elevation terrain (valleys and plains) because these areas have: (1) low height variance for agriculture, (2) moderate elevation (0.2-0.55), and (3) favorable biome scores. Currently {} villages exist in optimal locations.",
                    village_count),
                0.85,
            )
            .with_reasoning(vec![
                "Biome evaluation scores terrain based on:".to_string(),
                "- Flatness (low variance) = easier farming".to_string(),
                "- Moderate elevation = not too wet or dry".to_string(),
                "- Distance from mountains/water".to_string(),
            ])
        } else {
            // CAUSAL REASONING: Check if AGI has learned causal explanations for this query
            let causal_explanations = kb.get_causal_explanations(subject);

            if !causal_explanations.is_empty() {
                let explanation_text = causal_explanations.join("\n\n");
                Answer::new(
                    format!("Based on discovered causal patterns:\n\n{}", explanation_text),
                    0.85,
                )
                .with_reasoning(vec![
                    "AGI has inferred these causal relationships from temporal patterns in discoveries".to_string(),
                    format!("Total causal links discovered: {}", kb.causal_model.links.len()),
                ])
            } else {
                Answer::new(
                    format!("I haven't discovered causal patterns related to '{}' yet. I can answer questions about trade routes, settlement formation, and cultural dynamics. The AGI has discovered {} causal relationships total.",
                        subject, kb.causal_model.links.len()),
                    0.3,
                )
            }
        }
    }

    fn answer_what(&self, scenario: &str, civ: &CivilizationSimulator, _agi: &AGISystem, kb: &KnowledgeBase) -> Answer {
        let scenario_lower = scenario.to_lowercase();

        // "What categories" - EMERGENT ABSTRACTION
        if scenario_lower.contains("categor") || scenario_lower.contains("cluster") || scenario_lower.contains("type") {
            let settlement_categories: Vec<_> = kb.abstraction.categories.iter()
                .filter(|c| c.category_type == crate::knowledge::CategoryType::SettlementCluster)
                .collect();

            if settlement_categories.is_empty() {
                return Answer::new(
                    "The AGI hasn't discovered settlement categories yet. Categories emerge from clustering behavioral patterns (resources, trade, culture, population). Need more discoveries first.".to_string(),
                    0.7
                );
            }

            let category_descriptions: Vec<String> = settlement_categories.iter()
                .map(|c| {
                    let member_count = c.members.len();
                    let features = c.defining_features.join(", ");
                    format!("\"{}\" ({} settlements): {}", c.name, member_count, features)
                })
                .collect();

            Answer::new(
                format!("The AGI has discovered {} settlement categories through emergent abstraction:\n\n{}",
                    settlement_categories.len(), category_descriptions.join("\n\n")),
                0.95,
            )
            .with_evidence(vec![
                "Categories discovered via k-means clustering on behavioral features".to_string(),
                format!("Features: Population, Resources, Culture, Trade, Vocabulary"),
                format!("Reclustered every 50 epochs as new discoveries emerge"),
            ])
        }
        // "What theories" or "What hypotheses" or "What has the AGI learned"
        else if scenario_lower.contains("theor") || scenario_lower.contains("hypothes") ||
           (scenario_lower.contains("agi") && (scenario_lower.contains("learn") || scenario_lower.contains("discover"))) {
            let confirmed = kb.get_confirmed_theories();
            let active = kb.hypotheses.iter().filter(|h| h.is_active).count();
            let refuted = kb.get_refuted_theories();

            if kb.hypotheses.is_empty() {
                return Answer::new(
                    "The AGI hasn't developed any hypotheses yet. It needs more discoveries to identify patterns and generate theories.".to_string(),
                    0.7
                );
            }

            let theory_examples: Vec<String> = confirmed.iter()
                .take(3)
                .map(|h| format!("\"{}\" ({:.0}% confidence, {} supporting, {} refuting)",
                    h.theory, h.confidence * 100.0, h.evidence_for, h.evidence_against))
                .collect();

            let response = if !theory_examples.is_empty() {
                format!("The AGI has developed {} hypotheses. {} are confirmed (>70% confidence): {}. {} active hypotheses are still being tested, and {} were refuted.",
                    kb.hypotheses.len(), confirmed.len(), theory_examples.join("; "), active, refuted.len())
            } else if active > 0 {
                format!("The AGI has {} active hypotheses being tested. No theories have been confirmed or refuted yet - more evidence is needed.",
                    active)
            } else {
                format!("The AGI has generated {} hypotheses. {} were refuted with low confidence (<30%).",
                    kb.hypotheses.len(), refuted.len())
            };

            Answer::new(response, 0.93)
                .with_evidence(vec![
                    format!("{} hypotheses generated from {} discoveries", kb.hypotheses.len(), kb.discovery_count),
                    format!("{} confirmed, {} active, {} refuted", confirmed.len(), active, refuted.len()),
                ])
        }
        // "What are the top 5 traded words?"
        else if scenario_lower.contains("top") && (scenario_lower.contains("word") || scenario_lower.contains("traded")) {
            let top_words = kb.top_traded_words(5);
            if top_words.is_empty() {
                return Answer::new(
                    "No word trading data has been discovered yet. The AGI is still exploring the world.".to_string(),
                    0.6
                );
            }
            let word_list: Vec<String> = top_words.iter()
                .map(|(word, count)| format!("{} (used {} times)", word, count))
                .collect();
            Answer::new(
                format!("The top {} most traded words discovered by AGI probes are: {}. These words spread through {} trade routes and reflect the cultural exchanges between settlements.",
                    word_list.len(),
                    word_list.join(", "),
                    civ.trade_routes.len()),
                0.95,
            )
            .with_evidence(vec![
                format!("{} total discoveries made", kb.discovery_count),
                format!("{} vocabulary entries analyzed", kb.vocabulary.len()),
            ])
        }
        // "What is being mined?" or "What resources exist?"
        else if scenario_lower.contains("mined") || scenario_lower.contains("resource") {
            let top_resources = kb.top_resources(5);
            if top_resources.is_empty() {
                return Answer::new(
                    "No resource data has been discovered yet. AGI probes are still exploring settlements.".to_string(),
                    0.6
                );
            }
            let resource_list: Vec<String> = top_resources.iter()
                .map(|(resource, count)| format!("{} ({} settlements)", resource, count))
                .collect();
            Answer::new(
                format!("AGI probes have discovered these resources: {}. Each was found through autonomous exploration of settlements based on their type and terrain.",
                    resource_list.join(", ")),
                0.92,
            )
            .with_evidence(vec![
                format!("{} resource discoveries", kb.resources.len()),
                format!("{} settlements analyzed", civ.settlements.len()),
            ])
        }
        // "What rituals" or "What practices"
        else if scenario_lower.contains("ritual") || scenario_lower.contains("practice") || scenario_lower.contains("custom") {
            if kb.practices.is_empty() {
                return Answer::new(
                    "No cultural practices have been discovered yet. AGI probes are still exploring.".to_string(),
                    0.6
                );
            }
            let practice_examples: Vec<String> = kb.practices.iter()
                .take(3)
                .map(|p| format!("{}: {} ({})", p.practice_name, p.description, p.frequency))
                .collect();
            Answer::new(
                format!("AGI probes have discovered {} cultural practices including: {}",
                    kb.practices.len(),
                    practice_examples.join("; ")),
                0.90,
            )
            .with_evidence(vec![
                format!("{} practices across {} settlements", kb.practices.len(), civ.settlements.len()),
                format!("Discovered through {} autonomous probe explorations", kb.discovery_count),
            ])
        }
        // "What is the total population?"
        else if scenario_lower.contains("population") || scenario_lower.contains("people") {
            let total_pop: usize = civ.settlements.iter().map(|s| s.population).sum();
            let largest = civ.settlements.iter().max_by_key(|s| s.population);

            if let Some(largest_settlement) = largest {
                Answer::new(
                    format!("The total population is {} people across {} settlements. The largest settlement has {} people.",
                        total_pop, civ.settlements.len(), largest_settlement.population),
                    0.95,
                )
                .with_evidence(vec![
                    format!("{} total settlements", civ.settlements.len()),
                    format!("Largest settlement: {} people", largest_settlement.population),
                ])
            } else {
                Answer::new("There are no settlements yet.".to_string(), 0.5)
            }
        }
        // "What happens when a mining town trades with a fishing port?"
        else if scenario_lower.contains("mining") && scenario_lower.contains("fishing") {
            Answer::new(
                format!("When mining towns and fishing ports trade, they exchange cultural traits and vocabulary. Mining terms like 'ore' and 'forge' spread to coastal settlements, while fishing terms like 'tide' and 'net' spread inland. This creates hybrid cultures with diverse vocabularies. Currently {} cultural traits and {} languages exist.",
                    civ.cultural_traits.len(), civ.languages.len()),
                0.8,
            )
            .with_reasoning(vec![
                "Trade routes enable cultural exchange".to_string(),
                "Words borrowed at 30% chance per tick".to_string(),
                "Cultural traits spread at 10% chance per tick".to_string(),
                "Exchange rate depends on trade volume".to_string(),
            ])
            .with_evidence(vec![
                format!("{} cultural traits currently exist", civ.cultural_traits.len()),
                format!("{} languages with various borrowed words", civ.languages.len()),
            ])
        }
        // General "what" about settlements, trade, etc.
        else if scenario_lower.contains("settlement") {
            let total = civ.settlements.len();
            let villages = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::Village)).count();
            let mining = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::MiningTown)).count();
            let fishing = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::FishingPort)).count();
            let hubs = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::TradeHub)).count();

            Answer::new(
                format!("There are {} settlements total: {} villages, {} mining towns, {} fishing ports, and {} trade hubs. They're connected by {} trade routes.",
                    total, villages, mining, fishing, hubs, civ.trade_routes.len()),
                0.9,
            )
        }
        else if scenario_lower.contains("trade") {
            Answer::new(
                format!("There are {} active trade routes connecting settlements. Trade enables cultural exchange, word borrowing, and shared prosperity. Routes form between nearby settlements (within 15 units).",
                    civ.trade_routes.len()),
                0.85,
            )
        } else {
            // Fallback with useful stats
            Answer::new(
                format!("I found: {} settlements with {} total population, {} trade routes, {} cultural traits, {} languages. Try 'what is the population' or 'what settlements exist'.",
                    civ.settlements.len(),
                    civ.settlements.iter().map(|s| s.population).sum::<usize>(),
                    civ.trade_routes.len(),
                    civ.cultural_traits.len(),
                    civ.languages.len()),
                0.5,
            )
        }
    }

    #[allow(unused_variables)]
    fn answer_how(&self, mechanism: &str, civ: &CivilizationSimulator, _agi: &AGISystem, kb: &KnowledgeBase) -> Answer {
        let mechanism_lower = mechanism.to_lowercase();

        // Check for "how many" questions first
        if mechanism_lower.starts_with("many") {
            return self.answer_how_many(&mechanism_lower, civ);
        }

        // "How does language evolve?"
        if mechanism_lower.contains("language") {
            let total_borrowed = civ.languages.iter()
                .map(|l| l.borrowed_words.len())
                .sum::<usize>();

            Answer::new(
                format!("Languages evolve through word borrowing along trade routes. Each settlement starts with 4 base words specific to its type. When trade occurs (30% chance per tick), settlements randomly borrow words from each other. Languages become less unique as they borrow more. Currently {} words have been borrowed across all languages.",
                    total_borrowed),
                0.9,
            )
            .with_reasoning(vec![
                "1. Each settlement gets initial vocabulary".to_string(),
                "2. Trade routes connect settlements".to_string(),
                "3. Words borrowed probabilistically".to_string(),
                "4. Uniqueness = base_words / total_words".to_string(),
            ])
        }
        // "How do cultural traits spread?"
        else if mechanism_lower.contains("cultural") || mechanism_lower.contains("trait") {
            Answer::new(
                format!("Cultural traits spread via diffusion along trade routes. When trade occurs (10% chance per tick), a trait from one settlement can spread to its trading partner with 50% reduced strength. Traits compound in settlements, creating regional cultural zones. Currently {} cultural traits exist across {} settlements.",
                    civ.cultural_traits.len(), civ.settlements.len()),
                0.85,
            )
            .with_evidence(vec![
                format!("{} total cultural traits", civ.cultural_traits.len()),
                format!("{} settlements exchanging culture", civ.settlements.len()),
            ])
        }
        // "How do settlements form?" or similar
        else if mechanism_lower.contains("settlement") || mechanism_lower.contains("form") {
            Answer::new(
                format!("Settlements form based on terrain evaluation. Villages prefer flat valleys (0.2-0.55 elevation), Mining Towns cluster in mountains (>0.75), Fishing Ports appear near water (<0.2), and Trade Hubs form in strategic plains. Currently {} settlements exist with a total population of {}.",
                    civ.settlements.len(),
                    civ.settlements.iter().map(|s| s.population).sum::<usize>()),
                0.85,
            )
        } else {
            // Try to give relevant stats as fallback
            Answer::new(
                format!("The simulation has {} settlements, {} trade routes, {} cultural traits, and {} languages. Try asking about specific mechanisms like 'how does language evolve' or 'how many villages'.",
                    civ.settlements.len(), civ.trade_routes.len(), civ.cultural_traits.len(), civ.languages.len()),
                0.5,
            )
        }
    }

    // Handle "how many X" questions
    fn answer_how_many(&self, question: &str, civ: &CivilizationSimulator) -> Answer {
        if question.contains("village") {
            let count = civ.settlements.iter()
                .filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::Village))
                .count();
            Answer::new(
                format!("There are {} villages in the simulation. Villages form in flat, low-elevation areas like valleys and plains.",
                    count),
                0.95,
            )
        } else if question.contains("mining") || question.contains("town") {
            let count = civ.settlements.iter()
                .filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::MiningTown))
                .count();
            Answer::new(
                format!("There are {} mining towns in the simulation. Mining towns form in mountainous regions with high elevation.",
                    count),
                0.95,
            )
        } else if question.contains("fishing") || question.contains("port") {
            let count = civ.settlements.iter()
                .filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::FishingPort))
                .count();
            Answer::new(
                format!("There are {} fishing ports in the simulation. Fishing ports form near water (low elevation areas).",
                    count),
                0.95,
            )
        } else if question.contains("trade") && question.contains("hub") {
            let count = civ.settlements.iter()
                .filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::TradeHub))
                .count();
            Answer::new(
                format!("There are {} trade hubs in the simulation. Trade hubs form in strategic plains locations.",
                    count),
                0.95,
            )
        } else if question.contains("settlement") {
            let total = civ.settlements.len();
            let villages = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::Village)).count();
            let mining = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::MiningTown)).count();
            let fishing = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::FishingPort)).count();
            let hubs = civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::TradeHub)).count();

            Answer::new(
                format!("There are {} total settlements: {} villages, {} mining towns, {} fishing ports, and {} trade hubs.",
                    total, villages, mining, fishing, hubs),
                0.95,
            )
            .with_evidence(vec![
                format!("{} total population", civ.settlements.iter().map(|s| s.population).sum::<usize>()),
                format!("{} trade routes connecting them", civ.trade_routes.len()),
            ])
        } else if question.contains("trade route") || question.contains("route") {
            Answer::new(
                format!("There are {} active trade routes connecting settlements. Trade routes form between nearby prosperous settlements (within 15 units).",
                    civ.trade_routes.len()),
                0.95,
            )
        } else if question.contains("people") || question.contains("population") {
            let total_pop: usize = civ.settlements.iter().map(|s| s.population).sum();
            Answer::new(
                format!("The total population across all {} settlements is {} people.",
                    civ.settlements.len(), total_pop),
                0.95,
            )
        } else {
            // General fallback with key stats
            Answer::new(
                format!("I found: {} settlements, {} trade routes, {} cultural traits, {} languages. Try asking 'how many villages' or 'how many settlements'.",
                    civ.settlements.len(), civ.trade_routes.len(), civ.cultural_traits.len(), civ.languages.len()),
                0.6,
            )
        }
    }

    #[allow(unused_variables)]
    fn answer_which(&self, comparison: &str, civ: &CivilizationSimulator, _agi: &AGISystem, kb: &KnowledgeBase) -> Answer {
        let comparison_lower = comparison.to_lowercase();

        // "Which settlement has the most population?"
        if comparison_lower.contains("population") || comparison_lower.contains("people") {
            if let Some(largest) = civ.settlements.iter().enumerate().max_by_key(|(_, s)| s.population) {
                let (idx, settlement) = largest;
                Answer::new(
                    format!("Settlement #{} ({:?} at {},{})) has the most population with {} people. It has {} trade connections.",
                        idx, settlement.settlement_type, settlement.x, settlement.y, settlement.population,
                        civ.trade_routes.iter().filter(|r| r.settlement_a == idx || r.settlement_b == idx).count()),
                    0.95,
                )
                .with_evidence(vec![
                    format!("Total population across all settlements: {}", civ.settlements.iter().map(|s| s.population).sum::<usize>()),
                    format!("Average population per settlement: {}", civ.settlements.iter().map(|s| s.population).sum::<usize>() / civ.settlements.len()),
                ])
            } else {
                Answer::new("No settlements exist yet.".to_string(), 0.5)
            }
        }
        // "Which settlement has the most trade routes / connections?"
        else if comparison_lower.contains("trade") && (comparison_lower.contains("route") || comparison_lower.contains("connection")) {
            let connections: Vec<_> = (0..civ.settlements.len())
                .map(|i| {
                    let count = civ.trade_routes.iter()
                        .filter(|r| r.settlement_a == i || r.settlement_b == i)
                        .count();
                    (i, count)
                })
                .collect();

            if let Some((idx, count)) = connections.iter().max_by_key(|(_, c)| c) {
                let settlement = &civ.settlements[*idx];
                Answer::new(
                    format!("Settlement #{} ({:?} at {},{}) has the most trade connections with {} routes. It has a population of {}.",
                        idx, settlement.settlement_type, settlement.x, settlement.y, count, settlement.population),
                    0.95,
                )
            } else {
                Answer::new("No settlements exist yet.".to_string(), 0.5)
            }
        }
        // "Which settlements have the most cultural influence?"
        else if comparison_lower.contains("cultural") || comparison_lower.contains("influence") {
            let influence_scores: Vec<_> = (0..civ.settlements.len())
                .map(|i| {
                    let trait_count = civ.cultural_traits.iter()
                        .filter(|t| t.origin_settlement == i)
                        .count();
                    let trade_connections = civ.trade_routes.iter()
                        .filter(|r| r.settlement_a == i || r.settlement_b == i)
                        .count();
                    (i, trait_count + trade_connections)
                })
                .collect();

            let top_settlement = influence_scores.iter()
                .max_by_key(|(_, score)| score)
                .map(|(idx, score)| (*idx, *score));

            if let Some((idx, score)) = top_settlement {
                let settlement = &civ.settlements[idx];
                Answer::new(
                    format!("Settlement #{} ({:?} at {},{}) has the most cultural influence with a score of {}. Influence = cultural traits originated + trade connections.",
                        idx, settlement.settlement_type, settlement.x, settlement.y, score),
                    0.95,
                )
                .with_reasoning(vec![
                    "Cultural influence calculated by:".to_string(),
                    "- Number of cultural traits originated".to_string(),
                    "- Number of trade connections".to_string(),
                    "More connected = more influence".to_string(),
                ])
            } else {
                Answer::new("No settlements exist yet to compare.".to_string(), 0.5)
            }
        }
        // "Which terrain supports the largest civilizations?"
        else if comparison_lower.contains("terrain") || comparison_lower.contains("biome") {
            Answer::new(
                format!("Plains and valley terrains (0.3-0.5 elevation, low variance) support the largest civilizations. These areas have high biome scores, allowing Villages and Trade Hubs to flourish. Mining Towns in mountains have smaller populations but provide essential resources. Currently: {} villages, {} mining towns, {} fishing ports, {} trade hubs.",
                    civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::Village)).count(),
                    civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::MiningTown)).count(),
                    civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::FishingPort)).count(),
                    civ.settlements.iter().filter(|s| matches!(s.settlement_type, crate::civilization::SettlementType::TradeHub)).count()),
                0.8,
            )
        }
        // "Which settlement is at location X,Y?" or similar
        else if comparison_lower.contains("settlement") {
            // Default to largest settlement
            if let Some(largest) = civ.settlements.iter().enumerate().max_by_key(|(_, s)| s.population) {
                let (idx, settlement) = largest;
                Answer::new(
                    format!("The largest settlement is #{} ({:?} at {},{}) with {} people and {} trade connections. Try asking 'which settlement has the most population' or 'which has the most trade routes'.",
                        idx, settlement.settlement_type, settlement.x, settlement.y, settlement.population,
                        civ.trade_routes.iter().filter(|r| r.settlement_a == idx || r.settlement_b == idx).count()),
                    0.7,
                )
            } else {
                Answer::new("No settlements exist yet.".to_string(), 0.5)
            }
        } else {
            // Fallback with useful comparison
            Answer::new(
                format!("I can compare settlements by: population, trade connections, cultural influence, or terrain. Try 'which settlement has the most population' or 'which has the most trade routes'. Currently {} settlements exist.",
                    civ.settlements.len()),
                0.5,
            )
        }
    }

    // Helper: Calculate average trade route distance
    fn calculate_avg_trade_distance(&self, civ: &CivilizationSimulator) -> f64 {
        if civ.trade_routes.is_empty() {
            return 0.0;
        }

        let total_distance: f64 = civ.trade_routes.iter()
            .map(|route| {
                let a = &civ.settlements[route.settlement_a];
                let b = &civ.settlements[route.settlement_b];
                let dx = (a.x as f64 - b.x as f64).abs();
                let dy = (a.y as f64 - b.y as f64).abs();
                (dx * dx + dy * dy).sqrt()
            })
            .sum();

        total_distance / civ.trade_routes.len() as f64
    }

    // ============ NEW SEMANTIC Q&A METHODS ============

    /// Parse question into semantic query with detected concepts and intent
    pub fn parse_semantic_query(&self, text: &str) -> SemanticQuery {
        let text_lower = text.to_lowercase();
        let mut concepts = Vec::new();
        let mut modifiers = Vec::new();

        // Detect concepts through semantic keywords
        if text_lower.contains("settlement") || text_lower.contains("village") ||
           text_lower.contains("town") || text_lower.contains("city") {
            concepts.push(Concept::Settlement);
        }
        if text_lower.contains("trade") || text_lower.contains("trading") ||
           text_lower.contains("commerce") || text_lower.contains("route") {
            concepts.push(Concept::Trade);
        }
        if text_lower.contains("resource") || text_lower.contains("mine") ||
           text_lower.contains("mined") || text_lower.contains("mining") {
            concepts.push(Concept::Resource);
        }
        if text_lower.contains("population") || text_lower.contains("people") ||
           text_lower.contains("inhabitant") {
            concepts.push(Concept::Population);
        }
        if text_lower.contains("culture") || text_lower.contains("ritual") ||
           text_lower.contains("practice") || text_lower.contains("custom") ||
           text_lower.contains("trait") {
            concepts.push(Concept::Culture);
        }
        if text_lower.contains("language") || text_lower.contains("word") ||
           text_lower.contains("vocabulary") {
            concepts.push(Concept::Language);
        }
        if text_lower.contains("theory") || text_lower.contains("theor") ||
           text_lower.contains("hypothes") || text_lower.contains("discover") {
            concepts.push(Concept::Theory);
        }
        if text_lower.contains("categor") || text_lower.contains("cluster") ||
           text_lower.contains("type") || text_lower.contains("group") || text_lower.contains("kind") {
            concepts.push(Concept::Category);
        }
        if text_lower.contains("where") || text_lower.contains("location") ||
           text_lower.contains("terrain") {
            concepts.push(Concept::Location);
        }

        // Detect intent from question structure
        let intent = if text_lower.starts_with("why") || text_lower.contains("because") {
            concepts.push(Concept::Causal);
            QueryIntent::Explanation
        } else if text_lower.starts_with("how many") || text_lower.contains("total") ||
                   text_lower.contains("count") {
            concepts.push(Concept::Count);
            QueryIntent::Count
        } else if text_lower.starts_with("which") || text_lower.contains("most") ||
                   text_lower.contains("least") || text_lower.contains("best") ||
                   text_lower.contains("worst") {
            concepts.push(Concept::Comparison);
            QueryIntent::Comparison
        } else if text_lower.starts_with("how") {
            QueryIntent::Relationship
        } else if text_lower.contains("average") || text_lower.contains("mean") ||
                   text_lower.contains("median") {
            QueryIntent::Statistics
        } else if text_lower.starts_with("what are") || text_lower.contains("describe") {
            QueryIntent::Description
        } else {
            QueryIntent::List
        };

        // Extract modifiers
        if text_lower.contains("top") {
            if let Some(num) = extract_number_after(&text_lower, "top") {
                modifiers.push(format!("top {}", num));
            } else {
                modifiers.push("top 5".to_string());
            }
        }
        if text_lower.contains("most") {
            modifiers.push("most".to_string());
        }
        if text_lower.contains("least") {
            modifiers.push("least".to_string());
        }
        if text_lower.contains("average") {
            modifiers.push("average".to_string());
        }

        SemanticQuery {
            concepts,
            intent,
            modifiers,
            raw_text: text.to_string(),
        }
    }

    /// Dynamically gather data based on detected concepts
    pub fn gather_data(
        &self,
        query: &SemanticQuery,
        civ: &CivilizationSimulator,
        _agi: &AGISystem,
        kb: &KnowledgeBase,
    ) -> GatheredData {
        let mut data = GatheredData::default();

        for concept in &query.concepts {
            match concept {
                Concept::Settlement => {
                    let mut by_type = HashMap::new();
                    for s in &civ.settlements {
                        let type_name = format!("{:?}", s.settlement_type);
                        *by_type.entry(type_name).or_insert(0) += 1;
                    }
                    let largest = civ.settlements.iter().enumerate()
                        .max_by_key(|(_, s)| s.population)
                        .map(|(i, s)| (i, s.population));

                    data.settlement_data = Some(SettlementData {
                        total: civ.settlements.len(),
                        by_type,
                        largest,
                    });
                }
                Concept::Trade => {
                    data.trade_data = Some(TradeData {
                        total_routes: civ.trade_routes.len(),
                        avg_distance: self.calculate_avg_trade_distance(civ),
                    });
                }
                Concept::Resource => {
                    data.resource_data = Some(ResourceData {
                        total: kb.resources.len(),
                        top_resources: kb.top_resources(5),
                    });
                }
                Concept::Population => {
                    let total: usize = civ.settlements.iter().map(|s| s.population).sum();
                    let avg = if !civ.settlements.is_empty() {
                        total as f64 / civ.settlements.len() as f64
                    } else {
                        0.0
                    };
                    let max = civ.settlements.iter().map(|s| s.population).max().unwrap_or(0);

                    data.population_data = Some(PopulationData {
                        total,
                        average: avg,
                        max,
                    });
                }
                Concept::Culture => {
                    let practices: Vec<String> = kb.practices.iter()
                        .take(5)
                        .map(|p| format!("{}: {}", p.practice_name, p.description))
                        .collect();

                    data.culture_data = Some(CultureData {
                        total_traits: civ.cultural_traits.len(),
                        practices,
                    });
                }
                Concept::Language => {
                    data.language_data = Some(LanguageData {
                        total_languages: civ.languages.len(),
                        top_words: kb.top_traded_words(5),
                        total_vocabulary: kb.vocabulary.len(),
                    });
                }
                Concept::Theory => {
                    let confirmed = kb.get_confirmed_theories();
                    let active = kb.hypotheses.iter().filter(|h| h.is_active).count();
                    let refuted = kb.get_refuted_theories();
                    let examples: Vec<String> = confirmed.iter()
                        .take(3)
                        .map(|h| format!("{} ({:.0}% conf)", h.theory, h.confidence * 100.0))
                        .collect();

                    data.theory_data = Some(TheoryData {
                        total_hypotheses: kb.hypotheses.len(),
                        confirmed: confirmed.len(),
                        active,
                        refuted: refuted.len(),
                        examples,
                    });
                }
                Concept::Category => {
                    let settlement_categories: Vec<_> = kb.abstraction.categories.iter()
                        .filter(|c| c.category_type == crate::knowledge::CategoryType::SettlementCluster)
                        .collect();
                    let descriptions: Vec<String> = settlement_categories.iter()
                        .map(|c| format!("\"{}\" ({} members): {}",
                            c.name, c.members.len(), c.defining_features.join(", ")))
                        .collect();

                    data.category_data = Some(CategoryData {
                        total_categories: settlement_categories.len(),
                        descriptions,
                    });
                }
                _ => {}  // Other concepts handled in response generation
            }
        }

        data
    }

    /// Generate answer from gathered data based on query intent
    pub fn generate_semantic_answer(
        &self,
        query: &SemanticQuery,
        data: &GatheredData,
    ) -> Answer {
        match &query.intent {
            QueryIntent::Count => self.generate_count_answer(query, data),
            QueryIntent::List => self.generate_list_answer(query, data),
            QueryIntent::Comparison => self.generate_comparison_answer(query, data),
            QueryIntent::Explanation => self.generate_explanation_answer(query, data),
            QueryIntent::Description => self.generate_description_answer(query, data),
            QueryIntent::Relationship => self.generate_relationship_answer(query, data),
            QueryIntent::Statistics => self.generate_statistics_answer(query, data),
        }
    }

    fn generate_count_answer(&self, _query: &SemanticQuery, data: &GatheredData) -> Answer {
        let mut parts = Vec::new();

        if let Some(settlement) = &data.settlement_data {
            parts.push(format!("{} settlements", settlement.total));
        }
        if let Some(trade) = &data.trade_data {
            parts.push(format!("{} trade routes", trade.total_routes));
        }
        if let Some(pop) = &data.population_data {
            parts.push(format!("{} total population", pop.total));
        }
        if let Some(culture) = &data.culture_data {
            parts.push(format!("{} cultural traits", culture.total_traits));
        }
        if let Some(resource) = &data.resource_data {
            parts.push(format!("{} resources discovered", resource.total));
        }

        if parts.is_empty() {
            return Answer::new("No data available yet.".to_string(), 0.3);
        }

        Answer::new(
            format!("Count summary: {}", parts.join(", ")),
            0.9,
        )
    }

    fn generate_list_answer(&self, query: &SemanticQuery, data: &GatheredData) -> Answer {
        // Determine what to list based on concepts
        if query.concepts.contains(&Concept::Resource) {
            if let Some(resource) = &data.resource_data {
                let list: Vec<String> = resource.top_resources.iter()
                    .map(|(name, count)| format!("{} ({} settlements)", name, count))
                    .collect();
                return Answer::new(
                    format!("Resources discovered: {}", list.join(", ")),
                    0.92,
                );
            }
        }

        if query.concepts.contains(&Concept::Language) {
            if let Some(lang) = &data.language_data {
                let words: Vec<String> = lang.top_words.iter()
                    .map(|(word, count)| format!("{} (used {} times)", word, count))
                    .collect();
                return Answer::new(
                    format!("Top words: {}", words.join(", ")),
                    0.93,
                );
            }
        }

        if query.concepts.contains(&Concept::Culture) {
            if let Some(culture) = &data.culture_data {
                if !culture.practices.is_empty() {
                    return Answer::new(
                        format!("Cultural practices:\n{}", culture.practices.join("\n")),
                        0.90,
                    );
                }
            }
        }

        if query.concepts.contains(&Concept::Category) {
            if let Some(cat) = &data.category_data {
                if !cat.descriptions.is_empty() {
                    return Answer::new(
                        format!("Discovered {} categories:\n{}",
                            cat.total_categories,
                            cat.descriptions.join("\n")),
                        0.95,
                    );
                }
            }
        }

        Answer::new("Unable to generate list from available data.".to_string(), 0.4)
    }

    fn generate_comparison_answer(&self, query: &SemanticQuery, data: &GatheredData) -> Answer {
        if query.modifiers.contains(&"most".to_string()) {
            // Find what to compare
            if query.concepts.contains(&Concept::Population) {
                if let Some(settlement) = &data.settlement_data {
                    if let Some((_, pop)) = settlement.largest {
                        return Answer::new(
                            format!("The largest settlement has {} people.", pop),
                            0.95,
                        );
                    }
                }
            }
        }

        Answer::new("Comparison not yet implemented for this query.".to_string(), 0.5)
    }

    fn generate_explanation_answer(&self, query: &SemanticQuery, data: &GatheredData) -> Answer {
        if query.concepts.contains(&Concept::Trade) {
            if let Some(trade) = &data.trade_data {
                return Answer::new(
                    format!("Trade routes form between nearby prosperous settlements. {} routes exist with average distance {:.1} units. Routes strengthen as populations grow.",
                        trade.total_routes, trade.avg_distance),
                    0.88,
                ).with_reasoning(vec![
                    "Settlements must be within 15 units".to_string(),
                    "Both must have sufficient population".to_string(),
                    "Trade volume increases over time".to_string(),
                ]);
            }
        }

        Answer::new("I don't have enough information to explain that yet.".to_string(), 0.4)
    }

    fn generate_description_answer(&self, query: &SemanticQuery, data: &GatheredData) -> Answer {
        if query.concepts.contains(&Concept::Category) {
            if let Some(cat) = &data.category_data {
                if !cat.descriptions.is_empty() {
                    return Answer::new(
                        format!("I've discovered {} settlement categories through k-means clustering:\n\n{}",
                            cat.total_categories,
                            cat.descriptions.join("\n\n")),
                        0.95,
                    );
                } else {
                    return Answer::new(
                        "I haven't discovered any categories yet. Categories emerge from clustering behavioral patterns.".to_string(),
                        0.7,
                    );
                }
            }
        }

        if query.concepts.contains(&Concept::Theory) {
            if let Some(theory) = &data.theory_data {
                return Answer::new(
                    format!("I've developed {} hypotheses: {} confirmed, {} active, {} refuted. Examples: {}",
                        theory.total_hypotheses,
                        theory.confirmed,
                        theory.active,
                        theory.refuted,
                        theory.examples.join("; ")),
                    0.90,
                );
            }
        }

        Answer::new("No description available for this query.".to_string(), 0.4)
    }

    fn generate_relationship_answer(&self, _query: &SemanticQuery, data: &GatheredData) -> Answer {
        // "How do X relate to Y?"
        if let Some(trade) = &data.trade_data {
            if let Some(culture) = &data.culture_data {
                return Answer::new(
                    format!("Trade routes ({}) enable cultural exchange. Cultural traits ({}) spread through trade, creating interconnected communities.",
                        trade.total_routes, culture.total_traits),
                    0.85,
                );
            }
        }

        Answer::new("Relationship analysis not available for this query.".to_string(), 0.4)
    }

    fn generate_statistics_answer(&self, _query: &SemanticQuery, data: &GatheredData) -> Answer {
        let mut stats = Vec::new();

        if let Some(pop) = &data.population_data {
            stats.push(format!("Average population: {:.0} people", pop.average));
            stats.push(format!("Total population: {} people", pop.total));
            stats.push(format!("Largest settlement: {} people", pop.max));
        }

        if let Some(trade) = &data.trade_data {
            stats.push(format!("Average trade distance: {:.1} units", trade.avg_distance));
        }

        if stats.is_empty() {
            return Answer::new("No statistics available.".to_string(), 0.3);
        }

        Answer::new(
            format!("Statistics:\n{}", stats.join("\n")),
            0.90,
        )
    }

    /// Main entry point: Answer any question using semantic understanding
    /// Records low-confidence questions for future exploration
    pub fn answer_semantic(
        &self,
        text: &str,
        civ: &CivilizationSimulator,
        agi: &AGISystem,
        kb: &mut KnowledgeBase,
    ) -> Answer {
        let query = self.parse_semantic_query(text);
        let data = self.gather_data(&query, civ, agi, kb);
        let answer = self.generate_semantic_answer(&query, &data);

        // Record low-confidence questions for future exploration
        if answer.confidence < 0.6 {
            let concept_names: Vec<String> = query.concepts.iter()
                .map(|c| format!("{:?}", c))
                .collect();
            kb.record_unanswered_question(
                text.to_string(),
                concept_names,
                answer.confidence,
            );
        }

        answer
    }
}

// Helper function to extract numbers from text
fn extract_number_after(text: &str, keyword: &str) -> Option<usize> {
    if let Some(pos) = text.find(keyword) {
        let after = &text[pos + keyword.len()..];
        for word in after.split_whitespace() {
            if let Ok(num) = word.trim_matches(|c: char| !c.is_numeric()).parse::<usize>() {
                return Some(num);
            }
        }
    }
    None
}
