//! sage-consolidate: Run HDC → NCA consolidation sleep cycles.
//!
//! This is Step 6 of the v0.6.0 plan: the bridge between episodic memory
//! (HDC store) and semantic memory (NCA grid).
//!
//! Usage:
//!   cargo run --bin sage-consolidate              # One sleep cycle
//!   cargo run --bin sage-consolidate -- --cycles 5 # Multiple cycles
//!   cargo run --bin sage-consolidate -- --verbose  # Verbose output

use sage::consolidation::{ConsolidationConfig, ConsolidationEngine};
use sage::grid::ConsolidationParams;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");
    let max_cycles: usize = args
        .iter()
        .position(|a| a == "--cycles" || a == "-c")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    eprintln!("🌙 SAGE Consolidation Engine");
    eprintln!("   HDC → NCA sleep cycle bridge");
    eprintln!();

    // Load trained consolidation params if available
    let params = ConsolidationParams::load_or_default();
    eprintln!("   Params: decay={:.3} strengthen={:.3} spread={:.3} conf_boost={:.3} thresh={:.3}",
        params.decay_rate, params.strengthen_rate, params.spread_rate,
        params.confidence_boost, params.activation_threshold);

    let config = ConsolidationConfig {
        params,
        ..ConsolidationConfig::default()
    };

    let mut engine = ConsolidationEngine::new(config);

    if max_cycles > 1 {
        eprintln!("   Running up to {} sleep cycles (stable when <3 new clusters)", max_cycles);
        let reports = engine.sleep_until_stable(max_cycles, 3, verbose)?;

        eprintln!();
        eprintln!("📊 Sleep Summary:");
        for (i, r) in reports.iter().enumerate() {
            eprintln!("   Cycle {}: {} clusters encoded, coherence={:.3}, {:.1}s",
                i + 1, r.clusters_encoded, r.mean_coherence, r.duration_secs);
        }
    } else {
        let report = engine.sleep_cycle(verbose)?;

        eprintln!();
        eprintln!("📊 Sleep Report:");
        eprintln!("   Clusters found:    {}", report.clusters_found);
        eprintln!("   Clusters encoded:  {}", report.clusters_encoded);
        eprintln!("   Mean coherence:    {:.3}", report.mean_coherence);
        eprintln!("   HDC entries:       {}", report.hdc_entries);
        eprintln!("   NCA knowledge:     {} cells", report.nca_knowledge_cells);
        eprintln!("   Duration:          {:.1}s", report.duration_secs);
    }

    eprintln!();
    eprintln!("✅ Consolidation complete");

    Ok(())
}
