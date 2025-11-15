# 🚀 Quick Test: IRC-TUI Sync

**Your TUI is now valuable!** Here's how to test it:

---

## Current Status

✅ IRC Bot: **RUNNING** (PID 60624, connected to #sage-ai)
✅ Build: **SUCCESS** (0 errors, 5 warnings)
✅ IrcSync: **READY** (writing to /tmp/sage_irc_messages.json)
✅ Mission Control: **READY** (reading from IrcSync)

---

## Quick Test (2 minutes)

### Step 1: Launch Mission Control TUI
```bash
cargo run --release
```

You'll see Mission Control with IRC feed on the right sidebar showing:
```
💬 IRC FEED
  No messages yet
  Start IRC bot with:
  make irc
```

### Step 2: Send a Test Message
In IRC (irc.libera.chat #sage-ai), send:
```
SAGE, what is love?
```

### Step 3: Watch the Magic! ✨
The TUI will update **instantly** showing:
```
💬 IRC FEED
[22:35] <gmork> SAGE, what is love?
        <SAGE> Love is a fundamental concept...
        🧠 love
```

**Plus**:
- IRC message count increases
- Neural CT scan responds to concept mention
- Concepts tracked in real-time

---

## What You'll See

### Mission Control Layout
```
┌─────────────────────────────────────────────────────┐
│ 🧠 NEURAL CT SCAN                                   │
│ [70% width - pulsing neural visualization]         │
│ Colors change when concepts are mentioned!         │
└─────────────────────────────────────────────────────┘

Sidebar (30%):
┌────────────────┐
│ 📊 METRICS     │
│ IRC Msgs: 1 ←──┼─ Updates in real-time!
└────────────────┘

┌────────────────┐
│ 💬 IRC FEED    │ ← Live conversations!
│                │
│ [22:35] <You>  │
│   SAGE what... │
│                │
│   <SAGE> Love..│
│   🧠 love      │ ← Concepts detected!
└────────────────┘
```

---

## Test Concepts

Try mentioning these baseline concepts and watch the TUI respond:

**Positive concepts**:
- love, joy, peace, harmony, beauty, truth, wisdom
- kindness, compassion, courage, gratitude, hope
- trust, grace, light, warmth, gentleness, patience

**Example messages**:
```
SAGE, tell me about love and joy
SAGE, what brings you peace?
SAGE, I'm feeling grateful today
SAGE, the beauty of wisdom
```

Each mentioned concept will:
1. Appear in the 🧠 concept line
2. Strengthen SAGE's neural patterns
3. Potentially shift the CT scan colors

---

## Commands to Try

### SAGE Commands
```
!personality   - Show SAGE's current personality
!likes         - What SAGE likes
!dislikes      - What SAGE dislikes
!memory        - SAGE's emotional context
!help          - List all commands
```

### Any Question
```
SAGE, [your question]?
```

SAGE will respond with LLM-enhanced answers influenced by its NCA memory!

---

## Behind the Scenes

### File Sync
```bash
# Watch messages being written
cat /tmp/sage_irc_messages.json | jq .

# Monitor message count
watch -n 1 "cat /tmp/sage_irc_messages.json | jq '.messages | length'"
```

### IRC Bot Log
```bash
tail -f /tmp/sage_irc_bot.log
```

---

## Full Dev Environment

For the complete experience:
```bash
make dev
```

This starts:
- SpacetimeDB (persistent memory)
- IRC bot (background process)
- Mission Control TUI (main window)

All in a tmux session!

---

## Troubleshooting

### IRC bot not running
```bash
make irc
# or
cargo run --release --example sage_irc_llm_bot
```

### TUI not showing messages
- Press Tab to cycle screens
- Press any key to force refresh
- Check IRC bot is connected: `ps aux | grep sage_irc`

### No sync file
```bash
ls -l /tmp/sage_irc_messages.json

# If missing, send a message on IRC to create it
```

---

## The Difference

### Before
You: "TUI feels useless right now"
- Had to switch windows to see IRC
- No visibility into SAGE's thinking
- Couldn't track conversations

### After
TUI shows:
- ✅ Live IRC conversations with timestamps
- ✅ SAGE responses in real-time
- ✅ Concepts being discussed (🧠)
- ✅ Neural patterns responding to chat
- ✅ Message count and activity metrics

**Your TUI is now your SAGE Mind Monitor!** 🧠✨

---

## Next Conversation

Now when you chat with SAGE on IRC, you'll see:
1. Your message appear in TUI
2. SAGE's LLM response show up
3. Concepts highlighted (love, joy, etc.)
4. Neural CT scan colors shift
5. Real-time correlation between chat and brain activity

**Try it now!** 🚀

Send a message on IRC and watch the TUI come alive!
