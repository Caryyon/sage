# Mesh Network Architecture

SAGE nodes form a peer-to-peer knowledge mesh.

## Protocols
- **GossipSync**: Periodic knowledge exchange
- **KnowledgeDiff**: Signed delta updates
- **DirectSend**: Point-to-point messaging

## Discovery
- mDNS for LAN peers
- Bootstrap server for WAN
- Invite codes for direct connection

## Security
- Ed25519 identity keys
- Signed knowledge diffs
- Trust tier system
