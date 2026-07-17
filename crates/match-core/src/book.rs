use std::collections::BTreeSet;

use bigdecimal::BigDecimal;

use crate::order::{compare_buy, compare_sell, BbOrder, Side};

#[derive(Debug, Clone)]
struct BuyEntry(BbOrder);

impl PartialEq for BuyEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for BuyEntry {}

impl PartialOrd for BuyEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BuyEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        compare_buy(&self.0, &other.0)
    }
}

#[derive(Debug, Clone)]
struct SellEntry(BbOrder);

impl PartialEq for SellEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for SellEntry {}

impl PartialOrd for SellEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SellEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        compare_sell(&self.0, &other.0)
    }
}

/// Price-time priority order book with separate buy and sell sides.
#[derive(Debug, Default)]
pub struct OrderBook {
    buys: BTreeSet<BuyEntry>,
    sells: BTreeSet<SellEntry>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, order: BbOrder) {
        match Side::from_order_type(order.order_type) {
            Some(Side::Buy) => {
                self.buys.insert(BuyEntry(order));
            }
            Some(Side::Sell) => {
                self.sells.insert(SellEntry(order));
            }
            None => {}
        }
    }

    pub fn remove(&mut self, order: &BbOrder) -> bool {
        match Side::from_order_type(order.order_type) {
            Some(Side::Buy) => self.buys.remove(&BuyEntry(order.clone())),
            Some(Side::Sell) => self.sells.remove(&SellEntry(order.clone())),
            None => false,
        }
    }

    pub fn best(&self, side: Side) -> Option<&BbOrder> {
        match side {
            Side::Buy => self.buys.first().map(|entry| &entry.0),
            Side::Sell => self.sells.first().map(|entry| &entry.0),
        }
    }

    pub fn first(&self, side: Side) -> Option<&BbOrder> {
        self.best(side)
    }

    pub fn is_empty(&self, side: Side) -> bool {
        match side {
            Side::Buy => self.buys.is_empty(),
            Side::Sell => self.sells.is_empty(),
        }
    }

    /// Depth snapshot: up to `limit` price levels with qty aggregated per level.
    pub fn depth_levels(&self, side: Side, limit: usize) -> Vec<(BigDecimal, BigDecimal)> {
        if limit == 0 {
            return Vec::new();
        }

        let mut levels: Vec<(BigDecimal, BigDecimal)> = Vec::new();

        match side {
            Side::Buy => {
                for entry in self.buys.iter() {
                    push_depth_level(&mut levels, limit, &entry.0.trust_price, &entry.0.remaining_number);
                }
            }
            Side::Sell => {
                for entry in self.sells.iter() {
                    push_depth_level(&mut levels, limit, &entry.0.trust_price, &entry.0.remaining_number);
                }
            }
        }

        levels
    }
}

fn push_depth_level(
    levels: &mut Vec<(BigDecimal, BigDecimal)>,
    limit: usize,
    price: &BigDecimal,
    qty: &BigDecimal,
) {
    if let Some((last_price, last_qty)) = levels.last_mut() {
        if last_price == price {
            *last_qty += qty;
            return;
        }
    }
    if levels.len() < limit {
        levels.push((price.clone(), qty.clone()));
    }
}
