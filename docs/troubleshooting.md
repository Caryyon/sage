# Troubleshooting

## Installation Issues

### cargo build fails
```bash
# Update Rust
rustup update

# Check version
rustc --version  # Should be 1.75+
```

### Ollama not found
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Verify
ollama --version
```

## Runtime Issues

### "No trained weights found"
```bash
# Train the predictor
sage train --corpus data/training/qa-corpus.txt
```

### Node won't start
```bash
# Check port availability
lsof -i :4001
lsof -i :19175

# Use different ports
sage node start --port 4002
```

### High memory usage
- Reduce grid size: `--grid-size 128`
- Limit history: `--max-history 10`
