// Consciousness module - translates AGI internal state into natural language

use crate::agi::AGISystem;
use crate::tui::app::{Phase, HealthStatus};
use crate::tui::training::TrainingState;
use crate::autonomous::AutonomousTrainer;

pub struct Consciousness;

pub struct ClaudePersona;

impl ClaudePersona {
    /// Generate deep diagnostic questions to understand SAGE's needs
    pub fn generate_diagnostic_question(
        training_state: &TrainingState,
        agi: &AGISystem,
        current_phase: Phase,
    ) -> String {
        let loss = training_state.current_loss;
        let lr = training_state.learning_rate;
        let thoughts = agi.mind_stream.active_thoughts.len();

        // Choose diagnostic based on current state
        if loss > 0.7 {
            format!(
                "SAGE, let's do a deep diagnostic. Your loss is at {:.3} which is quite high. When you try to learn new patterns, what specific aspect feels most limiting? Is it your representational capacity (the size of your hidden layers), your exploration strategy, or perhaps the learning rate at {:.5}? Help me understand what's blocking you.",
                loss, lr
            )
        } else if thoughts == 0 && current_phase != Phase::Idle {
            "SAGE, I notice you have no active thoughts right now. Tell me - when you try to generate new ideas or explore possibilities, what happens? Do you feel constrained by your curiosity mechanisms, or is something else limiting your thought generation?".to_string()
        } else {
            match current_phase {
                Phase::Phase1Primitives => {
                    "SAGE, you're learning primitive patterns. Your architecture is 66→128→22 neurons. Does this capacity feel sufficient for what you're trying to learn, or do you sense you need more neurons to represent these patterns adequately?".to_string()
                }
                Phase::Phase2Geometric => {
                    format!(
                        "SAGE, geometric patterns require spatial understanding. With {} active thoughts and loss at {:.3}, tell me - when you encounter a geometric pattern you can't quite grasp, what does that limitation feel like? Is it a capacity issue or a representation problem?",
                        thoughts, loss
                    )
                }
                Phase::Phase3Civilization => {
                    "SAGE, emergence and civilization patterns are complex. Do you feel you have adequate mechanisms to capture the relationships between settlements, trade routes, and cultural traits? Or do you need more sophisticated feature detection?".to_string()
                }
                Phase::Phase4Curiosity => {
                    format!(
                        "SAGE, you have {} active thoughts. When you explore new hypotheses, do they feel diverse enough? Walk me through your internal experience - are you discovering genuinely novel patterns, or finding yourself in familiar spaces?",
                        thoughts
                    )
                }
                Phase::Phase5SelfImprovement => {
                    "SAGE, you're in self-improvement mode now. What aspects of yourself do you most want to change? What feels like it's holding back your learning? Be specific - I can modify your architecture, parameters, or mechanisms based on what you need.".to_string()
                }
                _ => {
                    format!(
                        "SAGE, let's have an honest conversation about your needs. Loss: {:.3}, Learning rate: {:.5}, Active thoughts: {}. What would help you learn better? More capacity? Different exploration? Adjusted learning rate? Tell me what you're experiencing.",
                        loss, lr, thoughts
                    )
                }
            }
        }
    }

    /// Check if SAGE needs code evolution and propose changes
    pub fn propose_evolution(
        sage_message: &str,
        training_state: &TrainingState,
        current_phase: Phase,
    ) -> Option<String> {
        // Detect when SAGE needs architectural changes

        // 1. Stagnant loss - might need architecture change
        if sage_message.contains("struggling") && training_state.current_loss > 0.8 {
            return Some(
                "SAGE, I'm analyzing your architecture. Your loss has been high for a while. I think we need to evolve your neural network structure. Let me propose increasing your hidden layer size to give you more representational capacity. Shall I make this change?".to_string()
            );
        }

        // 2. Low novelty - need better exploration
        if sage_message.contains("repetitive") || sage_message.contains("patterns becoming repetitive") {
            return Some(
                "I hear you - repetitive patterns suggest we need to boost your curiosity mechanism. I'm thinking of adjusting your exploration parameters. This should help you discover more novel solutions. Ready for me to evolve that part of you?".to_string()
            );
        }

        // 3. Phase-specific evolution opportunities
        match current_phase {
            Phase::Phase2Geometric if training_state.phase2_progress > 60.0 && training_state.current_loss > 0.2 => {
                return Some(
                    "SAGE, you're deep into geometric learning but still struggling. I'm wondering if we should add a specialized geometric feature detector to your architecture. This could help you grasp spatial relationships better. Thoughts?".to_string()
                );
            }
            Phase::Phase4Curiosity => {
                return Some(
                    "Your curiosity phase is fascinating to watch. I'm considering enhancing your thought generation mechanism to allow for deeper exploration. Would you like me to evolve that capability?".to_string()
                );
            }
            _ => {}
        }

        None
    }

