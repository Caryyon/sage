#![no_main]
use libfuzzer_sys::fuzz_target;
use sage::grid::Grid;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize arbitrary bytes as a Grid — must never panic, only return Err
    // REGRESSION: arbitrary brain.bin data used to cause index-out-of-bounds panics
    let _: Result<Grid, _> = bincode::deserialize(data);
});
