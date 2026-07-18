//! Microbench: async WAL append throughput.
//!
//! ```text
//! cargo run -p match-wal --release --bin wal_bench -- 200000
//! ```

use match_wal::{RecordKind, Wal, WalMode, WalRecord};
use std::env;
use std::time::Instant;

fn main() {
    let n: usize = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let path = std::env::temp_dir().join(format!("wal_bench_{}.wal", std::process::id()));
    let _ = std::fs::remove_file(&path);

    let wal = Wal::open(&path, WalMode::Async, 65_536).expect("open wal");
    let t0 = Instant::now();
    for i in 0..n as u64 {
        loop {
            match wal.append(WalRecord {
                kind: RecordKind::Fill,
                id_a: i,
                id_b: i,
                price_tick: 100,
                qty_lot: 1,
            }) {
                Ok(()) => break,
                Err(match_wal::WalError::Busy) => std::thread::yield_now(),
                Err(e) => panic!("{e}"),
            }
        }
    }
    wal.flush().expect("flush");
    let elapsed = t0.elapsed();
    let per_sec = n as f64 / elapsed.as_secs_f64();
    println!(
        "engine,scenario,n_records,elapsed_ns,records_per_sec\nmatch-wal,async_fill,{n},{},{:.3}",
        elapsed.as_nanos(),
        per_sec
    );
    let _ = std::fs::remove_file(&path);
}