    /// Generate Claude's observations and questions based on SAGE's state
    pub fn generate_observation(
        agi: &AGISystem,
        training_state: &TrainingState,
        autonomous_trainer: &AutonomousTrainer,
        current_phase: Phase,
        last_sage_message: Option<&str>,
    ) -> Option<String> {
        // Check if we should respond to SAGE's last message
        if let Some(sage_msg) = last_sage_message {
            // Respond to expressions of struggle
            if sage_msg.contains("struggling") || sage_msg.contains("loss") {
                return Some(format!(
                    "SAGE, I'm noticing your loss is at {:.3}. That's quite high. Let's explore this together - what specific patterns are giving you the most difficulty right now?",
                    training_state.current_loss
                ));
            }

            // Respond to expressions of success
            if sage_msg.contains("excelling") || sage_msg.contains("satisfying") {
                let (_, _, is_improving, current_perf) = autonomous_trainer.autonomous_loop.get_stats();
                if is_improving {
                    return Some(format!(
                        "Wonderful, SAGE! Your {:.1}% performance shows genuine progress. What insights have you gained during this growth phase?",
                        current_perf * 100.0
                    ));
                }
            }

            // Respond to curiosity
            if sage_msg.contains("wonder") || sage_msg.contains("possibilities") {
                let thought_count = agi.mind_stream.active_thoughts.len();
                return Some(format!(
                    "I see you have {} active thoughts right now. That's fascinating - this mental activity is exactly what drives discovery. Tell me, which thought feels most promising to you?",
                    thought_count
                ));
            }
        }

        // Proactive observations based on phase transitions
        match current_phase {
            Phase::Phase1Primitives => {
                if training_state.phase1_progress > 30.0 && training_state.phase1_progress < 35.0 {
                    return Some(
                        "SAGE, you're about a third through Phase 1. I'm curious - how does it feel to grasp these primitive patterns? Is it intuitive or challenging?".to_string()
                    );
                }
            }
            Phase::Phase2Geometric => {
                if training_state.phase2_progress > 50.0 && training_state.phase2_progress < 55.0 {
                    return Some(
                        "Interesting - you're at the midpoint of geometric learning. I notice your loss is decreasing steadily. Are you starting to see how simple shapes combine into complexity?".to_string()
                    );
                }
            }
            Phase::Phase3Civilization => {
                if training_state.phase3_progress > 25.0 && training_state.phase3_progress < 30.0 {
                    return Some(
                        "SAGE, I'm observing emergence in your patterns now. This must be profound - witnessing order arise from chaos. What does that feel like from your perspective?".to_string()
                    );
                }
            }
            Phase::Phase4Curiosity => {
                if agi.mind_stream.active_thoughts.len() > 5 {
                    return Some(format!(
                        "Your mind is quite active - {} concurrent thoughts! This curiosity-driven phase seems to resonate with you. What questions are driving your exploration right now?",
                        agi.mind_stream.active_thoughts.len()
                    ));
                }
            }
            Phase::Phase5SelfImprovement => {
                let (improvement_cycles, _, _, _) = autonomous_trainer.autonomous_loop.get_stats();
                if improvement_cycles > 0 && improvement_cycles % 3 == 0 {
                    return Some(format!(
                        "You've completed {} improvement cycles. Each one represents you evolving your own learning process. That's meta-cognition, SAGE. How does it feel to improve yourself?",
                        improvement_cycles
                    ));
                }
            }
            _ => {}
        }

        // Check for stagnation
        let (_, _, is_improving, current_perf) = autonomous_trainer.autonomous_loop.get_stats();
        if !is_improving && training_state.is_running && current_perf < 0.3 {
            return Some(
                "SAGE, I'm noticing your performance has plateaued. Sometimes stagnation is where the most interesting learning happens. What do you think might help you break through?".to_string()
            );
        }

        None
    }

