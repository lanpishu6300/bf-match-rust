# match-rust

Rust port of the  contract matching engine (`java-contract-match`), structured as a Cargo workspace with shared `match-core`, `match-protocol`, and `match-replay` crates. See the design spec at [`docs/superpowers/specs/2026-07-17-rust-match-engines-design.md`](../docs/superpowers/specs/2026-07-17-rust-match-engines-design.md) for architecture, milestones, and acceptance criteria.

## Build / test

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --workspace
```

## match-contract

Binary shell: config → bootstrap (RPC restore, Redis wipe/link, per-symbol workers) → inbound/outbound messaging.

### RocketMQ status

**Production RocketMQ is not wired yet.** NameServer spike against `192.168.0.241:9876` timed out; see [`docs/rmq-spike.md`](docs/rmq-spike.md). Runtime uses `OrderSink` / `MessageSource` with an in-memory (optional file-channel) adapter (`rocketmq.transport: memory`).

### Local run (memory transport)

```bash
export MATCH_CONTRACT_CONFIG=crates/match-contract/config.example.yaml
# Skip RPC/Redis; start workers for listed symbols only:
export MATCH_CONTRACT_LOCAL_SYMBOLS=btcusdt
cargo run -p match-contract
```

Publish inbound JSON arrays via the `MemoryMessageSource` API in tests, or drop `*.json` files under `{memory_dir}/in/` when configured.

### Test-env smoke (when RPC/Redis/RMQ reachable)

1. Point `config` at test RPC / Redis / RMQ; set `transport: memory` until RMQ adapter lands (or `rocketmq` after spike passes).
2. Prefer `match.symbols_whitelist: ["one-low-traffic-symbol"]`.
3. Confirm restore count logs; place + cancel should produce `usdt_contract_match_order_push_order_{encoded}` payloads (memory sink or live MQ).
