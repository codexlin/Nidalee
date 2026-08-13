# Rust 模块治理计划

## 目标

本计划用于指导后续 Rust 核心代码重构。重点不是按行数机械拆文件，而是让每个模块只负责一段明确的数据流，并保持现有行为、并发语义、IPC 契约和错误语义稳定。

核心数据流：

```text
LCU HTTP / WebSocket
  → transport / decoder
  → event_handler 状态归并
  → EventCache 当前会话状态
  → 玩家身份与战绩补全
  → analysis_data 实时分析编排
  → domains/analysis 纯业务计算
  → TeamAnalysisData 等 IPC 类型
  → Vue Store / UI
```

## 治理原则

1. 按职责和数据流拆分，不以文件行数作为唯一依据。
2. 一次只治理一个模块；先做纯结构移动，再单独调整行为。
3. 对外函数、事件名、错误文本和调用路径尽量保持不变。
4. 网络 `await` 期间不得持有全局写锁。
5. 少量重复优于错误抽象；稳定出现三次以上且语义一致时再提取公共能力。
6. 小型、高内聚测试保留内联；大型测试使用相邻 `*_tests.rs`。
7. 内部 service 使用结构化错误；只在 Tauri IPC 边界转换为用户可读字符串。
8. 每批独立提交，确保可单独 review、回滚和验证。

## 实施顺序

### 1. `analysis_data/service.rs` 职责拆分

当前文件同时承担会话解析、身份分类、队伍构建、网络补全、战绩缓存和重试编排，是下一批最高优先级。

建议边界：

```text
analysis_data/
├─ service.rs                 # 稳定门面与对外入口
├─ session.rs                 # session key、patch、位置规范化
├─ identity.rs                # 人类、机器人、匿名身份分类
├─ team_builder.rs            # TeamAnalysisData 构建流程
├─ player_enrichment.rs       # 召唤师、段位和战绩补全
├─ match_stats_cache.rs       # 缓存 key、命中和写入规则
└─ retry.rs                   # 单玩家分析重试
```

约束：身份模块保持纯函数；I/O 仅留在 enrichment；team builder 只负责编排，不堆积字段解析。

### 2. `evidence/types.rs` 按领域概念分组

建议边界：

```text
evidence/types/
├─ mod.rs
├─ diagnostics.rs
├─ opponent.rs
├─ events.rs
├─ match_evidence.rs
└─ summary.rs
```

必须保持公共类型名称、Serde 字段、`ts-rs export_to` 路径及外部 re-export 不变。类型同步后 `global.d.ts` 不应产生非预期差异。

### 3. `pipeline/process_insights.rs` 按分析产品拆分

建议边界：

```text
pipeline/process_insights/
├─ mod.rs
├─ key_moments.rs
├─ opponent_compare.rs
├─ deaths.rs
├─ laning.rs
├─ objectives.rs
├─ vision.rs
└─ actions.rs
```

按“关键时刻、对位、死亡、对线、资源、视野、建议”等完整业务概念拆分；不要为 `mean`、`round1` 等小函数单独建模块。

### 4. `evidence/events.rs` 按事件家族拆分

建议边界：

```text
evidence/events/
├─ mod.rs             # 时间线遍历和统一分发
├─ combat.rs          # 击杀、死亡、助攻
├─ objectives.rs      # 龙、先锋、防御塔
├─ items.rs           # 购买、出售、撤销
└─ position.rs        # 坐标、活动区域和距离
```

共享 JSON 读取 helper 可以保留，但不引入复杂通用事件框架或不必要的 trait。

### 5. 收敛 WebSocket enrichment 的稳定重复能力

候选公共能力仅包括：

- LiveClient 玩家列表等待策略；
- 召唤师技能和身份字段规范化。

只有 recovery 与 backfill 的重试、取消和降级语义一致时才共用实现。不得为了去重改变 generation、AbortHandle、锁或事件发布时序。

### 6. 分模块引入结构化错误

推进顺序：

1. `analysis_data`；
2. `liveclient`；
3. `matches`；
4. `auth/request`。

内部返回模块错误枚举，Tauri command 负责最终字符串化。禁止一次性全仓迁移 `Result<T, String>`。

### 7. Clippy 与 CI 门禁

1. 记录当前 Clippy 基线；
2. 先处理 correctness、suspicious 和 perf；
3. 再处理 API 与可读性告警；
4. 清零后在 CI 启用 `-D warnings`；
5. 必须压制时使用带原因的 `#[expect(...)]`，不添加无解释的全局 `allow`。

重点检查不必要 clone、锁跨 `await`、无意义中间集合、忽略 `Result`、大 enum、死代码及依赖错误文本进行分支。

### 8. 最后整理 Debug-only 开发工具

`common/commands/dev_tools/data_collection.rs` 虽然最大，但已受 `debug_assertions` 隔离，不进入 Release，优先级低于实时链路和分析领域。最终可按数据采集、fixture 写盘、阈值实验和命令入口拆分。

## 每批验收门禁

- [ ] 只包含一个明确主题。
- [ ] 对外 API、事件名和 IPC 契约未意外改变。
- [ ] 网络请求顺序、并发上限、重试与取消语义未改变。
- [ ] 网络 `await` 期间不持有全局写锁。
- [ ] 相关定向测试通过。
- [ ] `cargo fmt --all -- --check` 通过。
- [ ] `git diff --check` 通过。
- [ ] 公共 Rust 类型变化时执行类型生成与漂移检查。
- [ ] 高风险批次完成后再运行一次完整 Rust 测试，而不是每个机械步骤重复执行。

## 明确不做

- 不按行数一次性拆完所有大文件。
- 不为每个函数创建一个文件。
- 不在纯结构提交中顺手改变算法、错误文案或缓存策略。
- 不为消除两处相似代码引入抽象框架。
- 不使用 lint `allow` 掩盖可以删除的死代码。
- 不优先重构已隔离的 Debug 工具而延后核心实时链路。

## 下一批

从 `infrastructure/match_management/analysis_data/service.rs` 开始，只进行职责拆分和测试迁移。完成结构稳定与门禁验证后，再单独评估结构化错误和行为优化。