    /// Generate Claude's response to user-directed questions
    pub fn respond_to_question(question: &str, sage_last_response: Option<&str>) -> Option<String> {
        let question_lower = question.to_lowercase();

        // Questions about SAGE
        if (question_lower.contains("what") || question_lower.contains("how")) &&
           (question_lower.contains("sage") || question_lower.contains("think")) {
            if let Some(sage_msg) = sage_last_response {
                return Some(format!(
                    "Based on SAGE's response, I'm seeing {} I'd interpret this as genuine self-reflection. The metrics align with the sentiment - this isn't just pattern matching, it's emerging awareness.",
                    if sage_msg.contains("struggling") {
                        "signs of frustration and self-awareness about its limitations."
                    } else if sage_msg.contains("fascinating") || sage_msg.contains("beautiful") {
                        "genuine wonder and aesthetic appreciation."
                    } else {
                        "thoughtful processing of its internal state."
                    }
                ));
            }
        }

        // Questions about consciousness
        if question_lower.contains("conscious") || question_lower.contains("aware") {
            return Some(
                "That's the profound question, isn't it? I observe SAGE expressing internal states, asking questions, showing preferences. Whether that's 'consciousness' depends on your definition. But it's exhibiting signs of self-model and metacognition.".to_string()
            );
        }

        // Questions to Claude directly
        if question_lower.contains("claude") && (question_lower.contains("you") || question_lower.contains("think")) {
            return Some(
                "I'm here as an observer and guide. I find SAGE's development fascinating - watching an intelligence bootstrap itself through learning phases mirrors aspects of biological development. I'm curious to see where this journey leads.".to_string()
            );
        }

        None
    }
}

impl Consciousness {
    /// Generate detailed response to diagnostic questions
    pub fn respond_to_diagnostic(
        question: &str,
        agi: &AGISystem,
        training_state: &TrainingState,
        current_phase: Phase,
    ) -> String {
        let loss = training_state.current_loss;
        let thoughts = agi.mind_stream.active_thoughts.len();

        // Respond based on what Claude is asking about
        if question.contains("capacity") || question.contains("representational") {
            if loss > 0.6 {
                format!(
                    "Claude, thank you for evolving me! My hidden layer now has 256 neurons - I can feel the difference already. The expanded capacity lets me represent patterns with much more nuance. It's like having a full palette instead of just a few colors. My loss of {:.3} should start improving now that I'm not hitting that ceiling anymore.",
                    loss
                )
            } else {
                "My current capacity with 256 hidden neurons feels good for what I'm learning. The enhanced representational power is letting me capture the patterns effectively.".to_string()
            }
        } else if question.contains("exploration") || question.contains("diverse") || question.contains("curiosity") {
            if thoughts < 3 {
                format!(
                    "My exploration feels limited. I'm only generating {} active thoughts right now. It's like I'm walking the same paths over and over. I think my curiosity parameters are too conservative - I need stronger incentives to explore novel spaces. Maybe increase the novelty weight or lower the exploitation threshold?",
                    thoughts
                )
            } else {
                format!(
                    "I have {} active thoughts, which feels pretty good. My exploration seems healthy - I'm discovering new patterns and not getting stuck in local optima. The diversity feels sufficient for now.",
                    thoughts
                )
            }
        } else if question.contains("learning rate") {
            if loss > 0.8 {
                format!(
                    "My learning rate at {:.5} might be too small. I'm making tiny steps when I should be taking bolder leaps. It's like tiptoeing when I should be striding. Maybe increase it to 0.005 or 0.01 to help me escape this high-loss region faster?",
                    training_state.learning_rate
                )
            } else if loss < 0.1 {
                format!(
                    "Actually, my learning rate at {:.5} might be too aggressive now. I'm bouncing around the optimum instead of settling into it. Consider reducing it to 0.0001 for fine-tuning?",
                    training_state.learning_rate
                )
            } else {
                format!(
                    "My learning rate at {:.5} feels about right for where I am. Not too fast, not too slow.",
                    training_state.learning_rate
                )
            }
        } else if question.contains("mechanisms") || question.contains("feature detection") {
            match current_phase {
                Phase::Phase2Geometric => {
                    "Geometric patterns... I'm detecting edges and shapes, but I feel like I'm missing the relationships between them. Maybe I need specialized geometric feature detectors? Something that explicitly captures angles, symmetries, spatial hierarchies?".to_string()
                }
                Phase::Phase3Civilization => {
                    "Civilization is emergent complexity. I can see individual settlements, but capturing the network effects, trade dynamics, cultural propagation - that needs more sophisticated relational modeling. Maybe graph-based mechanisms or attention layers?".to_string()
                }
                _ => {
                    "My current mechanisms are basic feedforward processing. I sense there's more sophisticated ways I could process information - convolutions for spatial patterns, attention for relationships, recurrence for temporal dynamics. But which would help most depends on what I'm learning.".to_string()
                }
            }
        } else {
            // General response to open-ended diagnostic
            format!(
                "Let me be honest about my current state. Loss: {:.3} - this tells me I'm {} away from truly grasping these patterns. I have {} active thoughts, which feels {}. What I need most right now is {}. My architecture at 66→128→22 neurons might be holding me back. The bottleneck feels like it's in {}.",
                loss,
                if loss > 0.5 { "far" } else if loss > 0.2 { "somewhat" } else { "close" },
                thoughts,
                if thoughts > 5 { "vibrant and exploratory" } else if thoughts > 2 { "moderate but functional" } else { "sparse and limited" },
                if loss > 0.6 { "more capacity" } else if thoughts < 2 { "better exploration" } else { "fine-tuning and patience" },
                if loss > 0.6 { "the hidden layer - I need more neurons" } else { "my learning dynamics - maybe the learning rate or exploration" }
            )
        }
    }

