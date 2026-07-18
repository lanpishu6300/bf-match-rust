//! Exhaustive branch coverage for validate / convert / encode.

use bigdecimal::BigDecimal;
use match_protocol::{check_mq_order, encode_symbol_key, type_convert, MqOrder};
use std::str::FromStr;

fn valid_mq() -> MqOrder {
    MqOrder {
        user_id: Some(1),
        uid: Some(100),
        c_type: 1,
        deal_type: Some(1),
        r#type: Some(1),
        order_type: Some(1),
        market_id: Some(1),
        coin_id: Some(2),
        symbol_key: Some("btcusdt".into()),
        coin_market: Some("BTC/USDT".into()),
        trust_order_no: Some("10001".into()),
        close_position: Some(1),
        start_deposit: Some("10".into()),
        position_type: Some(0),
        taker_rate: Some("0.0005".into()),
        order_status: Some(0),
        order_form: Some(1),
        gear: None,
        lever_times: Some(10),
        trust_number: Some("1".into()),
        trust_price: Some("50000".into()),
        create_time: Some(1_700_000_000_000),
        face_value: Some(BigDecimal::from_str("0.001").unwrap()),
        handicap_type: None,
    }
}

#[test]
fn encode_empty_and_ascii_and_unicode() {
    assert_eq!(encode_symbol_key(""), "");
    assert_eq!(encode_symbol_key("btc_usdt"), "btc_usdt");
    assert_eq!(encode_symbol_key("btc-usdt"), "btc-usdt");
    // non-ascii token → base64url
    let enc = encode_symbol_key("币安");
    assert!(!enc.is_empty());
    assert_ne!(enc, "币安");
}

#[test]
fn encode_slash_passthrough_and_partial_encode() {
    assert_eq!(encode_symbol_key("BTC/USDT"), "BTC/USDT");
    let enc = encode_symbol_key("币/USDT");
    assert!(enc.contains('/'));
    assert!(enc.ends_with("USDT") || enc.contains("USDT"));
    let enc2 = encode_symbol_key("BTC/安");
    assert!(enc2.contains('/'));
    assert!(enc2.starts_with("BTC"));
    let both = encode_symbol_key("币/安");
    assert!(both.contains('/'));
    assert_ne!(both, "币/安");
}

#[test]
fn encode_leading_slash_parts() {
    // splitn: empty base, quote "USDT"
    let s = encode_symbol_key("/USDT");
    assert!(s.contains('/'));
}

#[test]
fn check_rejects_each_required_field() {
    let cases: Vec<Box<dyn Fn(&mut MqOrder)>> = vec![
        Box::new(|o| o.user_id = None),
        Box::new(|o| o.r#type = None),
        Box::new(|o| o.r#type = Some(99)),
        Box::new(|o| o.order_type = None),
        Box::new(|o| o.order_type = Some(99)),
        Box::new(|o| o.market_id = None),
        Box::new(|o| o.coin_id = None),
        Box::new(|o| o.order_form = None),
        Box::new(|o| {
            o.order_form = Some(2);
            o.gear = None;
        }),
        Box::new(|o| o.symbol_key = None),
        Box::new(|o| o.symbol_key = Some("  ".into())),
        Box::new(|o| o.coin_market = None),
        Box::new(|o| o.coin_market = Some("".into())),
        Box::new(|o| o.trust_order_no = None),
        Box::new(|o| o.order_status = None),
        Box::new(|o| o.order_status = Some(99)),
        Box::new(|o| o.trust_number = None),
        Box::new(|o| o.trust_number = Some("".into())),
        Box::new(|o| o.trust_price = None),
        Box::new(|o| o.create_time = None),
        Box::new(|o| o.create_time = Some(0)),
        Box::new(|o| o.create_time = Some(-1)),
        Box::new(|o| o.close_position = None),
        Box::new(|o| o.start_deposit = None),
        Box::new(|o| o.start_deposit = Some("   ".into())),
        Box::new(|o| o.taker_rate = None),
        Box::new(|o| o.position_type = None),
    ];
    for (i, mutate) in cases.into_iter().enumerate() {
        let mut o = valid_mq();
        mutate(&mut o);
        assert!(!check_mq_order(&o), "case {i} should reject");
    }
}

#[test]
fn check_accepts_market_with_gear() {
    let mut o = valid_mq();
    o.order_form = Some(2);
    o.gear = Some(5);
    assert!(check_mq_order(&o));
}

#[test]
fn type_convert_rejects_non_positive_and_missing() {
    let mut o = valid_mq();
    o.trust_number = Some("0".into());
    assert!(type_convert(&o).is_none());

    o = valid_mq();
    o.trust_price = Some("-1".into());
    assert!(type_convert(&o).is_none());

    o = valid_mq();
    o.uid = None;
    assert!(type_convert(&o).is_none());

    o = valid_mq();
    o.symbol_key = Some("BTC/USDT".into());
    let bb = type_convert(&o).unwrap();
    assert_eq!(bb.symbol_key, "btcusdt");

    o = valid_mq();
    o.trust_number = Some("not-a-number".into());
    assert!(type_convert(&o).is_none());
}
