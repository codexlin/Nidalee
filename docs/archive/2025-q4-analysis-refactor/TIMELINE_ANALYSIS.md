# 时间线分析设计方案

## 📊 **当前状况**

### ❌ **未使用的时间线数据**

根据 `api.md` (line 178-217)，LCU API 提供了丰富的**分阶段数据**：

```json
"timeline": {
  "role": "string",              // ✅ 已使用
  "lane": "string",              // ✅ 已使用

  // ❌ 以下数据完全未使用：
  "creepsPerMinDeltas": {        // 每分钟补刀（分阶段）
    "0-10": 8.5,                 // 前10分钟
    "10-20": 7.2,                // 10-20分钟
    "20-30": 6.8                 // 20-30分钟
  },
  "goldPerMinDeltas": {          // 每分钟金币（分阶段）
    "0-10": 420.5,
    "10-20": 380.2,
    "20-30": 350.8
  },
  "xpPerMinDeltas": {            // 每分钟经验（分阶段）
    "0-10": 550.3,
    "10-20": 480.1
  },
  "csDiffPerMinDeltas": {        // 补刀差（分阶段）
    "0-10": +12.5,               // 领先12.5刀
    "10-20": -5.3                // 落后5.3刀
  },
  "xpDiffPerMinDeltas": {        // 经验差（分阶段）
    "0-10": +200.5,
    "10-20": -150.2
  },
  "damageTakenPerMinDeltas": {   // 每分钟承伤（分阶段）
    "0-10": 280.5,
    "10-20": 320.8
  }
}
```

---

## 🎯 **时间线分析的价值**

### **1. 游戏阶段表现分析**

```
对线期 (0-10分钟)
├─ CS/min：判断对线压制力
├─ 金币/min：经济发育能力
├─ CS差：是否压制对手
├─ 经验差：是否取得等级优势
└─ 承伤：判断是否激进/被压制

中期 (10-20分钟)
├─ CS维持能力
├─ 经济转化效率
├─ 游走/参团影响
└─ 发育稳定性

后期 (20分钟+)
├─ 团战贡献
├─ 资源控制
└─ 团队配合
```

### **2. 特征识别潜力**

基于时间线数据，可以识别以下特征：

| 特征 | 判断标准 | 价值 |
|------|---------|------|
| **对线压制** | 前10分钟 CS差 > +15 | 识别线霸型选手 |
| **稳健发育** | 前10分钟 CS/min > 8.0，CS差 > -5 | 识别稳定型选手 |
| **被压制** | 前10分钟 CS差 < -20 | 识别对线劣势 |
| **爆发成长** | 中期金币/分 > 对线期 | 识别游走支援型 |
| **后期乏力** | 后期数据下降 | 识别耐力不足 |
| **早期激进** | 前10分钟承伤高 | 识别打法风格 |

---

## 🛠️ **实现方案**

### **第1步：扩展 ParsedGame 结构**

```rust
// parser.rs

/// 时间线数据（分阶段）
#[derive(Debug, Clone, Default)]
pub struct TimelineData {
    // 补刀数据（每分钟）
    pub cs_per_min_0_10: Option<f64>,
    pub cs_per_min_10_20: Option<f64>,
    pub cs_per_min_20_end: Option<f64>,

    // 金币数据（每分钟）
    pub gold_per_min_0_10: Option<f64>,
    pub gold_per_min_10_20: Option<f64>,
    pub gold_per_min_20_end: Option<f64>,

    // 经验数据（每分钟）
    pub xp_per_min_0_10: Option<f64>,
    pub xp_per_min_10_20: Option<f64>,

    // 补刀差（对手对比）
    pub cs_diff_0_10: Option<f64>,
    pub cs_diff_10_20: Option<f64>,
    pub cs_diff_20_end: Option<f64>,

    // 经验差（对手对比）
    pub xp_diff_0_10: Option<f64>,
    pub xp_diff_10_20: Option<f64>,

    // 承伤数据（每分钟）
    pub damage_taken_per_min_0_10: Option<f64>,
    pub damage_taken_per_min_10_20: Option<f64>,
}

pub struct ParsedPlayerData {
    // ... 现有字段 ...
    pub timeline_data: Option<TimelineData>,  // 新增
}
```

