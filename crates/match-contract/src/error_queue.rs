//! MQ send-failure queue backed by Redis list `match:poc_redis_send_mq_error_data_queue`.
//!
//! Java (`OrderProducer`, `SendErrorData`) uses `LPUSH` on failure and `RPOP` for retry.
//! List elements are Kryo-serialized `List<BBOrder>` in production Java. This module exposes
//! raw byte push/pop for wire compatibility and JSON helpers for Rust-only roundtrips/tests.

use crate::redis_store::{mq_error_queue_key, RedisStore, RedisStoreError};
use match_protocol::BbOrder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorQueueError {
    #[error("redis store: {0}")]
    Store(#[from] RedisStoreError),
    #[error("json encode/decode: {0}")]
    Json(#[from] serde_json::Error),
}

/// Push/pop failed outbound order batches on the Java error queue key.
pub struct ErrorQueue<'a> {
    store: &'a mut RedisStore,
    key: String,
}

impl<'a> ErrorQueue<'a> {
    pub fn new(store: &'a mut RedisStore) -> Self {
        Self {
            store,
            key: mq_error_queue_key(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// `LPUSH` — mirrors `RedisTemplateMatch.lpushList`.
    pub fn push_raw(&mut self, payload: &[u8]) -> Result<i64, ErrorQueueError> {
        Ok(self.store.lpush_bytes(&self.key, payload)?)
    }

    /// `RPOP` — mirrors `RedisTemplateMatch.rPopList`.
    pub fn pop_raw(&mut self) -> Result<Option<Vec<u8>>, ErrorQueueError> {
        Ok(self.store.rpop_bytes(&self.key)?)
    }

    /// JSON-encoded `Vec<BbOrder>` for Rust tests; Java expects Kryo on the same key.
    pub fn push_orders(&mut self, orders: &[BbOrder]) -> Result<i64, ErrorQueueError> {
        let payload = serde_json::to_vec(orders)?;
        self.push_raw(&payload)
    }

    pub fn pop_orders(&mut self) -> Result<Option<Vec<BbOrder>>, ErrorQueueError> {
        match self.pop_raw()? {
            None => Ok(None),
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    fn sample_order() -> BbOrder {
        BbOrder {
            user_id: 1,
            uid: 100,
            r#type: 1,
            order_type: 1,
            market_id: 1,
            coin_id: 2,
            symbol_key: "btcusdt".into(),
            coin_market: "BTC/USDT".into(),
            trust_order_no: "T001".into(),
            order_form: 1,
            gear: None,
            close_position: 1,
            start_deposit: BigDecimal::from_str("100").unwrap(),
            target_rate: BigDecimal::from_str("0.001").unwrap(),
            position_type: 1,
            lever_times: 10,
            order_status: 0,
            consumer_all_number: BigDecimal::from_str("0").unwrap(),
            current_deal_number: BigDecimal::from_str("0").unwrap(),
            trust_number: BigDecimal::from_str("1").unwrap(),
            trust_price: BigDecimal::from_str("50000").unwrap(),
            remaining_number: BigDecimal::from_str("1").unwrap(),
            create_time: 1_700_000_000,
            face_value: None,
            average_price: BigDecimal::from_str("0").unwrap(),
        }
    }

    #[test]
    fn error_queue_key_matches_java() {
        assert_eq!(
            mq_error_queue_key(),
            "match:poc_redis_send_mq_error_data_queue"
        );
    }

    #[test]
    fn orders_json_roundtrip_without_redis() {
        let orders = vec![sample_order()];
        let payload = serde_json::to_vec(&orders).unwrap();
        let decoded: Vec<BbOrder> = serde_json::from_slice(&payload).unwrap();
        assert_eq!(decoded, orders);
    }
}
