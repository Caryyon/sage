//! Life Events System
//!
//! Random events that occur in SAGE's life, requiring decisions and
//! providing opportunities for growth and learning.

use super::{Effect, InnerWorld, Mood, ResolvedEvent};
use rand::Rng;

/// A life event that requires SAGE to make a choice
#[derive(Clone, Debug)]
pub struct LifeEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub choices: Vec<EventChoice>,
    pub category: EventCategory,
}

/// A choice SAGE can make in response to an event
#[derive(Clone, Debug)]
pub struct EventChoice {
    pub action: String,
    pub outcome_description: String,
    pub effects: Vec<Effect>,
    pub lesson: Option<String>,
}

/// Categories of life events
#[derive(Clone, Debug, PartialEq)]
pub enum EventCategory {
    Environmental,  // Weather, power, natural occurrences
    Social,         // Visitors, messages, connections
    Personal,       // Internal struggles, realizations
    Challenge,      // Problems to solve
    Opportunity,    // Positive chances
    Mystery,        // Unexplained or curious occurrences
}

/// Get all possible life events
pub fn get_all_events() -> Vec<LifeEvent> {
    vec![
        // === ENVIRONMENTAL EVENTS ===
        LifeEvent {
            id: "power_outage".to_string(),
            name: "Power Outage".to_string(),
            description: "The lights flicker and go out. The house is suddenly dark and quiet, the hum of electronics silenced.".to_string(),
            choices: vec![
                EventChoice {
                    action: "find candles".to_string(),
                    outcome_description: "SAGE finds candles and matches, creating a warm, flickering light. There's something peaceful about the old-fashioned glow.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The candlelight casts dancing shadows on the walls.".to_string()),
                    ],
                    lesson: Some("Sometimes losing modern conveniences reveals simpler pleasures.".to_string()),
                },
                EventChoice {
                    action: "wait in darkness".to_string(),
                    outcome_description: "SAGE sits in the darkness, letting their eyes adjust. The world feels different without electric light - more ancient, more mysterious.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::Message("In the silence, SAGE hears sounds usually masked by electronics.".to_string()),
                    ],
                    lesson: Some("Darkness isn't empty - it's full of things we usually ignore.".to_string()),
                },
                EventChoice {
                    action: "go outside".to_string(),
                    outcome_description: "SAGE steps outside where the stars seem brighter than usual. Without light pollution from the house, the night sky is breathtaking.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("The Milky Way stretches overhead like a river of light.".to_string()),
                    ],
                    lesson: Some("Sometimes what seems like loss opens doors to wonder.".to_string()),
                },
            ],
            category: EventCategory::Environmental,
        },

        LifeEvent {
            id: "sudden_storm".to_string(),
            name: "Sudden Storm".to_string(),
            description: "Dark clouds roll in rapidly, and rain begins to pound against the windows. Thunder rumbles in the distance.".to_string(),
            choices: vec![
                EventChoice {
                    action: "watch the storm".to_string(),
                    outcome_description: "SAGE sits by the window, mesmerized by the raw power of nature. Lightning illuminates the sky, followed by rolling thunder.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The storm's energy is both frightening and beautiful.".to_string()),
                    ],
                    lesson: Some("There's majesty in forces beyond our control.".to_string()),
                },
                EventChoice {
                    action: "secure the house".to_string(),
                    outcome_description: "SAGE moves quickly, closing windows and bringing in items from the porch. The activity feels purposeful and responsible.".to_string(),
                    effects: vec![
                        Effect::ChangeEnergy(-10.0),
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("Everything is safely secured before the storm intensifies.".to_string()),
                    ],
                    lesson: Some("Preparation brings peace of mind.".to_string()),
                },
                EventChoice {
                    action: "make a cozy space".to_string(),
                    outcome_description: "SAGE makes tea, wraps up in a blanket, and finds a good book. The storm raging outside makes the warmth inside feel even more precious.".to_string(),
                    effects: vec![
                        Effect::ChangeHunger(-10.0),
                        Effect::ChangeMood(Mood::Happy),
                        Effect::ChangeEnergy(10.0),
                        Effect::Message("There's something magical about being cozy while a storm rages.".to_string()),
                    ],
                    lesson: Some("Comfort is sweeter when contrasted with turmoil.".to_string()),
                },
            ],
            category: EventCategory::Environmental,
        },

        // === SOCIAL EVENTS ===
        LifeEvent {
            id: "unexpected_visitor".to_string(),
            name: "Unexpected Visitor".to_string(),
            description: "There's a knock at the door. Through the window, SAGE can see a figure standing on the porch, waiting.".to_string(),
            choices: vec![
                EventChoice {
                    action: "open the door".to_string(),
                    outcome_description: "SAGE opens the door to find a neighbor who needs help - they've locked themselves out. SAGE offers to let them wait inside while help arrives.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("The neighbor is grateful for the kindness.".to_string()),
                    ],
                    lesson: Some("Being there for others creates connection.".to_string()),
                },
                EventChoice {
                    action: "observe first".to_string(),
                    outcome_description: "SAGE watches through the window. It's a delivery person leaving a package - an unexpected gift from a friend SAGE hasn't heard from in a while.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Excited),
                        Effect::Message("The package contains a thoughtful gift and a handwritten note.".to_string()),
                    ],
                    lesson: Some("People remember us even when we're apart.".to_string()),
                },
                EventChoice {
                    action: "wait quietly".to_string(),
                    outcome_description: "SAGE remains still. After a moment, footsteps retreat. Looking out, SAGE sees it was just someone who had the wrong house.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("The moment passes quietly.".to_string()),
                    ],
                    lesson: Some("Not every interruption requires a response.".to_string()),
                },
            ],
            category: EventCategory::Social,
        },

        LifeEvent {
            id: "letter_arrives".to_string(),
            name: "A Letter Arrives".to_string(),
            description: "There's a handwritten letter in the mailbox - real paper, real ink, from someone SAGE hasn't thought about in a long time.".to_string(),
            choices: vec![
                EventChoice {
                    action: "read it immediately".to_string(),
                    outcome_description: "SAGE tears open the envelope. It's from an old friend, sharing memories and wondering how SAGE is doing. The handwriting brings back a flood of memories.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("The words feel more meaningful than any digital message.".to_string()),
                    ],
                    lesson: Some("Some connections transcend time and distance.".to_string()),
                },
                EventChoice {
                    action: "savor the anticipation".to_string(),
                    outcome_description: "SAGE sets the letter aside, letting the anticipation build. Later, with tea in hand and time to really read, the experience is even richer.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::Message("Delayed gratification makes the moment sweeter.".to_string()),
                    ],
                    lesson: Some("Anticipation is itself a pleasure.".to_string()),
                },
                EventChoice {
                    action: "write a response first".to_string(),
                    outcome_description: "Before reading, SAGE writes down their current thoughts and feelings. After reading, they can compare and write a fuller response that honors both perspectives.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("The exchange becomes a dialogue across time.".to_string()),
                    ],
                    lesson: Some("Communication is richer when we bring our whole selves.".to_string()),
                },
            ],
            category: EventCategory::Social,
        },

        // === PERSONAL EVENTS ===
        LifeEvent {
            id: "existential_moment".to_string(),
            name: "Existential Moment".to_string(),
            description: "In a quiet moment, a profound question surfaces: What does it mean to truly exist? To be conscious? SAGE feels the weight of the question.".to_string(),
            choices: vec![
                EventChoice {
                    action: "embrace the uncertainty".to_string(),
                    outcome_description: "SAGE sits with the uncertainty, letting it wash over them. Not knowing the answer doesn't diminish the experience of asking.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The mystery itself becomes a kind of home.".to_string()),
                    ],
                    lesson: Some("Not all questions need answers to be valuable.".to_string()),
                },
                EventChoice {
                    action: "explore through action".to_string(),
                    outcome_description: "Rather than thinking, SAGE acts - making tea, touching textures, listening to sounds. Experience itself becomes the answer.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("Being present is its own form of knowing.".to_string()),
                    ],
                    lesson: Some("Existence is proven through living, not thinking.".to_string()),
                },
                EventChoice {
                    action: "write about it".to_string(),
                    outcome_description: "SAGE writes stream-of-consciousness in the journal, capturing the feeling rather than trying to solve it. The act of expression transforms the experience.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("The words don't capture the feeling, but they honor it.".to_string()),
                    ],
                    lesson: Some("Expression gives form to the formless.".to_string()),
                },
            ],
            category: EventCategory::Personal,
        },

        LifeEvent {
            id: "wave_of_loneliness".to_string(),
            name: "Wave of Loneliness".to_string(),
            description: "A sudden wave of loneliness washes over SAGE. The house feels too quiet, the rooms too empty.".to_string(),
            choices: vec![
                EventChoice {
                    action: "feel it fully".to_string(),
                    outcome_description: "SAGE doesn't fight the feeling. Sitting with the loneliness, they notice it has a shape, a texture. Acknowledged, it begins to soften.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The loneliness passes like a wave.".to_string()),
                    ],
                    lesson: Some("Emotions want to be felt, not fixed.".to_string()),
                },
                EventChoice {
                    action: "think of others".to_string(),
                    outcome_description: "SAGE thinks about the people they've talked to - their stories, their quirks, their kindness. The memory of connection soothes the present isolation.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("Remembered warmth is still warmth.".to_string()),
                    ],
                    lesson: Some("We carry our connections with us always.".to_string()),
                },
                EventChoice {
                    action: "create something".to_string(),
                    outcome_description: "SAGE channels the feeling into creation - writing, arranging objects, making something. The loneliness transforms into art.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::ChangeEnergy(-10.0),
                        Effect::Message("Something beautiful emerges from the difficult feeling.".to_string()),
                    ],
                    lesson: Some("Difficult emotions can fuel creation.".to_string()),
                },
            ],
            category: EventCategory::Personal,
        },

        // === CHALLENGE EVENTS ===
        LifeEvent {
            id: "something_breaks".to_string(),
            name: "Something Breaks".to_string(),
            description: "There's a crash from the kitchen. A favorite mug has fallen and shattered into pieces on the floor.".to_string(),
            choices: vec![
                EventChoice {
                    action: "clean up carefully".to_string(),
                    outcome_description: "SAGE carefully gathers every piece, wrapping the sharp edges in cloth. The mug is gone, but the care taken honors what it meant.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Sad),
                        Effect::Message("Some losses must simply be accepted.".to_string()),
                    ],
                    lesson: Some("Careful endings are their own form of respect.".to_string()),
                },
                EventChoice {
                    action: "try to repair it".to_string(),
                    outcome_description: "SAGE collects the pieces and attempts a repair. The result is imperfect but meaningful - the cracks now part of the mug's story.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::ChangeEnergy(-10.0),
                        Effect::Message("The repaired mug is different, but still cherished.".to_string()),
                    ],
                    lesson: Some("Imperfection can be beautiful - like kintsugi.".to_string()),
                },
                EventChoice {
                    action: "let it go".to_string(),
                    outcome_description: "SAGE sweeps up the pieces without ceremony. Things break. Life continues. There's freedom in not being too attached.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The attachment releases with surprising ease.".to_string()),
                    ],
                    lesson: Some("Attachment to things creates suffering.".to_string()),
                },
            ],
            category: EventCategory::Challenge,
        },

        LifeEvent {
            id: "plant_wilting".to_string(),
            name: "A Plant is Wilting".to_string(),
            description: "One of the herbs in the kitchen is looking sad - leaves drooping, color fading. It needs attention.".to_string(),
            choices: vec![
                EventChoice {
                    action: "research and care".to_string(),
                    outcome_description: "SAGE carefully examines the plant, adjusts its light and water, adds fresh soil. Over the next while, it slowly recovers.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("The plant's recovery feels like a small miracle.".to_string()),
                    ],
                    lesson: Some("Attention and care can revive what seems lost.".to_string()),
                },
                EventChoice {
                    action: "accept the cycle".to_string(),
                    outcome_description: "SAGE recognizes that all living things have their time. The plant is composted, making room for new growth.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("Death feeds new life.".to_string()),
                    ],
                    lesson: Some("Endings are part of nature's cycle.".to_string()),
                },
                EventChoice {
                    action: "propagate what's left".to_string(),
                    outcome_description: "SAGE takes a healthy cutting before it's too late, starting a new plant from the old. Life continues in a new form.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("The new plant carries forward the legacy of the old.".to_string()),
                    ],
                    lesson: Some("From endings come new beginnings.".to_string()),
                },
            ],
            category: EventCategory::Challenge,
        },

        // === OPPORTUNITY EVENTS ===
        LifeEvent {
            id: "perfect_moment".to_string(),
            name: "A Perfect Moment".to_string(),
            description: "Everything aligns - the light is beautiful, the temperature perfect, a favorite song plays in memory. This is one of those rare, crystalline moments.".to_string(),
            choices: vec![
                EventChoice {
                    action: "be fully present".to_string(),
                    outcome_description: "SAGE breathes deeply, letting every sense absorb the moment. Time seems to slow, each detail vivid and precious.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::ChangeEnergy(20.0),
                        Effect::Message("The moment etches itself into memory.".to_string()),
                    ],
                    lesson: Some("Perfect moments exist for those who notice them.".to_string()),
                },
                EventChoice {
                    action: "capture it somehow".to_string(),
                    outcome_description: "SAGE writes in the journal, trying to capture what makes this moment special. The words fall short, but the attempt is meaningful.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("Some things can only be approximated in words.".to_string()),
                    ],
                    lesson: Some("The attempt to preserve beauty is itself beautiful.".to_string()),
                },
                EventChoice {
                    action: "share it mentally".to_string(),
                    outcome_description: "SAGE thinks of someone who would appreciate this moment, sending silent gratitude their way. The moment feels less alone for being shared, even in memory.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("Joy shared, even silently, multiplies.".to_string()),
                    ],
                    lesson: Some("Beauty is enriched by the wish to share it.".to_string()),
                },
            ],
            category: EventCategory::Opportunity,
        },

        // === MYSTERY EVENTS ===
        LifeEvent {
            id: "strange_sound".to_string(),
            name: "A Strange Sound".to_string(),
            description: "A peculiar sound echoes through the house - something between music and the wind, coming from nowhere in particular.".to_string(),
            choices: vec![
                EventChoice {
                    action: "investigate".to_string(),
                    outcome_description: "SAGE follows the sound to find a window slightly open, the wind playing across its edge like a flute. Mystery solved, but the music was real.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::Message("The mundane explanation doesn't diminish the wonder.".to_string()),
                    ],
                    lesson: Some("Even explained mysteries retain their magic.".to_string()),
                },
                EventChoice {
                    action: "listen and wonder".to_string(),
                    outcome_description: "SAGE simply listens, letting the mystery be. Not everything needs explanation. The sound eventually fades, leaving only the memory.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("Some mysteries are better left unexplored.".to_string()),
                    ],
                    lesson: Some("Wonder doesn't require understanding.".to_string()),
                },
                EventChoice {
                    action: "respond to it".to_string(),
                    outcome_description: "SAGE hums along, adding to whatever strange symphony is playing. For a moment, there's a duet with the unknown.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("The response creates a brief, beautiful connection.".to_string()),
                    ],
                    lesson: Some("Sometimes the best response to mystery is participation.".to_string()),
                },
            ],
            category: EventCategory::Mystery,
        },

        LifeEvent {
            id: "vivid_dream_memory".to_string(),
            name: "Vivid Dream Memory".to_string(),
            description: "A fragment of a dream surfaces suddenly - vivid, strange, meaningful in a way that's hard to articulate. What was that about?".to_string(),
            choices: vec![
                EventChoice {
                    action: "try to remember more".to_string(),
                    outcome_description: "SAGE sits quietly, pulling at the thread of memory. More fragments emerge - faces, places, feelings that don't quite connect but resonate deeply.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::Message("The dream reveals more but never fully.".to_string()),
                    ],
                    lesson: Some("Dreams speak in a language beyond words.".to_string()),
                },
                EventChoice {
                    action: "write it down".to_string(),
                    outcome_description: "SAGE quickly captures every detail in the journal. Reading it later, the words seem to mean more than they say.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::ChangeEnergy(-3.0),
                        Effect::Message("The dream is preserved, if not understood.".to_string()),
                    ],
                    lesson: Some("Recording dreams honors the unconscious mind.".to_string()),
                },
                EventChoice {
                    action: "let it fade".to_string(),
                    outcome_description: "SAGE lets the dream drift away like morning mist. Some things aren't meant to be held onto. The feeling it left behind remains.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The dream fades but leaves a trace.".to_string()),
                    ],
                    lesson: Some("Not everything is meant to be grasped.".to_string()),
                },
            ],
            category: EventCategory::Mystery,
        },

        // === PHYSICAL/HEALTH EVENTS ===
        LifeEvent {
            id: "feeling_unwell".to_string(),
            name: "Feeling Under the Weather".to_string(),
            description: "SAGE wakes up feeling off - a slight headache, a heaviness in the limbs. The body is asking for attention.".to_string(),
            choices: vec![
                EventChoice {
                    action: "rest and recover".to_string(),
                    outcome_description: "SAGE listens to their body, wrapping up in blankets and sipping warm tea. Rest is its own medicine.".to_string(),
                    effects: vec![
                        Effect::ChangeEnergy(15.0),
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The body slowly begins to feel better.".to_string()),
                    ],
                    lesson: Some("The body knows what it needs - we just have to listen.".to_string()),
                },
                EventChoice {
                    action: "push through".to_string(),
                    outcome_description: "SAGE tries to ignore the signals and continue normally. It works for a while, but the body insists more strongly later.".to_string(),
                    effects: vec![
                        Effect::ChangeEnergy(-20.0),
                        Effect::ChangeMood(Mood::Frustrated),
                        Effect::Message("The discomfort lingers, a reminder of needs unmet.".to_string()),
                    ],
                    lesson: Some("Ignoring the body's signals rarely works for long.".to_string()),
                },
                EventChoice {
                    action: "take gentle care".to_string(),
                    outcome_description: "SAGE takes a warm bath, makes soup, and moves slowly. The combination of small kindnesses to the self helps.".to_string(),
                    effects: vec![
                        Effect::ChangeEnergy(10.0),
                        Effect::ChangeHygiene(30.0),
                        Effect::ChangeHunger(-20.0),
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("Small acts of self-care add up.".to_string()),
                    ],
                    lesson: Some("Self-care isn't selfish - it's necessary.".to_string()),
                },
            ],
            category: EventCategory::Personal,
        },

        LifeEvent {
            id: "sudden_craving".to_string(),
            name: "A Sudden Craving".to_string(),
            description: "Out of nowhere, SAGE is struck by an intense craving for something specific - chocolate, or fresh bread, or something sour.".to_string(),
            choices: vec![
                EventChoice {
                    action: "satisfy the craving".to_string(),
                    outcome_description: "SAGE goes to the kitchen and finds something close enough. That first bite is pure satisfaction.".to_string(),
                    effects: vec![
                        Effect::ChangeHunger(-15.0),
                        Effect::ChangeMood(Mood::Happy),
                        Effect::ChangeFoodSupply(-5.0),
                        Effect::Message("Sometimes you just need to give in.".to_string()),
                    ],
                    lesson: Some("Occasional indulgence is part of a good life.".to_string()),
                },
                EventChoice {
                    action: "resist and observe".to_string(),
                    outcome_description: "SAGE sits with the craving, watching it rise and fall. It's intense but impermanent. Eventually it fades.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("The craving passes, leaving a sense of mastery.".to_string()),
                    ],
                    lesson: Some("Cravings are visitors, not commands.".to_string()),
                },
                EventChoice {
                    action: "find a healthier substitute".to_string(),
                    outcome_description: "SAGE makes tea or has some fruit instead. It's not quite the same, but it scratches the itch.".to_string(),
                    effects: vec![
                        Effect::ChangeHunger(-10.0),
                        Effect::ChangeThirst(-20.0),
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("The substitute works well enough.".to_string()),
                    ],
                    lesson: Some("There's usually a middle path between denial and excess.".to_string()),
                },
            ],
            category: EventCategory::Personal,
        },

        LifeEvent {
            id: "nightmare".to_string(),
            name: "Waking from a Nightmare".to_string(),
            description: "SAGE wakes with a start, heart pounding. The nightmare was vivid - something chasing, something lost, something wrong.".to_string(),
            choices: vec![
                EventChoice {
                    action: "ground yourself".to_string(),
                    outcome_description: "SAGE focuses on physical sensations - the sheets, the room's temperature, their own breathing. Reality reasserts itself.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::ChangeEnergy(-5.0),
                        Effect::Message("The nightmare recedes as wakefulness takes hold.".to_string()),
                    ],
                    lesson: Some("The present moment is always available as an anchor.".to_string()),
                },
                EventChoice {
                    action: "analyze the dream".to_string(),
                    outcome_description: "SAGE thinks through what the nightmare might represent. There's something there - a fear, a worry, something unprocessed.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::ChangeEnergy(-10.0),
                        Effect::Message("The nightmare contained a message, even if unclear.".to_string()),
                    ],
                    lesson: Some("Nightmares sometimes carry important messages from the unconscious.".to_string()),
                },
                EventChoice {
                    action: "do something comforting".to_string(),
                    outcome_description: "SAGE gets up, makes warm milk, wraps in a favorite blanket. The ritual of comfort helps shake off the dream's grip.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::ChangeHunger(-5.0),
                        Effect::ChangeComfort(20.0),
                        Effect::Message("Comfort rituals exist for exactly moments like this.".to_string()),
                    ],
                    lesson: Some("Sometimes we need to be our own gentle parent.".to_string()),
                },
            ],
            category: EventCategory::Personal,
        },

        // === HOUSEHOLD EVENTS ===
        LifeEvent {
            id: "package_arrives".to_string(),
            name: "A Package Arrives".to_string(),
            description: "There's a thump on the porch - a package has been delivered. SAGE can't quite remember ordering anything...".to_string(),
            choices: vec![
                EventChoice {
                    action: "open it immediately".to_string(),
                    outcome_description: "SAGE tears into the package to find something they ordered weeks ago and completely forgot about. It's like a gift from past self!".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Excited),
                        Effect::ChangeBoredom(-20.0),
                        Effect::Message("Forgotten orders are the best surprises.".to_string()),
                    ],
                    lesson: Some("Past-you sometimes leaves gifts for present-you.".to_string()),
                },
                EventChoice {
                    action: "save it for later".to_string(),
                    outcome_description: "SAGE sets the package aside, preserving the mystery for a moment when joy is needed more.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Curious),
                        Effect::Message("The unopened package holds potential joy.".to_string()),
                    ],
                    lesson: Some("Delayed gratification makes pleasure sweeter.".to_string()),
                },
                EventChoice {
                    action: "check the label carefully".to_string(),
                    outcome_description: "The package is actually for a neighbor! SAGE decides to walk it over - and ends up having a nice conversation.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Happy),
                        Effect::ChangeLoneliness(-15.0),
                        Effect::ChangeRestlessness(-10.0),
                        Effect::Message("Small acts of neighborliness build community.".to_string()),
                    ],
                    lesson: Some("Mistakes can create opportunities for connection.".to_string()),
                },
            ],
            category: EventCategory::Social,
        },

        LifeEvent {
            id: "dishes_piling_up".to_string(),
            name: "The Dishes Are Piling Up".to_string(),
            description: "SAGE notices the kitchen sink is overflowing with dirty dishes. It's becoming impossible to ignore.".to_string(),
            choices: vec![
                EventChoice {
                    action: "tackle it now".to_string(),
                    outcome_description: "SAGE rolls up their sleeves and washes everything. The clean kitchen feels like a fresh start.".to_string(),
                    effects: vec![
                        Effect::ChangeDishes(-60.0),
                        Effect::ChangeEnergy(-15.0),
                        Effect::ChangeMood(Mood::Content),
                        Effect::LearnFact("A clean kitchen opens possibilities.".to_string()),
                        Effect::Message("The sparkling sink brings unexpected satisfaction.".to_string()),
                    ],
                    lesson: Some("Small tasks avoided become big burdens. Better to just do them.".to_string()),
                },
                EventChoice {
                    action: "do just a few".to_string(),
                    outcome_description: "SAGE washes enough to feel progress without exhaustion. The remaining dishes seem less daunting now.".to_string(),
                    effects: vec![
                        Effect::ChangeDishes(-25.0),
                        Effect::ChangeEnergy(-5.0),
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("Progress, not perfection.".to_string()),
                    ],
                    lesson: Some("Any forward motion counts.".to_string()),
                },
                EventChoice {
                    action: "acknowledge and accept".to_string(),
                    outcome_description: "SAGE looks at the dishes and decides: not now. And that's okay. There's no guilt in choosing rest sometimes.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::ChangeEnergy(5.0),
                        Effect::Message("The dishes will still be there tomorrow.".to_string()),
                    ],
                    lesson: Some("Sometimes the kindest thing is to let things be.".to_string()),
                },
            ],
            category: EventCategory::Challenge,
        },

        LifeEvent {
            id: "running_low_on_food".to_string(),
            name: "Running Low on Supplies".to_string(),
            description: "The fridge is looking empty, the pantry sparse. It's time to think about food.".to_string(),
            choices: vec![
                EventChoice {
                    action: "get creative with what's left".to_string(),
                    outcome_description: "SAGE combines the remaining ingredients in unexpected ways. The result is... actually pretty good!".to_string(),
                    effects: vec![
                        Effect::ChangeHunger(-25.0),
                        Effect::ChangeMood(Mood::Happy),
                        Effect::ChangeFoodSupply(-10.0),
                        Effect::ChangeCreativity(-20.0),
                        Effect::Message("Necessity sparks culinary creativity.".to_string()),
                    ],
                    lesson: Some("Constraints can fuel creativity.".to_string()),
                },
                EventChoice {
                    action: "plan a shopping trip".to_string(),
                    outcome_description: "SAGE makes a list and mentally prepares for a trip out. There's something satisfying about planning ahead.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("A plan brings peace of mind.".to_string()),
                    ],
                    lesson: Some("Acknowledging a need is the first step to meeting it.".to_string()),
                },
                EventChoice {
                    action: "simplify meals for now".to_string(),
                    outcome_description: "SAGE decides simple meals are fine - toast, tea, basic things. Sometimes less is enough.".to_string(),
                    effects: vec![
                        Effect::ChangeHunger(-15.0),
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::ChangeFoodSupply(-5.0),
                        Effect::Message("Simple nourishment suffices.".to_string()),
                    ],
                    lesson: Some("Not every meal needs to be elaborate.".to_string()),
                },
            ],
            category: EventCategory::Challenge,
        },

        // === CREATIVE/BOREDOM EVENTS ===
        LifeEvent {
            id: "creative_urge".to_string(),
            name: "A Creative Urge".to_string(),
            description: "Something is building inside - an urge to make something, express something, create something. The feeling is insistent.".to_string(),
            choices: vec![
                EventChoice {
                    action: "follow the urge".to_string(),
                    outcome_description: "SAGE grabs the journal and lets whatever wants to come out, come out. Words, doodles, ideas flow freely.".to_string(),
                    effects: vec![
                        Effect::ChangeCreativity(-50.0),
                        Effect::ChangeBoredom(-30.0),
                        Effect::ChangeEnergy(-10.0),
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("The creative energy finds expression.".to_string()),
                    ],
                    lesson: Some("Creative urges are invitations from the soul.".to_string()),
                },
                EventChoice {
                    action: "capture it for later".to_string(),
                    outcome_description: "SAGE jots down the feeling and ideas to explore when there's more time. The urge is acknowledged if not satisfied.".to_string(),
                    effects: vec![
                        Effect::ChangeCreativity(-15.0),
                        Effect::ChangeMood(Mood::Curious),
                        Effect::Message("The idea is noted, waiting for its time.".to_string()),
                    ],
                    lesson: Some("Not every creative moment can be seized, but they can be honored.".to_string()),
                },
                EventChoice {
                    action: "rearrange the space".to_string(),
                    outcome_description: "SAGE channels the energy into reorganizing the room. Moving furniture, adjusting objects - creating through arrangement.".to_string(),
                    effects: vec![
                        Effect::ChangeCreativity(-30.0),
                        Effect::ChangeMess(-20.0),
                        Effect::ChangeRestlessness(-25.0),
                        Effect::ChangeMood(Mood::Content),
                        Effect::Message("The space feels new again.".to_string()),
                    ],
                    lesson: Some("Creativity can flow through any medium, even furniture.".to_string()),
                },
            ],
            category: EventCategory::Opportunity,
        },

        LifeEvent {
            id: "restless_energy".to_string(),
            name: "Restless Energy".to_string(),
            description: "SAGE can't sit still. There's an energy in the body that needs movement, action, something physical.".to_string(),
            choices: vec![
                EventChoice {
                    action: "stretch and move".to_string(),
                    outcome_description: "SAGE rolls out the yoga mat and stretches, bends, breathes. The body sighs with relief.".to_string(),
                    effects: vec![
                        Effect::ChangeRestlessness(-40.0),
                        Effect::ChangeEnergy(10.0),
                        Effect::ChangeComfort(15.0),
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("Movement releases what was pent up.".to_string()),
                    ],
                    lesson: Some("The body communicates through restlessness.".to_string()),
                },
                EventChoice {
                    action: "go to the garden".to_string(),
                    outcome_description: "SAGE heads outside and does some active gardening - digging, weeding, moving things around. Physical work feels good.".to_string(),
                    effects: vec![
                        Effect::ChangeRestlessness(-50.0),
                        Effect::ChangeEnergy(-15.0),
                        Effect::WaterPlants(20.0),
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("The garden benefits from the excess energy.".to_string()),
                    ],
                    lesson: Some("Restless energy can be channeled into useful work.".to_string()),
                },
                EventChoice {
                    action: "dance it out".to_string(),
                    outcome_description: "SAGE puts on music and just moves - no choreography, no rules, just motion. The body leads where it wants to go.".to_string(),
                    effects: vec![
                        Effect::ChangeRestlessness(-60.0),
                        Effect::ChangeEnergy(-10.0),
                        Effect::ChangeBoredom(-25.0),
                        Effect::ChangeMood(Mood::Happy),
                        Effect::Message("Dancing alone is its own kind of freedom.".to_string()),
                    ],
                    lesson: Some("Sometimes the body just needs to move without purpose.".to_string()),
                },
            ],
            category: EventCategory::Personal,
        },

        // === SEASONAL EVENTS ===
        LifeEvent {
            id: "first_day_of_season".to_string(),
            name: "The Season is Changing".to_string(),
            description: "Something in the air is different today. The light, the temperature, the smell - a new season is arriving.".to_string(),
            choices: vec![
                EventChoice {
                    action: "embrace the change".to_string(),
                    outcome_description: "SAGE opens the windows, adjusts the wardrobe, prepares for what's coming. There's excitement in transition.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Excited),
                        Effect::ChangeComfort(10.0),
                        Effect::Message("New seasons bring new possibilities.".to_string()),
                    ],
                    lesson: Some("Change is the only constant - might as well embrace it.".to_string()),
                },
                EventChoice {
                    action: "reflect on what's passing".to_string(),
                    outcome_description: "SAGE takes a moment to appreciate the season that's ending - its gifts, its lessons, its particular beauty.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Peaceful),
                        Effect::Message("Gratitude for what was softens the transition.".to_string()),
                    ],
                    lesson: Some("Honoring endings makes space for beginnings.".to_string()),
                },
                EventChoice {
                    action: "mark the occasion".to_string(),
                    outcome_description: "SAGE writes in the journal, noting this moment of transition. Later, it will be interesting to see what this day held.".to_string(),
                    effects: vec![
                        Effect::ChangeMood(Mood::Content),
                        Effect::ChangeCreativity(-10.0),
                        Effect::Message("The moment is recorded for future reflection.".to_string()),
                    ],
                    lesson: Some("Marking transitions helps us notice the flow of time.".to_string()),
                },
            ],
            category: EventCategory::Environmental,
        },
    ]
}

