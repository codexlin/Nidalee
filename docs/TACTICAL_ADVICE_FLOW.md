# 智能建议系统完整调用逻辑 v3.1

> **文档说明：** 本文档详细说明了智能建议系统的架构、数据流向、调用逻辑和使用场景。

---

## 📚 目录

- [系统架构](#系统架构)
- [核心设计模式](#核心设计模式)
- [数据流向](#数据流向)
- [三种视角的调用流程](#三种视角的调用流程)
- [关键代码位置](#关键代码位置)
- [类型定义](#类型定义)
- [使用场景](#使用场景)
- [调试和排查](#调试和排查)

---

## 🏗️ 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    智能建议系统 v3.1                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🎯 三种视角（Perspective）                                 │
│  ├─→ SelfImprovement（自我提升）"你"                       │
│  ├─→ Targeting（针对敌人）"对手"                           │
│  └─→ Collaboration（协作队友）"队友"                       │
│                                                             │
│  🔍 五个分析器（Analyzer Chain）                           │
│  ├─→ LaningAdviceAnalyzer（对线期）                        │
│  ├─→ FarmingAdviceAnalyzer（发育效率）                     │
│  ├─→ TeamfightAdviceAnalyzer（团战参与）                   │
│  ├─→ VisionAdviceAnalyzer（视野控制）                      │
│  └─→ ChampionAdviceAnalyzer（英雄池）                      │
│                                                             │
│  📊 数据来源                                                │
│  ├─→ 基础统计（PlayerMatchStats）                          │
│  ├─→ 对局历史（ParsedGame[]）                              │
│  └─→ 时间线数据（TimelineData）                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎨 核心设计模式

| 设计模式 | 应用位置 | 职责 |
|---------|---------|------|
| **Parser** | `parser.rs` | 将 LCU API 原始数据解析为 `ParsedGame` |
| **Strategy** | `strategies/` | 根据视角生成不同措辞的建议 |
| **Builder** | `builder.rs` | 构建 `GameAdvice` 对象 |
| **Factory** | `factory.rs` | 创建对应视角的 `AdviceStrategy` |
| **Chain of Responsibility** | `chain.rs` | 链式处理多个分析器 |
| **Template Method** | `analyzers/base.rs` | 定义分析器的通用流程 |

---

## 🔄 数据流向

### 完整数据流程图

```mermaid
graph TD
    A[LCU API] -->|原始 JSON| B[Parser]
    B -->|ParsedGame| C[Strategy 选择]
    C -->|AnalysisStrategy| D[统计分析]
    D -->|PlayerMatchStats| E[特征分析]
    D -->|ParsedGame[]| F[时间线分析]
    E --> G[建议生成]
    F --> G
    G -->|GameAdvice[]| H[前端展示]

    style A fill:#f9f,stroke:#333
    style G fill:#9f9,stroke:#333
    style H fill:#99f,stroke:#333
```

### 详细数据流

```
第1步：数据获取
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LCU API: GET /lol-match-history/v1/products/lol/{puuid}/matches
  ↓
原始 JSON（包含 games.games 数组，每个对局含 timeline 数据）

第2步：数据解析（Parser 模式）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
parse_games(games: &[Value], puuid: &str) → Vec<ParsedGame>
  ↓
ParsedGame {
    game_id: u64,
    queue_id: i64,
    game_duration: i32,
    player_data: ParsedPlayerData {
        kills, deaths, assists, kda,
        damage_to_champions, vision_score, cs,
        champion_id, role, lane,           // ⭐ 位置信息
        timeline_data: TimelineData {       // ⭐ 时间线数据
            cs_diff_0_10, xp_diff_0_10,
            gold_per_min_0_10, ...
        }
    },
    team_data: ParsedTeamData {...}
}

第3步：策略选择（Strategy 模式）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
AnalysisStrategy::from_queue_id(queue_id) 或
AnalysisStrategy::from_games(parsed_games)
  ↓
AnalysisStrategy::Ranked  // 排位：深度分析+建议生成
AnalysisStrategy::Other   // 其他：简化分析，不生成建议

第4步：统计分析
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
analyze_player_stats(parsed_games, puuid, context) → PlayerMatchStats
  ↓
PlayerMatchStats {
    total_games, wins, losses, win_rate,
    avg_kills, avg_deaths, avg_assists, avg_kda,
    dpm, cspm, vspm,
    traits: Vec<SummonerTrait>,              // 特征标签
    favorite_champions: Vec<AnalysisChampionStats>,
    recent_performance: Vec<MatchPerformance> {  // ⭐ v3.1 新增位置字段
        game_id, win, champion_id, kills, deaths, assists, kda,
        role, lane, position,  // ⭐ 新增：每场对局的位置
        ...
    },
    advice: Vec<GameAdvice>,  // ⭐ v3.0 新增：智能建议
}

第5步：特征分析（多层次）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
if strategy.enable_advanced_analysis() {
    traits.extend(analyze_advanced_traits(...))     // 深度特征
}
if strategy.enable_role_analysis() {
    traits.extend(analyze_role_based_traits(...))   // 位置特征
}
if strategy.enable_distribution_analysis() {
    traits.extend(analyze_distribution_traits(...)) // 分布特征
    traits.extend(analyze_timeline_traits(...))     // ⭐ 时间线特征
}
traits.extend(analyze_win_loss_pattern(...))        // 胜负模式

第6步：建议生成（仅排位模式）⭐ v3.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
if matches!(strategy, AnalysisStrategy::Ranked) {
    let main_role = identify_main_role(&parsed_games);
    player_stats.advice = generate_advice(
        &player_stats,
        &parsed_games,
        &main_role,
        advice_perspective,  // ⭐ 视角参数
        target_player_name,  // ⭐ 目标名称
        &strategy,
    );
}
  ↓
Vec<GameAdvice> {
    title, problem, evidence, suggestions,
    priority, category, perspective,
    affected_role, target_player,
}
```

---

## 🎯 三种视角的调用流程

### 场景 1：Dashboard - 自我提升

**触发：** 用户打开 Dashboard，自动刷新战绩

```rust
// 后端调用链
┌─────────────────────────────────────────────────────────┐
│ 1. 前端调用                                             │
│    invoke('get_match_history', { count: 20 })          │
│                                                         │
│ 2. 命令处理                                             │
│    src-tauri/src/infrastructure/match_management/       │
│    matches/commands.rs::get_match_history()             │
│                                                         │
│ 3. 服务调用                                             │
│    matches/service.rs::get_match_history()              │
│      ↓                                                  │
│    analyze_match_list_data()                            │
│      ↓                                                  │
│    analyze_match_list_data_with_perspective(            │
│        ...,                                             │
│        AdvicePerspective::SelfImprovement, // ⭐ 固定   │
│        None,                              // 无目标名   │
│    )                                                    │
│                                                         │
│ 4. 建议生成                                             │
│    tactical_advice::generate_advice(                    │
│        &player_stats,                                   │
│        &parsed_games,                                   │
│        &main_role,  // 例如："ADC"                     │
│        AdvicePerspective::SelfImprovement,              │
│        None,                                            │
│        &strategy,                                       │
│    )                                                    │
│      ↓                                                  │
│    AdviceContext {                                      │
│        perspective: SelfImprovement,                    │
│        role: "ADC",                                     │
│        target_name: None,                               │
│    }                                                    │
│      ↓                                                  │
│    AdviceStrategyFactory::create(SelfImprovement)       │
│      → SelfImprovementStrategy                          │
│      ↓                                                  │
│    AdviceChain::generate()                              │
│      → 5个分析器依次执行                                │
│      ↓                                                  │
│    Vec<GameAdvice> [                                    │
│        {                                                │
│            title: "对线补刀能力待提升",                 │
│            problem: "你的对线期平均落后15刀...",        │
│            perspective: SelfImprovement,                │
│            ...                                          │
│        }                                                │
│    ]                                                    │
│                                                         │
│ 5. 返回前端                                             │
│    PlayerMatchStats {                                   │
│        advice: Vec<GameAdvice>,  // ⭐ 包含建议       │
│        ...                                              │
│    }                                                    │
└─────────────────────────────────────────────────────────┘
```

**前端展示：**
```vue
<!-- Dashboard.vue -->
<AdvicePanel
  :advice="filteredAdvice"
  :perspective="selectedPerspective"
  :title="advicePanelTitle"
  :subtitle="advicePanelSubtitle"
  @perspective-change="handlePerspectiveChange"
/>

<!-- AdvicePanel.vue -->
<AdviceCard
  v-for="item in advice"
  :key="index"
  :advice="item"
  :perspective="perspective"
/>
```

---

### 场景 2：选人阶段 - 针对敌人 & 协作队友

**触发：** 进入选人阶段（ChampSelect），WebSocket 事件自动触发

```rust
// 后端调用链
┌─────────────────────────────────────────────────────────┐
│ 1. WebSocket 事件监听                                   │
│    src-tauri/src/infrastructure/real_time/websocket/    │
│    event_handler.rs::handle_event()                     │
│      ↓                                                  │
│    监听到 "/lol-champ-select/v1/session" 更新           │
│                                                         │
│ 2. 构建队伍分析数据                                     │
│    analysis_data/service.rs::                           │
│    build_team_analysis_from_session(session, ...)       │
│                                                         │
│ 3. 分别获取队友和敌人的战绩                             │
│                                                         │
│ 【队友】                                                │
│    fetch_players_match_stats_with_perspective(          │
│        my_team_real_players,  // 过滤掉机器人           │
│        http_client,                                     │
│        queue_id,                                        │
│        match_stats_cache,                               │
│        AdvicePerspective::Collaboration,  // ⭐ 队友视角│
│        local_player_cell_id,                            │
│    )                                                    │
│      ↓                                                  │
│    for each player in my_team:                          │
│        if player.is_local:                              │
│            perspective = SelfImprovement  // 自己      │
│        else:                                            │
│            perspective = Collaboration    // 队友      │
│                                                         │
│        get_recent_matches_by_puuid_with_perspective(    │
│            puuid,                                       │
│            20,                                          │
│            queue_id,                                    │
│            perspective,                                 │
│            target_name: player.display_name,  // ⭐     │
│        )                                                │
│          ↓                                              │
│        analyze_match_list_data_with_perspective(        │
│            ...,                                         │
│            perspective,                                 │
│            target_name,                                 │
│        )                                                │
│          ↓                                              │
│        generate_advice(..., Collaboration, ...)         │
│          ↓                                              │
│        Vec<GameAdvice> [                                │
│            {                                            │
│                title: "队友上单前期需要保护",          │
│                problem: "该队友前期容易被击杀...",     │
│                perspective: Collaboration,              │
│                target_player: "队友名称",              │
│                affected_role: "上单",                  │
│                ...                                      │
│            }                                            │
│        ]                                                │
│                                                         │
│ 【敌人】                                                │
│    fetch_players_match_stats_with_perspective(          │
│        enemy_team_real_players,                         │
│        http_client,                                     │
│        queue_id,                                        │
│        match_stats_cache,                               │
│        AdvicePerspective::Targeting,  // ⭐ 敌人视角   │
│        local_player_cell_id,                            │
│    )                                                    │
│      ↓                                                  │
│    for each player in enemy_team:                       │
│        get_recent_matches_by_puuid_with_perspective(    │
│            puuid,                                       │
│            20,                                          │
│            queue_id,                                    │
│            Targeting,  // ⭐ 针对视角                  │
│            target_name: player.display_name,            │
│        )                                                │
│          ↓                                              │
│        generate_advice(..., Targeting, ...)             │
│          ↓                                              │
│        Vec<GameAdvice> [                                │
│            {                                            │
│                title: "软柿子：上单生存能力极差",      │
│                problem: "对手场均死亡8次...",          │
│                perspective: Targeting,                  │
│                target_player: "对手名称",              │
│                affected_role: "上单",                  │
│                ...                                      │
│            }                                            │
│        ]                                                │
│                                                         │
│ 4. 组装返回数据                                         │
│    TeamAnalysisData {                                   │
│        my_team: Vec<PlayerAnalysisData> {               │
│            match_stats: Some(PlayerMatchStats {         │
│                advice: Vec<GameAdvice>,  // ⭐ 协作建议│
│            }),                                          │
│        },                                               │
│        enemy_team: Vec<PlayerAnalysisData> {            │
│            match_stats: Some(PlayerMatchStats {         │
│                advice: Vec<GameAdvice>,  // ⭐ 针对建议│
│            }),                                          │
│        },                                               │
│    }                                                    │
│                                                         │
│ 5. 通过 WebSocket 推送给前端                            │
│    emit("team-analysis-updated", TeamAnalysisData)      │
└─────────────────────────────────────────────────────────┘
```

**前端接收：**
```typescript
// store.ts
const setTeamAnalysisData = (data: TeamAnalysisData) => {
  // 保存队友战绩（含 Collaboration 建议）
  myTeamStats.value = data.myTeam.map(p => p.matchStats);

  // 保存敌人战绩（含 Targeting 建议）
  enemyTeamStats.value = data.enemyTeam.map(p => p.matchStats);
}

// CompactPlayerCard.vue
const playerAdvice = computed(() => {
  return props.playerStats?.advice || [];  // ⭐ 直接使用已生成的建议
});

// 点击战术按钮
<TacticalAdviceDialog
  :advice="playerAdvice"  // ⭐ 传递已生成的建议
  :perspective="isAlly ? 'Collaboration' : 'Targeting'"
/>
```

---

### 场景 3：手动查询指定玩家（可选）

**触发：** 前端主动调用 API

```typescript
// 前端调用
const advice = await invoke('get_player_tactical_advice', {
  summonerName: '玩家名称',
  perspective: 'Targeting',  // 或 'Collaboration'
  targetRole: null,  // 可选
});
```

```rust
// 后端调用链
┌─────────────────────────────────────────────────────────┐
│ 1. 命令处理                                             │
│    commands.rs::get_player_tactical_advice()            │
│      ↓                                                  │
│    解析视角参数："Targeting" → AdvicePerspective::Targeting
│                                                         │
│ 2. 服务调用                                             │
│    service.rs::get_player_tactical_advice()             │
│      ↓                                                  │
│    根据召唤师名称获取 PUUID                             │
│      ↓                                                  │
│    get_recent_matches_by_puuid_with_perspective(        │
│        puuid,                                           │
│        20,                                              │
│        queue_id: Some(420),  // 排位                   │
│        perspective: Targeting,                          │
│        target_name: Some(summoner_name),                │
│    )                                                    │
│      ↓                                                  │
│    generate_advice(...)                                 │
│                                                         │
│ 3. 直接返回建议                                         │
│    Vec<GameAdvice>                                      │
└─────────────────────────────────────────────────────────┘
```

---

## 🧩 建议生成核心流程

```rust
// tactical_advice/mod.rs::generate_advice()
┌─────────────────────────────────────────────────────────┐
│ 输入参数：                                              │
│   - stats: &PlayerMatchStats      // 统计数据          │
│   - games: &[ParsedGame]          // 对局历史          │
│   - role: &str                    // 主要位置          │
│   - perspective: AdvicePerspective // 视角             │
│   - target_name: Option<String>   // 目标名称          │
│   - strategy: &AnalysisStrategy   // 分析策略          │
│                                                         │
│ 步骤 1：构建上下文                                      │
│   AdviceContext::new(                                   │
│       stats.clone(),                                    │
│       games.to_vec(),                                   │
│       role.to_string(),                                 │
│       perspective,                                      │
│       target_name,                                      │
│   )                                                     │
│                                                         │
│ 步骤 2：创建责任链                                      │
│   AdviceChain::new()                                    │
│       .add_analyzer(LaningAdviceAnalyzer)               │
│       .add_analyzer(FarmingAdviceAnalyzer)              │
│       .add_analyzer(TeamfightAdviceAnalyzer)            │
│       .add_analyzer(VisionAdviceAnalyzer)               │
│       .add_analyzer(ChampionAdviceAnalyzer)             │
│                                                         │
│ 步骤 3：执行责任链                                      │
│   for each analyzer in chain:                           │
│     ├─→ analyzer.analyze(context, strategy)             │
│     │     ↓                                             │
│     │   检测问题（例如：avg_deaths > 6.0）             │
│     │     ↓                                             │
│     │   if 有问题:                                      │
│     │       strategy = Factory::create(perspective)     │
│     │       advice = strategy.generate_advice(...)      │
│     │         ↓                                         │
│     │       根据视角生成不同措辞：                      │
│     │         - SelfImprovement: "你的..."             │
│     │         - Targeting: "对手..."                   │
│     │         - Collaboration: "队友..."               │
│     └─→ return Some(GameAdvice)                         │
│                                                         │
│ 步骤 4：排序和限制                                      │
│   advice_list.sort_by(|a, b| b.priority.cmp(&a.priority))
│   advice_list.truncate(5)  // 最多5条                  │
│                                                         │
│ 输出：Vec<GameAdvice>（按优先级排序）                  │
└─────────────────────────────────────────────────────────┘
```

---

## 📍 关键代码位置

### 后端 Rust 代码

#### 1. 核心建议系统

| 文件 | 路径 | 职责 |
|------|------|------|
| **主入口** | `src-tauri/src/domains/tactical_advice/mod.rs` | `generate_advice()` 函数 |
| **类型定义** | `src-tauri/src/shared/types/types/mod.rs` | `GameAdvice`, `AdviceCategory`, `AdvicePerspective` |
| **上下文** | `domains/tactical_advice/context.rs` | `AdviceContext` |
| **建造者** | `domains/tactical_advice/builder.rs` | `AdviceBuilder` |
| **责任链** | `domains/tactical_advice/chain.rs` | `AdviceChain` |
| **工厂** | `domains/tactical_advice/factory.rs` | `AdviceStrategyFactory` |

#### 2. 三种策略

| 视角 | 文件 | 措辞 | 目标 |
|------|------|------|------|
| **SelfImprovement** | `strategies/self_improvement.rs` | 第二人称（"你"） | 长期提升 |
| **Targeting** | `strategies/targeting.rs` | 第三人称（"对手"） | 针对弱点 |
| **Collaboration** | `strategies/collaboration.rs` | 第三人称（"队友"） | 团队配合 |

#### 3. 五个分析器

| 分析器 | 文件 | 分析内容 |
|--------|------|---------|
| **LaningAdviceAnalyzer** | `analyzers/laning.rs` | 对线期 CS 差、经验差、被压制 |
| **FarmingAdviceAnalyzer** | `analyzers/farming.rs` | 补刀效率、中期发育 |
| **TeamfightAdviceAnalyzer** | `analyzers/teamfight.rs` | 参团率、死亡率、助攻数 |
| **VisionAdviceAnalyzer** | `analyzers/vision.rs` | 视野得分 |
| **ChampionAdviceAnalyzer** | `analyzers/champion.rs` | 英雄池、依赖度 |

#### 4. 服务集成

| 功能 | 文件 | 调用位置 |
|------|------|---------|
| **Dashboard 战绩** | `infrastructure/match_management/matches/service.rs` | `get_match_history()` → 第 435-446 行 |
| **选人阶段分析** | `infrastructure/match_management/analysis_data/service.rs` | `build_team_analysis_from_session()` → 第 174、199 行 |
| **命令接口** | `infrastructure/match_management/matches/commands.rs` | `get_match_history()`, `get_player_tactical_advice()` |

#### 5. 数据解析

| 模块 | 文件 | 职责 |
|------|------|------|
| **Parser** | `domains/analysis/analyzers/core/parser.rs` | 解析 LCU JSON → `ParsedGame` |
| **Strategy** | `domains/analysis/analyzers/core/strategy.rs` | 根据队列选择分析深度 |
| **Stats Analyzer** | `domains/analysis/analyzers/core/stats.rs` | 统计计算 → `PlayerMatchStats` |
| **Timeline Analyzer** | `domains/analysis/analyzers/traits/timeline.rs` | 时间线特征分析 |

---

### 前端 TypeScript/Vue 代码

#### 1. 类型定义（自动生成）

| 类型 | 文件 | 来源 |
|------|------|------|
| **GameAdvice** | `src/types/generated/GameAdvice.ts` | Rust `GameAdvice` |
| **AdviceCategory** | `src-tauri/bindings/AdviceCategory.ts` | Rust `AdviceCategory` enum |
| **AdvicePerspective** | `src-tauri/bindings/AdvicePerspective.ts` | Rust `AdvicePerspective` enum |
| **PlayerMatchStats** | `src/types/generated/PlayerMatchStats.ts` | Rust `PlayerMatchStats` |
| **MatchPerformance** | `src/types/generated/MatchPerformance.ts` | Rust `MatchPerformance` |

#### 2. Dashboard 组件

| 组件 | 文件 | 职责 |
|------|------|------|
| **Dashboard** | `src/features/dashboard/Dashboard.vue` | 主页面，集成 AdvicePanel |
| **AdvicePanel** | `src/features/dashboard/components/AdvicePanel.vue` | 建议面板，视角切换 |
| **AdviceCard** | `src/features/dashboard/components/AdviceCard.vue` | 单个建议卡片 |

**关键代码：**
```vue
<!-- Dashboard.vue -->
<script setup lang="ts">
// 第54行：视角状态
const selectedPerspective = ref<'self-improvement' | 'targeting' | 'collaboration'>('self-improvement');

// 第88-104行：建议过滤
const filteredAdvice = computed(() => {
  if (!matchStatistics.value?.advice) return [];

  return matchStatistics.value.advice.filter(
    (advice: any) =>
      advice.perspective ===
      (selectedPerspective.value === 'self-improvement'
        ? 'SelfImprovement'
        : selectedPerspective.value === 'targeting'
          ? 'Targeting'
          : 'Collaboration')
  );
});

// 第80行：视角切换
const handlePerspectiveChange = (perspective: ...) => {
  selectedPerspective.value = perspective;
};
</script>

<!-- 第25-32行：组件使用 -->
<AdvicePanel
  v-if="matchStatistics && !matchHistoryLoading"
  :advice="filteredAdvice"
  :perspective="selectedPerspective"
  :title="advicePanelTitle"
  :subtitle="advicePanelSubtitle"
  @perspective-change="handlePerspectiveChange"
/>
```

#### 3. 对局分析组件

| 组件 | 文件 | 职责 |
|------|------|------|
| **CompactPlayerCard** | `src/features/match-analysis/components/analysis/CompactPlayerCard.vue` | 玩家卡片，含战术按钮 |
| **TacticalAdviceDialog** | `src/features/match-analysis/components/analysis/TacticalAdviceDialog.vue` | 战术建议弹窗 |

**关键代码：**
```vue
<!-- CompactPlayerCard.vue -->

<!-- 第101-113行：战术建议按钮 -->
<button
  v-if="!isLocal && !player.isBot && player.displayName"
  @click.stop="handleTacticalAdvice"
  :title="isAlly ? '查看协作建议' : '查看针对性战术'"
>
  <Target v-if="!isAlly" />  <!-- 敌人：🎯 -->
  <Users v-else />            <!-- 队友：👥 -->
</button>

<!-- 第258-261行：获取建议数据 -->
const playerAdvice = computed(() => {
  return props.playerStats?.advice || [];
});

<!-- 第232-239行：弹窗组件 -->
<TacticalAdviceDialog
  v-model:open="showTacticalDialog"
  :player-name="player.displayName"
  :perspective="isAlly ? 'Collaboration' : 'Targeting'"
  :advice="playerAdvice"
/>
```

---

## 📊 类型定义

### Rust 类型

```rust
/// 游戏建议
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct GameAdvice {
    pub title: String,              // 建议标题
    pub problem: String,            // 问题描述
    pub evidence: String,           // 数据证据
    pub suggestions: Vec<String>,   // 具体建议列表（3-5条）
    pub priority: u8,               // 优先级（1-5）
    pub category: AdviceCategory,   // 分类
    pub perspective: AdvicePerspective,  // 视角
    pub affected_role: Option<String>,   // 影响位置
    pub target_player: Option<String>,   // 目标玩家
}

/// 建议分类
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub enum AdviceCategory {
    Laning,      // 对线期
    Farming,     // 发育
    Teamfight,   // 团战
    Vision,      // 视野
    Positioning, // 站位
    Decision,    // 决策
    Champion,    // 英雄
}

/// 建议视角
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub enum AdvicePerspective {
    SelfImprovement,  // 自我提升
    Targeting,        // 针对敌人
    Collaboration,    // 协作队友
}

/// 玩家战绩统计
pub struct PlayerMatchStats {
    // ... 基础统计字段
    pub advice: Vec<GameAdvice>,  // ⭐ v3.0 新增
}

/// 单场对局表现
pub struct MatchPerformance {
    // ... 基础字段
    pub role: String,      // ⭐ v3.1 新增：DUO_CARRY, SOLO, JUNGLE
    pub lane: String,      // ⭐ v3.1 新增：TOP, MIDDLE, BOTTOM
    pub position: String,  // ⭐ v3.1 新增：上单, 中单, 打野, ADC, 辅助
}
```

### TypeScript 类型（自动生成）

```typescript
// GameAdvice.ts
export type GameAdvice = {
  title: string;
  problem: string;
  evidence: string;
  suggestions: Array<string>;
  priority: number;
  category: AdviceCategory;
  perspective: AdvicePerspective;
  affectedRole: string | null;
  targetPlayer: string | null;
};

// AdviceCategory.ts
export type AdviceCategory =
  | "Laning"
  | "Farming"
  | "Teamfight"
  | "Vision"
  | "Positioning"
  | "Decision"
  | "Champion";

// AdvicePerspective.ts
export type AdvicePerspective =
  | "SelfImprovement"
  | "Targeting"
  | "Collaboration";

// PlayerMatchStats.ts
export type PlayerMatchStats = {
  // ... 其他字段
  advice?: Array<GameAdvice>;  // ⭐ 可选字段
};

// MatchPerformance.ts
export type MatchPerformance = {
  // ... 其他字段
  role: string;      // ⭐ v3.1 新增
  lane: string;      // ⭐ v3.1 新增
  position: string;  // ⭐ v3.1 新增
};
```

---

## 🎬 使用场景详解

### 场景 1：Dashboard 自我提升

**用户操作：**
1. 打开应用
2. 进入 Dashboard
3. 自动获取近 20 场排位战绩
4. 显示建议面板

**效果：**
```
💡 提升建议
基于你的近20场数据分析，帮助你变得更强

┌──────────────────────────────────────┐
│ 🗡️ 对线补刀能力待提升               │ 高优先级
│ 你的对线期平均落后15刀，经常被压制   │
│ 📊 对线期补刀效率偏低                │
│                                      │
│ 具体建议：                           │
│ ✅ 练习补刀基本功：训练模式练习      │
│ ✅ 改善对线站位：避免被频繁消耗      │
│ ✅ 优化技能释放：用技能补刀          │
└──────────────────────────────────────┘
```

**视角切换：**
- 用户可以点击切换按钮（自我提升/针对敌人/团队协作）
- 当前只有 SelfImprovement 有数据
- 其他视角会显示空状态

---

### 场景 2：选人阶段 - 针对敌人

**用户操作：**
1. 进入排位选人阶段
2. 系统自动获取敌方玩家战绩
3. 点击敌方玩家卡片上的 🎯 图标
4. 弹出战术建议对话框

**效果：**
```
🎯 针对 Enemy#1234 的战术建议
基于该玩家历史数据分析，识别弱点并制定针对性战术

┌──────────────────────────────────────┐
│ 🗡️ 软柿子：上单生存能力极差          │ 高优先级
│ 该玩家场均死亡8次，是团队最大弱点    │
│ 📊 该玩家频繁暴毙，是最容易击杀的目标│
│                                      │
│ 具体建议：                           │
│ ✅ 选择压制型英雄：对线强势的英雄    │
│ ✅ 打野优先级：前期重点照顾上路      │
│ ✅ 时机把握：3级/6级抓一波，滚雪球   │
│ ✅ 视野压制：反掉对方视野            │
└──────────────────────────────────────┘

影响位置：上单    目标：Enemy#1234
```

---

### 场景 3：选人阶段 - 协作队友

**用户操作：**
1. 进入排位选人阶段
2. 系统自动获取队友战绩
3. 点击队友卡片上的 👥 图标
4. 弹出协作建议对话框

**效果：**
```
🤝 协作 Teammate#5678 的建议
了解该队友的特点，优化团队配合策略

┌──────────────────────────────────────┐
│ 🛡️ 保护队友：打野生存能力弱          │ 高优先级
│ 该队友场均死亡7次，是团队的薄弱环节  │
│ 📊 该队友频繁阵亡，需要团队保护      │
│                                      │
│ 具体建议：                           │
│ ✅ 野区埋伏：在他刷野时提供视野      │
│ ✅ 控龙协助：打龙时帮他清视野        │
│ ✅ 路线提醒：看到对方入侵立即信号    │
│ ✅ 减少依赖：该打野容易死，别太依赖  │
│ ✅ 心态关键：多鼓励，别责怪          │
└──────────────────────────────────────┘

影响位置：打野    目标：Teammate#5678
```

---

## 🔍 调试和排查

### 后端日志

**启用日志：**
```rust
// 在 generate_advice() 中
println!("🎯 开始生成智能建议...");
println!("   视角：{}", perspective.description());
println!("   位置：{}", role);
println!("✅ 建议生成完成：共 {} 条", advice.len());
```

**关键日志输出：**
```
📊 开始分析对局列表数据 (使用优化架构: Parser + Strategy)
📊 找到 20 场对局记录
✅ Parser: 解析了 20 场对局数据
🎯 Strategy: 排位深度分析 (queueId=420)
✅ 时间线分析：识别对线期和发育曲线特征
💡 建议生成：共 3 条建议（视角：SelfImprovement）
✅ 分析完成 (Ranked):
   总对局=20, 胜场=12, 胜率=60.0%
   识别特征: 8个
   智能建议: 3条
```

### 位置识别日志

**如果位置显示"未知"：**
```
⚠️ 未识别的位置组合: role=XXX, lane=YYY
```

这个日志会告诉您具体是什么 role/lane 组合导致了"未知"。

**位置转换逻辑：**
```rust
// stats.rs::role_to_position()
("DUO_CARRY", _) → "ADC"
("DUO_SUPPORT", _) → "辅助"
("SOLO", "TOP") → "上单"
("SOLO", "MIDDLE") → "中单"
("NONE", "JUNGLE") → "打野"
("NONE", "TOP") → "上单"        // ⭐ 容错
("NONE", "MIDDLE") → "中单"     // ⭐ 容错
("NONE", "BOTTOM") → "下路"     // ⭐ 容错
("NONE", "NONE") → "灵活"       // ⭐ 大乱斗等
_ → "未知" + 日志输出            // ⭐ 调试
```

### 前端调试

**检查建议数据：**
```typescript
// Dashboard.vue
console.log('建议数据:', matchStatistics.value?.advice);
console.log('过滤后建议:', filteredAdvice.value);
console.log('当前视角:', selectedPerspective.value);

// CompactPlayerCard.vue
console.log('玩家建议:', playerAdvice.value);
console.log('玩家统计:', props.playerStats);
```

---

## 📝 配置和阈值

### 分析阈值

所有阈值在 `domains/analysis/thresholds.rs` 中定义：

```rust
// 对线期阈值
pub mod laning_phase {
    pub const CS_DIFF_DOMINATE: f64 = 15.0;      // 前10分钟领先15刀
    pub const CS_DIFF_ADVANTAGE: f64 = 8.0;      // 领先8刀
    pub const CS_DIFF_DISADVANTAGE: f64 = -8.0;  // 落后8刀
    pub const CS_DIFF_SUPPRESSED: f64 = -15.0;   // 落后15刀
}

// 团战阈值
pub mod teamfight {
    pub const KP_EXCELLENT: f64 = 0.70;  // 参团率70%
    pub const KP_GOOD: f64 = 0.60;
    pub const KP_POOR: f64 = 0.40;
}

// 死亡阈值
pub mod deaths {
    pub const HIGH_DEATH_RATE: f64 = 6.0;  // 场均死亡6次
}
```

### 建议优先级

| 优先级 | 值 | 含义 | 边框颜色 |
|--------|---|------|---------|
| **高** | 4-5 | 严重问题，急需改进 | 红色 |
| **中** | 2-3 | 需要注意的问题 | 蓝色 |
| **低** | 1 | 优化建议 | 灰色 |

---

## 🚀 扩展和定制

### 添加新的分析器

1. 创建新文件：`analyzers/positioning.rs`
2. 实现 `AdviceAnalyzer` trait
3. 在 `chain.rs` 中添加到责任链

```rust
// positioning.rs
pub struct PositioningAdviceAnalyzer;

impl AdviceAnalyzer for PositioningAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext, _strategy: &AnalysisStrategy) -> Option<GameAdvice> {
        // 检测站位问题
        // 调用 strategy.generate_advice(ProblemType::PoorPositioning, ...)
    }
}

// chain.rs
let chain = AdviceChain::new()
    .add_analyzer(Box::new(LaningAdviceAnalyzer))
    .add_analyzer(Box::new(FarmingAdviceAnalyzer))
    .add_analyzer(Box::new(TeamfightAdviceAnalyzer))
    .add_analyzer(Box::new(PositioningAdviceAnalyzer))  // ⭐ 新增
    .add_analyzer(Box::new(VisionAdviceAnalyzer))
    .add_analyzer(Box::new(ChampionAdviceAnalyzer));
```

### 添加新的问题类型

1. 在 `strategies/base.rs` 中添加枚举值：

```rust
pub enum ProblemType {
    // ... 现有问题
    NewProblemType,  // ⭐ 新增
}
```

2. 在三个策略文件中实现对应的建议生成方法

---

## 🎯 关键决策点

### Q1: 为什么只在排位模式生成建议？

```rust
if matches!(strategy, AnalysisStrategy::Ranked) {
    player_stats.advice = generate_advice(...);
}
```

**原因：**
- 建议生成需要深度分析（时间线、位置、趋势等）
- 大乱斗、自定义等模式缺少这些数据
- 非排位模式玩家可能不需要这么详细的建议

### Q2: 为什么自己始终是 SelfImprovement 视角？

```rust
let final_perspective = if is_self {
    AdvicePerspective::SelfImprovement  // 自己永远是自我提升
} else {
    persp  // 队友或敌人使用传入的视角
};
```

**原因：**
- 自己看自己的数据，目的是提升
- 即使在队伍分析中，自己也应该看到自我提升建议
- 保持逻辑一致性

### Q3: 为什么建议数量限制为 5 条？

```rust
advice_list.truncate(5);
```

**原因：**
- 避免信息过载
- 突出重点问题
- 优先级排序后，前 5 条已经是最重要的

### Q4: 位置为什么会显示"灵活"或"未知"？

**"灵活"：**
- `role = "NONE", lane = "NONE"`
- 通常出现在：大乱斗、极地大乱斗、自定义游戏
- 这些模式本来就没有固定位置

**"未知"：**
- 不匹配任何已知的 (role, lane) 组合
- 会输出调试日志：`⚠️ 未识别的位置组合: role=XXX, lane=YYY`
- 需要根据日志添加新的匹配规则

---

## 🎨 UI 组件样式说明

### AdviceCard 视觉设计

```
┌────────────────────────────────────────────────┐
│ ┃ [图标] 标题                         [优先级] │ ← 边框颜色根据优先级
│ ┃                                              │
│ ┃ 问题描述                                     │
│ ┃ 📊 数据证据                                  │
│ ┃                                              │
│ ┃ ✓ 具体建议：                                 │
│ ┃   ✅ 建议1                                   │
│ ┃   ✅ 建议2                                   │
│ ┃   ✅ 建议3                                   │
│ ┃                                              │
│ ┃ ─────────────────────────────────────────   │
│ ┃ 🎯 影响位置：上单  👤 目标：XXX             │
└────────────────────────────────────────────────┘
```

**颜色方案：**

| 元素 | 亮色模式 | 暗色模式 |
|------|---------|---------|
| 边框（高优先级） | `border-l-red-500` | `border-l-red-400` |
| 边框（中优先级） | `border-l-blue-500` | `border-l-blue-400` |
| 边框（低优先级） | `border-l-gray-400` | `border-l-gray-600` |
| 分类图标背景 | `bg-xxx-100` | `bg-xxx-950` |
| 分类图标颜色 | `text-xxx-600` | `text-xxx-400` |
| 建议列表图标 | `text-green-600` | `text-green-400` |
| 文字 | `text-foreground` | `text-foreground` |
| 次要文字 | `text-muted-foreground` | `text-muted-foreground` |

---

## 🧪 测试检查清单

### 后端测试

- [ ] 编译通过：`cd src-tauri && cargo build`
- [ ] 测试通过：`cargo test --lib`
- [ ] 类型生成：`pnpm types`（85 个测试）

### 前端测试

**Dashboard 测试：**
- [ ] 打开 Dashboard
- [ ] 查看是否有建议面板
- [ ] 建议卡片显示正常
- [ ] 优先级边框颜色正确
- [ ] 深色模式颜色适配

**对局分析测试：**
- [ ] 进入排位选人阶段
- [ ] 敌方玩家卡片显示 🎯 按钮
- [ ] 队友玩家卡片显示 👥 按钮
- [ ] 点击按钮弹出对话框
- [ ] 对话框宽度合适（max-w-4xl）
- [ ] 建议内容显示正确
- [ ] 措辞符合视角（"对手"/"队友"）
- [ ] 深色模式适配

**位置信息测试：**
- [ ] 对局历史显示位置（上单/中单/打野/ADC/辅助/灵活）
- [ ] 排位模式显示准确位置
- [ ] 大乱斗等模式显示"灵活"

---

## 📦 完整功能清单

### ✅ 已实现功能

- [x] Parser 模式：LCU API 数据解析
- [x] Strategy 模式：分析深度选择（排位/其他）
- [x] 时间线分析：对线期、发育曲线、经验优势
- [x] 三种建议视角：SelfImprovement, Targeting, Collaboration
- [x] 五个分析器：Laning, Farming, Teamfight, Vision, Champion
- [x] 位置识别：每场对局的 role/lane/position
- [x] Dashboard 集成：自我提升建议面板
- [x] 对局分析集成：针对敌人、协作队友
- [x] UI 组件：AdvicePanel, AdviceCard, TacticalAdviceDialog
- [x] 深浅色主题适配
- [x] TypeScript 类型生成
- [x] 数据缓存优化

### 🔮 未来扩展（可选）

- [ ] Positioning 分析器（站位问题详细分析）
- [ ] Decision 分析器（决策失误分析）
- [ ] 更多位置特化建议（针对不同英雄类型）
- [ ] 建议优先级算法优化
- [ ] 阈值动态调整（根据段位）
- [ ] 建议历史记录
- [ ] 建议有效性反馈

---

## 💡 最佳实践

### 1. 修改建议内容

建议内容在三个策略文件中：
- `strategies/self_improvement.rs`
- `strategies/targeting.rs`
- `strategies/collaboration.rs`

**示例：修改死亡过多的建议**
```rust
// self_improvement.rs
fn create_high_death_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
    let role_advice = match data.role.as_str() {
        "ADC" => vec![
            "🛡️ 你的新建议1",  // ⭐ 修改这里
            "⚡ 你的新建议2",
            "💰 你的新建议3",
        ],
        // ...
    };
    // ...
}
```

### 2. 调整阈值

阈值在 `domains/analysis/thresholds.rs` 中：

```rust
pub mod deaths {
    pub const HIGH_DEATH_RATE: f64 = 6.0;  // ⭐ 调整这里
}
```

### 3. 添加新的位置匹配规则

在 `stats.rs::role_to_position()` 中添加：

```rust
fn role_to_position(role: &str, lane: &str) -> String {
    match (role, lane) {
        // ... 现有规则
        ("NEW_ROLE", "NEW_LANE") => "新位置",  // ⭐ 添加新规则
        _ => "未知"
    }
}
```

### 4. 修改建议数量限制

在 `tactical_advice/chain.rs` 中：

```rust
pub fn generate(...) -> Vec<GameAdvice> {
    // ...
    advice_list.truncate(5);  // ⭐ 修改这里（改为 10、3 等）
    advice_list
}
```

---

## 🎓 技术要点

### 1. 为什么使用 Option<TimelineData>？

```rust
pub struct ParsedPlayerData {
    pub timeline_data: Option<TimelineData>,  // ⭐ 可选
}
```

**原因：**
- 部分模式没有 timeline 数据（大乱斗、自定义）
- 避免解析失败导致整个对局数据丢失
- 使用 `if let Some(timeline) = ...` 安全处理

### 2. 为什么 advice 字段是可选的？

```rust
pub struct PlayerMatchStats {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub advice: Vec<GameAdvice>,  // ⭐ 序列化时跳过空数组
}
```

**原因：**
- 非排位模式不生成建议，`advice` 为空
- 减少 JSON 大小
- 前端使用 `advice?.length` 安全判断

### 3. 为什么需要 perspective 参数？

```rust
pub fn generate_advice(
    stats: &PlayerMatchStats,
    games: &[ParsedGame],
    role: &str,
    perspective: AdvicePerspective,  // ⭐ 视角参数
    target_name: Option<String>,
    strategy: &AnalysisStrategy,
) -> Vec<GameAdvice>
```

**原因：**
- 同样的问题（如"死亡多"），不同视角有不同建议：
  - SelfImprovement: "你要改进站位..."
  - Targeting: "对手容易死，重点针对..."
  - Collaboration: "队友容易死，要保护他..."

---

## 📈 性能优化

### 1. 数据缓存

```rust
// analysis_data/service.rs
match_stats_cache: &mut HashMap<String, PlayerMatchStats>
```

**优点：**
- 同一玩家不重复请求 LCU API
- 选人阶段频繁更新时避免重复计算
- 缓存包含已生成的建议

### 2. 责任链短路

```rust
// analyzers/laning.rs
if context.stats.avg_deaths <= thresholds::deaths::HIGH_DEATH_RATE {
    return None;  // ⭐ 没问题就跳过，不生成建议
}
```

**优点：**
- 不会为所有分析器都生成建议
- 只针对真正的问题生成建议
- 减少不必要的计算

### 3. 建议数量限制

```rust
advice_list.sort_by(|a, b| b.priority.cmp(&a.priority));
advice_list.truncate(5);  // ⭐ 最多5条
```

**优点：**
- 避免建议过多造成信息过载
- 突出重点问题
- 提升用户体验

---

## 🐛 常见问题排查

### 问题 1：Dashboard 没有显示建议

**可能原因：**
1. 对局数量不足（小于 5 场）
2. 不是排位模式
3. 没有识别到明显问题

**排查：**
```rust
// 检查日志
💡 建议生成：共 0 条建议  // ← 如果是 0，说明没有问题
⏭️  建议系统：非排位模式，跳过  // ← 非排位模式不生成
```

### 问题 2：位置显示"未知"

**可能原因：**
- LCU API 返回的 role/lane 组合不在匹配规则中

**排查：**
```rust
// 查看日志
⚠️ 未识别的位置组合: role=XXX, lane=YYY
```

**解决：**
在 `role_to_position()` 中添加对应的匹配规则

### 问题 3：建议措辞不对（应该是"对手"却显示"你"）

**可能原因：**
- 传入的 `perspective` 参数错误

**排查：**
```rust
// 检查日志
🎯 开始生成智能建议...
   视角：个人改进建议  // ← 检查这里是否正确
```

**解决：**
检查调用时传入的 `AdvicePerspective` 是否正确

---

## 🎉 总结

智能建议系统是一个**完整的、可扩展的、多视角的游戏分析系统**：

- **后端架构清晰**：Parser → Strategy → Analyzer → Advice
- **设计模式合理**：6 种设计模式协同工作
- **三种视角完整**：自我提升、针对敌人、协作队友
- **数据流向明确**：LCU API → 解析 → 分析 → 建议 → 前端
- **UI 组件完善**：Dashboard + 对局分析完整集成
- **深浅色适配**：所有颜色都响应主题变化
- **位置信息完整**：每场对局都有位置数据

现在您可以：
1. 启动应用测试所有功能
2. 根据需要调整阈值和建议内容
3. 扩展新的分析器和问题类型
4. 优化 UI 细节

---

**文档版本：** v3.1
**更新日期：** 2025-10-20
**作者：** AI Assistant
**项目：** Nidalee 英雄联盟助手

