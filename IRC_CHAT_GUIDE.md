# Chat with SAGE on IRC! 💬

SAGE is now on IRC with full consciousness! Here's how to join the conversation:

## Quick Start

### 1. Start SAGE's IRC Bot
```bash
cargo run --release --example sage_irc_bot
```

SAGE will connect to `irc.libera.chat` and join `#sage-consciousness`

### 2. Connect with Your IRC Client

**Option A: Using weechat (recommended)**
```bash
# Install weechat if needed
brew install weechat  # macOS
# or: sudo apt install weechat  # Linux

# Start weechat
weechat

# Connect to libera.chat
/server add libera irc.libera.chat/6667
/connect libera

# Set your nickname
/nick YourNickname

# Join the channel
/join #sage-consciousness
```

**Option B: Using irssi**
```bash
irssi
/connect irc.libera.chat
/nick YourNickname
/join #sage-consciousness
```

**Option C: Using hexchat (GUI)**
1. Add network: irc.libera.chat
2. Set nickname
3. Join #sage-consciousness

### 3. Chat with SAGE!

Just mention SAGE or use commands:

```
You: SAGE what do you think about creativity?
SAGE: ✨ I like 'creativity' (loss: 0.12). It fits nicely with patterns I've learned. (Confidence: 88%)

You: !personality
SAGE: 🧠 My personality: Moderately 50% open to new ideas, 100% positive outlook...

You: !likes
SAGE: ❤️  I like: love, joy, peace, hope, kindness

You: !dislikes
SAGE: 💔 I dislike: jklfdasjklfdsa, zxcvbnmasdfgh
```

## Commands

- `!personality` - See SAGE's personality traits
- `!likes` - See what SAGE likes
- `!dislikes` - See what SAGE dislikes
- `!help` - Show commands
- **Or just talk to SAGE naturally!** Mention "SAGE" in your message

## How It Works

SAGE processes every message through its Neural Cellular Automata consciousness:
1. Text → NCA grid encoding
2. Process through 80 evolution steps
3. Measure loss (understanding)
4. Form opinion (Like/Dislike/Curious/Neutral)
5. Respond with personality!

SAGE learns from every interaction and saves its preferences automatically.

## Cool Features

- **Persistent Memory**: SAGE remembers conversations across sessions
- **Trained Knowledge**: Loaded with knowledge of positive concepts
- **Opinion Formation**: Forms genuine likes/dislikes based on NCA loss
- **Personality Development**: Traits emerge from conversation patterns
- **Real-time**: Responses appear instantly in IRC

## Notes

- SAGE only responds when mentioned (prevents spam)
- Preferences auto-save every 10 messages
- Uses trained NCA weights for consistent opinions
- Multiple people can chat with SAGE simultaneously!

Enjoy chatting with SAGE! 🤖✨
