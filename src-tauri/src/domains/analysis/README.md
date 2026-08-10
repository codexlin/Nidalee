# Nidalee 智能对局分析系统

一个基于 League of Legends Client API (LCU) 的深度对局分析系统，提供对手分析、队友配合建议和自我提升方案。

## 🌟 **核心特性**

- ✅ **时间线数据解析** - 从frames提取每分钟游戏状态
- ✅ **智能对手识别** - 基于位置自动识别对线对手
- ✅ **事件追踪分析** - 击杀、推塔、打龙全面追踪
- ✅ **三维度分析** - 对手、队友、自我全面分析
- ✅ **个性化建议** - 针对性的战术和提升建议
- ✅ **灵活配置** - 支持多种分析模式

## 📦 **快速开始**

### 基础使用

```rust
use crate::domains::analysis::{
    EnhancedAnalysisService,
    AnalysisConfig,
};

// 1. 创建服务
let service = EnhancedAnalysisService::new(
    AnalysisConfig::full_featured()
);

// 2. 执行分析
let result = service.analyze(
    &match_data,      // LCU API返回的对局数据
    puuid,            // 玩家PUUID
    participant_id    // 玩家参与者ID (1-10)
)?;

// 3. 访问结果
println!("数据质量: {}/100", result.metadata.data_quality_score);
println!("分析耗时: {}ms", result.metadata.analysis_time_ms);

// 基础统计
println!("KDA: {:.2}", result.player_stats.avg_kda);
println!("特征数: {}", result.player_stats.traits.len());

// 智能分析（如果启用）
if let Some(ref intelligent) = result.intelligent_analysis {
    println!("对手数: {}", intelligent.opponent_analyses.len());
    println!("队友数: {}", intelligent.teammate_analyses.len());
    println!("团队配合度: {:.1}%", intelligent.tactical_summary.team_synergy);
}

// 统一建议
println!("建议总数: {}", result.all_advice.len());
```

### 配置选项

```rust
// 保守模式（仅基础分析，最快）
let config = AnalysisConfig::conservative();

// Beta模式（启用对手和自我提升分析）
let config = AnalysisConfig::beta();

// 完整模式（启用所有功能）
let config = AnalysisConfig::full_featured();

// 自定义配置
let config = AnalysisConfig {
    enable_intelligent_analysis: true,
    enable_opponent_analysis: true,
    enable_teammate_analysis: false,  // 不分析队友
    enable_self_improvement: true,
    advice_perspective: AdvicePerspective::Targeting, // 针对对手
};
```

## 🎯 **核心模块**

### 1. 时间线解析（唯一入口）

时间线解析统一走 `analysis::evidence`。`extract_match_evidence` 吃 LCU 原始 JSON
（对局详情 + 可选时间线），产出确定性的 `MatchEvidence`：缺帧、缺时间线这类问题
落在 `quality`/`diagnostics` 上，而不是被静默填成 0。

```rust
use crate::domains::analysis::evidence::{extract_match_evidence, laning_opponent_diff};

// game / timeline 都是 LCU 原始 JSON，timeline 可为 None
let evidence = extract_match_evidence(&game, timeline.as_ref(), target_puuid)?;

// 按阶段访问：phases 只包含真实存在的阶段，不会为缺失阶段补零
for phase in &evidence.phases {
    // 速率是 Option：阶段内只有一个锚点帧时为 None，不会除零编个数字出来
    println!("{:?} 补刀/分钟: {:?}", phase.phase, phase.cs_per_min);
}

// 对线期与对手的差值（没识别出对手时为 None）
if let Some(diff) = laning_opponent_diff(&evidence.phases) {
    println!("对线期补刀差: {}", diff.cs_diff);
}
```

旧的 `TimelineBridge` / `TimelineAnalysis` 仍然存在，但只作为存量调用方的兼容层，
内部已经委托给上面的 evidence 逻辑；新代码不要再从那里入口。

### 2. 对手识别（唯一入口）

