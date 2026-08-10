# 多位置分组分析系统

## 📋 概述

多位置分组分析系统是对原有战绩分析的重大升级，解决了将不同位置的对局数据混合分析导致的建议不准确问题。

### ❌ 原有问题

之前的分析系统会将玩家所有对局（无论位置）混合在一起计算平均值：
- 打野的10场对局 + 中单的5场对局 + 辅助的3场对局 → 混合平均
- 导致建议针对性差，甚至出现"未知位置"的建议
- 无法准确反映玩家在特定位置的真实水平

### ✅ 新系统优势

1. **按位置独立分析**：每个位置的数据单独计算
2. **精准建议**：基于特定位置的表现生成针对性建议
3. **位置识别优化**：
   - 排位赛（420/440）：使用API返回的role和lane精确识别
   - 其他模式：标记为"灵活"位置
4. **主位置识别**：自动识别场次最多的位置作为主要位置

## 🏗️ 系统架构

### 数据结构

```rust
/// 多位置分组分析结果
pub struct MultiPositionAnalysis {
    /// 所有位置的统计（按场次从多到少排序）
    pub position_stats: Vec<PositionStats>,

    /// 主要位置（场次最多的）
    pub main_position: String,

    /// 总览数据（所有位置合计）
    pub overall_stats: PlayerMatchStats,
}

/// 单个位置的统计数据
pub struct PositionStats {
    /// 位置名称（打野、上单、中单、ADC、辅助、灵活）
    pub position: String,

    /// 该位置的场次
    pub games: u32,

    /// 该位置的胜场
    pub wins: u32,

    /// 该位置的胜率
    pub win_rate: f64,

    /// 该位置的统计数据（包含KDA、建议等）
    pub stats: PlayerMatchStats,
}
```

### 核心模块

1. **位置识别** (`src/domains/analysis/analyzers/core/parser.rs`)
   - `identify_position_from_game`: 统一的位置识别逻辑
   - 支持排位/非排位模式的不同识别策略

2. **分组分析** (`src/infrastructure/match_management/matches/position_analysis.rs`)
   - `analyze_with_position_grouping`: 核心分组分析函数
   - 按位置分组 → 独立分析 → 生成建议

3. **API命令** (`src/infrastructure/match_management/matches/commands_v2.rs`)
   - `get_match_history_with_positions`: 新API命令
   - 返回完整的多位置分析结果

## 🔧 使用方法

### 后端调用

```rust
use crate::infrastructure::match_management::matches::commands_v2::get_match_history_with_positions;

// 获取最近20场排位赛的多位置分析
let result = get_match_history_with_positions(Some(20), Some(420)).await?;

println!("主要位置: {}", result.main_position);
println!("位置数量: {}", result.position_stats.len());

for pos_stat in result.position_stats {
    println!("{}: {}场 (胜率{:.1}%)",
        pos_stat.position,
        pos_stat.games,
        pos_stat.win_rate
    );
}
```

### 前端调用

```typescript
import { invoke } from '@tauri-apps/api/core'

// 获取多位置分析
const result = await invoke<MultiPositionAnalysis>('get_match_history_with_positions', {
  count: 20,
  queueId: 420  // 420=单排, 440=灵活组排, null=所有模式
})

console.log('主要位置:', result.mainPosition)
console.log('位置统计:', result.positionStats)
console.log('总览:', result.overallStats)
```

### Vue组件集成

```vue
<template>
  <PositionStatsCard
    :position-stats="positionAnalysis.positionStats"
    :main-position="positionAnalysis.mainPosition"
    @view-details="handlePositionDetails"
  />
</template>

<script setup lang="ts">
const { positionAnalysis, fetchPositionAnalysis } = usePositionAnalysis()

// 加载排位赛数据
await fetchPositionAnalysis(20, 420)
</script>
```

## 📊 位置识别逻辑

### 排位赛模式 (Queue ID: 420, 440)

| role | lane | 识别结果 |
|------|------|---------|
| TOP | TOP | 上单 |
| JUNGLE | JUNGLE | 打野 |
| MIDDLE | MIDDLE | 中单 |
| BOTTOM | BOTTOM | ADC |
| DUO_SUPPORT | BOTTOM | 辅助 |
| SUPPORT | BOTTOM | 辅助 |
| SOLO | NONE | 未知 (罕见) |

