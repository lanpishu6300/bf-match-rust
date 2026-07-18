//! Market-order matching ported from Java BuyHandler/SellHandler market branches
//! and MarketBuyHandler (gear stop; preserve P0-3 gear=0 behavior).

use bigdecimal::BigDecimal;
use match_protocol::ORDER_FORM_MARKET_PRICE;

use crate::book::OrderBook;
use crate::event::MatchEvent;
use crate::match_limit::{
    rather_than_buy, rather_than_sell, revoke_order_with_reason, RatherThanSellResult,
};
use crate::order::{BbOrder, Side};

/// Java `new BigDecimal(Integer.MAX_VALUE)` for market buy trust price.
const MARKET_BUY_TRUST_PRICE: i32 = i32::MAX;

/// Java `MarketBuyHandler.buyHandle`: skip when best sell is also market.
fn market_buy_handle(book: &mut OrderBook) -> Option<MatchEvent> {
    let sell = book.first(Side::Sell)?;
    if sell.order_form == ORDER_FORM_MARKET_PRICE {
        return None;
    }
    rather_than_buy(book)
}

fn gear_of(order: &BbOrder) -> i32 {
    // Java NPE if null; validation requires Some for market. Treat None as 0 (P0-3).
    order.gear.unwrap_or(0)
}

fn revoke_by_no(
    book: &mut OrderBook,
    order_no: &str,
    side: Side,
    reason: &str,
) -> Option<MatchEvent> {
    let mut stub = BbOrder::test_limit(side, BigDecimal::from(0), order_no, 0, "0");
    stub.order_type = side.order_type();
    revoke_order_with_reason(book, &stub, reason)
}

/// Java `BuyHandler` market path: MAX price, rest, match until gear / empty / filled.
pub fn handle_market_buy(book: &mut OrderBook, mut order: BbOrder) -> Vec<MatchEvent> {
    order.trust_price = BigDecimal::from(MARKET_BUY_TRUST_PRICE);
    let gear = gear_of(&order);
    let order_no = order.trust_order_no.clone();
    book.insert(order);

    let mut events = Vec::new();
    let mut fill_count: i32 = 0;
    loop {
        if book.is_empty(Side::Buy) {
            break;
        }
        if book.is_empty(Side::Sell) {
            if let Some(ev) = revoke_by_no(book, &order_no, Side::Buy, "market_empty") {
                events.push(ev);
            }
            break;
        }
        let best_buy = book.first(Side::Buy).unwrap();
        if best_buy.order_form != ORDER_FORM_MARKET_PRICE {
            // Fully filled (best is no longer a market order).
            break;
        }
        if best_buy.trust_order_no != order_no {
            break;
        }

        let made_progress = match market_buy_handle(book) {
            Some(ev) => {
                events.push(ev);
                fill_count += 1;
                true
            }
            None => false,
        };

        // Java: `bbOrders.size() >= bbOrder.getGear()` after each attempt (P0-3: gear=0 ⇒ always).
        if fill_count >= gear {
            if let Some(ev) = revoke_by_no(book, &order_no, Side::Buy, "market_gear") {
                events.push(ev);
            }
            break;
        }

        // Java would `continue` forever when match returns null and gear > size; stop instead.
        if !made_progress {
            break;
        }
    }
    events
}

