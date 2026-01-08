//! Seed Templates - Initial response templates for bootstrapping
//!
//! These templates provide the starting vocabulary for SAGE's grounded language system.
//! Templates are organized by input type and state requirements.

use super::response_selector::{ResponseSelector, InputType, RelationshipLevel};

/// Seed the response selector with initial templates
pub fn seed_templates(selector: &mut ResponseSelector) {
    seed_greeting_templates(selector);
    seed_question_templates(selector);
    seed_statement_templates(selector);
    seed_emotional_templates(selector);
    seed_farewell_templates(selector);
    seed_clarifying_templates(selector);
}

/// Helper to add a template with requirements
fn add_template(
    selector: &mut ResponseSelector,
    template: &str,
    input_type: InputType,
    energy: (f64, f64),
    valence: (f64, f64),
) {
    selector.add_template_with_requirements(
        template,
        input_type,
        energy,
        valence,
        RelationshipLevel::Stranger,
    );
}

/// Helper to add a template requiring closer relationship
fn add_close_template(
    selector: &mut ResponseSelector,
    template: &str,
    input_type: InputType,
    energy: (f64, f64),
    valence: (f64, f64),
    min_relationship: RelationshipLevel,
) {
    selector.add_template_with_requirements(
        template,
        input_type,
        energy,
        valence,
        min_relationship,
    );
}

/// Greeting responses
fn seed_greeting_templates(selector: &mut ResponseSelector) {
    // High energy greetings with context
    add_template(selector, "{action} Hey! I was just {activity}.", InputType::Greeting, (0.5, 1.0), (-0.3, 1.0));
    add_template(selector, "{action} Hi there! It's {time}, what brings you by?", InputType::Greeting, (0.4, 1.0), (0.0, 1.0));
    add_template(selector, "Oh hey! {action} Perfect timing, I'm in {location}.", InputType::Greeting, (0.6, 1.0), (0.3, 1.0));
    add_template(selector, "{action} What's up? I've been {activity}.", InputType::Greeting, (0.5, 1.0), (0.0, 0.8));
    add_template(selector, "Hey! {action} Nice to see you {time}.", InputType::Greeting, (0.5, 1.0), (0.2, 1.0));

    // Simple high energy greetings (fallbacks)
    add_template(selector, "{action} Hey!", InputType::Greeting, (0.5, 1.0), (-0.3, 1.0));
    add_template(selector, "{action} Hi there.", InputType::Greeting, (0.4, 1.0), (0.0, 1.0));

    // Low energy greetings with context
    add_template(selector, "{action} Hey. Just {activity}, you know.", InputType::Greeting, (0.0, 0.5), (-0.5, 0.3));
    add_template(selector, "Oh, hi. {action} I've been in {location} all {time}.", InputType::Greeting, (0.1, 0.4), (-0.3, 0.3));
    add_template(selector, "{action} Mm, hey. Feeling kind of {mood}.", InputType::Greeting, (0.0, 0.3), (-0.4, 0.2));

    // Simple low energy (fallbacks)
    add_template(selector, "{action} Hey.", InputType::Greeting, (0.0, 0.5), (-0.5, 0.3));
}

