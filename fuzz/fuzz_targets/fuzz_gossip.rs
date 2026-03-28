#![no_main]
use libfuzzer_sys::fuzz_target;
use sage::network::diff::KnowledgeDiff;

fuzz_target!(|data: &[u8]| {
    // Gossip messages from untrusted peers must never panic on malformed input
    // KnowledgeDiff is the core message type exchanged between nodes
    let _: Result<KnowledgeDiff, _> = bincode::deserialize(data);
});