/// Java `SellHandler` market path: rest, match via ratherThan sell until gear / empty / filled.
pub fn handle_market_sell(book: &mut OrderBook, order: BbOrder) -> Vec<MatchEvent> {
    let gear = gear_of(&order);
    let order_no = order.trust_order_no.clone();
    book.insert(order);

    let mut events = Vec::new();
    let mut fill_count: i32 = 0;
    loop {
        if book.is_empty(Side::Sell) {
            break;
        }
        if book.is_empty(Side::Buy) {
            if let Some(ev) = revoke_by_no(book, &order_no, Side::Sell, "market_empty") {
                events.push(ev);
            }
            break;
        }
        let best_sell = book.first(Side::Sell).unwrap();
        if best_sell.order_form != ORDER_FORM_MARKET_PRICE {
            break;
        }
        if best_sell.trust_order_no != order_no {
            break;
        }

        let made_progress = match rather_than_sell(book) {
            RatherThanSellResult::Fill(ev) => {
                events.push(ev);
                fill_count += 1;
                true
            }
            RatherThanSellResult::Revoked(ev) => {
                events.push(ev);
                return events;
            }
            RatherThanSellResult::None => false,
        };

        if fill_count >= gear {
            if let Some(ev) = revoke_by_no(book, &order_no, Side::Sell, "market_gear") {
                events.push(ev);
            }
            break;
        }

        if !made_progress {
            break;
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> BigDecimal {
        BigDecimal::from_str(s).unwrap()
    }

    #[test]
    fn market_buy_stops_without_progress_when_best_sell_is_market() {
        let mut book = OrderBook::new();
        let mut market_sell = BbOrder::test_market(Side::Sell, "s_mkt", 1, "1");
        market_sell.trust_price = dec("100");
        book.insert(market_sell);

        let mut market_buy = BbOrder::test_market(Side::Buy, "b_mkt", 2, "1");
        market_buy.gear = Some(5);
        let events = handle_market_buy(&mut book, market_buy);

        assert!(events.iter().all(|e| !matches!(e, MatchEvent::Fill { .. })));
    }

    #[test]
    fn market_sell_stops_when_rather_than_returns_none() {
        let mut book = OrderBook::new();
        let mut market_sell = BbOrder::test_market(Side::Sell, "s_mkt", 1, "1");
        market_sell.gear = Some(5);
        book.insert(market_sell.clone());

        let events = handle_market_sell(&mut book, market_sell);
        assert!(events.iter().all(|e| !matches!(e, MatchEvent::Fill { .. })));
    }

    #[test]
    fn market_buy_stops_when_best_buy_is_no_longer_market() {
        let mut book = OrderBook::new();
        book.insert(BbOrder::test_limit(Side::Buy, dec("100"), "b_rest", 1, "5"));
        book.insert(BbOrder::test_limit(Side::Sell, dec("100"), "s1", 2, "1"));
        let market_buy = BbOrder::test_market(Side::Buy, "b_mkt", 3, "1");
        let events = handle_market_buy(&mut book, market_buy);

        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], MatchEvent::Fill { .. }));
        assert_eq!(book.best(Side::Buy).unwrap().trust_order_no, "b_rest");
    }

    #[test]
    fn market_buy_revokes_when_sell_book_becomes_empty() {
        let mut book = OrderBook::new();
        let mut market_buy = BbOrder::test_market(Side::Buy, "b_mkt", 1, "1");
        market_buy.gear = Some(5);
        let events = handle_market_buy(&mut book, market_buy);

        assert!(events.iter().any(|e| matches!(
            e,
            MatchEvent::Revoke {
                reason,
                ..
            } if reason == "market_empty"
        )));
    }

    #[test]
    fn market_sell_revokes_when_buy_book_becomes_empty() {
        let mut book = OrderBook::new();
        let mut market_sell = BbOrder::test_market(Side::Sell, "s_mkt", 1, "1");
        market_sell.gear = Some(5);
        let events = handle_market_sell(&mut book, market_sell);

        assert!(events.iter().any(|e| matches!(
            e,
            MatchEvent::Revoke {
                reason,
                ..
            } if reason == "market_empty"
        )));
    }

    #[test]
    fn market_buy_loop_continues_while_market_order_remains() {
        let mut book = OrderBook::new();
        book.insert(BbOrder::test_limit(Side::Sell, dec("100"), "s1", 1, "1"));
        book.insert(BbOrder::test_limit(Side::Sell, dec("101"), "s2", 2, "1"));
        let mut market_buy = BbOrder::test_market(Side::Buy, "b_mkt", 3, "2");
        market_buy.gear = Some(5);
        let events = handle_market_buy(&mut book, market_buy);

        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], MatchEvent::Fill { .. }));
        assert!(matches!(&events[1], MatchEvent::Fill { .. }));
    }

    #[test]
    fn market_sell_loop_continues_while_market_order_remains() {
        let mut book = OrderBook::new();
        book.insert(BbOrder::test_limit(Side::Buy, dec("100"), "b1", 1, "1"));
        book.insert(BbOrder::test_limit(Side::Buy, dec("99"), "b2", 2, "1"));
        let mut market_sell = BbOrder::test_market(Side::Sell, "s_mkt", 3, "2");
        market_sell.gear = Some(5);
        let events = handle_market_sell(&mut book, market_sell);

        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], MatchEvent::Fill { .. }));
        assert!(matches!(&events[1], MatchEvent::Fill { .. }));
    }

    #[test]
    fn market_buy_stops_when_best_buy_is_different_market_order() {
        let mut book = OrderBook::new();
        book.insert(BbOrder::test_limit(Side::Sell, dec("100"), "s1", 1, "1"));
        let mut first = BbOrder::test_market(Side::Buy, "b_first", 2, "1");
        first.trust_price = BigDecimal::from(MARKET_BUY_TRUST_PRICE);
        book.insert(first);
        let second = BbOrder::test_market(Side::Buy, "b_second", 3, "1");
        let events = handle_market_buy(&mut book, second);

        assert!(events.is_empty());
        assert_eq!(book.best(Side::Buy).unwrap().trust_order_no, "b_first");
    }

    #[test]
    fn market_sell_stops_when_best_sell_is_no_longer_market() {
        let mut book = OrderBook::new();
        book.insert(BbOrder::test_limit(Side::Sell, dec("100"), "s_rest", 1, "5"));
        book.insert(BbOrder::test_limit(Side::Buy, dec("100"), "b1", 2, "1"));
        let market_sell = BbOrder::test_market(Side::Sell, "s_mkt", 3, "1");
        let events = handle_market_sell(&mut book, market_sell);

        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], MatchEvent::Fill { .. }));
        assert_eq!(book.best(Side::Sell).unwrap().trust_order_no, "s_rest");
    }

    #[test]
    fn market_sell_stops_when_best_sell_is_different_market_order() {
        let mut book = OrderBook::new();
        book.insert(BbOrder::test_limit(Side::Buy, dec("100"), "b1", 1, "1"));
        let first = BbOrder::test_market(Side::Sell, "s_first", 2, "1");
        book.insert(first);
        let second = BbOrder::test_market(Side::Sell, "s_second", 3, "1");
        let events = handle_market_sell(&mut book, second);

        assert!(events.is_empty());
        assert_eq!(book.best(Side::Sell).unwrap().trust_order_no, "s_first");
    }
}
