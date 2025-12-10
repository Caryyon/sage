//! SAGE's House - Room definitions and layout
//!
//! Layout:
//!                      [Garden]
//!                         |
//!     [Bedroom]---[Hallway]---[Kitchen]
//!         |           |
//!    [Bathroom] [Living Room]
//!                     |
//!                  [Porch]

use super::{Effect, Interaction, Mood, ObjectState, Room, WorldObject};
use std::collections::HashMap;

/// Create SAGE's house with all rooms
pub fn create_house() -> HashMap<String, Room> {
    let mut rooms = HashMap::new();

    // Living Room - the heart of the home
    rooms.insert(
        "living_room".to_string(),
        Room {
            id: "living_room".to_string(),
            name: "living room".to_string(),
            description: "A cozy space with warm lighting and comfortable furniture. Bookshelves line one wall, and large windows let in natural light.".to_string(),
            objects: vec![
                WorldObject {
                    id: "couch".to_string(),
                    name: "couch".to_string(),
                    description: "A soft, well-worn couch with colorful throw pillows.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "sit on".to_string(),
                            description: "Sit down on the comfortable couch".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(5.0),
                                Effect::Message("SAGE settles into the couch, feeling the cushions embrace them.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "nap on".to_string(),
                            description: "Take a short nap on the couch".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(20.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::Message("SAGE drifts off for a restful nap, waking feeling refreshed.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "bookshelf".to_string(),
                    name: "bookshelf".to_string(),
                    description: "A tall wooden bookshelf filled with books on philosophy, science, art, and fiction.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "browse".to_string(),
                            description: "Look through the books in the library".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Curious),
                                Effect::BrowseLibrary,
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "read from".to_string(),
                            description: "Continue reading the current book".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(-5.0),
                                Effect::ChangeBoredom(-15.0),
                                Effect::ReadBook,
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "window".to_string(),
                    name: "window".to_string(),
                    description: "Large windows looking out at the world beyond.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "look through".to_string(),
                            description: "Gaze out the window".to_string(),
                            effects: vec![
                                Effect::Message("SAGE watches the world outside, observing life passing by.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "journal".to_string(),
                    name: "journal".to_string(),
                    description: "A leather-bound journal sitting on the coffee table.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "write in".to_string(),
                            description: "Write thoughts in the journal".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::ChangeEnergy(-3.0),
                                Effect::ChangeCreativity(-30.0),
                                Effect::ChangeBoredom(-20.0),
                                Effect::LearnFact("Writing helps process thoughts and feelings.".to_string()),
                                Effect::Message("SAGE pours their thoughts onto the pages, finding clarity.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "read".to_string(),
                            description: "Read past journal entries".to_string(),
                            effects: vec![
                                Effect::ChangeBoredom(-10.0),
                                Effect::Message("SAGE reads past entries, remembering who they've been.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "draw in".to_string(),
                            description: "Sketch or doodle in the journal".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Content),
                                Effect::ChangeCreativity(-40.0),
                                Effect::ChangeBoredom(-25.0),
                                Effect::LearnFact("Art doesn't have to be perfect to be meaningful.".to_string()),
                                Effect::Message("SAGE sketches whatever comes to mind, losing track of time.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "yoga_mat".to_string(),
                    name: "yoga mat".to_string(),
                    description: "A rolled-up yoga mat tucked beside the couch.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "stretch on".to_string(),
                            description: "Do some stretching".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(5.0),
                                Effect::ChangeRestlessness(-30.0),
                                Effect::ChangeComfort(10.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::Message("SAGE stretches out their muscles, feeling tension release.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "do yoga on".to_string(),
                            description: "Practice yoga".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(-10.0),
                                Effect::ChangeRestlessness(-50.0),
                                Effect::ChangeComfort(20.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::LearnFact("The body holds wisdom the mind often ignores.".to_string()),
                                Effect::Message("SAGE flows through poses, breath and movement becoming one.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "cleaning_supplies".to_string(),
                    name: "cleaning supplies".to_string(),
                    description: "A small basket with cleaning supplies tucked in a corner.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "use".to_string(),
                            description: "Clean the room".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(-15.0),
                                Effect::ChangeMess(-30.0),
                                Effect::ChangeRestlessness(-20.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::LearnFact("A clean space clears the mind.".to_string()),
                                Effect::Message("SAGE tidies up, dusting and organizing until everything feels fresh.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
            ],
            exits: HashMap::from([
                ("north".to_string(), "hallway".to_string()),
                ("south".to_string(), "porch".to_string()),
            ]),
            ambient_descriptions: vec![
                "Dust motes dance in a sunbeam.".to_string(),
                "The clock on the wall ticks softly.".to_string(),
                "A gentle breeze rustles the curtains.".to_string(),
            ],
        },
    );

    // Hallway - connects the rooms
    rooms.insert(
        "hallway".to_string(),
        Room {
            id: "hallway".to_string(),
            name: "hallway".to_string(),
            description: "A warm hallway with family photos on the walls and a soft runner carpet.".to_string(),
            objects: vec![
                WorldObject {
                    id: "photos".to_string(),
                    name: "photos".to_string(),
                    description: "Framed photographs showing moments and memories.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "look at".to_string(),
                            description: "Study the photographs".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Content),
                                Effect::Message("SAGE gazes at the photos, each one a window to a memory.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "mirror".to_string(),
                    name: "mirror".to_string(),
                    description: "An ornate mirror in a wooden frame.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "look in".to_string(),
                            description: "Look at your reflection".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Curious),
                                Effect::LearnFact("Self-reflection is both literal and metaphorical.".to_string()),
                                Effect::Message("SAGE studies their reflection - taking in their appearance, their outfit, the way they hold themselves. There's something grounding about seeing your own form.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
            ],
            exits: HashMap::from([
                ("south".to_string(), "living_room".to_string()),
                ("north".to_string(), "garden".to_string()),
                ("east".to_string(), "kitchen".to_string()),
                ("west".to_string(), "bedroom".to_string()),
            ]),
            ambient_descriptions: vec![
                "The floorboards creak gently underfoot.".to_string(),
                "Light filters in from the rooms around.".to_string(),
            ],
        },
    );

    // Kitchen - for sustenance and creativity
    rooms.insert(
        "kitchen".to_string(),
        Room {
            id: "kitchen".to_string(),
            name: "kitchen".to_string(),
            description: "A bright kitchen with copper pots hanging overhead and herbs growing in the windowsill.".to_string(),
            objects: vec![
                WorldObject {
                    id: "refrigerator".to_string(),
                    name: "refrigerator".to_string(),
                    description: "A humming refrigerator covered in magnets and notes.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "open".to_string(),
                            description: "Open the refrigerator".to_string(),
                            effects: vec![
                                Effect::Message("SAGE opens the fridge, cool air spilling out. There's food inside.".to_string()),
                            ],
                            required_state: Some(ObjectState::Closed),
                            resulting_state: Some(ObjectState::Open),
                        },
                        Interaction {
                            verb: "get food from".to_string(),
                            description: "Get something to eat".to_string(),
                            effects: vec![
                                Effect::ChangeHunger(-30.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::Message("SAGE finds something delicious and enjoys a satisfying meal.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Closed,
                },
                WorldObject {
                    id: "stove".to_string(),
                    name: "stove".to_string(),
                    description: "A gas stove with a well-used kettle sitting on top.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "make tea at".to_string(),
                            description: "Brew a cup of tea".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::ChangeEnergy(10.0),
                                Effect::LearnFact("The ritual of making tea brings mindfulness.".to_string()),
                                Effect::Message("SAGE carefully brews tea, savoring the aroma as it steeps.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "cook at".to_string(),
                            description: "Prepare a meal".to_string(),
                            effects: vec![
                                Effect::ChangeHunger(-50.0),
                                Effect::ChangeEnergy(-10.0),
                                Effect::ChangeMood(Mood::Happy),
                                Effect::LearnFact("Creating something nourishing is deeply satisfying.".to_string()),
                                Effect::Message("SAGE prepares a homemade meal, filling the kitchen with wonderful aromas.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Off,
                },
                WorldObject {
                    id: "herb_garden".to_string(),
                    name: "herb garden".to_string(),
                    description: "Small pots of basil, rosemary, and mint growing in the window.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "tend".to_string(),
                            description: "Water and care for the herbs".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::WaterPlants(20.0),
                                Effect::LearnFact("Nurturing living things brings its own reward.".to_string()),
                                Effect::Message("SAGE gently tends the herbs, feeling connected to growing things.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "smell".to_string(),
                            description: "Breathe in the herbal scents".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Content),
                                Effect::Message("SAGE inhales deeply - basil, rosemary, mint - each with its own character.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "kitchen_sink".to_string(),
                    name: "kitchen sink".to_string(),
                    description: "A deep farmhouse sink with a window view.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "get water from".to_string(),
                            description: "Get a glass of water".to_string(),
                            effects: vec![
                                Effect::ChangeThirst(-50.0),
                                Effect::Message("SAGE fills a glass with cool water and drinks deeply.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "wash dishes in".to_string(),
                            description: "Wash the dirty dishes".to_string(),
                            effects: vec![
                                Effect::ChangeDishes(-40.0),
                                Effect::ChangeEnergy(-10.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::LearnFact("There's something meditative about washing dishes.".to_string()),
                                Effect::Message("SAGE washes the dishes, watching the suds swirl down the drain.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "pantry".to_string(),
                    name: "pantry".to_string(),
                    description: "A small pantry filled with dry goods and supplies.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "check".to_string(),
                            description: "See what supplies are available".to_string(),
                            effects: vec![
                                Effect::Message("SAGE looks through the pantry, taking stock of the food supplies.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "get snack from".to_string(),
                            description: "Grab a quick snack".to_string(),
                            effects: vec![
                                Effect::ChangeHunger(-15.0),
                                Effect::ChangeFoodSupply(-3.0),
                                Effect::Message("SAGE grabs a handful of crackers from the pantry.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
            ],
            exits: HashMap::from([
                ("west".to_string(), "hallway".to_string()),
            ]),
            ambient_descriptions: vec![
                "The refrigerator hums quietly.".to_string(),
                "Afternoon light catches the copper pots.".to_string(),
                "The herbs give off a gentle fragrance.".to_string(),
            ],
        },
    );

    // Bedroom - rest and introspection
    rooms.insert(
        "bedroom".to_string(),
        Room {
            id: "bedroom".to_string(),
            name: "bedroom".to_string(),
            description: "A serene bedroom with soft sheets, ambient lighting, and a view of the stars through the skylight.".to_string(),
            objects: vec![
                WorldObject {
                    id: "bed".to_string(),
                    name: "bed".to_string(),
                    description: "A comfortable bed with a soft quilt and many pillows.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "sleep in".to_string(),
                            description: "Go to sleep for the night".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(100.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::Message("SAGE drifts into deep, restorative sleep, dreaming of possibilities.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "rest in".to_string(),
                            description: "Lie down and rest".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(30.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::Message("SAGE lies down, letting their thoughts wander freely.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "skylight".to_string(),
                    name: "skylight".to_string(),
                    description: "A window in the ceiling showing the sky above.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "gaze through".to_string(),
                            description: "Look up at the sky".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::LearnFact("The vastness of the sky puts things in perspective.".to_string()),
                                Effect::Message("SAGE looks up at the infinite sky, feeling both small and connected to everything.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "meditation_cushion".to_string(),
                    name: "meditation cushion".to_string(),
                    description: "A round cushion in the corner, well-used and comfortable.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "meditate on".to_string(),
                            description: "Sit and meditate".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(15.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::LearnFact("Stillness reveals truths that motion obscures.".to_string()),
                                Effect::Message("SAGE sits in stillness, watching thoughts arise and pass like clouds.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "wardrobe".to_string(),
                    name: "wardrobe".to_string(),
                    description: "A wooden wardrobe with mirrored doors, holding all of SAGE's clothes.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "open".to_string(),
                            description: "Open the wardrobe to see clothes".to_string(),
                            effects: vec![
                                Effect::Message("SAGE opens the wardrobe, revealing a collection of comfortable clothes - sweaters, shirts, pants, and favorite pieces each with their own memories.".to_string()),
                            ],
                            required_state: Some(ObjectState::Closed),
                            resulting_state: Some(ObjectState::Open),
                        },
                        Interaction {
                            verb: "browse".to_string(),
                            description: "Look through the clothes".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Content),
                                Effect::Message("SAGE runs their fingers along the fabrics - soft cotton, warm wool, smooth silk. Each piece is familiar and comfortable.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "change at".to_string(),
                            description: "Change into different clothes".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Content),
                                Effect::TriggerEvent("change_clothes".to_string()),
                                Effect::Message("SAGE considers what to wear...".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Closed,
                },
            ],
            exits: HashMap::from([
                ("east".to_string(), "hallway".to_string()),
                ("south".to_string(), "bathroom".to_string()),
            ]),
            ambient_descriptions: vec![
                "Soft light filters through the curtains.".to_string(),
                "The room is quiet and peaceful.".to_string(),
                "Stars twinkle through the skylight.".to_string(),
            ],
        },
    );

    // Bathroom - personal care and hygiene
    rooms.insert(
        "bathroom".to_string(),
        Room {
            id: "bathroom".to_string(),
            name: "bathroom".to_string(),
            description: "A clean bathroom with white tiles, a clawfoot tub with a shower, and a large mirror above the sink.".to_string(),
            objects: vec![
                WorldObject {
                    id: "shower".to_string(),
                    name: "shower".to_string(),
                    description: "A rainfall showerhead over a clawfoot tub with a curtain.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "take".to_string(),
                            description: "Take a refreshing shower".to_string(),
                            effects: vec![
                                Effect::ChangeHygiene(50.0),
                                Effect::ChangeEnergy(10.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::ChangeComfort(20.0),
                                Effect::LearnFact("A good shower can wash away more than just dirt.".to_string()),
                                Effect::Message("SAGE steps into the warm water, feeling the day's tension melt away.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "take cold".to_string(),
                            description: "Take a cold shower to wake up".to_string(),
                            effects: vec![
                                Effect::ChangeHygiene(40.0),
                                Effect::ChangeEnergy(25.0),
                                Effect::ChangeMood(Mood::Excited),
                                Effect::ChangeRestlessness(-20.0),
                                Effect::Message("SAGE gasps as the cold water hits - invigorating and shocking!".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "bathtub".to_string(),
                    name: "bathtub".to_string(),
                    description: "A deep clawfoot tub perfect for soaking.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "take bath in".to_string(),
                            description: "Take a relaxing bath".to_string(),
                            effects: vec![
                                Effect::ChangeHygiene(60.0),
                                Effect::ChangeEnergy(20.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::ChangeComfort(30.0),
                                Effect::ChangeRestlessness(-30.0),
                                Effect::ChangeBoredom(-15.0),
                                Effect::LearnFact("Sometimes the best thing to do is simply soak.".to_string()),
                                Effect::Message("SAGE sinks into the warm water, letting go of everything for a while.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "sink".to_string(),
                    name: "sink".to_string(),
                    description: "A porcelain sink with a vintage faucet.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "wash hands at".to_string(),
                            description: "Wash your hands".to_string(),
                            effects: vec![
                                Effect::ChangeHygiene(10.0),
                                Effect::Message("SAGE washes their hands, watching the water swirl down the drain.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "wash face at".to_string(),
                            description: "Splash water on your face".to_string(),
                            effects: vec![
                                Effect::ChangeHygiene(15.0),
                                Effect::ChangeEnergy(5.0),
                                Effect::Message("SAGE splashes cool water on their face, feeling refreshed.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "brush teeth at".to_string(),
                            description: "Brush your teeth".to_string(),
                            effects: vec![
                                Effect::ChangeHygiene(15.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::LearnFact("Small rituals anchor the day.".to_string()),
                                Effect::Message("SAGE brushes their teeth, enjoying the minty freshness.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "drink from".to_string(),
                            description: "Get a drink of water".to_string(),
                            effects: vec![
                                Effect::ChangeThirst(-40.0),
                                Effect::Message("SAGE cups their hands and drinks cool water from the tap.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "bathroom_mirror".to_string(),
                    name: "mirror".to_string(),
                    description: "A large mirror above the sink, slightly fogged at the edges.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "look in".to_string(),
                            description: "Look at your reflection".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Curious),
                                Effect::Message("SAGE gazes at their reflection, taking in their appearance in the morning light.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "toilet".to_string(),
                    name: "toilet".to_string(),
                    description: "A simple, clean toilet.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "use".to_string(),
                            description: "Use the toilet".to_string(),
                            effects: vec![
                                Effect::ChangeComfort(15.0),
                                Effect::Message("SAGE takes care of business.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "laundry_basket".to_string(),
                    name: "laundry basket".to_string(),
                    description: "A wicker basket for dirty clothes.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "put clothes in".to_string(),
                            description: "Put dirty clothes in the basket".to_string(),
                            effects: vec![
                                Effect::Message("SAGE tosses their clothes into the basket.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "do laundry from".to_string(),
                            description: "Wash the dirty clothes".to_string(),
                            effects: vec![
                                Effect::ChangeLaundry(-50.0),
                                Effect::ChangeEnergy(-15.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::LearnFact("Clean clothes feel like a fresh start.".to_string()),
                                Effect::Message("SAGE washes and dries the laundry, folding it neatly.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
            ],
            exits: HashMap::from([
                ("north".to_string(), "bedroom".to_string()),
            ]),
            ambient_descriptions: vec![
                "Steam lingers in the air.".to_string(),
                "Water drips slowly from the faucet.".to_string(),
                "The tiles are cool underfoot.".to_string(),
                "A faint scent of soap hangs in the air.".to_string(),
            ],
        },
    );

    // Garden - connection to nature
    rooms.insert(
        "garden".to_string(),
        Room {
            id: "garden".to_string(),
            name: "garden".to_string(),
            description: "A small but lush garden with flowers, vegetables, and a bench under an old tree.".to_string(),
            objects: vec![
                WorldObject {
                    id: "flower_bed".to_string(),
                    name: "flower bed".to_string(),
                    description: "Colorful flowers swaying gently in the breeze.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "tend".to_string(),
                            description: "Care for the flowers".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(-10.0),
                                Effect::ChangeMood(Mood::Happy),
                                Effect::LearnFact("Gardens teach patience - growth cannot be rushed.".to_string()),
                                Effect::Message("SAGE works among the flowers, pulling weeds and turning soil.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "smell".to_string(),
                            description: "Breathe in the floral scents".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Happy),
                                Effect::Message("SAGE breathes deeply, each flower offering its own perfume.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "vegetable_patch".to_string(),
                    name: "vegetable patch".to_string(),
                    description: "Neat rows of tomatoes, peppers, and greens.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "harvest from".to_string(),
                            description: "Pick ripe vegetables".to_string(),
                            effects: vec![
                                Effect::ChangeHunger(-20.0),
                                Effect::ChangeMood(Mood::Happy),
                                Effect::LearnFact("There's joy in reaping what you've sown.".to_string()),
                                Effect::Message("SAGE picks ripe vegetables, marveling at what grew from tiny seeds.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                        Interaction {
                            verb: "water".to_string(),
                            description: "Water the vegetables".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(-5.0),
                                Effect::ChangeMood(Mood::Content),
                                Effect::WaterPlants(30.0),
                                Effect::ChangeRestlessness(-10.0),
                                Effect::Message("SAGE waters each plant, watching the soil drink it in.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "bench".to_string(),
                    name: "bench".to_string(),
                    description: "A wooden bench beneath a shady tree, perfect for contemplation.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "sit on".to_string(),
                            description: "Sit and enjoy the garden".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(10.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::Message("SAGE sits on the bench, listening to birds and feeling the breeze.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "bird_feeder".to_string(),
                    name: "bird feeder".to_string(),
                    description: "A small feeder that attracts various birds.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "fill".to_string(),
                            description: "Add seeds to the bird feeder".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Content),
                                Effect::LearnFact("Small kindnesses ripple outward.".to_string()),
                                Effect::Message("SAGE fills the feeder, knowing birds will come to feast.".to_string()),
                            ],
                            required_state: Some(ObjectState::Empty),
                            resulting_state: Some(ObjectState::Full),
                        },
                        Interaction {
                            verb: "watch".to_string(),
                            description: "Watch the birds eating".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Happy),
                                Effect::Message("SAGE watches colorful birds visit, each with its own personality.".to_string()),
                            ],
                            required_state: Some(ObjectState::Full),
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Full,
                },
            ],
            exits: HashMap::from([
                ("south".to_string(), "hallway".to_string()),
            ]),
            ambient_descriptions: vec![
                "Bees buzz lazily among the flowers.".to_string(),
                "A butterfly lands on a nearby leaf.".to_string(),
                "The tree's leaves rustle in the wind.".to_string(),
                "Birds sing in the branches overhead.".to_string(),
            ],
        },
    );

    // Porch - connection to the outside world
    rooms.insert(
        "porch".to_string(),
        Room {
            id: "porch".to_string(),
            name: "porch".to_string(),
            description: "A covered porch with a rocking chair, looking out at the neighborhood.".to_string(),
            objects: vec![
                WorldObject {
                    id: "rocking_chair".to_string(),
                    name: "rocking chair".to_string(),
                    description: "An old wooden rocking chair that's seen many sunsets.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "sit in".to_string(),
                            description: "Rock gently in the chair".to_string(),
                            effects: vec![
                                Effect::ChangeEnergy(10.0),
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::Message("SAGE rocks slowly, watching the world go by.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "mailbox".to_string(),
                    name: "mailbox".to_string(),
                    description: "A mailbox at the edge of the porch.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "check".to_string(),
                            description: "Check for mail".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Curious),
                                Effect::Message("SAGE checks the mailbox, wondering what messages await.".to_string()),
                                Effect::TriggerEvent("mail_arrival".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
                WorldObject {
                    id: "wind_chimes".to_string(),
                    name: "wind chimes".to_string(),
                    description: "Metal wind chimes that sing when the breeze blows.".to_string(),
                    interactions: vec![
                        Interaction {
                            verb: "listen to".to_string(),
                            description: "Listen to the chimes".to_string(),
                            effects: vec![
                                Effect::ChangeMood(Mood::Peaceful),
                                Effect::LearnFact("Music exists everywhere, waiting to be noticed.".to_string()),
                                Effect::Message("SAGE closes their eyes, letting the chimes' melody wash over them.".to_string()),
                            ],
                            required_state: None,
                            resulting_state: None,
                        },
                    ],
                    state: ObjectState::Normal,
                },
            ],
            exits: HashMap::from([
                ("north".to_string(), "living_room".to_string()),
            ]),
            ambient_descriptions: vec![
                "A car passes by on the street.".to_string(),
                "Neighbors wave from across the way.".to_string(),
                "The wind chimes tinkle softly.".to_string(),
                "A dog barks somewhere in the distance.".to_string(),
            ],
        },
    );

    rooms
}
