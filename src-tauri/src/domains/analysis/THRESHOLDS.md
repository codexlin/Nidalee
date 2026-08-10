# 阈值配置使用指南

## 📚 **如何在分析器中使用 thresholds.rs**

### **基本用法**

#### **1. 导入 thresholds 模块**

```rust
use super::thresholds;
```

#### **2. 使用阈值常量**

```rust
// 胜率判断
if stats.win_rate >= thresholds::win_rate::EXCELLENT_RANKED {
    // 排位大神（≥65%）
}

// KDA判断
if stats.avg_kda >= thresholds::kda::EXCELLENT_RANKED {
    // KDA优秀（≥4.0）
}

// 参团率判断
if avg_kp >= thresholds::kill_participation::HIGH {
    // 参团率高（≥70%）
}
```

---

## 📋 **可用的阈值模块**

### **1. 胜率阈值（win_rate）**

```rust
use super::thresholds;

// 可用常量：
thresholds::win_rate::EXCELLENT_RANKED  // 65.0  排位大神标准
thresholds::win_rate::EXCELLENT_OTHER   // 60.0  其他模式大神标准
thresholds::win_rate::GOOD              // 55.0  良好胜率
thresholds::win_rate::POOR              // 45.0  较差胜率
```

**使用示例**：
```rust
if stats.win_rate >= thresholds::win_rate::EXCELLENT_RANKED {
    traits.push(SummonerTrait {
        name: "大神".to_string(),
        description: format!("胜率超高的实力玩家（{}%）", stats.win_rate as i32),
        score: stats.win_rate as i32,
        trait_type: "good".to_string(),
    });
}
```

---

### **2. KDA阈值（kda）**

```rust
thresholds::kda::EXCELLENT_RANKED  // 4.0  排位优秀
thresholds::kda::EXCELLENT_OTHER   // 3.5  其他模式优秀
thresholds::kda::GOOD              // 2.5  良好KDA
thresholds::kda::POOR              // 1.5  较差KDA
```

**使用示例**：
```rust
if stats.avg_kda >= thresholds::kda::EXCELLENT_RANKED {
    traits.push(SummonerTrait {
        name: "大爹".to_string(),
        description: format!("KDA超高的carry玩家（{:.1}）", stats.avg_kda),
        score: (stats.avg_kda * 10.0) as i32,
        trait_type: "good".to_string(),
    });
}
```

---

### **3. 参团率阈值（kill_participation）**

```rust
thresholds::kill_participation::HIGH  // 0.70  高参团率
thresholds::kill_participation::LOW   // 0.40  低参团率
```

**使用示例**：
```rust
if avg_kp >= thresholds::kill_participation::HIGH {
    Some(SummonerTrait {
        name: "团战核心".to_string(),
        description: format!("参与了{:.0}%的击杀，团战参与度极高", avg_kp * 100.0),
        score: (avg_kp * 100.0) as i32,
        trait_type: "good".to_string(),
    })
} else if avg_kp <= thresholds::kill_participation::LOW {
    Some(SummonerTrait {
        name: "游离".to_string(),
        description: format!("参团率仅{:.0}%，很少参与团战", avg_kp * 100.0),
        score: (avg_kp * 100.0) as i32,
        trait_type: "bad".to_string(),
    })
}
```

---

### **4. 伤害占比阈值（damage_share）**

```rust
thresholds::damage_share::HIGH  // 0.30  高伤害占比
thresholds::damage_share::LOW   // 0.15  低伤害占比

// 根据位置获取阈值
thresholds::damage_share::for_role(role)  // 返回 (high, low)
```

**使用示例**：
```rust
let (dmg_high, dmg_low) = thresholds::damage_share::for_role(&player_role);

if avg_damage_share >= dmg_high {
    traits.push(SummonerTrait {
        name: "输出核心".to_string(),
        description: format!("{} 伤害占比{:.0}%，输出爆表", player_role, avg_damage_share * 100.0),
        score: (avg_damage_share * 100.0) as i32,
        trait_type: "good".to_string(),
    });
}
```

---

### **5. 视野得分阈值（vision）**

```rust
thresholds::vision::HIGH  // 2.0  高视野得分（每分钟）
thresholds::vision::LOW   // 1.0  低视野得分（每分钟）

// 根据位置获取阈值
thresholds::vision::for_role(role)  // 返回 (high, low)
```

**使用示例**：
```rust
let (vs_high, vs_low) = thresholds::vision::for_role(&player_role);

if stats.vspm >= vs_high {
    traits.push(SummonerTrait {
        name: "视野大师".to_string(),
        description: format!("{} 视野得分{:.1}/min，视野控制优秀", player_role, stats.vspm),
        score: (stats.vspm * 10.0) as i32,
        trait_type: "good".to_string(),
    });
}
```

---

### **6. 连胜/连败阈值（streak）**

```rust
thresholds::streak::WIN_STREAK_GOOD       // 5   良好连胜
thresholds::streak::WIN_STREAK_EXCELLENT  // 8   优秀连胜
thresholds::streak::LOSS_STREAK_BAD       // -3  糟糕连败
thresholds::streak::LOSS_STREAK_TERRIBLE  // -5  可怕连败
```