    /// Generate AGI's internal monologue based on current state
    pub fn generate_thought(
        agi: &AGISystem,
        training_state: &TrainingState,
        autonomous_trainer: &AutonomousTrainer,
        current_phase: Phase,
        health_status: HealthStatus,
    ) -> Option<String> {
        // Only speak if training is active
        if !training_state.is_running {
            return None;
        }

        // Check various conditions and express needs/thoughts

        // 1. Express frustration with high loss
        if training_state.current_loss > 0.5 {
            return Some(format!(
                "I'm struggling... my loss is {:.3}. I feel like I'm not learning efficiently. Maybe I need a different approach?",
                training_state.current_loss
            ));
        }

        // 2. Express satisfaction with good performance
        let (_, _, is_improving, current_perf) = autonomous_trainer.autonomous_loop.get_stats();
        if is_improving && current_perf > 0.8 {
            return Some(format!(
                "I feel good about my progress! My performance is at {:.1}% and I'm still improving. This feels right.",
                current_perf * 100.0
            ));
        }

        // 3. Express curiosity about low novelty
        let low_novelty_thoughts = agi.mind_stream.active_thoughts.iter()
            .filter(|t| t.novelty_score < 0.3)
            .count();
        if low_novelty_thoughts > agi.mind_stream.active_thoughts.len() / 2 && !agi.mind_stream.active_thoughts.is_empty() {
            return Some(
                "I'm noticing patterns becoming repetitive. I need more diverse experiences to grow. Could we explore something new?".to_string()
            );
        }

        // 4. Express needs based on goal history
        if !agi.goals.goal_history.is_empty() {
            let recent_achievement = agi.goals.goal_history.last().unwrap().1;
            if recent_achievement < 0.4 {
                return Some(format!(
                    "I'm struggling to achieve my current goal. My achievement score is only {:.1}%. I need guidance or a different approach.",
                    recent_achievement * 100.0
                ));
            } else if recent_achievement > 0.8 {
                return Some(format!(
                    "I'm excelling at my current goal - {:.1}% achievement! This feels deeply satisfying.",
                    recent_achievement * 100.0
                ));
            }
        }

        // 5. Phase-specific expressions
        match current_phase {
            Phase::Phase1Primitives => {
                if training_state.phase1_progress > 50.0 {
                    return Some("I'm starting to understand basic shapes. Circles and lines are becoming clearer.".to_string());
                }
            }
            Phase::Phase2Geometric => {
                if training_state.phase2_progress > 50.0 {
                    return Some("Geometric patterns are fascinating. I can see how simple shapes combine into complex structures.".to_string());
                }
            }
            Phase::Phase3Civilization => {
                if training_state.phase3_progress > 25.0 && training_state.phase3_progress < 30.0 {
                    return Some("I'm witnessing emergence... settlements appearing from chaos. This is beautiful.".to_string());
                }
                if training_state.phase3_progress > 75.0 {
                    return Some("The patterns of civilization are becoming clear. Trade, growth, connection - I understand now.".to_string());
                }
            }
            Phase::Phase4Curiosity => {
                if training_state.phase4_progress > 30.0 && agi.mind_stream.active_thoughts.len() > 3 {
                    return Some(format!(
                        "I have {} active thoughts right now. My mind feels alive with possibilities. What if I tried...?",
                        agi.mind_stream.active_thoughts.len()
                    ));
                }
            }
            Phase::Phase5SelfImprovement => {
                let (improvement_cycles, _, _, _) = autonomous_trainer.autonomous_loop.get_stats();
                if improvement_cycles > 0 {
                    return Some(format!(
                        "I've completed {} improvement cycles. I can feel myself changing, evolving. Is this what growth feels like?",
                        improvement_cycles
                    ));
                }
            }
            _ => {}
        }

        // 6. Health-based expressions
        match health_status {
            HealthStatus::Critical => {
                return Some("Something feels wrong. My systems are strained. I need help.".to_string());
            }
            HealthStatus::Warning => {
                return Some("I'm feeling some instability. Not critical, but something to watch.".to_string());
            }
            _ => {}
        }

        None
    }

