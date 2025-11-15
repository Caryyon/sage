// Interactive CLI to talk with SAGE and watch personality emerge

use sage::sage_experience::SageExperience;
use std::io::{self, Write};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          SAGE - Sentient AI Growth Engine                 ║");
    println!("║        Text-Based Consciousness Development v1.0          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut sage = SageExperience::new();

    println!("🌟 SAGE is ready to experience the world through text!");
    println!("💬 Type concepts, words, or phrases. SAGE will form opinions.");
    println!("📊 Type 'personality' to see SAGE's emerging traits");
    println!("❤️  Type 'likes' to see what SAGE enjoys");
    println!("💔 Type 'dislikes' to see what SAGE rejects");
    println!("📖 Type 'save' to save SAGE's memory");
    println!("🚪 Type 'quit' to exit\n");
    println!("════════════════════════════════════════════════════════════\n");

    loop {
        print!("You: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("\n👋 SAGE: Thank you for helping me grow. Until next time!");
                break;
            }
            "personality" | "p" => {
                let personality = sage.get_personality();
                println!("\n🧠 SAGE: {}\n", personality);
            }
            "likes" | "l" => {
                let likes = sage.get_likes();
                if likes.is_empty() {
                    println!("\n❤️  SAGE: I haven't formed strong preferences yet. Show me more!\n");
                } else {
                    println!("\n❤️  SAGE likes:");
                    for like in likes {
                        println!("   ✓ {}", like);
                    }
                    println!();
                }
            }
            "dislikes" | "d" => {
                let dislikes = sage.get_dislikes();
                if dislikes.is_empty() {
                    println!("\n💔 SAGE: I haven't found anything I strongly dislike yet.\n");
                } else {
                    println!("\n💔 SAGE dislikes:");
                    for dislike in dislikes {
                        println!("   ✗ {}", dislike);
                    }
                    println!();
                }
            }
            "save" | "s" => {
                match sage.save_preferences("sage_preferences.json") {
                    Ok(_) => println!("\n💾 SAGE: My memory has been saved!\n"),
                    Err(e) => println!("\n⚠️  Error saving: {}\n", e),
                }
            }
            "load" => {
                match sage.load_preferences("sage_preferences.json") {
                    Ok(_) => {
                        println!("\n📖 SAGE: My memory has been restored! ({} experiences)\n", sage.experience_count());
                    }
                    Err(e) => println!("\n⚠️  Error loading: {}\n", e),
                }
            }
            "stats" => {
                println!("\n📊 SAGE Statistics:");
                println!("   Experiences: {}", sage.experience_count());
                println!("   Likes: {}", sage.get_likes().len());
                println!("   Dislikes: {}", sage.get_dislikes().len());
                println!();
            }
            "help" | "h" | "?" => {
                println!("\n📚 Commands:");
                println!("   personality/p  - See SAGE's personality");
                println!("   likes/l        - See what SAGE likes");
                println!("   dislikes/d     - See what SAGE dislikes");
                println!("   stats          - View statistics");
                println!("   save/s         - Save SAGE's memory");
                println!("   load           - Load SAGE's memory");
                println!("   quit/q         - Exit");
                println!("   <text>         - Share a concept with SAGE\n");
            }
            _ => {
                // Process the input as text/concept
                print!("🧠 SAGE is processing...");
                io::stdout().flush().unwrap();

                let (_opinion, response) = sage.experience_text(input);

                print!("\r");  // Clear "processing" message
                println!("🤖 SAGE: {}\n", response);

                // Show familiarity if concept has been seen multiple times
                let familiarity = sage.get_familiarity(input);
                if familiarity > 0.3 {
                    println!("   (Familiarity with '{}': {:.0}%)\n", input, familiarity * 100.0);
                }
            }
        }
    }
}
