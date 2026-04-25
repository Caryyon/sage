# Install SAGE

## Quick Install
```bash
curl -fsSL https://whatssage.ai/install.sh | bash
```

## Manual Install
```bash
git clone https://github.com/Caryyon/sage.git
cd sage
cargo build --release
sudo cp target/release/sage-cli /usr/local/bin/sage
```

## Verify
```bash
sage --version
```

## Start Chatting
```bash
sage chat
```

## Join the Mesh
```bash
sage node start
```
