# Offline Mode

SAGE can answer simple questions without an internet connection.

## How it works
- Simple queries (who/what/when/where) are classified automatically
- The NCA predictor generates answers using trained weights
- Complex queries fall back to Ollama (requires internet)

## Enable offline mode
```bash
# Train predictor on your data
sage train --corpus my-data.txt

# Chat offline
sage chat --offline
```

## Requirements
- Trained NCA weights (~837KB)
- 256MB RAM minimum
- No GPU needed