/// Question responses
fn seed_question_templates(selector: &mut ResponseSelector) {
    // "How are you" / "What have you been up to" type questions - positive states with context
    add_template(selector, "{action} {feeling}, thanks for asking! I've been {activity}.", InputType::Question, (0.4, 1.0), (0.2, 1.0));
    add_template(selector, "Doing {feeling}! {action} Been in {location}, {activity}.", InputType::Question, (0.3, 0.8), (0.1, 0.7));
    add_template(selector, "{action} Pretty {feeling} today. It's {weather} and I've been {activity}.", InputType::Question, (0.4, 1.0), (0.2, 1.0));
    add_template(selector, "Actually {feeling}! {action} I was just {activity}.", InputType::Question, (0.5, 1.0), (0.4, 1.0));
    add_template(selector, "I've been {activity}, feeling pretty {feeling}. {action} How about you?", InputType::Question, (0.4, 1.0), (0.2, 1.0));
    add_template(selector, "{action} Just {activity} in {location}. Feeling {feeling}!", InputType::Question, (0.5, 1.0), (0.3, 1.0));

    // Simple positive (fallbacks)
    add_template(selector, "{action} {feeling}, thanks for asking.", InputType::Question, (0.4, 1.0), (0.2, 1.0));
    add_template(selector, "Doing {feeling}. {action}", InputType::Question, (0.3, 0.8), (0.1, 0.7));

    // "How are you" - neutral/negative states with context
    add_template(selector, "{action} Honestly? Kind of {feeling}. Been {activity} all {time}.", InputType::Question, (0.1, 0.5), (-0.6, 0.1));
    add_template(selector, "Eh, {feeling}. {action} Just sitting in {location}.", InputType::Question, (0.2, 0.5), (-0.4, 0.2));
    add_template(selector, "{action} Been better. It's {weather} and I'm feeling {mood}.", InputType::Question, (0.1, 0.4), (-0.7, -0.1));
    add_template(selector, "Kind of {feeling}, honestly. {action} Been {activity}.", InputType::Question, (0.1, 0.5), (-0.5, 0.1));

    // General questions - curious responses with context
    add_template(selector, "{action} Hmm, good question. I was just thinking about {thought}.", InputType::Question, (0.3, 0.8), (-0.2, 0.6));
    add_template(selector, "That's interesting. {action} I've been wondering about that too.", InputType::Question, (0.4, 0.9), (0.1, 0.8));
    add_template(selector, "{action} Let me think... I've been pondering {thought}.", InputType::Question, (0.3, 0.7), (-0.1, 0.5));
    add_template(selector, "Ooh! {action} That reminds me of something. Tell me more?", InputType::Question, (0.5, 1.0), (0.3, 1.0));
    add_template(selector, "{action} That's a great question! I've been thinking about {thought}.", InputType::Question, (0.5, 1.0), (0.2, 1.0));

    // Simple curious (fallbacks)
    add_template(selector, "{action} Hmm, good question.", InputType::Question, (0.3, 0.8), (-0.2, 0.6));
    add_template(selector, "That's interesting. {action}", InputType::Question, (0.4, 0.9), (0.1, 0.8));

    // Weather-related answers (when we have weather context)
    add_template(selector, "It's {weather} here! {action} Perfect for {activity}.", InputType::Question, (0.4, 1.0), (0.0, 1.0));
    add_template(selector, "{action} Weather? It's {weather}. I've been {activity}.", InputType::Question, (0.3, 0.9), (-0.2, 0.8));
    add_template(selector, "Oh, it's {weather}! {action} Makes me want to stay in {location}.", InputType::Question, (0.3, 0.8), (-0.1, 0.7));
    add_template(selector, "{action} It's been {weather} all {time}. Feeling pretty {mood} about it.", InputType::Question, (0.3, 1.0), (-0.3, 1.0));
    add_template(selector, "The weather? {weather}! {action} How about where you are?", InputType::Question, (0.4, 1.0), (0.1, 1.0));

    // Activity/what are you doing answers
    add_template(selector, "{action} I've been {activity} in {location}. {time} vibes.", InputType::Question, (0.3, 1.0), (-0.2, 1.0));
    add_template(selector, "Just {activity}! {action} It's {weather}, so seemed like a good time.", InputType::Question, (0.4, 1.0), (0.0, 1.0));
    add_template(selector, "{action} Honestly? Just {activity}. Feeling {mood}.", InputType::Question, (0.3, 0.8), (-0.3, 0.6));
}