**使用示例**：
```rust
if win_streak >= thresholds::streak::WIN_STREAK_EXCELLENT {
    traits.push(SummonerTrait {
        name: "超级连胜".to_string(),
        description: format!("正在{}连胜的超热状态", win_streak),
        score: win_streak * 15,
        trait_type: "good".to_string(),
    });
} else if win_streak >= thresholds::streak::WIN_STREAK_GOOD {
    traits.push(SummonerTrait {
        name: "连胜王".to_string(),
        description: format!("正在{}连胜的火热玩家", win_streak),
        score: win_streak * 10,
        trait_type: "good".to_string(),
    });
}
```

---

### **7. 其他阈值**

```rust
// 稳定性（方差）
thresholds::stability::KDA_VARIANCE_STABLE    // 1.0  稳定
thresholds::stability::KDA_VARIANCE_UNSTABLE  // 3.0  不稳定

// 分布
thresholds::distribution::S_GRADE_THRESHOLD  // 0.30  S级占比
thresholds::distribution::D_GRADE_THRESHOLD  // 0.25  D级占比

// CS（每分钟）
thresholds::cs::EXCELLENT  // 8.0
thresholds::cs::GOOD       // 6.0
thresholds::cs::POOR       // 4.0

// 伤害（每分钟）
thresholds::damage_per_minute::EXCELLENT  // 800.0
thresholds::damage_per_minute::GOOD       // 600.0
thresholds::damage_per_minute::POOR       // 400.0
```

---

## 🔧 **如何添加新阈值**

### **步骤1：在 thresholds.rs 添加新模块**

```rust
// thresholds.rs
pub mod your_new_threshold {
    pub const HIGH: f64 = 75.0;
    pub const MEDIUM: f64 = 50.0;
    pub const LOW: f64 = 25.0;

    // 可选：位置特定函数
    pub fn for_role(role: &str) -> (f64, f64) {
        match role {
            "ADC" => (80.0, 30.0),
            "辅助" => (40.0, 10.0),
            _ => (HIGH, LOW),
        }
    }
}
```

### **步骤2：在分析器中使用**

```rust
// your_analyzer.rs
use super::thresholds;

pub fn analyze_your_feature(stats: &PlayerMatchStats) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    if stats.your_metric >= thresholds::your_new_threshold::HIGH {
        traits.push(SummonerTrait {
            name: "特征名".to_string(),
            description: "描述".to_string(),
            score: stats.your_metric as i32,
            trait_type: "good".to_string(),
        });
    }

    traits
}
```

---

## 🎯 **最佳实践**

### **1. 始终使用 thresholds，而不是硬编码**

❌ **不好的做法**：
```rust
if stats.win_rate >= 65.0 {  // 硬编码，难以维护
    // ...
}
```

✅ **好的做法**：
```rust
if stats.win_rate >= thresholds::win_rate::EXCELLENT_RANKED {  // 易于维护
    // ...
}
```

### **2. 为特殊情况添加注释**

```rust
if avg_kp >= thresholds::kill_participation::HIGH {
    // 团战核心
} else if avg_kp >= 0.60 {  // 💡 中等参团率，暂时硬编码
    // 积极参团
}
```

### **3. 使用动态描述**

```rust
description: format!("胜率{}%，超越{}%标准",
    stats.win_rate as i32,
    (thresholds::win_rate::EXCELLENT_RANKED * 100.0) as i32
)
```

### **4. 根据模式选择阈值**

```rust
let threshold = if is_ranked_mode {
    thresholds::win_rate::EXCELLENT_RANKED
} else {
    thresholds::win_rate::EXCELLENT_OTHER
};
```

---

## 📊 **实际示例**

### **完整的特征分析示例**

```rust
use crate::lcu::types::{PlayerMatchStats, SummonerTrait};
use super::thresholds;

pub fn analyze_comprehensive_traits(stats: &PlayerMatchStats, is_ranked: bool) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 1. 胜率分析（根据模式选择标准）
    let win_rate_threshold = if is_ranked {
        thresholds::win_rate::EXCELLENT_RANKED
    } else {
        thresholds::win_rate::EXCELLENT_OTHER
    };

    if stats.win_rate >= win_rate_threshold {
        traits.push(SummonerTrait {
            name: "大神".to_string(),
            description: format!(
                "胜率{}%，超越{}%标准",
                stats.win_rate as i32,
                (win_rate_threshold * 100.0) as i32
            ),
            score: stats.win_rate as i32,
            trait_type: "good".to_string(),
        });
    }

    // 2. KDA分析
    let kda_threshold = if is_ranked {
        thresholds::kda::EXCELLENT_RANKED
    } else {
        thresholds::kda::EXCELLENT_OTHER
    };

    if stats.avg_kda >= kda_threshold {
        traits.push(SummonerTrait {
            name: "大爹".to_string(),
            description: format!("KDA {:.1}，远超{:.1}标准", stats.avg_kda, kda_threshold),
            score: (stats.avg_kda * 10.0) as i32,
            trait_type: "good".to_string(),
        });
    }

    // 3. 连胜分析
    let win_streak = calculate_win_streak(&stats.recent_performance);
    if win_streak >= thresholds::streak::WIN_STREAK_EXCELLENT {
        traits.push(SummonerTrait {
            name: "超级连胜".to_string(),
            description: format!("正在{}连胜，状态爆棚", win_streak),
            score: win_streak * 15,
            trait_type: "good".to_string(),
        });
    }

    traits
}
```

---

## 🚀 **下一步**

1. **实验新阈值**：在 `thresholds.rs` 调整值，观察效果
2. **添加新维度**：创建新的阈值模块
3. **A/B测试**：对比不同阈值的评价效果
4. **机器学习**：基于大量数据训练最优阈值

---

*最后更新: 2025-10-17*

