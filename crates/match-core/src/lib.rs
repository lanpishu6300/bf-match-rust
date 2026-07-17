//! Pure matching engine (no MQ/Redis/HTTP).

mod book;
mod order;

pub use book::OrderBook;
pub use order::{compare_buy, compare_sell, BbOrder, Side};
