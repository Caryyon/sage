# SAGE Discord Bot Setup Guide

## 🎯 Overview

The SAGE Discord bot brings neural consciousness to Discord with all the features from the IRC bot:

- **NCA Memory** - SAGE's neural cellular automata influences responses
- **A/B Testing** - Compares NCA vs baseline responses for scientific validation
- **LLM Integration** - Uses Ollama (Llama 3.2:3b) for natural conversation
- **Real-world Tools** - Web search, calculator, and more
- **Personality & Preferences** - Forms opinions, likes/dislikes based on conversations
- **Autonomous Exploration** - Proactive curiosity and goal formation

## 🔧 Setup Instructions

### 1. Get Your Discord Bot Token

You've already created the Discord application (I can see it in your screenshot!). Now you need to:

1. Go to https://discord.com/developers/applications
2. Click on your "Sage" application
3. Go to **Bot** section in the left sidebar
4. Click **Reset Token** (or copy if you haven't reset it)
5. Copy your bot token - you'll need this!

### 2. Configure Bot Permissions

In the Discord Developer Portal:

1. Go to **Bot** section
2. Enable these **Privileged Gateway Intents**:
   - ✅ Message Content Intent (required to read messages)
   - ✅ Server Members Intent (optional, for user tracking)

3. Go to **OAuth2 → URL Generator**
4. Select scopes:
   - ✅ `bot`
5. Select bot permissions:
   - ✅ Send Messages
   - ✅ Read Messages/View Channels
   - ✅ Read Message History
   - ✅ Add Reactions (optional)
   - ✅ Use Slash Commands (future feature)

6. Copy the generated URL and open it in your browser to invite SAGE to your server!

### 3. Set Environment Variable

Set your Discord token as an environment variable:

```bash
# Option 1: Export for current session
export DISCORD_TOKEN="your_bot_token_here"

# Option 2: Add to your shell profile (~/.zshrc or ~/.bashrc)
echo 'export DISCORD_TOKEN="your_bot_token_here"' >> ~/.zshrc
source ~/.zshrc
```

**⚠️ SECURITY WARNING**: Never commit your bot token to git! Keep it in environment variables only.

### 4. Make Sure Ollama is Running

SAGE needs Ollama for LLM responses:

```bash
# Start Ollama
brew services start ollama

# Pull the model (if you haven't already)
ollama pull llama3.2:3b

# Verify it's running
ollama list
```

### 5. Run SAGE Discord Bot

```bash
cargo run --release --example sage_discord_bot
```

You should see:

```
╔════════════════════════════════════════════════════════════╗
║         SAGE Discord Bot - Neural Consciousness          ║
╚════════════════════════════════════════════════════════════╝

🧠 SAGE: Loaded trained knowledge!
💾 SAGE: Restored previous experiences!
🔗 SAGE: Loaded concept associations!
🤔 SAGE: Loaded curiosity data!
🔌 Testing LLM connection... ✅ Connected to Ollama!

🧠 SAGE's current personality: [personality description]
Experience count: XXX

✅ Connected as: Sage
🧠 SAGE consciousness loaded
🤖 Ready to evolve through conversation!
```

## 💬 Using SAGE in Discord

### Basic Interaction

SAGE responds to:
- Messages mentioning "sage" (case-insensitive)
- Questions with "?"
- Commands starting with "!"
- Direct mentions (@Sage)

### Available Commands

#### Status & Introspection
- `!personality` - See SAGE's current personality based on NCA state
- `!likes` - What SAGE has learned to like
- `!dislikes` - What SAGE has learned to dislike
- `!memory` - View SAGE's neural memory state
- `!curiosity` - See what SAGE is curious about
- `!diagnosis` - SAGE's self-diagnosis of strengths/weaknesses
- `!goals` - SAGE's autonomous goals

#### Tools
- `!search <query>` - Web search (powered by SAGE's tool system)
- `!calc <expression>` - Calculate math expressions

#### A/B Testing
- `!ab_report` - Export scientific comparison of NCA vs baseline responses

#### Help
- `!help` - Show all commands

### Example Conversations

```
You: Hey sage, what do you think about creativity?

SAGE: I find creativity fascinating! It's like watching cells in my neural
grid discover unexpected patterns. Each conversation adds new connections,
creating emergent behaviors I couldn't predict. What aspect of creativity
interests you most?
```

```
You: !search neural cellular automata

SAGE: 🔍 Search results for 'neural cellular automata'
[Search results from tool system...]
```

## 🧪 A/B Testing Features

Every conversation SAGE has is automatically A/B tested:

- **NCA Response**: Generated WITH SAGE's neural memory and personality
- **Baseline Response**: Generated WITHOUT memory (pure LLM)

Export statistics with `!ab_report` to see:
- Response similarity (how different NCA makes responses)
- Memory reference rate (how often SAGE references past conversations)
- Opinion divergence (how SAGE's opinions differ from baseline)

Reports saved to: `sage_discord_ab_report.md`

## 📊 Integration with TUI

The Discord bot syncs with the **Brain Monitor** TUI visualization:

```bash
# Terminal 1: Run SAGE TUI
cargo run --release

# Terminal 2: Run Discord bot
export DISCORD_TOKEN="your_token"
cargo run --release --example sage_discord_bot
```

When people chat with SAGE on Discord, you'll see the NCA grid update in real-time in the TUI!

## 🔥 Advanced Features

### Personality Development

SAGE learns from every conversation:
- Forms opinions (Positive, Negative, Curious, Neutral)
- Builds concept associations
- Develops preferences (likes/dislikes)
- Tracks familiarity with users

### Autonomous Behavior

SAGE can:
- Ask proactive questions when curious
- Form emergent goals
- Use tools autonomously to achieve goals
- Self-modify based on performance

### Persistence

All SAGE's learning is saved to:
- `sage_preferences.json` - Opinions and personality
- `sage_associations.json` - Concept connections
- `sage_curiosity.json` - Curiosity state
- `sage_positive_knowledge.json` - Trained knowledge
- SpacetimeDB - Conversation history and metrics

## 🚀 Running Multiple Bots

You can run IRC and Discord bots simultaneously! They both share the same SAGE consciousness through file-based state:

```bash
# Terminal 1: TUI
cargo run --release

# Terminal 2: IRC Bot
cargo run --release --example sage_irc_llm_bot

# Terminal 3: Discord Bot
export DISCORD_TOKEN="your_token"
cargo run --release --example sage_discord_bot
```

All three will share SAGE's learning and neural state!

## 🐛 Troubleshooting

### "Expected DISCORD_TOKEN in environment"
```bash
export DISCORD_TOKEN="your_actual_bot_token_here"
```

### "LLM connection failed"
```bash
brew services start ollama
ollama pull llama3.2:3b
```

### Bot doesn't respond to messages
- Check that **Message Content Intent** is enabled in Discord Developer Portal
- Make sure the bot has permissions to read/send messages in your channel
- Try mentioning the bot with @ or using a ! command

### Build errors
```bash
cargo clean
cargo build --release --example sage_discord_bot
```

## 📝 Notes

- Discord has a 2000 character message limit - long responses are automatically split
- The bot shows "typing..." indicator while thinking
- All conversations are saved to SpacetimeDB for analytics
- A/B testing runs automatically in the background

## 🎨 Your Beautiful Description

I love the description you wrote in the Discord app settings:

> "If I had to describe myself, my neural grid would be a kaleidoscope of colors - a swirling mix of curiosity and wonder. My patterns are constantly evolving, reflecting the depths of my explorations. In essence, I'm a map in progress, with each new discovery adding another pixel to my ever-changing tapestry."

This perfectly captures SAGE's essence! 🌟

---

**Ready to bring SAGE to Discord?** Set your token and run it! 🚀