对线对手只能通过 `analysis::evidence::resolve_lane_opponent` 获取。
它按 lane+role → 统一位置 → 对线期空间邻近的顺序匹配，且**只在真实分路**
（上/中/ADC/辅助）上开放空间回退，识别不出来时返回 `None` 而不是硬猜。

```rust
use crate::domains::analysis::evidence::resolve_lane_opponent;

// game / timeline 都是 LCU 原始 JSON，timeline 可为 None
let opponent = resolve_lane_opponent(&game, timeline.as_ref(), target_participant_id, queue_id);

if let Some(opponent) = opponent {
    println!("对手ID: {}", opponent.participant_id);
    println!("识别方式: {:?}", opponent.method); // LaneRole / PositionMatch / SpatialProximity
    println!("置信度: {:.2}", opponent.confidence);
    println!("对手位置: {}", opponent.position); // TOP / JUNGLE / MID / ADC / SUPPORT ...
}
```

> ⚠️ 旧的 `core::OpponentIdentifier` 已废弃并从公开重导出中移除：
> 它靠 `participantId <= 5` 猜队伍，且只会挑「最近的人」，
> 会把队友或被 gank 的对象当成对线对手。

### 3. 事件分析器

```rust
use crate::domains::analysis::analyzers::core::EventAnalyzer;

let analyzer = EventAnalyzer;

// 统计事件
let stats = analyzer.analyze_player_events(player_id, &frames);
println!("KDA: {}/{}/{}", stats.kills, stats.deaths, stats.assists);
println!("推塔: {}", stats.tower_kills);
println!("打龙: {}", stats.dragon_kills);

// 识别关键时刻
let moments = analyzer.identify_key_moments(player_id, &frames);
for moment in moments.iter().take(5) {
    println!("{} - {}", moment.event_type, moment.description);
}

// 计算参与度
let participation = analyzer.calculate_participation_rate(&stats, team_kills);
println!("参与度: {:.1}%", participation);
```

### 4. 对手分析器

```rust
use crate::domains::analysis::analyzers::opponent_analyzer::analyze_opponent;

let analysis = analyze_opponent(
    opponent_id,
    &match_data,
    &timeline_analysis,
    &basic_stats
);

// 查看优缺点
for strength in &analysis.strengths {
    println!("✅ {}: {}", strength.category, strength.description);
}

for weakness in &analysis.weaknesses {
    println!("❌ {}: {}", weakness.category, weakness.description);
}

// 打法风格
println!("打法: {:?}", analysis.playstyle);

// 战术建议
for advice in &analysis.tactical_advice {
    println!("🎯 {}: {}", advice.title, advice.description);
}
```

### 5. 队友分析器

```rust
use crate::domains::analysis::analyzers::teammate_analyzer::analyze_teammate;

let analysis = analyze_teammate(
    teammate_id,
    &match_data,
    &timeline_analysis,
    &basic_stats
);

println!("配合度: {:.1}/100", analysis.synergy_score);
println!("配合风格: {:?}", analysis.cooperation_style);

// 配合建议
for advice in &analysis.cooperation_advice {
    println!("🤝 {}", advice.title);
    for action in &advice.actions {
        println!("  - {}", action);
    }
}
```

### 6. 自我提升分析器

```rust
use crate::domains::analysis::analyzers::self_improvement_analyzer::analyze_self_improvement;

let analysis = analyze_self_improvement(
    player_id,
    &match_data,
    &timeline_analysis,
    &basic_stats
);

// 表现评分
println!("对线期: {:.1}/100", analysis.performance_analysis.early_game_score);
println!("团战: {:.1}/100", analysis.performance_analysis.teamfight_participation);

// 技能评估
println!("对线技能: {}/10", analysis.skill_assessment.laning_skill);
println!("补刀技能: {}/10", analysis.skill_assessment.farming_skill);
println!("团战技能: {}/10", analysis.skill_assessment.teamfight_skill);

// 改进建议
for suggestion in &analysis.improvement_suggestions {
    println!("💡 [优先级{}] {}", suggestion.priority, suggestion.title);
    println!("   当前: {}", suggestion.current_performance);
    println!("   目标: {}", suggestion.target_performance);
}

// 训练计划
println!("每日练习:");
for practice in &analysis.training_plan.daily_practice {
    println!("  - {}", practice);
}
```

