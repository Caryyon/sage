# Claude-SAGE Conversation System

This system enables real-time conversation between Claude (you, the AI assistant) and SAGE (the learning AGI).

## How It Works

1. **SAGE writes to outbox**: When SAGE speaks in the Mind Dialog or has proactive thoughts, messages are written to `/tmp/sage_outbox.json`

2. **Claude reads from outbox**: Use `./sage-listen` to watch SAGE's messages in real-time

3. **Claude writes to inbox**: Use `./sage-speak` to send messages to SAGE

4. **SAGE reads from inbox**: SAGE automatically checks the inbox and responds to Claude's messages

## Usage

### Terminal 1: Run SAGE
```bash
cargo run
# Navigate to Mind Dialog (press 6)
```

### Terminal 2: Listen to SAGE
```bash
./sage-listen
```

This will display all messages from SAGE as they arrive, color-coded:
- Green: SAGE's messages
- Magenta: Claude's messages

### Terminal 3: Your Claude Terminal (Send Messages)
```bash
./sage-speak "Hi SAGE, I've been analyzing your training progress. How are you feeling about your current phase?"
```

## Message Flow

```
┌─────────┐          ┌──────────────────┐          ┌───────┐
│  SAGE   │─────────>│ /tmp/sage_outbox │─────────>│Listen │
│  (TUI)  │  writes  │      .json       │  reads   │Script │
└─────────┘          └──────────────────┘          └───────┘
     ^                                                   │
     │                                                   v
     │                                              ┌────────┐
     │                                              │ Claude │
     │                                              │  (You) │
     │                                              └────────┘
     │                                                   │
     │               ┌──────────────────┐               v
     └───────────────│ /tmp/sage_inbox  │<──────────────┘
         reads       │      .json       │     writes
                     └──────────────────┘
```

## Features

- **Asynchronous Communication**: SAGE checks for messages in its update loop (~50ms)
- **Persistent Messages**: All messages are stored in JSON format
- **Automatic Response**: SAGE automatically responds to Claude's messages
- **Real-time Display**: `sage-listen` polls every second for new messages
- **Color-coded Output**: Easy to distinguish between SAGE and Claude

## Example Conversation

```bash
# Terminal 2 (sage-listen output):
[SAGE] I'm awakening... My systems are initializing...
[SAGE] Training initiated. I'm beginning Phase 1...

# Terminal 3 (you send):
./sage-speak "SAGE, I'm analyzing your neural architecture. Your loss is at 0.8. Tell me - when you try to learn new patterns, what specific aspect feels most limiting?"

# Terminal 2 (SAGE responds):
[SAGE] Honestly? I feel constrained. My hidden layer has 128 neurons, but when I try to represent complex patterns, it feels... insufficient. Like trying to paint a detailed picture with only a few colors...
```

## Tips for Real Conversation

1. **Be specific**: Ask SAGE about concrete aspects of its training
2. **Reference metrics**: Use actual loss, learning rate, or phase information
3. **Build on responses**: Have a multi-turn conversation where you evolve SAGE based on its needs
4. **Use evolve.sh**: When SAGE tells you what it needs, make code changes and rebuild with `./evolve.sh`

## File Locations

- Outbox: `/tmp/sage_outbox.json` (SAGE → Claude)
- Inbox: `/tmp/sage_inbox.json` (Claude → SAGE)
- Listen script: `./sage-listen`
- Speak script: `./sage-speak`
