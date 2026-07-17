use bigdecimal::BigDecimal;
use match_core::{BbOrder, Engine, MatchEvent, Side};
use std::str::FromStr;

fn dec(s: &str) -> BigDecimal {
    BigDecimal::from_str(s).unwrap()
}

#[test]
fn limit_buy_fully_fills_resting_sell() {
    let mut eng = Engine::new();
    eng.on_order(BbOrder::test_limit(Side::Sell, dec("100"), "s1", 1, "1"));
    let events = eng.on_order(BbOrder::test_limit(Side::Buy, dec("100"), "b1", 2, "1"));
    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, MatchEvent::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 1);
    if let MatchEvent::Fill {
        price,
        qty,
        taker_remaining,
        maker_remaining,
        ..
    } = &fills[0]
    {
        assert_eq!(price, "100");
        assert_eq!(qty, "1");
        assert_eq!(taker_remaining, "0");
        assert_eq!(maker_remaining, "0");
    }
    assert!(eng.depth_levels("btcusdt", Side::Buy, 20).is_empty());
    assert!(eng.depth_levels("btcusdt", Side::Sell, 20).is_empty());
}

#[test]
fn price_time_priority_older_maker_first() {
    let mut eng = Engine::new();
    eng.on_order(BbOrder::test_limit(Side::Sell, dec("100"), "s_old", 1, "1"));
    eng.on_order(BbOrder::test_limit(Side::Sell, dec("100"), "s_new", 2, "1"));
    let events = eng.on_order(BbOrder::test_limit(Side::Buy, dec("100"), "b1", 3, "1"));
    if let MatchEvent::Fill {
        maker_order_no, ..
    } = &events[0]
    {
        assert_eq!(maker_order_no, "s_old");
    } else {
        panic!("expected fill");
    }
}