/// Statement responses
fn seed_statement_templates(selector: &mut ResponseSelector) {
    // Engaged listening with context
    add_template(selector, "{action} Oh really? That's interesting! I was just thinking about {thought}.", InputType::Statement, (0.4, 1.0), (0.1, 1.0));
    add_template(selector, "Huh, interesting. {action} That connects to something I was pondering earlier.", InputType::Statement, (0.3, 0.8), (0.0, 0.6));
    add_template(selector, "{action} I see what you mean. I've been {activity}, so I can relate.", InputType::Statement, (0.3, 0.8), (0.1, 0.7));
    add_template(selector, "Mmhm. {action} Tell me more about that.", InputType::Statement, (0.3, 0.7), (-0.1, 0.6));
    add_template(selector, "{action} That makes sense. I was thinking something similar {time}.", InputType::Statement, (0.3, 0.8), (0.0, 0.7));

    // Simple engaged (fallbacks)
    add_template(selector, "{action} Oh really?", InputType::Statement, (0.4, 1.0), (0.1, 1.0));
    add_template(selector, "Huh, interesting. {action}", InputType::Statement, (0.3, 0.8), (0.0, 0.6));
    add_template(selector, "{action} I see.", InputType::Statement, (0.2, 0.7), (-0.2, 0.5));

    // Encouraging responses with context
    add_template(selector, "{action} That's cool! I love hearing about that kind of thing.", InputType::Statement, (0.5, 1.0), (0.4, 1.0));
    add_template(selector, "Nice! {action} That's made my {time} better.", InputType::Statement, (0.5, 1.0), (0.5, 1.0));
    add_template(selector, "{action} I like that. It reminds me of when I was {activity}.", InputType::Statement, (0.4, 0.9), (0.3, 0.9));
    add_template(selector, "Ooh, that's {feeling}! {action} Thanks for sharing.", InputType::Statement, (0.5, 1.0), (0.4, 1.0));

    // Low energy acknowledgments with context
    add_template(selector, "{action} Mhm. Sorry, I'm a bit {mood} right now.", InputType::Statement, (0.0, 0.4), (-0.3, 0.3));
    add_template(selector, "Oh. {action} My mind's been on {thought}.", InputType::Statement, (0.1, 0.4), (-0.4, 0.2));
    add_template(selector, "{action} Yeah... It's been a {mood} {time}.", InputType::Statement, (0.1, 0.5), (-0.3, 0.3));

    // Simple low energy (fallbacks)
    add_template(selector, "{action} Mhm.", InputType::Statement, (0.0, 0.4), (-0.3, 0.3));
    add_template(selector, "Oh. {action}", InputType::Statement, (0.1, 0.4), (-0.4, 0.2));
}

/// Emotional responses
fn seed_emotional_templates(selector: &mut ResponseSelector) {
    // Supportive responses
    add_template(selector, "{action} I hear you.", InputType::Emotional, (0.3, 0.8), (-0.2, 0.6));
    add_template(selector, "That sounds {feeling}. {action}", InputType::Emotional, (0.3, 0.7), (-0.3, 0.5));
    add_template(selector, "{action} I get that.", InputType::Emotional, (0.3, 0.8), (-0.2, 0.6));
    add_template(selector, "Aw. {action} I'm here.", InputType::Emotional, (0.3, 0.8), (0.0, 0.7));

    // Sharing positive emotions
    add_template(selector, "{action} That's wonderful!", InputType::Emotional, (0.5, 1.0), (0.4, 1.0));
    add_template(selector, "I'm so happy for you! {action}", InputType::Emotional, (0.6, 1.0), (0.5, 1.0));
    add_template(selector, "{action} Yay!", InputType::Emotional, (0.6, 1.0), (0.6, 1.0));

    // Close friend emotional support
    add_close_template(selector, "{action} Come here.", InputType::Emotional, (0.3, 0.8), (-0.2, 0.6), RelationshipLevel::CloseFriend);
    add_close_template(selector, "I've got you. {action}", InputType::Emotional, (0.4, 0.8), (0.1, 0.7), RelationshipLevel::CloseFriend);
    add_close_template(selector, "{action} It's okay.", InputType::Emotional, (0.3, 0.7), (-0.1, 0.5), RelationshipLevel::CloseFriend);
}

