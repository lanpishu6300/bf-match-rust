//! Contract match engine process shell (config, restore RPC, MQ/Redis, bootstrap).

pub mod bootstrap;
pub mod config;
pub mod error_queue;
pub mod inbound;
pub mod mq;
pub mod outbound;
pub mod redis_store;
pub mod rpc;
pub mod symbol_worker;