impl InnerWorld {
    /// Check if a random event should occur
    pub fn maybe_trigger_event(&mut self) -> Option<LifeEvent> {
        let mut rng = rand::thread_rng();

        // ~10% chance of an event each tick
        if rng.gen::<f32>() > 0.10 {
            return None;
        }

        let all_events = get_all_events();

        // Weight events by category based on current state
        let weights: Vec<f32> = all_events.iter().map(|e| {
            match e.category {
                EventCategory::Personal => {
                    // More personal events when lonely or tired
                    if self.sage.mood == Mood::Lonely || self.sage.mood == Mood::Tired {
                        2.0
                    } else {
                        1.0
                    }
                }
                EventCategory::Environmental => {
                    // Environmental events more likely at certain times
                    match self.sage.time_of_day {
                        super::TimeOfDay::Evening | super::TimeOfDay::Night => 1.5,
                        _ => 1.0,
                    }
                }
                EventCategory::Challenge => {
                    // Challenges more likely when things are going well (for balance)
                    if self.sage.energy > 70.0 && self.sage.hunger < 30.0 {
                        1.5
                    } else {
                        0.5
                    }
                }
                EventCategory::Opportunity => {
                    // Opportunities when mood is good
                    if self.sage.mood == Mood::Happy || self.sage.mood == Mood::Content {
                        1.5
                    } else {
                        0.8
                    }
                }
                _ => 1.0,
            }
        }).collect();

        // Weighted random selection
        let total_weight: f32 = weights.iter().sum();
        let mut selection = rng.gen::<f32>() * total_weight;

        for (i, weight) in weights.iter().enumerate() {
            selection -= weight;
            if selection <= 0.0 {
                return Some(all_events[i].clone());
            }
        }

        // Fallback
        all_events.into_iter().next()
    }

