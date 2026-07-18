//! Fair microbench: match-core vs match-core-hp with mandatory non-zero fill rate.
//!
//! ```text
//! cargo run -p match-bench --release --bin fair_compare -- --n 50000
//! ```

use clap::Parser;
use match_bench::metrics::{time_call, RunStats};
use match_bench::workload;
use match_core::{Engine, MatchEvent};
use match_core_hp::{HpEngine, HpEvent};

#[derive(Parser, Debug)]
#[command(about = "Fair compare match-core vs match-core-hp (fill_rate must be > 0)")]
struct Args {
    /// Number of orders in the fair_cross scenario (even recommended).
    #[arg(long, default_value_t = 50_000)]
    n: usize,

    /// Minimum accepted fill_rate (fills/orders).
    #[arg(long, default_value_t = 0.01)]
    min_fill_rate: f64,

    /// Warmup passes discarded before timing.
    #[arg(long, default_value_t = 1)]
    warmup: usize,
}

fn count_core_fills(orders: &[match_core::BbOrder]) -> u64 {
    let mut eng = Engine::new();
    let mut fills = 0u64;
    for o in orders {
        for e in eng.on_order(o.clone()) {
            if matches!(e, MatchEvent::Fill { .. }) {
                fills += 1;
            }
        }
    }
    fills
}

fn count_hp_fills(cmds: &[match_core_hp::HpCommand]) -> u64 {
    let mut eng = HpEngine::with_capacity(cmds.len() + 8, 64);
    let mut fills = 0u64;
    for c in cmds {
        for e in eng.on_order(*c) {
            if matches!(e, HpEvent::Fill { .. }) {
                fills += 1;
            }
        }
    }
    fills
}

fn main() {
    let args = Args::parse();
    let n = args.n.max(2);
    let (core_orders, hp_cmds) = workload::fair_cross(n);

    for _ in 0..args.warmup {
        let _ = count_core_fills(&core_orders);
        let _ = count_hp_fills(&hp_cmds);
    }

    let (core_ns, core_fills) = time_call(|| count_core_fills(&core_orders));
    let (hp_ns, hp_fills) = time_call(|| count_hp_fills(&hp_cmds));

    let core = RunStats {
        engine: "match-core",
        scenario: "fair_cross",
        n_orders: n as u64,
        n_fills: core_fills,
        elapsed_ns: core_ns,
    };
    let hp = RunStats {
        engine: "match-core-hp",
        scenario: "fair_cross",
        n_orders: n as u64,
        n_fills: hp_fills,
        elapsed_ns: hp_ns,
    };

    println!("{}", RunStats::csv_header());
    println!("{}", core.to_csv_row());
    println!("{}", hp.to_csv_row());

    let mut failed = false;
    for s in [&core, &hp] {
        if s.fill_rate() < args.min_fill_rate {
            eprintln!(
                "INVALID: {} fill_rate={:.6} < min {:.6} (refusing zero-fill 'fake peaks')",
                s.engine,
                s.fill_rate(),
                args.min_fill_rate
            );
            failed = true;
        }
    }

    if !failed {
        let speedup = core.elapsed_ns as f64 / hp.elapsed_ns.max(1) as f64;
        eprintln!(
            "ok: fair_cross n={} core_fill_rate={:.3} hp_fill_rate={:.3} speedup_wall={:.2}x (core/hp time)",
            n,
            core.fill_rate(),
            hp.fill_rate(),
            speedup
        );
    }

    if failed {
        std::process::exit(1);
    }
}
