# HP L1 热路径说明（2026-07-21）

**English：** [perf-hotpath-2026-07.md](./perf-hotpath-2026-07.md)

范围：仅 `match-core-hp` 的撤单、撮合环、档位索引。
做法：先测 → 改一类成本 → 再测。没有撤单重负载证据时，不在成交路径加 HashMap。

## 方法

| 原则 | 本仓库做法 |
|------|------------|
| Profiler 嘈杂时找热点 | 手工计时 + 聚焦微基准：`fair_compare`、criterion `cancel_hot` / `cross_full` |
| 少假共享、少多余写 | 本地 `taker_open`；成交路径不加第二张 id→client 表 |
| 避免全表扫描 | 撤单：从 store 读 `client_id`，再 O(1) `HashMap::remove` |
| 内核快 ≠ 端到端快 | 分层预算见 [`e2e-budget.zh-CN.md`](./e2e-budget.zh-CN.md)；生产仍被 L4 MQ 主导 |
| 只报公平峰值 | Criterion + `fill_rate` 门禁；禁止 `fill_rate == 0` 的 HP 峰值 |

## 改动

1. **撤单：** 去掉 `client_to_id.retain`（O(n)）。解析 slot → 从 store 读 `client_id` → O(1) `remove`。试过反向 `id_to_client`，已回退（拖慢 `fair_cross`）。
2. **撮合环：** 传入 `taker_client`；本地维护 `taker_open`；maker 字段一次读；仅 `coverage_nightly` 强制 `inline(never)`。
3. **Book：** `LevelIndex::get_or_insert_with`（BTree `entry`；ART 先插再取），减少 miss 双探。

## 当前 L1 瓶颈

`fair_cross`（fill_rate = 0.5）上大致残留：

1. **`client_to_id` HashMap**（挂单 insert、全成/撤 remove）
2. **`fills_order` + BTree 档位**（FIFO、最优价缓存、空档删除）
3. **事件 `Vec` push**
4. **L1 之外：** RocketMQ + JSON（L4）— 再抠 L1 对 e2e 几乎不动

## 数字（2026-07-21）

```bash
cargo run -p match-bench --release --bin fair_compare -- --n 50000
```

### HP `fair_cross` 连续 8 次（fill_rate = 0.5）

| 指标 | 值 |
|------|-----|
| ns/order（排序） | 47.2, 49.5, 49.6, 53.5, 55.2, 55.4, 56.5, 58.6 |
| 中位数 | ~54.3 ns/order |
| 同日改动前基线 | ~56.3 ns/order（~17.8M/s） |
| 相对变化 | L1 约 4%（噪声敏感） |

偏离中位数 3× 以上的 run 丢弃后再报。

### 同一次 `fair_compare` 对打样例

| engine | fill_rate | ns/order | orders/s |
|--------|-----------|----------|----------|
| match-core | 0.50 | ~45.8 µs | ~22K |
| match-core-hp | 0.50 | ~48 ns | ~21M |

core 墙钟随负载波动大；只在同进程、fill_rate 对齐时谈倍率。门禁：fill_rate ≈ 0.5。

### criterion `cancel_hot`（sample-size 15）

| bench | median |
|-------|--------|
| `core_cancel_hot` | ~1.98 s |
| `hp_cancel_hot` | ~22 ms |

与 2026-07-18 文档数字只作数量级对比。

## 压测指标

| 层 | 指标 | 门禁 / 目标 | 命令 |
|----|------|-------------|------|
| L1 fair | `ns_per_order`, `orders_per_sec` | fill_rate = 0.5 ± 0.01；≥5 次中位数 | `fair_compare --n 50000` |
| L1 场景 | criterion core/hp 中位比 | `cross_full` / `partial_walk` ≥5× | `make bench` |
| L1 撤单 | `hp_cancel_hot` | 无 O(n) retain | `engine_cmp -- cancel_hot` |
| L1+L2 | SPSC worker | 可选 | `hp_cross_full_spsc` |
| L3 | `match.span.l3_adapt_ns_total` | `hp-engine` 后 | `/metrics` |
| L4 | p99 入出站 | 先测再继续抠 L1 | RMQ 落地后 |

## L1 上暂不做

- 无撤单重负载证据就加成交路径 HashMap
- 用纯 L1 数字宣称 e2e 提升
- 用零成交 ART/SIMD 峰值当吞吐