### 非排位模式 (其他Queue ID)

所有对局统一标记为 **"灵活"** 位置，因为匹配、大乱斗等模式不强制位置分配。

## 🎯 建议生成策略

### 排位赛

- ✅ 为每个位置单独生成建议
- ✅ 建议包含具体位置信息
- ❌ 跳过"未知"位置（数据不可靠）

### 非排位赛

- ✅ 为"灵活"位置生成基础建议
- ✅ 简化分析，不深入位置特定细节

## 🔄 向后兼容

原有的 `get_match_history` 命令仍然可用，返回 `PlayerMatchStats`（总览数据）。

```rust
// 旧API（兼容）
pub async fn get_match_history(count: Option<u32>, queue_id: Option<i32>)
    -> Result<PlayerMatchStats, String>

// 新API（多位置）
pub async fn get_match_history_with_positions(count: Option<u32>, queue_id: Option<i32>)
    -> Result<MultiPositionAnalysis, String>
```

新系统内部仍会调用多位置分析，然后返回 `overall_stats` 以保持兼容。

## 📝 实现细节

### 分组流程

```
1. 解析对局数据 (parse_games)
   ↓
2. 按 queue_id 过滤
   ↓
3. 确定分析策略 (Ranked / Other)
   ↓
4. 按位置分组 (identify_position_from_game)
   ↓
5. 为每个位置独立分析
   - 计算统计数据
   - 生成建议（仅排位赛）
   ↓
6. 按场次排序
   ↓
7. 返回结果
```

### 关键函数

#### `identify_position_from_game`

```rust
pub fn identify_position_from_game(role: &str, lane: &str, queue_id: i64) -> String {
    // 排位赛：精确识别
    if queue_id == 420 || queue_id == 440 {
        match (role, lane) {
            ("TOP", _) => "上单",
            ("JUNGLE", _) => "打野",
            ("MIDDLE", _) => "中单",
            ("BOTTOM", _) | ("DUO_CARRY", _) => "ADC",
            ("DUO_SUPPORT", _) | ("SUPPORT", _) => "辅助",
            _ => "未知"
        }
    }
    // 非排位：灵活
    else {
        "灵活"
    }
}
```

#### `analyze_with_position_grouping`

```rust
pub fn analyze_with_position_grouping(
    match_list_data: Value,
    current_puuid: &str,
    queue_id: Option<i32>,
) -> Result<MultiPositionAnalysis, String> {
    // 1. 解析对局
    let parsed_games = parse_games(games, current_puuid);

    // 2. 过滤
    let filtered_games = filter_by_queue_id(parsed_games, queue_id);

    // 3. 分组
    let position_groups = group_by_position(filtered_games);

    // 4. 分析每个位置
    let position_stats = position_groups.map(|(pos, games)| {
        let stats = analyze_player_stats(games, current_puuid, context);
        let advice = generate_advice(&stats, games, pos, perspective, strategy);
        PositionStats { position: pos, games, wins, win_rate, stats }
    });

    // 5. 确定主位置
    let main_position = position_stats.first().position;

    // 6. 计算总览
    let overall_stats = analyze_player_stats(&filtered_games, current_puuid, context);

    Ok(MultiPositionAnalysis { position_stats, main_position, overall_stats })
}
```

## 🧪 测试

### 单元测试

```bash
cd src-tauri
cargo test position_analysis
```

### 集成测试

1. 启动应用
2. 进入"战绩查询"页面
3. 查询召唤师
4. 切换到"位置分组"标签
5. 验证：
   - 位置正确识别
   - 数据准确分组
   - 建议针对性强

## 🔮 未来优化

1. **位置推荐**：根据胜率推荐最佳位置
2. **位置对比**：横向对比不同位置的表现
3. **历史趋势**：追踪位置胜率变化
4. **英雄池分析**：每个位置的英雄使用情况
5. **图表可视化**：echarts雷达图展示能力维度

## 📚 相关文档

- [算法架构](./ARCHITECTURE.md)
- [战术建议流程](./TACTICAL_ADVICE_FLOW.md)
- [时间线分析](./TIMELINE_ANALYSIS.md)
