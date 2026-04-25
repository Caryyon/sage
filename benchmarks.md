# SAGE Performance Benchmarks

## Inference Speed

| Query Type | Backend | Time | Hardware |
|-----------|---------|------|----------|
| Simple | NCA | <10ms | x86_64 |
| Moderate | NCA + LLM | 50-200ms | x86_64 |
| Complex | LLM | 500-2000ms | x86_64 |

## Memory Usage

| Component | RAM | Notes |
|-----------|-----|-------|
| NCA Grid | ~8MB | 256×256×12×f64 |
| Predictor | ~1MB | Weights |
| Chat History | Variable | Depends on length |
| Total | ~64MB | Base usage |

## Comparison

| | SAGE | ChatGPT | Local LLM |
|---|------|---------|-----------|
| Offline | ✅ | ❌ | ✅ |
| Privacy | ✅ | ❌ | ✅ |
| Cost | Free | $20/mo | Free |
| Mesh | ✅ | ❌ | ❌ |
| Pi 4 | ✅ | ❌ | ❌ |
