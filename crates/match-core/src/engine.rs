use std::collections::HashMap;

use bigdecimal::BigDecimal;

use crate::book::OrderBook;
use crate::event::MatchEvent;
use crate::order::{BbOrder, Side};

/// Per-symbol matching engine facade (L1: rest limit orders only).
#[derive(Debug, Default)]
pub struct Engine {
    books: HashMap<String, OrderBook>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Accept an incoming order. L1: limit orders rest on the book; no matching yet.
    pub fn on_order(&mut self, order: BbOrder) -> Vec<MatchEvent> {
        let symbol = order.symbol_key.clone();
        self.books
            .entry(symbol)
            .or_insert_with(OrderBook::new)
            .insert(order);
        Vec::new()
    }

    /// Aggregated depth for `symbol` and `side`: best prices first, qty summed per level.
    pub fn depth_levels(
        &self,
        symbol: &str,
        side: Side,
        limit: usize,
    ) -> Vec<(BigDecimal, BigDecimal)> {
        self.books
            .get(symbol)
            .map(|book| book.depth_levels(side, limit))
            .unwrap_or_default()
    }
}
