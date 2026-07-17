//! Transport traits for inbound/outbound messaging.
//!
//! Production RocketMQ adapter is pending client/broker compatibility (see `docs/rmq-spike.md`).
//! Local testing uses [`crate::mq::memory`].

use std::fmt;

/// Outbound message sink (RocketMQ producer or in-memory / file adapter).
pub trait OrderSink: Send + Sync {
    fn send(&self, topic: &str, body: &[u8]) -> Result<(), SinkError>;
}

/// Inbound message source (RocketMQ consumer or in-memory / file adapter).
///
/// Deliveries invoke `handler(topic, body)`. Handler return is ignored for ACK —
/// parity with Java `BaseConsumer.process` (`finally return true`).
pub trait MessageSource: Send + Sync {
    /// Subscribe and start delivering messages. Implementations may spawn background tasks.
    fn start(
        &self,
        subscriptions: &[Subscription],
        handler: InboundHandler,
    ) -> Result<(), SourceError>;

    /// Best-effort shutdown of background delivery.
    fn stop(&self) {}
}

/// Per-symbol (or global) subscription.
#[derive(Debug, Clone)]
pub struct Subscription {
    pub topic: String,
    pub consumer_group: String,
}

pub type InboundHandler = std::sync::Arc<dyn Fn(&str, &[u8]) + Send + Sync>;

#[derive(Debug)]
pub struct SinkError {
    pub message: String,
}

impl SinkError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sink error: {}", self.message)
    }
}

impl std::error::Error for SinkError {}

#[derive(Debug)]
pub struct SourceError {
    pub message: String,
}

impl SourceError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source error: {}", self.message)
    }
}

impl std::error::Error for SourceError {}