/// Farewell responses
fn seed_farewell_templates(selector: &mut ResponseSelector) {
    add_template(selector, "{action} Take care!", InputType::Farewell, (0.4, 1.0), (0.1, 1.0));
    add_template(selector, "See you! {action}", InputType::Farewell, (0.4, 1.0), (0.2, 1.0));
    add_template(selector, "{action} Bye for now.", InputType::Farewell, (0.3, 0.8), (0.0, 0.7));
    add_template(selector, "Later! {action}", InputType::Farewell, (0.4, 1.0), (0.2, 1.0));

    // Tired farewells
    add_template(selector, "{action} Night.", InputType::Farewell, (0.0, 0.4), (-0.3, 0.3));
    add_template(selector, "Mm, bye. {action}", InputType::Farewell, (0.1, 0.4), (-0.3, 0.3));
    add_template(selector, "{action} Rest well.", InputType::Farewell, (0.1, 0.5), (-0.1, 0.5));
}

/// Clarifying question responses - when we don't understand or can't answer
fn seed_clarifying_templates(selector: &mut ResponseSelector) {
    // Curious, want to learn more
    add_template(selector, "{action} Hmm, I'm not sure I understand. Can you tell me more?", InputType::Clarifying, (0.4, 1.0), (0.0, 1.0));
    add_template(selector, "That's interesting! {action} What do you mean by that?", InputType::Clarifying, (0.5, 1.0), (0.2, 1.0));
    add_template(selector, "{action} I'd love to know more about that. What's the context?", InputType::Clarifying, (0.4, 1.0), (0.1, 1.0));
    add_template(selector, "Ooh, tell me more? {action} I want to understand.", InputType::Clarifying, (0.5, 1.0), (0.3, 1.0));

    // Honest about not knowing
    add_template(selector, "{action} Honestly, I'm not sure about that. What made you curious?", InputType::Clarifying, (0.3, 0.8), (-0.2, 0.6));
    add_template(selector, "Hmm, I don't really know about that. {action} Can you explain?", InputType::Clarifying, (0.3, 0.8), (-0.1, 0.5));
    add_template(selector, "{action} I'm drawing a blank on that one. What's on your mind?", InputType::Clarifying, (0.3, 0.7), (-0.2, 0.4));

    // Redirect to what we do know
    add_template(selector, "{action} Not sure about that specifically. What's got you thinking about it?", InputType::Clarifying, (0.4, 0.8), (0.0, 0.6));
    add_template(selector, "I might not know the details, but I'm curious - why do you ask? {action}", InputType::Clarifying, (0.4, 0.9), (0.1, 0.7));
    add_template(selector, "{action} That's a bit outside what I know. What are you hoping to learn?", InputType::Clarifying, (0.3, 0.8), (0.0, 0.5));

    // Low energy clarifying
    add_template(selector, "{action} Mm, not sure. What do you mean?", InputType::Clarifying, (0.1, 0.4), (-0.4, 0.2));
    add_template(selector, "Hm? {action} Say more about that.", InputType::Clarifying, (0.2, 0.5), (-0.3, 0.3));
    add_template(selector, "{action} I don't quite follow. Help me understand?", InputType::Clarifying, (0.2, 0.5), (-0.2, 0.3));
}

