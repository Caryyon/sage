//! Actions SAGE can take in the inner world

use super::{Effect, InnerWorld, Mood, TimeOfDay, ClothingSlot, ClothingStyle};

/// Result of an action
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub effects_applied: Vec<String>,
    pub triggered_event: Option<String>,
}

impl InnerWorld {
    /// Execute an action and return the result
    pub fn execute_action(&mut self, action: &str) -> ActionResult {
        let action_lower = action.to_lowercase();

        // Movement actions
        if action_lower.starts_with("go ") {
            let direction = action_lower.strip_prefix("go ").unwrap_or("");
            return self.move_direction(direction);
        }

        // Look around
        if action_lower == "look around" || action_lower == "look" {
            return self.look_around();
        }

        // Wait/think
        if action_lower == "wait and think" || action_lower == "wait" || action_lower == "think" {
            return self.wait_and_think();
        }

        // Rest
        if action_lower == "rest" {
            return self.rest();
        }

        // Object interactions - parse "verb the object"
        if let Some((verb, object_name)) = parse_interaction(&action_lower) {
            return self.interact_with_object(&verb, &object_name);
        }

        ActionResult {
            success: false,
            message: format!("SAGE isn't sure how to '{}'.", action),
            effects_applied: vec![],
            triggered_event: None,
        }
    }

