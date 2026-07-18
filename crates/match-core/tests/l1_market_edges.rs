use bigdecimal::BigDecimal;
use match_core::{BbOrder, Engine, MatchEvent, Side};
use std::str::FromStr;

fn dec(s: &str) -> BigDecimal {
    BigDecimal::from_str(s).unwrap()
}

#[test]
fn market_sell_empty_book_revokes() {
    let mut eng = Engine::new();
    let mut taker = BbOrder::test_market(Side::Sell, "s_empty", 1, "1");
    taker.gear = Some(5);
    let events = eng.on_order(taker);

    assert!(events.iter().all(|e| !matches!(e, MatchEvent::Fill { .. })));
    assert!(events.iter().any(|e| matches!(
        e,
        MatchEvent::Revoke {
            reason,
            ..
        } if reason == "market_empty"
    )));
}

#[test]
fn market_sell_gear_none_treated_as_zero() {
    let mut eng = Engine::new();
    for i in 0..3 {
        eng.on_order(BbOrder::test_limit(
            Side::Buy,
            dec(&(100 - i).to_string()),
            &format!("b{i}"),
            i,
            "1",
        ));
    }
    let mut taker = BbOrder::test_market(Side::Sell, "s_gear_none", 10, "10");
    taker.gear = None;
    let events = eng.on_order(taker);
    let fill_count = events
        .iter()
        .filter(|e| matches!(e, MatchEvent::Fill { .. }))
        .count();
    assert_eq!(fill_count, 1);
    assert!(events.iter().any(|e| matches!(
        e,
        MatchEvent::Revoke {
            reason,
            ..
        } if reason == "market_gear"
    )));
}