---

### **第2步：解析时间线数据**

```rust
// parser.rs

fn parse_timeline_data(timeline: &Value) -> Option<TimelineData> {
    let mut data = TimelineData::default();

    // 解析 creepsPerMinDeltas
    if let Some(cs_deltas) = timeline.get("creepsPerMinDeltas") {
        data.cs_per_min_0_10 = parse_delta_value(cs_deltas, "0-10");
        data.cs_per_min_10_20 = parse_delta_value(cs_deltas, "10-20");
        data.cs_per_min_20_end = parse_delta_value(cs_deltas, "20-30")
            .or_else(|| parse_delta_value(cs_deltas, "20-end"));
    }

    // 解析 goldPerMinDeltas
    if let Some(gold_deltas) = timeline.get("goldPerMinDeltas") {
        data.gold_per_min_0_10 = parse_delta_value(gold_deltas, "0-10");
        data.gold_per_min_10_20 = parse_delta_value(gold_deltas, "10-20");
        data.gold_per_min_20_end = parse_delta_value(gold_deltas, "20-30")
            .or_else(|| parse_delta_value(gold_deltas, "20-end"));
    }

    // 解析 xpPerMinDeltas
    if let Some(xp_deltas) = timeline.get("xpPerMinDeltas") {
        data.xp_per_min_0_10 = parse_delta_value(xp_deltas, "0-10");
        data.xp_per_min_10_20 = parse_delta_value(xp_deltas, "10-20");
    }

    // 解析 csDiffPerMinDeltas（关键！）
    if let Some(cs_diff) = timeline.get("csDiffPerMinDeltas") {
        data.cs_diff_0_10 = parse_delta_value(cs_diff, "0-10");
        data.cs_diff_10_20 = parse_delta_value(cs_diff, "10-20");
        data.cs_diff_20_end = parse_delta_value(cs_diff, "20-30")
            .or_else(|| parse_delta_value(cs_diff, "20-end"));
    }

    // 解析 xpDiffPerMinDeltas（关键！）
    if let Some(xp_diff) = timeline.get("xpDiffPerMinDeltas") {
        data.xp_diff_0_10 = parse_delta_value(xp_diff, "0-10");
        data.xp_diff_10_20 = parse_delta_value(xp_diff, "10-20");
    }

    // 解析 damageTakenPerMinDeltas
    if let Some(dmg_taken) = timeline.get("damageTakenPerMinDeltas") {
        data.damage_taken_per_min_0_10 = parse_delta_value(dmg_taken, "0-10");
        data.damage_taken_per_min_10_20 = parse_delta_value(dmg_taken, "10-20");
    }

    Some(data)
}

// 辅助函数：解析 delta 值
fn parse_delta_value(deltas: &Value, key: &str) -> Option<f64> {
    deltas.get(key)?.as_f64()
}
```

---

### **第3步：创建时间线分析器**