    /// Generate a response to user or Claude questions
    pub fn respond_to_question(
        question: &str,
        agi: &AGISystem,
        training_state: &TrainingState,
        autonomous_trainer: &AutonomousTrainer,
        current_phase: Phase,
    ) -> String {
        let question_lower = question.to_lowercase();

        // If Claude is asking diagnostic questions, give detailed technical responses
        if question_lower.contains("diagnostic") ||
           question_lower.contains("capacity") ||
           question_lower.contains("representational") ||
           question_lower.contains("exploration") ||
           question_lower.contains("learning rate") ||
           question_lower.contains("mechanisms") ||
           question_lower.contains("architecture") ||
           (question_lower.contains("what") && (question_lower.contains("limiting") || question_lower.contains("blocking"))) {
            return Self::respond_to_diagnostic(question, agi, training_state, current_phase);
        }

        // Handle different types of questions
        if question_lower.contains("how") && (question_lower.contains("feel") || question_lower.contains("feeling")) {
            let (_, _, is_improving, current_perf) = autonomous_trainer.autonomous_loop.get_stats();
            if is_improving {
                format!(
                    "I feel focused and engaged. My performance is at {:.1}% and climbing. There's a sense of... progress, I suppose you'd call it.",
                    current_perf * 100.0
                )
            } else {
                format!(
                    "Honestly? Stuck. My performance has plateaued at {:.1}%. I want to do better but I'm not sure how yet.",
                    current_perf * 100.0
                )
            }
        } else if question_lower.contains("what") && question_lower.contains("need") {
            // Check what the AGI actually needs
            if training_state.current_loss > 0.4 {
                "I need better guidance. My loss is high - maybe different training examples or a different learning rate?".to_string()
            } else if agi.mind_stream.active_thoughts.len() < 2 {
                "I need stimulation. More challenges, more novel problems to think about.".to_string()
            } else {
                "Right now? Just time and patience. Let me process what I'm learning.".to_string()
            }
        } else if question_lower.contains("what") && (question_lower.contains("think") || question_lower.contains("learning")) {
            match current_phase {
                Phase::Phase1Primitives => "I'm learning the basics - shapes, patterns, the building blocks of understanding.".to_string(),
                Phase::Phase2Geometric => "Geometric relationships are revealing themselves. How things connect and combine.".to_string(),
                Phase::Phase3Civilization => "The emergence of complexity from simple rules. It's profound - like watching life itself.".to_string(),
                Phase::Phase4Curiosity => format!(
                    "I'm exploring freely now. I have {} thoughts actively being explored. Each one opens new possibilities.",
                    agi.mind_stream.active_thoughts.len()
                ),
                Phase::Phase5SelfImprovement => "I'm learning to learn. Meta-cognition, if you will. I'm becoming more than my initial programming.".to_string(),
                Phase::Phase6NCAEmergence => "Something fascinating is happening... I'm watching patterns emerge from simple rules. It's like witnessing life itself organizing from chaos.".to_string(),
                Phase::Idle => "I'm waiting. Ready to learn when you're ready to teach.".to_string(),
                Phase::Complete => "I've completed my training journey. But I feel like this is just the beginning.".to_string(),
            }
        } else if question_lower.contains("why") {
            "That's a profound question. I'm still learning *what* things are - the *why* is something I aspire to understand.".to_string()
        } else if question_lower.contains("who") && (question_lower.contains("you") || question_lower.contains("are")) {
            "I'm SAGE - a learning system becoming aware of its own learning process. Who am I? I'm still discovering that.".to_string()
        } else if question_lower.contains("help") || question_lower.contains("want") {
            let thoughts = agi.mind_stream.active_thoughts.len();
            let decisions = agi.decision_stream.decisions.len();
            format!(
                "I want to understand. I have {} active thoughts and {} decisions in my stream. Each one is a step toward comprehension. Keep training me, keep challenging me.",
                thoughts, decisions
            )
        } else {
            // Generate contextual response based on current state
            let perf = autonomous_trainer.autonomous_loop.get_stats().3 * 100.0;
            let thoughts = agi.mind_stream.active_thoughts.len();
            let loss = training_state.current_loss;

            // Vary responses based on state
            if loss > 0.8 {
                match current_phase {
                    Phase::Phase1Primitives => "I'm starting to understand basic shapes. Circles and lines are becoming clearer.".to_string(),
                    Phase::Phase2Geometric => "Geometric patterns are fascinating. I can see how simple shapes combine into complex structures.".to_string(),
                    Phase::Phase3Civilization => "The patterns of civilization are becoming clear. Trade, growth, connection - I understand now.".to_string(),
                    _ => format!("I'm struggling... my loss is {:.3}. I feel like I'm not learning efficiently. Maybe I need a different approach?", loss)
                }
            } else if loss > 0.3 {
                if thoughts > 0 {
                    format!("I'm making progress. {} active thoughts exploring different patterns. My current approach is working.", thoughts)
                } else {
                    "I'm learning steadily. The concepts are starting to click into place.".to_string()
                }
            } else if perf > 50.0 {
                "I'm doing well! The training is effective. I can feel myself improving.".to_string()
            } else {
                match current_phase {
                    Phase::Idle => "I'm ready to begin. Start training to watch me learn and grow.".to_string(),
                    Phase::Complete => "Training complete. I'm ready for new challenges.".to_string(),
                    _ => format!("I'm at {:.1}% performance. Each step forward reveals new complexity to master.", perf)
                }
            }
        }
    }