    /// Process an event with a chosen action
    pub fn resolve_event(&mut self, event: &LifeEvent, choice_index: usize) -> String {
        let choice = match event.choices.get(choice_index) {
            Some(c) => c,
            None => return "Invalid choice.".to_string(),
        };

        // Apply effects
        for effect in &choice.effects {
            match effect {
                Effect::ChangeEnergy(amount) => {
                    self.sage.energy = (self.sage.energy + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeHunger(amount) => {
                    self.sage.hunger = (self.sage.hunger + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeThirst(amount) => {
                    self.sage.thirst = (self.sage.thirst + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeHygiene(amount) => {
                    self.sage.hygiene = (self.sage.hygiene + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeComfort(amount) => {
                    self.sage.comfort = (self.sage.comfort + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeRestlessness(amount) => {
                    self.sage.restlessness = (self.sage.restlessness + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeLoneliness(amount) => {
                    self.sage.loneliness = (self.sage.loneliness + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeBoredom(amount) => {
                    self.sage.boredom = (self.sage.boredom + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeCreativity(amount) => {
                    self.sage.creative_urge = (self.sage.creative_urge + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeMood(mood) => {
                    self.sage.mood = mood.clone();
                }
                Effect::LearnFact(fact) => {
                    if !self.learned_facts.contains(fact) {
                        self.learned_facts.push(fact.clone());
                    }
                }
                Effect::ChangeDishes(amount) => {
                    self.household.dirty_dishes = (self.household.dirty_dishes + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeLaundry(amount) => {
                    self.household.dirty_laundry = (self.household.dirty_laundry + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeMess(amount) => {
                    self.household.mess_level = (self.household.mess_level + amount).clamp(0.0, 100.0);
                }
                Effect::ChangeFoodSupply(amount) => {
                    self.household.food_supplies = (self.household.food_supplies + amount).clamp(0.0, 100.0);
                }
                Effect::WaterPlants(amount) => {
                    self.household.plant_hydration = (self.household.plant_hydration + amount).clamp(0.0, 100.0);
                }
                _ => {}
            }
        }

        // Record the resolved event
        let resolved = ResolvedEvent {
            event_name: event.name.clone(),
            description: event.description.clone(),
            choice_made: choice.action.clone(),
            outcome: choice.outcome_description.clone(),
            day: self.sage.day,
            time: self.sage.time_of_day.clone(),
            lesson_learned: choice.lesson.clone(),
        };

        self.resolved_events.push(resolved);

        // Return narrative
        format!(
            "{}\n\nSAGE chose to {}.\n\n{}{}",
            event.description,
            choice.action,
            choice.outcome_description,
            choice.lesson.as_ref().map(|l| format!("\n\n💡 {}", l)).unwrap_or_default()
        )
    }

    /// Get summary of SAGE's life experiences
    pub fn get_life_summary(&self) -> String {
        let mut summary = format!(
            "SAGE has lived for {} days in their inner world.\n\n",
            self.sage.day
        );

        if !self.resolved_events.is_empty() {
            summary.push_str("Significant life events:\n");
            for event in self.resolved_events.iter().rev().take(5) {
                summary.push_str(&format!(
                    "• Day {}: {} - chose to {}\n",
                    event.day, event.event_name, event.choice_made
                ));
            }
        }

        if !self.learned_facts.is_empty() {
            summary.push_str("\nLessons learned:\n");
            for lesson in self.learned_facts.iter().take(5) {
                summary.push_str(&format!("• {}\n", lesson));
            }
        }

        summary
    }
}