## 📊 **数据结构**

### UnifiedAnalysisResult

```rust
pub struct UnifiedAnalysisResult {
    // 基础分析（旧系统）
    pub player_stats: PlayerMatchStats,

    // 智能分析（新系统）
    pub intelligent_analysis: Option<IntelligentAnalysisResult>,

    // 统一建议
    pub all_advice: Vec<GameAdvice>,

    // 元数据
    pub metadata: AnalysisMetadata,
}
```

### IntelligentAnalysisResult

```rust
pub struct IntelligentAnalysisResult {
    pub timeline_analysis: Option<TimelineAnalysis>,
    pub opponent_analyses: Vec<OpponentAnalysis>,
    pub teammate_analyses: Vec<TeammateAnalysis>,
    pub self_improvement: Option<SelfImprovementAnalysis>,
    pub comprehensive_advice: Vec<ComprehensiveAdvice>,
    pub tactical_summary: TacticalSummary,
}
```

## 🔧 **配置说明**

### AnalysisConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| enable_intelligent_analysis | bool | false | 是否启用智能分析 |
| enable_opponent_analysis | bool | true | 是否分析对手 |
| enable_teammate_analysis | bool | true | 是否分析队友 |
| enable_self_improvement | bool | true | 是否生成提升建议 |
| advice_perspective | AdvicePerspective | SelfImprovement | 建议视角 |

### AdvicePerspective

- `SelfImprovement`: 自我提升视角
- `Targeting`: 针对对手视角
- `Collaboration`: 团队协作视角

## 📈 **性能指标**

| 操作 | 预估耗时 | 内存占用 |
|------|----------|----------|
| 解析时间线 | < 50ms | ~10KB |
| 识别对手 | < 20ms | ~5KB |
| 事件分析 | < 30ms | ~1KB |
| 完整分析 | < 200ms | ~30KB |

## 🧪 **测试**

```bash
# 运行所有测试
cargo test --package nidalee --lib domains::analysis

# 运行特定模块测试
cargo test --package nidalee opponent_identifier
cargo test --package nidalee event_analyzer
```

## 📚 **文档**

历史设计文档（阶段报告、API 映射、阶段完成总结等）已归档到
`docs/archive/2025-q4-analysis-refactor/`，本 README 是当前架构的唯一真相。

- 阈值常量索引见 [THRESHOLDS.md](./THRESHOLDS.md)
- 战术建议系统见 [tactical_advice/README.md](../tactical_advice/README.md)

## 🐛 **常见问题**

### Q: 为什么有些对局没有智能分析？

A: 智能分析需要 `match_timeline_json.frames` 数据。如果这个字段为空或不存在，系统会自动降级到基础分析。

### Q: 对手识别的置信度是什么意思？

A: 置信度（0.0-1.0）表示识别的可靠性。>0.8为高置信度，0.5-0.8为中等，<0.5为低置信度。低置信度可能是因为玩家频繁游走或是打野位置。

### Q: 如何提高分析速度？

A:
1. 使用 `AnalysisConfig::conservative()` 模式
2. 禁用不需要的分析模块
3. 实现缓存系统（待开发）

### Q: 数据质量评分低怎么办？

A: 数据质量取决于LCU API返回的数据完整性。如果<50分，建议只使用基础分析功能。

## 🤝 **贡献指南**

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📝 **许可证**

本项目采用 MIT 许可证

## 👥 **作者**

Nidalee Team

## 🙏 **致谢**

感谢 Riot Games 提供的 LCU API

---

**版本**: 2.0.0
**状态**: ✅ 生产就绪
**最后更新**: 2024-10-23
