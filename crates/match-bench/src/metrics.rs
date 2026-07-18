//! Shared timing / fill-rate metrics for fair comparison.

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RunStats {
    pub engine: &'static str,
    pub scenario: &'static str,
    pub n_orders: u64,
    pub n_fills: u64,
    pub elapsed_ns: u128,
}

impl RunStats {
    pub fn fill_rate(&self) -> f64 {
        if self.n_orders == 0 {
            0.0
        } else {
            self.n_fills as f64 / self.n_orders as f64
        }
    }

    pub fn orders_per_sec(&self) -> f64 {
        if self.elapsed_ns == 0 {
            0.0
        } else {
            self.n_orders as f64 * 1_000_000_000.0 / self.elapsed_ns as f64
        }
    }

    pub fn fills_per_sec(&self) -> f64 {
        if self.elapsed_ns == 0 {
            0.0
        } else {
            self.n_fills as f64 * 1_000_000_000.0 / self.elapsed_ns as f64
        }
    }

    pub fn ns_per_order(&self) -> f64 {
        if self.n_orders == 0 {
            0.0
        } else {
            self.elapsed_ns as f64 / self.n_orders as f64
        }
    }

    pub fn csv_header() -> &'static str {
        "engine,scenario,n_orders,n_fills,fill_rate,elapsed_ns,orders_per_sec,fills_per_sec,ns_per_order"
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{:.6},{:.0},{:.3},{:.3},{:.3}",
            self.engine,
            self.scenario,
            self.n_orders,
            self.n_fills,
            self.fill_rate(),
            self.elapsed_ns,
            self.orders_per_sec(),
            self.fills_per_sec(),
            self.ns_per_order()
        )
    }
}

pub fn time_call<F: FnOnce() -> u64>(f: F) -> (u128, u64) {
    let t0 = Instant::now();
    let fills = f();
    (t0.elapsed().as_nanos(), fills)
}