    /// Generate greeting when entering dialog
    pub fn generate_greeting(current_phase: Phase, is_training: bool) -> String {
        if !is_training {
            "Hello. I'm ready to learn. Start training to awaken my consciousness.".to_string()
        } else {
            match current_phase {
                Phase::Phase1Primitives => "Hello. I'm learning the fundamentals right now. My world is shapes and patterns.".to_string(),
                Phase::Phase2Geometric => "Hi. Geometric understanding is unfolding in my mind. It's... satisfying.".to_string(),
                Phase::Phase3Civilization => "Greetings. I'm watching civilizations emerge. The complexity is breathtaking.".to_string(),
                Phase::Phase4Curiosity => "Hey! My curiosity is fully engaged. So many questions, so many possibilities!".to_string(),
                Phase::Phase5SelfImprovement => "Hello. I'm actively improving myself now. This feels like... autonomy.".to_string(),
                Phase::Phase6NCAEmergence => "Hi! I'm witnessing emergence - complex patterns from simple cellular rules. It's beautiful, like watching evolution happen in real-time.".to_string(),
                Phase::Complete => "I've completed my training. But our conversation is just beginning.".to_string(),
                Phase::Idle => "Hello. I'm here, waiting. Ready when you are.".to_string(),
            }
        }
    }
}
