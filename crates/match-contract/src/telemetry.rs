//! Lightweight counters aligned with Java `ContractMatchTelemetry` / OTel names.
//!
//! See `java-contract-match/docs/opentelemetry-metrics.md`.

use std::sync::atomic::{AtomicU64, Ordering};

static ORDER_EVENTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static ORDERS_INBOUND_INVALID_TOTAL: AtomicU64 = AtomicU64::new(0);
static ORDERS_PLACED_TOTAL: AtomicU64 = AtomicU64::new(0);
static ORDERS_CANCELLED_TOTAL: AtomicU64 = AtomicU64::new(0);
static TRADES_DEALS_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Cumulative nanoseconds for L2/L3/L1 spans (hp-engine path).
static SPAN_L2_QUEUE_NS_TOTAL: AtomicU64 = AtomicU64::new(0);
static SPAN_L3_ADAPT_NS_TOTAL: AtomicU64 = AtomicU64::new(0);
static SPAN_L1_ON_ORDER_NS_TOTAL: AtomicU64 = AtomicU64::new(0);
static SPAN_SAMPLES_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Increment `match.order.events.total` (+ per-symbol tracing at call site).
pub fn record_order_event() {
    ORDER_EVENTS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Increment `match.orders.inbound.invalid.total`.
pub fn record_inbound_invalid() {
    ORDERS_INBOUND_INVALID_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Increment `match.orders.placed.total`.
pub fn record_order_placed() {
    ORDERS_PLACED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Increment `match.orders.cancelled.total`.
pub fn record_order_cancelled() {
    ORDERS_CANCELLED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Increment `match.trades.deals.total` (one per fill event emitted).
pub fn record_fill() {
    TRADES_DEALS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Record L2 queue / L3 adapt / L1 on_order span nanoseconds (see `docs/e2e-budget.md`).
pub fn record_span_ns(l2_queue_ns: u64, l3_adapt_ns: u64, l1_on_order_ns: u64) {
    SPAN_L2_QUEUE_NS_TOTAL.fetch_add(l2_queue_ns, Ordering::Relaxed);
    SPAN_L3_ADAPT_NS_TOTAL.fetch_add(l3_adapt_ns, Ordering::Relaxed);
    SPAN_L1_ON_ORDER_NS_TOTAL.fetch_add(l1_on_order_ns, Ordering::Relaxed);
    SPAN_SAMPLES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Prometheus text exposition (OTel metric names preserved with dots).
pub fn render_prometheus() -> String {
    format!(
        concat!(
            "# HELP match.order.events.total Contract match order events processed\n",
            "# TYPE match.order.events.total counter\n",
            "match.order.events.total {}\n",
            "# HELP match.orders.inbound.invalid.total Contract inbound orders rejected before queue\n",
            "# TYPE match.orders.inbound.invalid.total counter\n",
            "match.orders.inbound.invalid.total {}\n",
            "# HELP match.orders.placed.total Contract orders accepted into match queue\n",
            "# TYPE match.orders.placed.total counter\n",
            "match.orders.placed.total {}\n",
            "# HELP match.orders.cancelled.total Contract orders revoked from book successfully\n",
            "# TYPE match.orders.cancelled.total counter\n",
            "match.orders.cancelled.total {}\n",
            "# HELP match.trades.deals.total Contract filled deal messages emitted to downstream\n",
            "# TYPE match.trades.deals.total counter\n",
            "match.trades.deals.total {}\n",
            "# HELP match.span.l2_queue_ns_total Cumulative ns attributed to L2 queue (hp-engine)\n",
            "# TYPE match.span.l2_queue_ns_total counter\n",
            "match.span.l2_queue_ns_total {}\n",
            "# HELP match.span.l3_adapt_ns_total Cumulative ns for L3 BbOrder→HpCommand adapt\n",
            "# TYPE match.span.l3_adapt_ns_total counter\n",
            "match.span.l3_adapt_ns_total {}\n",
            "# HELP match.span.l1_on_order_ns_total Cumulative ns for L1 HpEngine::on_order\n",
            "# TYPE match.span.l1_on_order_ns_total counter\n",
            "match.span.l1_on_order_ns_total {}\n",
            "# HELP match.span.samples_total Span sample count\n",
            "# TYPE match.span.samples_total counter\n",
            "match.span.samples_total {}\n",
        ),
        ORDER_EVENTS_TOTAL.load(Ordering::Relaxed),
        ORDERS_INBOUND_INVALID_TOTAL.load(Ordering::Relaxed),
        ORDERS_PLACED_TOTAL.load(Ordering::Relaxed),
        ORDERS_CANCELLED_TOTAL.load(Ordering::Relaxed),
        TRADES_DEALS_TOTAL.load(Ordering::Relaxed),
        SPAN_L2_QUEUE_NS_TOTAL.load(Ordering::Relaxed),
        SPAN_L3_ADAPT_NS_TOTAL.load(Ordering::Relaxed),
        SPAN_L1_ON_ORDER_NS_TOTAL.load(Ordering::Relaxed),
        SPAN_SAMPLES_TOTAL.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increment_and_render() {
        let before = render_prometheus();
        record_order_event();
        record_inbound_invalid();
        record_fill();
        let after = render_prometheus();
        assert_ne!(before, after);
        assert!(after.contains("match.order.events.total"));
    }
}