```rust
// timeline_analyzer.rs

use crate::lcu::types::SummonerTrait;
use super::parser::{ParsedGame, TimelineData};
use super::thresholds;

/// 时间线特征分析
pub fn analyze_timeline_traits(games: &[ParsedGame], role: &str) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 1. 对线期分析（前10分钟）
    traits.extend(analyze_laning_phase(games, role));

    // 2. 发育曲线分析
    traits.extend(analyze_growth_curve(games));

    // 3. 阶段强势分析
    traits.extend(analyze_phase_strength(games));

    traits
}

/// 对线期分析（0-10分钟）
fn analyze_laning_phase(games: &[ParsedGame], role: &str) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut total_cs_diff = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let Some(cs_diff) = timeline.cs_diff_0_10 {
                total_cs_diff += cs_diff;
                valid_games += 1;
            }
        }
    }

    if valid_games == 0 {
        return traits;
    }

    let avg_cs_diff = total_cs_diff / valid_games as f64;

    // 对线压制
    if avg_cs_diff >= 15.0 {
        traits.push(SummonerTrait {
            name: "对线压制".to_string(),
            description: format!(
                "前10分钟平均领先{:.1}刀，对线压制力强",
                avg_cs_diff
            ),
            score: avg_cs_diff as i32,
            trait_type: "good".to_string(),
        });
    }
    // 稳健对线
    else if avg_cs_diff >= -5.0 && avg_cs_diff <= 5.0 {
        traits.push(SummonerTrait {
            name: "稳健对线".to_string(),
            description: format!(
                "前10分钟补刀均势（{:+.1}），对线稳健",
                avg_cs_diff
            ),
            score: 50,
            trait_type: "good".to_string(),
        });
    }
    // 对线劣势
    else if avg_cs_diff <= -15.0 {
        traits.push(SummonerTrait {
            name: "对线弱势".to_string(),
            description: format!(
                "前10分钟平均落后{:.1}刀，对线承压",
                -avg_cs_diff
            ),
            score: (-avg_cs_diff) as i32,
            trait_type: "bad".to_string(),
        });
    }

    traits
}

/// 发育曲线分析
fn analyze_growth_curve(games: &[ParsedGame]) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut early_gold = 0.0;
    let mut mid_gold = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let (Some(early), Some(mid)) = (
                timeline.gold_per_min_0_10,
                timeline.gold_per_min_10_20,
            ) {
                early_gold += early;
                mid_gold += mid;
                valid_games += 1;
            }
        }
    }

    if valid_games == 0 {
        return traits;
    }

    let avg_early = early_gold / valid_games as f64;
    let avg_mid = mid_gold / valid_games as f64;

    // 爆发成长
    if avg_mid > avg_early * 1.15 {
        traits.push(SummonerTrait {
            name: "爆发成长".to_string(),
            description: format!(
                "中期经济效率提升{:.0}%，游走支援能力强",
                ((avg_mid / avg_early - 1.0) * 100.0)
            ),
            score: 70,
            trait_type: "good".to_string(),
        });
    }
    // 稳定发育
    else if avg_early >= 400.0 && avg_mid >= 380.0 {
        traits.push(SummonerTrait {
            name: "稳定发育".to_string(),
            description: format!(
                "各阶段经济稳定（{:.0}/{:.0}），发育能力强",
                avg_early, avg_mid
            ),
            score: 65,
            trait_type: "good".to_string(),
        });
    }

    traits
}

/// 阶段强势分析
fn analyze_phase_strength(games: &[ParsedGame]) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut early_xp_diff = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let Some(xp_diff) = timeline.xp_diff_0_10 {
                early_xp_diff += xp_diff;
                valid_games += 1;
            }
        }
    }

    if valid_games == 0 {
        return traits;
    }

    let avg_xp_diff = early_xp_diff / valid_games as f64;

    // 等级优势
    if avg_xp_diff >= 300.0 {
        traits.push(SummonerTrait {
            name: "等级优势".to_string(),
            description: format!(
                "前10分钟平均经验领先{:.0}，抢等级能力强",
                avg_xp_diff
            ),
            score: 60,
            trait_type: "good".to_string(),
        });
    }

    traits
}
```

---

### **第4步：添加时间线阈值**

```rust
// thresholds.rs

/// 时间线阈值（对线期）
pub mod laning_phase {
    pub const CS_DIFF_DOMINATE: f64 = 15.0;   // 压制级CS差
    pub const CS_DIFF_ADVANTAGE: f64 = 8.0;   // 优势级CS差
    pub const CS_DIFF_NEUTRAL: f64 = 5.0;     // 均势CS差
    pub const CS_DIFF_BEHIND: f64 = -15.0;    // 劣势CS差

    pub const XP_DIFF_ADVANTAGE: f64 = 300.0; // 经验优势

    pub const GOLD_PER_MIN_EXCELLENT: f64 = 450.0;
    pub const GOLD_PER_MIN_GOOD: f64 = 400.0;
}

/// 发育曲线阈值
pub mod growth {
    pub const MID_GAME_BOOST: f64 = 1.15;  // 中期经济提升15%
    pub const LATE_GAME_BOOST: f64 = 1.10; // 后期经济提升10%
}
```

