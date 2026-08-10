# Tactical Advice（智能建议系统）

针对 LoL 排位对局的多视角战术建议生成器。基于 5 个 GoF 设计模式 + 3 个玩家视角 + 5 类对局分析 + 7 个建议类别。

> 仅在排位模式（`SoloRanked` / `FlexRanked` / `MixedRanked`）下生成建议，其他模式直接返回空数组。

## 架构

```
            ┌────────────────────┐
            │  generate_advice   │  ← 主入口
            └─────────┬──────────┘
                      │
                      ▼
        ┌──────────────────────────┐
   1.   │  AdviceContext (context) │  上下文对象，承载 stats / games / role / perspective / target
        └─────────────┬────────────┘
                      │
                      ▼
        ┌──────────────────────────┐
   2.   │ AdviceChain (chain)      │  责任链：依次跑 5 个分析器
        └─────────────┬────────────┘
                      │
        ┌─────────────┼─────────────┬──────────────┬──────────────┐
        ▼             ▼             ▼              ▼              ▼
   LaningAdvice  FarmingAdvice  TeamfightAdvice  VisionAdvice  ChampionAdvice
   (对线)         (发育)        (团战)           (视野)        (英雄池)
        │             │             │              │              │
        └─────────────┴─────────────┴──────────────┴──────────────┘
                              │
                              ▼
                  ┌────────────────────────┐
                  │ 5 个 Analyzer 各自产出  │
                  │ Vec<GameAdvice>        │
                  └────────────┬───────────┘
                               ▼
                  ┌────────────────────────┐
                  │ strategies/ 按 perspective │
                  │ 过滤/重排/合并（策略模式） │
                  └────────────┬───────────┘
                              ▼
                  Vec<GameAdvice> (≤ 5 条)
```

| 模式 | 文件 | 职责 |
|---|---|---|
| **Builder** | `builder.rs` | `AdviceBuilder::new().title().problem().evidence().suggestion().priority().category().perspective().build()` 链式构造 `GameAdvice` payload（含必填字段校验：title/problem/evidence 不能缺，suggestions 至少 1 条） |
| **Chain of Responsibility** | `chain.rs` | `AdviceChain` 持 `Vec<Box<dyn AdviceAnalyzer>>`，按顺序执行每个分析器，收集建议、按 `priority` 降序排、截断到前 5 条 |
| **Factory** | `factory.rs` | `AdviceStrategyFactory::create(perspective)` 按 `AdvicePerspective` 选 3 种策略之一 |
| **Strategy** | `strategies/` | 3 个策略：`SelfImprovement` / `Targeting` / `Collaboration`，都实现 `AdviceStrategy` trait |

注：analyzers/ 下 5 个分析器只实现 `AdviceAnalyzer` trait（`analyzers/base.rs`），本身没有 Template Method 骨架——它们是责任链的"节点"。

## 主入口

```rust
use crate::domains::tactical_advice::{generate_advice, AdvicePerspective};
use crate::domains::analysis::{AnalysisStrategy, ParsedGame};
use crate::shared::types::PlayerMatchStats;

let advice = generate_advice(
    stats,                    // &PlayerMatchStats
    &games,                   // &[ParsedGame]
    "MID",                    // role: 主要位置
    AdvicePerspective::SelfImprovement,
    Some("敌方 Faker".into()), // target_name（Targeting/Collaboration 用）
    &AnalysisStrategy::SoloRanked,
);
// Vec<GameAdvice>，最多 5 条，按优先级排
```

## 三种建议视角

| `AdvicePerspective` | 含义 | 策略 | 适用场景 |
|---|---|---|---|
| `SelfImprovement` | 个人改进建议 | `SelfImprovementStrategy` | 自己想提升 |
| `Targeting` | 针对战术建议 | `TargetingStrategy` | 准备对线某个敌人 |
| `Collaboration` | 团队协作建议 | `CollaborationStrategy` | 团队配合 |

实际名称/描述见 `types.rs` 的 `description()` 方法。

## 七种建议类别

`AdviceCategory` 枚举（来自 `shared::types::`）：

| 类别 | 图标 | 名称 |
|---|---|---|
| `Laning` | ⚔️ | 对线 |
| `Farming` | 💰 | 发育 |
| `Teamfight` | 🤝 | 团战 |
| `Vision` | 👁️ | 视野 |
| `Positioning` | 📍 | 站位 |
| `Decision` | 🧠 | 决策 |
| `Champion` | 🎮 | 英雄池 |

## 五种分析器（责任链节点）

| 分析器 | 文件 | 关注维度 |
|---|---|---|
| `LaningAdviceAnalyzer` | `analyzers/laning.rs` | 对线期补刀、换血、节奏 |
| `FarmingAdviceAnalyzer` | `analyzers/farming.rs` | 发育效率、金币转化 |
| `TeamfightAdviceAnalyzer` | `analyzers/teamfight.rs` | 团战参与、输出占比 |
| `VisionAdviceAnalyzer` | `analyzers/vision.rs` | 视野得分、关键位置 |
| `ChampionAdviceAnalyzer` | `analyzers/champion.rs` | 英雄池深度、版本契合 |

## 如何扩展

### 加新建议视角（如 `Coaching`）

1. 在 `shared::types/` 的 `AdvicePerspective` 加一个 variant
2. 在 `strategies/` 新建 `coaching.rs`，实现 `AdviceStrategy` trait
3. 在 `factory.rs` 的 match 加一个分支
4. 类型层（`ts-rs`）会自动同步；前端 `AdvicePerspective.ts` 会更新

### 加新分析器（如 `ObjectiveAdviceAnalyzer`）

1. 在 `analyzers/` 新建 `objective.rs`
2. 实现 `AdviceAnalyzer` trait（定义在 `analyzers/base.rs`，3 个方法：`analyze` / `name` / 可选 `is_enabled`）
3. 在 `mod.rs::generate_advice` 的 chain 里 `.add_analyzer(Box::new(...))`

### 加新建议类别（如 `Macro`）

1. 在 `shared::types/` 的 `AdviceCategory` 加一个 variant
2. 在 `types.rs` 的 `icon()` 和 `name()` 方法补全 match 分支
3. 需要产出新类别的 analyzer 在自己内部用新 variant

## 测试

```bash
cargo test --lib domains::tactical_advice
```

`factory.rs` 自带一个 factory 测试；其他分析器/策略建议补齐单测覆盖边界条件。

## 相关

- `domains/analysis/` — 上游，提供 `AnalysisStrategy`、`ParsedGame` 与时间线分析结果
- `shared/types/` — `AdviceCategory` / `AdvicePerspective` / `GameAdvice` 的实际定义
- `infrastructure/data_services/external/ai/` — AI 路径：建议可作为 LLM prompt 上下文