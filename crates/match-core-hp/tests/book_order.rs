use match_core_hp::{Book, HpOrder, Side};

#[test]
fn bid_best_is_highest_tick() {
    let mut b = Book::new();
    b.insert_limit(HpOrder::limit(Side::Buy, 10000, 1, 1));
    b.insert_limit(HpOrder::limit(Side::Buy, 10100, 1, 2));
    assert_eq!(b.best_bid(), Some(10100));
}

#[test]
fn ask_best_is_lowest_tick() {
    let mut b = Book::new();
    b.insert_limit(HpOrder::limit(Side::Sell, 10200, 1, 1));
    b.insert_limit(HpOrder::limit(Side::Sell, 10100, 1, 2));
    assert_eq!(b.best_ask(), Some(10100));
}

#[test]
fn same_price_fifo_cancel_middle() {
    let mut b = Book::new();
    let id1 = b.insert_limit(HpOrder::limit(Side::Buy, 10000, 5, 1));
    let id2 = b.insert_limit(HpOrder::limit(Side::Buy, 10000, 3, 2));
    let id3 = b.insert_limit(HpOrder::limit(Side::Buy, 10000, 2, 3));

    assert!(b.cancel(id1));
    assert_eq!(b.best_bid(), Some(10000));

    // Front of FIFO is now id2.
    let front = b.front_id(Side::Buy, 10000).unwrap();
    assert_eq!(front, id2);

    assert!(b.cancel(id2));
    assert_eq!(b.front_id(Side::Buy, 10000), Some(id3));
    assert!(!b.cancel(id1)); // already gone
}

#[test]
fn depth_aggregates_same_tick() {
    let mut b = Book::new();
    b.insert_limit(HpOrder::limit(Side::Buy, 100, 5, 1));
    b.insert_limit(HpOrder::limit(Side::Buy, 100, 3, 2));
    b.insert_limit(HpOrder::limit(Side::Buy, 90, 1, 3));
    let d = b.depth(Side::Buy, 2);
    assert_eq!(d, vec![(100, 8), (90, 1)]);
}

#[test]
fn cancel_empties_level() {
    let mut b = Book::new();
    let id = b.insert_limit(HpOrder::limit(Side::Sell, 50, 1, 1));
    assert_eq!(b.best_ask(), Some(50));
    assert!(b.cancel(id));
    assert_eq!(b.best_ask(), None);
}
