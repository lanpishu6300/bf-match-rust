use bigdecimal::BigDecimal;
use match_core::{BbOrder, OrderBook, Side};
use std::str::FromStr;

fn order(side: Side, price: &str, no: &str, t: i64) -> BbOrder {
    BbOrder::test_limit(side, BigDecimal::from_str(price).unwrap(), no, t, "1")
}

#[test]
fn buy_book_best_is_highest_price_then_earliest_time() {
    let mut book = OrderBook::new();
    book.insert(order(Side::Buy, "100", "3", 200));
    book.insert(order(Side::Buy, "101", "1", 300));
    book.insert(order(Side::Buy, "101", "2", 100));
    let best = book.best(Side::Buy).unwrap();
    assert_eq!(best.trust_order_no, "2"); // price 101, earlier time
}

#[test]
fn sell_book_best_is_lowest_price() {
    let mut book = OrderBook::new();
    book.insert(order(Side::Sell, "100", "1", 1));
    book.insert(order(Side::Sell, "99", "2", 1));
    assert_eq!(book.best(Side::Sell).unwrap().trust_order_no, "2");
}