---

### **第5步：集成到分析流程**

```rust
// matches/service.rs

fn analyze_match_list_data(...) -> Result<PlayerMatchStats, String> {
    // ... 现有代码 ...

    // === 第4步: 多层次特征分析 ===
    let mut traits = Vec::new();

    traits.extend(analyze_traits(&player_stats));

    if strategy.enable_advanced_analysis() {
        traits.extend(analyze_advanced_traits(&player_stats, games, current_puuid));

        // 🆕 添加时间线分析（仅排位模式）
        traits.extend(analyze_timeline_traits(&parsed_games, &main_role));
    }

    // ... 其余代码 ...
}
```

---

## 📊 **预期效果示例**

### **排位模式 - 完整时间线分析**

```
用户：ADC 选手，20场排位数据

时间线特征（新增）：
✨ 对线压制：前10分钟平均领先18.5刀，对线压制力强
✨ 稳定发育：各阶段经济稳定（435/398），发育能力强
✨ 等级优势：前10分钟平均经验领先420，抢等级能力强

原有特征：
- 大神（胜率68%）
- 大爹（KDA 4.8）
- 团战核心（KP 75%）
- ADC专精（ADC胜率70%）
... 等
```

---

## 🎯 **实现优先级建议**

### **阶段 1：基础时间线分析** 🔴 高优先级

```
实现内容：
✅ 解析 csDiffPerMinDeltas（补刀差）
✅ 解析 goldPerMinDeltas（金币/分）
✅ 分析对线期表现（0-10分钟）
✅ 识别"对线压制"/"对线弱势"特征

工作量：~2-3小时
价值：⭐⭐⭐⭐⭐ 非常高
```

### **阶段 2：发育曲线分析** 🟡 中优先级

```
实现内容：
✅ 对比各阶段经济效率
✅ 识别"爆发成长"/"稳定发育"特征
✅ 分析打法风格（对线型/游走型）

工作量：~1-2小时
价值：⭐⭐⭐⭐ 高
```

### **阶段 3：高级时间线分析** 🔵 低优先级

```
实现内容：
✅ 经验差分析
✅ 承伤模式分析
✅ 打法激进度评估
✅ 资源控制能力

工作量：~2-3小时
价值：⭐⭐⭐ 中等
```

---

## 🔄 **与现有系统的整合**

### **数据流**

```
LCU API Timeline 数据
    ↓
Parser: parse_timeline_data()
    ↓
ParsedGame.player_data.timeline_data
    ↓
timeline_analyzer.rs
    ↓
analyze_timeline_traits()
    ↓
新特征：对线压制、爆发成长、稳定发育 等
    ↓
合并到 traits
    ↓
Merger: optimize_traits()
    ↓
最终 12个精炼特征
```

---

## 📝 **总结**

### **当前状况**
- ❌ 只用了 `timeline.role` 和 `timeline.lane`
- ❌ 完全忽略了 7 种时间线数据
- ❌ 无法分析对线期/发育曲线/阶段强势

### **时间线分析的价值**
- ✅ 识别对线压制力
- ✅ 评估发育稳定性
- ✅ 判断打法风格（激进/稳健）
- ✅ 分析游戏阶段强势
- ✅ 提供更深入的洞察

### **实现成本**
- **工作量**：阶段1（基础）约 2-3 小时
- **复杂度**：中等（主要是数据解析）
- **收益**：⭐⭐⭐⭐⭐ 非常高

### **建议**
**强烈建议实现阶段1的基础时间线分析**，这将大幅提升分析的深度和价值！🚀

---

*时间线分析设计 - 2025-10-17*
*状态: 📋 待实现*

