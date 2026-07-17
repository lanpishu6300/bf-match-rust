//! High-performance matching core (fixed-point, price-level book).
//! Not Java-equivalent; not used by match-contract by default.

mod scale;
mod types;

pub use scale::{from_lot, from_tick, to_lot, to_tick, ScaleError};
pub use types::{HpCommand, HpEvent, HpOrder, Side, SymbolScale};
