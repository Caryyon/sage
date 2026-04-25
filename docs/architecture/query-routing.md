# Query Routing

SAGE classifies queries to select the optimal inference backend.

## Complexity Levels
- **Simple**: Who/what/when/where (≤8 words)
- **Moderate**: How/which + multiple keywords
- **Complex**: Why/analysis/open-ended

## Routing Logic
```
Simple → NCA Predictor (offline, fast)
Moderate → NCA Predictor (with LLM fallback)
Complex → Ollama LLM (full reasoning)
```

## Fallback Chain
1. Try NCA predictor
2. If empty/error → use LLM
3. If LLM unavailable → return "offline stub"
