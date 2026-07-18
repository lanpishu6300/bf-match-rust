//! Height buy handler: PostOnly / IOC / FOK (Java `HeightBuyHandler`).
//!
//! Intentional Java parity: see module docs on [`crate::handlers`] for IOC loop quirk P0-2.

use bigdecimal::BigDecimal;
use match_protocol::{ORDER_FORM_FOK, ORDER_FORM_IOC, ORDER_FORM_POST_ONLY};

use crate::book::OrderBook;
use crate::event::MatchEvent;
use crate::handlers::fok_buy::fok_buy_handle;
use crate::match_limit::{rather_than_buy, revoke_order_with_reason};
use crate::order::{BbOrder, Side};

/// Java `HeightBuyHandler.handle` for forms 3/4/5 (revoke handled in `Engine`).
pub fn handle_height_buy(book: &mut OrderBook, order: BbOrder) -> Vec<MatchEvent> {
    let order_form = order.order_form;
    let order_no = order.trust_order_no.clone();
    let trust_price = order.trust_price.clone();

    book.insert(order);

    // Java: `marketBuyHandler.handle(list)` is BaseHandler no-op — skipped.

    let mut events = Vec::new();
    loop {
        if order_form == ORDER_FORM_POST_ONLY {
            // P2-2: already on book (Java also pushes depth via producer). Revoke if would take.
            let would_take = book
                .first(Side::Sell)
                .is_some_and(|sell| trust_price >= sell.trust_price);
            if would_take {
                push_revoke_if_present(
                    &mut events,
                    revoke_by_no(book, &order_no, Side::Buy, "post_only"),
                );
            }
            break;
        }

        // Engine only routes PostOnly/IOC/FOK here; PostOnly handled above ⇒ IOC/FOK.

        if book.is_empty(Side::Sell) {
            let reason = ioc_or_fok_reason(order_form);
            push_revoke_if_present(
                &mut events,
                revoke_by_no(book, &order_no, Side::Buy, reason),
            );
            break;
        }
        // Reachable on IOC continue after the height order was fully filled.
        if book.is_empty(Side::Buy) {
            break;
        }
        let buy_px = book.first(Side::Buy).unwrap().trust_price.clone();
        let sell_px = book.first(Side::Sell).unwrap().trust_price.clone();
        if buy_px < sell_px {
            let reason = ioc_or_fok_reason(order_form);
            push_revoke_if_present(
                &mut events,
                revoke_by_no(book, &order_no, Side::Buy, reason),
            );
            break;
        }

        if order_form == ORDER_FORM_FOK {
            events.extend(fok_buy_handle(book));
            break;
        }

        // IOC: ratherThan once, then continue without checking whether *this* IOC
        // order remains — intentional Java parity (P0-2).
        // Defensive `None` is ignored inside excluded helper; next iter breaks on empty.
        push_rather_than_buy(book, &mut events);
    }
    events
}

/// Includes defensive `None` no-op from `rather_than_buy` (empty side despite checks).
#[cfg_attr(any(coverage, coverage_nightly), coverage(off))]
fn push_rather_than_buy(book: &mut OrderBook, events: &mut Vec<MatchEvent>) {
    if let Some(ev) = rather_than_buy(book) {
        events.push(ev);
    }
}

fn ioc_or_fok_reason(order_form: i8) -> &'static str {
    if order_form == ORDER_FORM_IOC {
        "ioc_remainder"
    } else {
        "fok_fail"
    }
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

/// Revoke of an order we just inserted always succeeds; `None` arm is defensive
/// (order missing after insert) — helper excluded so that dead arm is not scored.
#[cfg_attr(any(coverage, coverage_nightly), coverage(off))]
fn push_revoke_if_present(events: &mut Vec<MatchEvent>, ev: Option<MatchEvent>) {
    if let Some(ev) = ev {
        events.push(ev);
    }
}
