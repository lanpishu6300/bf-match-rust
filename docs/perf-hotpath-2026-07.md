# HP L1 hot-path notes (2026-07-21)

**中文：** [perf-hotpath-2026-07.zh-CN.md](./perf-hotpath-2026-07.zh-CN.md)

Scope: `match-core-hp` cancel, match loop, and level index.
Approach: measure → change one cost class → re-measure. Do not add HashMap traffic on the fill path unless a cancel-heavy workload proves it.

## Method

| Principle | Practice here |
|-----------|---------------|
| Find heat when profilers are noisy | Hand timing + focused microbenches: `fair_compare`, criterion `cancel_hot` / `cross_full` |
| Avoid false sharing and extra stores | Local `taker_open`; no second id→client map on fill |
| Avoid full-map scans | Cancel: read `client_id` from store, then O(1) `HashMap::remove` |
| Kernel wins ≠ e2e wins | Layered budget in [`e2e-budget.md`](./e2e-budget.md); L4 MQ still dominates production |
| Publish only fair peaks | Criterion + `fill_rate` gate; never report HP with `fill_rate == 0` |

## Changes

1. **Cancel:** drop `client_to_id.retain` (O(n)). Resolve slot, read `client_id` from the store, then O(1) `remove`. A reverse `id_to_client` map was tried and reverted — it slowed `fair_cross`.
2. **Match loop:** pass `taker_client` in; keep `taker_open` local; one store read for maker fields; `inline(never)` only under `coverage_nightly`.
3. **Book:** `LevelIndex::get_or_insert_with` (BTree `entry`; ART insert-then-get) so `level_mut` does not double-probe on miss.

## Bottlenecks (current L1)

On `fair_cross` (aggressive limit cross, fill_rate = 0.5):

1. **`client_to_id` HashMap** — insert on rest, remove on full fill / cancel. Required for external-id cancel.
2. **`fills_order` + BTree level walk** — FIFO debit, best-price cache, empty-level remove.
3. **Event `Vec` push** — Fill / Rest / Revoke memory traffic.
4. **Outside L1:** RocketMQ + JSON (L4). Cutting L1 from ~50 ns to ~25 ns does not move e2e until L4 is measured.

## Numbers (2026-07-21)

Host: macOS, Apple Silicon. Rust release:

```bash
cargo run -p match-bench --release --bin fair_compare -- --n 50000
```

### `fair_cross` — HP, 8 consecutive runs (fill_rate = 0.5)

| Metric | Value |
|--------|-------|
| ns/order (sorted) | 47.2, 49.5, 49.6, 53.5, 55.2, 55.4, 56.5, 58.6 |
| median | ~54.3 ns/order |
| orders/s (at median) | ~18–19M |
| Same-day baseline before changes | ~56.3 ns/order (~17.8M/s) |
| Delta | ~4% wall on L1 `fair_cross` (noise-aware) |

Discard runs that deviate more than 3× the median before reporting.

### vs `match-core` (same `fair_compare` process)

| engine | fill_rate | ns/order (sample) | orders/s |
|--------|-----------|-------------------|----------|
| match-core | 0.50 | ~45.8 µs | ~22K |
| match-core-hp | 0.50 | ~48 ns | ~21M |

Core wall time varies with host load; use ratios only when both lines share a process and fill_rate. Gate: fill_rate ≈ 0.5.

### Criterion `cancel_hot` (sample-size 15)

| bench | median wall |
|-------|-------------|
| `core_cancel_hot` | ~1.98 s |
| `hp_cancel_hot` | ~22 ms (wide CI; cancel path uses O(1) map ops) |

Compare to 2026-07-18 numbers in [`bench-results.md`](./bench-results.md) only as order of magnitude.

## Pressure metrics

| Layer | Metric | Gate / target | Command |
|-------|--------|---------------|---------|
| L1 fair | `ns_per_order`, `orders_per_sec` | fill_rate = 0.5 ± 0.01; median of ≥5 runs | `fair_compare --n 50000` |
| L1 scenario | criterion median ratio core/hp | ≥5× on `cross_full`, `partial_walk` | `make bench` / `engine_cmp` |
| L1 cancel | `hp_cancel_hot` median | no O(n) retain | `cargo bench -p match-bench --bench engine_cmp -- cancel_hot` |
| L1+L2 | SPSC worker bench | optional; contract still tokio | `hp_cross_full_spsc` |
| L3 | `match.span.l3_adapt_ns_total` | after `hp-engine` | `/metrics` |
| L4 | p99 inbound + outbound | measure before more L1 micro-opts | after RMQ adapter |

Publish checklist: same `n`, same scenario, fill_rate gate, multi-run median, host + rustc date.

## Do not on L1 next

- Extra HashMaps on the fill path without a cancel-heavy workload proof.
- Claim e2e wins from L1-only benches.
- Treat zero-fill ART/SIMD peaks as throughput.