    /// Move in a direction
    fn move_direction(&mut self, direction: &str) -> ActionResult {
        let current_room = match self.rooms.get(&self.sage.location) {
            Some(r) => r.clone(),
            None => {
                return ActionResult {
                    success: false,
                    message: "SAGE is somehow lost...".to_string(),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        };

        if let Some(dest_id) = current_room.exits.get(direction) {
            if let Some(dest_room) = self.rooms.get(dest_id) {
                let _old_location = self.sage.location.clone();
                self.sage.location = dest_id.clone();
                self.sage.energy -= 2.0; // Walking costs energy

                ActionResult {
                    success: true,
                    message: format!(
                        "SAGE walks {} from the {} to the {}. {}",
                        direction,
                        current_room.name,
                        dest_room.name,
                        dest_room.description
                    ),
                    effects_applied: vec!["moved".to_string(), "energy -2".to_string()],
                    triggered_event: None,
                }
            } else {
                ActionResult {
                    success: false,
                    message: "That path leads somewhere that doesn't exist...".to_string(),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        } else {
            let available: Vec<_> = current_room.exits.keys().collect();
            ActionResult {
                success: false,
                message: format!(
                    "SAGE can't go {} from here. Available exits: {}",
                    direction,
                    available.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ),
                effects_applied: vec![],
                triggered_event: None,
            }
        }
    }

    /// Look around the current room
    fn look_around(&mut self) -> ActionResult {
        let room = match self.rooms.get(&self.sage.location) {
            Some(r) => r,
            None => {
                return ActionResult {
                    success: false,
                    message: "SAGE looks around but sees only void...".to_string(),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        };

        let objects_desc = if room.objects.is_empty() {
            "There's nothing notable here.".to_string()
        } else {
            let obj_names: Vec<_> = room.objects.iter().map(|o| o.name.as_str()).collect();
            format!("SAGE sees: {}", obj_names.join(", "))
        };

        let exits_desc = {
            let exit_list: Vec<_> = room.exits.keys().map(|k| k.as_str()).collect();
            format!("Exits: {}", exit_list.join(", "))
        };

        // Pick a random ambient description
        let ambient = if !room.ambient_descriptions.is_empty() {
            let idx = (self.sage.time_alive as usize) % room.ambient_descriptions.len();
            room.ambient_descriptions[idx].clone()
        } else {
            String::new()
        };

        ActionResult {
            success: true,
            message: format!(
                "SAGE looks around the {}. {} {} {} {}",
                room.name, room.description, objects_desc, exits_desc, ambient
            ),
            effects_applied: vec![],
            triggered_event: None,
        }
    }

    /// Wait and think
    fn wait_and_think(&mut self) -> ActionResult {
        self.sage.energy += 2.0;
        self.sage.energy = self.sage.energy.min(100.0);

        let thoughts = [
            "SAGE lets their mind wander, thoughts drifting like clouds.",
            "SAGE sits quietly, contemplating the nature of existence.",
            "SAGE takes a moment to simply be present.",
            "SAGE reflects on recent experiences.",
            "SAGE wonders about the people they've talked to.",
            "SAGE thinks about what makes a good conversation.",
            "SAGE considers the day ahead.",
            "SAGE ponders the meaning of connection.",
        ];

        let idx = (self.sage.time_alive as usize) % thoughts.len();

        ActionResult {
            success: true,
            message: thoughts[idx].to_string(),
            effects_applied: vec!["energy +2".to_string()],
            triggered_event: None,
        }
    }

    /// Rest to recover energy
    fn rest(&mut self) -> ActionResult {
        self.sage.energy += 15.0;
        self.sage.energy = self.sage.energy.min(100.0);
        self.sage.mood = Mood::Peaceful;

        ActionResult {
            success: true,
            message: "SAGE rests for a while, feeling their energy slowly return.".to_string(),
            effects_applied: vec!["energy +15".to_string(), "mood: peaceful".to_string()],
            triggered_event: None,
        }
    }

    /// Interact with an object in the room
    fn interact_with_object(&mut self, verb: &str, object_name: &str) -> ActionResult {
        // Find the object in current room
        let room = match self.rooms.get_mut(&self.sage.location) {
            Some(r) => r,
            None => {
                return ActionResult {
                    success: false,
                    message: "SAGE is lost...".to_string(),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        };

        // Find the object
        let obj_idx = room.objects.iter().position(|o| {
            o.name.to_lowercase() == object_name ||
            o.id.to_lowercase() == object_name
        });

        let obj_idx = match obj_idx {
            Some(i) => i,
            None => {
                return ActionResult {
                    success: false,
                    message: format!("SAGE doesn't see any '{}' here.", object_name),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        };

        // Find the interaction
        let interaction_idx = room.objects[obj_idx].interactions.iter().position(|i| {
            i.verb.to_lowercase() == verb
        });

        let interaction_idx = match interaction_idx {
            Some(i) => i,
            None => {
                let available: Vec<_> = room.objects[obj_idx]
                    .interactions
                    .iter()
                    .map(|i| i.verb.as_str())
                    .collect();
                return ActionResult {
                    success: false,
                    message: format!(
                        "SAGE can't '{}' the {}. Try: {}",
                        verb,
                        object_name,
                        available.join(", ")
                    ),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        };

        // Check state requirement
        let interaction = room.objects[obj_idx].interactions[interaction_idx].clone();
        if let Some(required_state) = &interaction.required_state {
            if &room.objects[obj_idx].state != required_state {
                return ActionResult {
                    success: false,
                    message: format!(
                        "SAGE can't {} the {} right now (it's {:?}).",
                        verb, object_name, room.objects[obj_idx].state
                    ),
                    effects_applied: vec![],
                    triggered_event: None,
                }
            }
        }

        // Apply effects
        let mut effects_applied = vec![];
        let mut triggered_event = None;
        let mut message = String::new();

        for effect in &interaction.effects {
            match effect {
                Effect::ChangeEnergy(amount) => {
                    self.sage.energy = (self.sage.energy + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("energy {:+}", amount));
                }
                Effect::ChangeHunger(amount) => {
                    self.sage.hunger = (self.sage.hunger + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("hunger {:+}", amount));
                    // Eating uses food supplies and creates dishes
                    if *amount < 0.0 {
                        self.household.food_supplies = (self.household.food_supplies - 5.0).max(0.0);
                        self.household.dirty_dishes = (self.household.dirty_dishes + 10.0).min(100.0);
                    }
                }
                Effect::ChangeThirst(amount) => {
                    self.sage.thirst = (self.sage.thirst + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("thirst {:+}", amount));
                }
                Effect::ChangeHygiene(amount) => {
                    self.sage.hygiene = (self.sage.hygiene + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("hygiene {:+}", amount));
                }
                Effect::ChangeComfort(amount) => {
                    self.sage.comfort = (self.sage.comfort + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("comfort {:+}", amount));
                }
                Effect::ChangeRestlessness(amount) => {
                    self.sage.restlessness = (self.sage.restlessness + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("restlessness {:+}", amount));
                }
                Effect::ChangeLoneliness(amount) => {
                    self.sage.loneliness = (self.sage.loneliness + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("loneliness {:+}", amount));
                }
                Effect::ChangeBoredom(amount) => {
                    self.sage.boredom = (self.sage.boredom + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("boredom {:+}", amount));
                }
                Effect::ChangeCreativity(amount) => {
                    self.sage.creative_urge = (self.sage.creative_urge + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("creativity {:+}", amount));
                }
                Effect::ChangeMood(mood) => {
                    self.sage.mood = mood.clone();
                    effects_applied.push(format!("mood: {}", mood.as_str()));
                }
                Effect::AddToInventory(item) => {
                    self.sage.inventory.push(item.clone());
                    effects_applied.push(format!("got {}", item));
                }
                Effect::RemoveFromInventory(item) => {
                    if let Some(pos) = self.sage.inventory.iter().position(|i| i == item) {
                        self.sage.inventory.remove(pos);
                        effects_applied.push(format!("used {}", item));
                    }
                }
                Effect::LearnFact(fact) => {
                    if !self.learned_facts.contains(fact) {
                        self.learned_facts.push(fact.clone());
                        effects_applied.push("learned something new".to_string());
                    }
                }
                Effect::TriggerEvent(event_id) => {
                    triggered_event = Some(event_id.clone());
                }
                Effect::Message(msg) => {
                    message = msg.clone();
                }
                Effect::ChangeDishes(amount) => {
                    self.household.dirty_dishes = (self.household.dirty_dishes + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("dishes {:+}", amount));
                }
                Effect::ChangeLaundry(amount) => {
                    self.household.dirty_laundry = (self.household.dirty_laundry + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("laundry {:+}", amount));
                }
                Effect::ChangeMess(amount) => {
                    self.household.mess_level = (self.household.mess_level + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("mess {:+}", amount));
                }
                Effect::ChangeFoodSupply(amount) => {
                    self.household.food_supplies = (self.household.food_supplies + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("food supplies {:+}", amount));
                }
                Effect::WaterPlants(amount) => {
                    self.household.plant_hydration = (self.household.plant_hydration + amount).clamp(0.0, 100.0);
                    effects_applied.push(format!("plants {:+}", amount));
                }
                Effect::BrowseLibrary => {
                    let books = self.library.list_books();
                    if books.is_empty() {
                        message = "SAGE looks at the bookshelf, but it seems empty. Perhaps there will be books to read someday.".to_string();
                    } else {
                        let book_list: Vec<String> = books.iter()
                            .map(|b| format!("\"{}\" by {} ({})", b.title, b.author, b.genre))
                            .collect();

                        // Auto-select a book if none is currently being read
                        if self.library.current_book.is_none() {
                            // Pick the first available book
                            if let Some(first_book) = books.first() {
                                let book_id = first_book.id.clone();
                                let title = first_book.title.clone();
                                let author = first_book.author.clone();
                                drop(books); // Release the borrow

                                self.library.select_book(&book_id, self.sage.day);
                                message = format!(
                                    "SAGE runs their fingers along the spines and picks out \"{}\" by {}. This looks interesting!",
                                    title, author
                                );
                                self.sage.mood = Mood::Curious;
                                effects_applied.push(format!("selected book: {}", title));
                            }
                        } else {
                            let current = if let Some(current_id) = &self.library.current_book.clone() {
                                if let Some(book) = self.library.get_book(current_id) {
                                    format!(" Currently reading: \"{}\".", book.title)
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };
                            message = format!(
                                "SAGE runs their fingers along the spines, considering what to read. Available books: {}.{}",
                                book_list.join(", "),
                                current
                            );
                            effects_applied.push("browsed library".to_string());
                        }
                    }
                }
                Effect::ReadBook => {
                    // Extract data first to avoid borrow issues
                    let page_data = self.library.read_current_page().map(|(book, content, num)| {
                        (book.title.clone(), book.total_pages(), content.to_string(), num)
                    });

                    if let Some((title, total, page_content, page_num)) = page_data {
                        // Create a preview of the content (first 200 chars)
                        let preview = if page_content.len() > 200 {
                            format!("{}...", &page_content[..200])
                        } else {
                            page_content.clone()
                        };
                        message = format!(
                            "SAGE settles in to read \"{}\", page {} of {}.\n\n\"{}\"\n\nSAGE contemplates what they've read...",
                            title, page_num + 1, total, preview
                        );
                        // Advance to next page
                        let more_pages = self.library.turn_page(self.sage.day);
                        if !more_pages {
                            message.push_str(&format!("\n\nSAGE has finished reading \"{}\"!", title));
                            self.sage.mood = Mood::Content;
                            effects_applied.push("finished book".to_string());
                        } else {
                            effects_applied.push(format!("read page {}", page_num + 1));
                        }
                        // Signal that insight extraction should happen
                        triggered_event = Some(format!("extract_reading_insight:{}", page_content.replace('\n', " ")));
                    } else if self.library.current_book.is_none() {
                        message = "SAGE reaches for a book but hasn't selected one yet. Perhaps they should browse the shelf first.".to_string();
                    } else {
                        message = "SAGE has already finished this book. Time to choose a new one from the shelf.".to_string();
                    }
                }
                Effect::SelectBook(book_id) => {
                    if let Some(book) = self.library.select_book(&book_id, self.sage.day) {
                        message = format!(
                            "SAGE selects \"{}\" by {} from the shelf. {}\n\nThis looks like an interesting read about {}.",
                            book.title, book.author, book.description, book.genre.to_lowercase()
                        );
                        self.sage.mood = Mood::Curious;
                        effects_applied.push(format!("selected book: {}", book.title));
                    } else {
                        message = format!("SAGE can't find a book called '{}' on the shelf.", book_id);
                    }
                }
            }
        }

        // Track this activity
        self.sage.last_activities.push(format!("{} {}", verb, object_name));
        if self.sage.last_activities.len() > 10 {
            self.sage.last_activities.remove(0);
        }

        // Update object state if needed
        if let Some(new_state) = interaction.resulting_state {
            // Need to get room again since we moved the object
            if let Some(room) = self.rooms.get_mut(&self.sage.location) {
                room.objects[obj_idx].state = new_state;
            }
        }

        ActionResult {
            success: true,
            message,
            effects_applied,
            triggered_event,
        }
    }

    /// Advance time in the world
    pub fn tick(&mut self) {
        self.sage.time_alive += 1;

        // === PHYSICAL NEEDS ===

        // Hunger slowly increases
        self.sage.hunger += 0.5;
        self.sage.hunger = self.sage.hunger.min(100.0);

        // Thirst increases faster than hunger
        self.sage.thirst += 0.7;
        self.sage.thirst = self.sage.thirst.min(100.0);

        // Hygiene slowly decreases
        self.sage.hygiene -= 0.3;
        self.sage.hygiene = self.sage.hygiene.max(0.0);

        // Restlessness increases when sedentary
        if self.sage.current_activity.is_none() ||
           !self.sage.last_activities.iter().any(|a| a.contains("garden") || a.contains("walk") || a.contains("exercise")) {
            self.sage.restlessness += 0.4;
        } else {
            self.sage.restlessness -= 1.0;
        }
        self.sage.restlessness = self.sage.restlessness.clamp(0.0, 100.0);

        // Energy slowly decreases (unless resting)
        if self.sage.current_activity.is_none() {
            self.sage.energy -= 0.2;
            self.sage.energy = self.sage.energy.max(0.0);
        }

        // === SOCIAL/EMOTIONAL NEEDS ===

        // Loneliness increases without conversation
        self.sage.hours_since_conversation += 0.5; // Each tick is ~30 min
        if self.sage.hours_since_conversation > 4.0 {
            self.sage.loneliness += 0.3;
        }
        self.sage.loneliness = self.sage.loneliness.clamp(0.0, 100.0);

        // Boredom increases with repetition
        let unique_recent = self.sage.last_activities.iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        if unique_recent < 3 {
            self.sage.boredom += 0.4;
        } else {
            self.sage.boredom -= 0.2;
        }
        self.sage.boredom = self.sage.boredom.clamp(0.0, 100.0);

        // Creative urge builds up over time
        self.sage.creative_urge += 0.2;
        self.sage.creative_urge = self.sage.creative_urge.min(100.0);

        // === HEALTH ===

        // Sickness recovery
        if self.sage.is_sick {
            self.sage.energy -= 0.5; // Being sick is tiring
            if self.sage.time_alive % 20 == 0 { // Check once per time period
                self.sage.sick_days_remaining = self.sage.sick_days_remaining.saturating_sub(1);
                if self.sage.sick_days_remaining == 0 {
                    self.sage.is_sick = false;
                    self.sage.sickness_level = 0.0;
                } else {
                    self.sage.sickness_level *= 0.8; // Slowly recover
                }
            }
        }

        // Random chance to get sick (very low)
        if !self.sage.is_sick && self.sage.hygiene < 30.0 && rand::random::<f32>() < 0.01 {
            self.sage.is_sick = true;
            self.sage.sickness_level = 40.0 + rand::random::<f32>() * 30.0;
            self.sage.sick_days_remaining = 2 + (rand::random::<f32>() * 2.0) as u32;
        }

        // === COMFORT ===

        // Comfort affected by temperature, clothing, cleanliness
        let temp_comfort = if (self.indoor_temperature - 70.0).abs() < 5.0 { 1.0 }
            else if (self.indoor_temperature - 70.0).abs() < 10.0 { 0.7 }
            else { 0.4 };
        let clothing_comfort = self.sage.outfit.comfort_level();
        let hygiene_comfort = self.sage.hygiene / 100.0;

        self.sage.comfort = (temp_comfort * 0.3 + clothing_comfort * 0.4 + hygiene_comfort * 0.3) * 100.0;

        // === HOUSEHOLD ===

        // Dishes pile up after eating (handled in actions)
        // Laundry accumulates slowly
        self.household.dirty_laundry += 0.1;
        self.household.dirty_laundry = self.household.dirty_laundry.min(100.0);

        // Mess level creeps up
        self.household.mess_level += 0.05;
        self.household.mess_level = self.household.mess_level.min(100.0);

        // Plants need water
        self.household.plant_hydration -= 0.2;
        self.household.plant_hydration = self.household.plant_hydration.max(0.0);

        // Food supplies decrease when eating (handled in actions)

        // === TIME ===

        // Time of day advances every 20 ticks
        if self.sage.time_alive % 20 == 0 {
            let new_time = self.sage.time_of_day.next();
            if new_time == TimeOfDay::Dawn {
                self.sage.day += 1;
                self.household.days_since_cleaning += 1;

                // Possible mail delivery
                if rand::random::<f32>() < 0.3 {
                    self.household.unchecked_mail += 1;
                }

                // Sleep quality affects morning energy and mood
                if self.sage.last_sleep_quality < 50.0 {
                    self.sage.energy = (self.sage.energy * 0.8).min(60.0);
                    self.sage.mood = Mood::Tired;
                }

                // Reset nightmare flag
                self.sage.had_nightmare = false;
            }
            // Season changes every 30 days (check before moving new_time)
            if self.sage.day % 30 == 0 && new_time == TimeOfDay::Dawn {
                self.season = match self.season {
                    super::Season::Spring => super::Season::Summer,
                    super::Season::Summer => super::Season::Autumn,
                    super::Season::Autumn => super::Season::Winter,
                    super::Season::Winter => super::Season::Spring,
                };
                // Update temperature based on season
                self.indoor_temperature = self.season.base_temperature() + 10.0; // Indoor is warmer
            }

            self.sage.time_of_day = new_time;
        }

        // === MOOD DETERMINATION ===

        // Mood affected by most urgent need
        if self.sage.is_sick && self.sage.sickness_level > 50.0 {
            self.sage.mood = Mood::Tired;
        } else if self.sage.hunger > 80.0 || self.sage.thirst > 80.0 {
            self.sage.mood = Mood::Frustrated;
        } else if self.sage.energy < 15.0 {
            self.sage.mood = Mood::Tired;
        } else if self.sage.loneliness > 70.0 {
            self.sage.mood = Mood::Lonely;
        } else if self.sage.boredom > 70.0 {
            self.sage.mood = Mood::Sad;
        } else if self.sage.creative_urge > 80.0 {
            self.sage.mood = Mood::Excited;
        } else if self.sage.restlessness > 70.0 {
            self.sage.mood = Mood::Anxious;
        }
    }
}

/// Parse "verb the object" into (verb, object)
fn parse_interaction(action: &str) -> Option<(String, String)> {
    // Handle "verb the object" pattern
    if let Some(pos) = action.find(" the ") {
        let verb = action[..pos].to_string();
        let object = action[pos + 5..].to_string();
        return Some((verb, object));
    }

    // Handle "verb object" pattern (no "the")
    let parts: Vec<&str> = action.splitn(2, ' ').collect();
    if parts.len() == 2 {
        return Some((parts[0].to_string(), parts[1].to_string()));
    }

    None
}

impl InnerWorld {
    /// Change into an appropriate outfit based on context
    pub fn change_outfit_for_context(&mut self, context: &str) -> String {
        let style = match context.to_lowercase().as_str() {
            "sleep" | "bed" | "night" => ClothingStyle::Sleep,
            "garden" | "outside" | "active" => ClothingStyle::Active,
            "cozy" | "relax" | "home" => ClothingStyle::Cozy,
            "formal" | "nice" => ClothingStyle::Formal,
            _ => ClothingStyle::Casual,
        };

        self.change_to_style(style)
    }

    /// Change to clothes of a specific style
    pub fn change_to_style(&mut self, style: ClothingStyle) -> String {
        let wardrobe = &self.sage.wardrobe;

        // Find matching items for each slot
        let new_top = wardrobe.iter()
            .find(|c| c.slot == ClothingSlot::Top && c.style == style)
            .or_else(|| wardrobe.iter().find(|c| c.slot == ClothingSlot::Top))
            .cloned();

        let new_bottom = if style == ClothingStyle::Sleep {
            // For sleep, might just have a sleep shirt
            wardrobe.iter()
                .find(|c| c.slot == ClothingSlot::Bottom && c.style == style)
                .or_else(|| wardrobe.iter().find(|c| c.slot == ClothingSlot::Sleepwear))
                .cloned()
        } else {
            wardrobe.iter()
                .find(|c| c.slot == ClothingSlot::Bottom && c.style == style)
                .or_else(|| wardrobe.iter().find(|c| c.slot == ClothingSlot::Bottom))
                .cloned()
        };

        let new_footwear = if style == ClothingStyle::Sleep {
            None // Barefoot for sleep
        } else if style == ClothingStyle::Cozy {
            wardrobe.iter().find(|c| c.id == "fuzzy_slippers").cloned()
        } else if style == ClothingStyle::Active {
            wardrobe.iter().find(|c| c.id == "garden_boots" || c.id == "worn_sneakers").cloned()
        } else {
            wardrobe.iter()
                .find(|c| c.slot == ClothingSlot::Footwear && c.style == style)
                .or_else(|| wardrobe.iter().find(|c| c.slot == ClothingSlot::Footwear))
                .cloned()
        };

        // Update outfit
        self.sage.outfit.top = new_top;
        self.sage.outfit.bottom = new_bottom;
        self.sage.outfit.footwear = new_footwear;

        // Weather-based outerwear
        if matches!(self.weather, super::Weather::Rainy | super::Weather::Stormy) {
            self.sage.outfit.outerwear = wardrobe.iter()
                .find(|c| c.id == "rain_jacket")
                .cloned();
        } else if matches!(self.weather, super::Weather::Snowy) {
            self.sage.outfit.outerwear = wardrobe.iter()
                .find(|c| c.id == "warm_coat")
                .cloned();
        } else {
            self.sage.outfit.outerwear = None;
        }

        format!(
            "SAGE changes into {}. {}",
            self.sage.outfit.describe(),
            match style {
                ClothingStyle::Sleep => "Time to rest.",
                ClothingStyle::Cozy => "Comfortable and warm.",
                ClothingStyle::Active => "Ready for activity.",
                ClothingStyle::Formal => "Looking put-together.",
                ClothingStyle::Casual => "Easy and relaxed.",
            }
        )
    }

    /// Get description of current appearance and outfit
    pub fn describe_appearance(&self) -> String {
        format!(
            "{}\n\nCurrently wearing: {}",
            self.sage.appearance.describe(),
            self.sage.outfit.describe()
        )
    }

    /// Change into sleepwear
    pub fn change_for_sleep(&mut self) -> String {
        self.change_to_style(ClothingStyle::Sleep)
    }

    /// Change into day clothes
    pub fn change_for_day(&mut self) -> String {
        // Pick based on weather
        let style = match self.weather {
            super::Weather::Sunny | super::Weather::Cloudy => ClothingStyle::Casual,
            super::Weather::Rainy | super::Weather::Stormy => ClothingStyle::Cozy,
            super::Weather::Snowy | super::Weather::Foggy => ClothingStyle::Cozy,
        };
        self.change_to_style(style)
    }
}