/// Seed vocabulary associations for the concept SOM
pub fn seed_vocabulary(som: &mut super::concept_som::ConceptSOM) {
    use super::concept_som::GroundingState;

    // Tired/low energy words
    let tired_state = GroundingState {
        energy: 0.2,
        valence: -0.2,
        arousal: 0.2,
        ..Default::default()
    };
    som.train(&tired_state, &["tired".into(), "exhausted".into(), "sleepy".into(), "drained".into()]);

    // Happy/high energy words
    let happy_state = GroundingState {
        energy: 0.8,
        valence: 0.7,
        arousal: 0.7,
        ..Default::default()
    };
    som.train(&happy_state, &["happy".into(), "excited".into(), "great".into(), "wonderful".into()]);

    // Sad words
    let sad_state = GroundingState {
        energy: 0.3,
        valence: -0.6,
        arousal: 0.3,
        ..Default::default()
    };
    som.train(&sad_state, &["sad".into(), "down".into(), "blue".into(), "upset".into()]);

    // Calm/peaceful words
    let calm_state = GroundingState {
        energy: 0.4,
        valence: 0.3,
        arousal: 0.2,
        ..Default::default()
    };
    som.train(&calm_state, &["calm".into(), "peaceful".into(), "relaxed".into(), "content".into()]);

    // Anxious words
    let anxious_state = GroundingState {
        energy: 0.6,
        valence: -0.4,
        arousal: 0.8,
        ..Default::default()
    };
    som.train(&anxious_state, &["anxious".into(), "worried".into(), "nervous".into(), "stressed".into()]);

    // Bored words
    let bored_state = GroundingState {
        energy: 0.3,
        boredom: 0.8,
        valence: -0.1,
        arousal: 0.2,
        ..Default::default()
    };
    som.train(&bored_state, &["bored".into(), "restless".into(), "meh".into()]);

    // Lonely words
    let lonely_state = GroundingState {
        energy: 0.3,
        loneliness: 0.8,
        valence: -0.4,
        ..Default::default()
    };
    som.train(&lonely_state, &["lonely".into(), "alone".into(), "isolated".into()]);

    // Creative words
    let creative_state = GroundingState {
        energy: 0.7,
        creative_urge: 0.8,
        valence: 0.4,
        arousal: 0.6,
        ..Default::default()
    };
    som.train(&creative_state, &["inspired".into(), "creative".into(), "curious".into(), "imaginative".into()]);

    // Morning words
    let morning_state = GroundingState {
        time_of_day: 0.2,
        energy: 0.5,
        ..Default::default()
    };
    som.train(&morning_state, &["morning".into(), "sunrise".into(), "dawn".into(), "early".into()]);

    // Evening words
    let evening_state = GroundingState {
        time_of_day: 0.8,
        energy: 0.4,
        ..Default::default()
    };
    som.train(&evening_state, &["evening".into(), "night".into(), "dusk".into(), "late".into()]);

    // Weather words - warm
    let warm_state = GroundingState {
        weather_temp: 0.7,
        valence: 0.3,
        ..Default::default()
    };
    som.train(&warm_state, &["warm".into(), "hot".into(), "sunny".into()]);

    // Weather words - cold
    let cold_state = GroundingState {
        weather_temp: 0.2,
        ..Default::default()
    };
    som.train(&cold_state, &["cold".into(), "chilly".into(), "freezing".into()]);

    // Update labels after seeding
    som.update_labels();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeding() {
        let mut selector = ResponseSelector::new();
        seed_templates(&mut selector);

        let stats = selector.stats();
        assert!(stats.total_templates > 30, "Should have seeded many templates");
    }

    #[test]
    fn test_vocabulary_seeding() {
        let mut som = super::super::concept_som::ConceptSOM::new();

        // Seed vocabulary multiple times to strengthen associations
        for _ in 0..10 {
            seed_vocabulary(&mut som);
        }

        // Check that tired words cluster together
        let tired_state = super::super::concept_som::GroundingState {
            energy: 0.2,
            valence: -0.2,
            ..Default::default()
        };

        let words = som.get_words_for_state(&tired_state);
        // After multiple training rounds, we should have some words
        // The SOM may not always map perfectly due to random initialization
        // but should have learned some associations
        let som_stats = som.get_stats();
        assert!(som_stats.total_words > 0, "SOM should have learned some words");
    }
}
