//! Per-symbol single-threaded worker: `recv → engine.on_order → outbound`.

use std::sync::Arc;

use match_core::{BbOrder as CoreOrder, Engine};
use match_protocol::BbOrder;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::outbound::Outbound;

/// Spawn a worker that owns an [`Engine`] for `symbol`.
pub fn spawn_symbol_worker(
    symbol: String,
    mut rx: mpsc::UnboundedReceiver<BbOrder>,
    outbound: Arc<Outbound>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut engine = Engine::new();
        info!(symbol = %symbol, "symbol worker started");
        while let Some(order) = rx.recv().await {
            let order_no = order.trust_order_no.clone();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                engine.on_order(CoreOrder(order))
            })) {
                Ok(events) => {
                    outbound.handle_events(&symbol, &events, &engine);
                }
                Err(_) => {
                    error!(symbol = %symbol, order_no = %order_no, "engine panic on order");
                }
            }
        }
        info!(symbol = %symbol, "symbol worker stopped");
    })
}
