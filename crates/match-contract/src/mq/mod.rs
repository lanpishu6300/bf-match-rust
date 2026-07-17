//! Messaging layer: topics, transport traits, memory adapter, producer/consumer facades.

pub mod consumer;
pub mod memory;
pub mod producer;
pub mod topics;
pub mod traits;

pub use consumer::{start_consumers, subscriptions_for_symbols};
pub use memory::{MemoryMessageSource, MemoryOrderSink};
pub use producer::Producer;
pub use topics::*;
pub use traits::{MessageSource, OrderSink, SinkError, SourceError, Subscription};
