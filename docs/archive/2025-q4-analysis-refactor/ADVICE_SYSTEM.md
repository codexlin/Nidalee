# 智能建议系统设计

## 🎯 **核心理念**

```
分析 → 识别问题/优势 → 提供针对性建议
```

不仅告诉玩家"你哪里不行"，更要告诉玩家"怎么改进"！

---

## 📊 **基于时间线的智能诊断**

### **诊断维度**

```
┌─────────────────────────────────────────────────────────┐
│ 时间线数据分析                                           │
├─────────────────────────────────────────────────────────┤
│ 1. 对线期诊断 (0-10分钟)                                 │
│    ├─ CS差分析 → 对线压制力/被压制                       │
│    ├─ 经验差分析 → 等级优势/劣势                         │
│    ├─ 死亡时间 → 前期存活能力                            │
│    └─ 承伤模式 → 打法激进度                              │
│                                                          │
│ 2. 发育阶段诊断 (10-20分钟)                              │
│    ├─ 经济效率 → 游走/刷钱平衡                           │
│    ├─ CS维持 → 补刀稳定性                                │
│    └─ 参团频率 → 团队配合                                │
│                                                          │
│ 3. 团战期诊断 (20分钟+)                                  │
│    ├─ 伤害输出 → 团战定位                                │
│    ├─ 存活能力 → 站位意识                                │
│    └─ 参团率 → 团队意识                                  │
└─────────────────────────────────────────────────────────┘
```

---

## 💡 **建议系统架构**

### **数据结构**

```rust
/// 游戏建议
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/GameAdvice.ts")]
#[serde(rename_all = "camelCase")]
pub struct GameAdvice {
    /// 建议标题
    pub title: String,

    /// 问题描述
    pub problem: String,

    /// 数据支持
    pub evidence: String,

    /// 具体建议（多条）
    pub suggestions: Vec<String>,

    /// 优先级（1-5，5最高）
    pub priority: u8,

    /// 分类（对线/团战/发育/意识）
    pub category: AdviceCategory,

    /// 影响的位置（如果位置相关）
    pub affected_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AdviceCategory {
    Laning,      // 对线
    Farming,     // 发育
    Teamfight,   // 团战
    Vision,      // 视野
    Positioning, // 站位
    Decision,    // 决策
    Champion,    // 英雄池
}

/// 扩展 PlayerMatchStats
pub struct PlayerMatchStats {
    // ... 现有字段 ...

    /// 智能建议列表
    pub advice: Vec<GameAdvice>,
}
```

---

## 🔍 **实际诊断示例**

### **示例1：上单前期容易死**

#### **数据分析**
```rust
// timeline_advice_analyzer.rs

// 分析死亡时间分布
let early_deaths = recent_performance
    .iter()
    .filter(|game| {
        if let Some(timeline) = &game.timeline_data {
            // 假设我们跟踪了死亡时间
            game.deaths >= 2 && game.game_duration <= 15 * 60
        } else {
            false
        }
    })
    .count();

let early_death_rate = early_deaths as f64 / recent_performance.len() as f64;

// 诊断：前期容易死
if early_death_rate >= 0.6 && role == "上单" {
    advice.push(GameAdvice {
        title: "前期存活能力需提升".to_string(),
        problem: format!(
            "近20场中有{}场在15分钟前死亡2次以上（占比{:.0}%）",
            early_deaths, early_death_rate * 100.0
        ),
        evidence: format!(
            "前期平均死亡{:.1}次，对线期压力大",
            avg_early_deaths
        ),
        suggestions: vec![
            "🛡️ 加强视野控制：前期在河道和三角草做好眼位".to_string(),
            "📊 学习兵线管理：控制兵线在塔下，减少被gank机会".to_string(),
            "⚠️ 提升警惕性：对方打野消失时立即后撤".to_string(),
            "🎯 练习换血技巧：避免无意义的消耗战".to_string(),
            "💪 选择稳健英雄：考虑使用塞恩、奥恩等抗压型英雄".to_string(),
        ],
        priority: 5, // 最高优先级
        category: AdviceCategory::Laning,
        affected_role: Some("上单".to_string()),
    });
}
```

#### **前端显示效果**
```
┌────────────────────────────────────────────────┐
│ 🚨 前期存活能力需提升 [高优先级]              │
├────────────────────────────────────────────────┤
│ 问题：                                         │
│ 近20场中有12场在15分钟前死亡2次以上（占比60%）│
│                                                │
│ 数据：                                         │
│ 前期平均死亡2.3次，对线期压力大                │
│                                                │
│ 建议：                                         │
│ 🛡️ 加强视野控制：前期在河道和三角草做好眼位   │
│ 📊 学习兵线管理：控制兵线在塔下，减少被gank机会│
│ ⚠️ 提升警惕性：对方打野消失时立即后撤         │
│ 🎯 练习换血技巧：避免无意义的消耗战            │
│ 💪 选择稳健英雄：考虑使用塞恩、奥恩等抗压型英雄│
└────────────────────────────────────────────────┘
```

---

### **示例2：ADC对线被压制**

#### **数据分析**
```rust
// 分析对线期CS差
let avg_cs_diff_early = calculate_avg_timeline_metric(
    games,
    |timeline| timeline.cs_diff_0_10
);

if avg_cs_diff_early <= -15.0 && role == "ADC" {
    advice.push(GameAdvice {
        title: "对线补刀能力待提升".to_string(),
        problem: format!(
            "对线期平均落后{:.1}刀，被对手压制",
            -avg_cs_diff_early
        ),
        evidence: format!(
            "前10分钟CS效率仅{:.1}/分钟，低于平均水平",
            avg_cs_per_min_early
        ),
        suggestions: vec![
            "🎯 练习补刀基本功：训练模式练习尾刀时机".to_string(),
            "🤝 加强辅助配合：沟通压制时机和换血节奏".to_string(),
            "⚡ 优化技能释放：用技能补远程兵和炮车".to_string(),
            "📍 改善站位：避免被频繁消耗而漏刀".to_string(),
            "🛡️ 选择稳健对线英雄：EZ、韦鲁斯等手长安全英雄".to_string(),
            "📚 学习对线组合：了解不同组合的强弱势期".to_string(),
        ],
        priority: 4,
        category: AdviceCategory::Laning,
        affected_role: Some("ADC".to_string()),
    });
}
```

---

### **示例3：中期经济效率下降**

#### **数据分析**
```rust
// 对比对线期和中期经济
let early_gold = avg_timeline_metric(games, |t| t.gold_per_min_0_10);
let mid_gold = avg_timeline_metric(games, |t| t.gold_per_min_10_20);

if mid_gold < early_gold * 0.85 {
    advice.push(GameAdvice {
        title: "中期发育节奏需优化".to_string(),
        problem: format!(
            "中期经济效率下降{:.0}%（{}→{}金币/分）",
            (1.0 - mid_gold / early_gold) * 100.0,
            early_gold as i32,
            mid_gold as i32
        ),
        evidence: "频繁游走但参团效率不高，导致经济落后".to_string(),
        suggestions: vec![
            "⏰ 优化游走时机：只在炮车线或兵线推进时游走".to_string(),
            "💰 提升参团收益：参团后优先吃野怪和兵线".to_string(),
            "🎯 平衡游走和发育：避免无意义的游走".to_string(),
            "👁️ 提升地图意识：提前判断团战位置，减少赶路时间".to_string(),
            "📊 学习资源分配：野区资源及时清理".to_string(),
        ],
        priority: 3,
        category: AdviceCategory::Farming,
        affected_role: None,
    });
}
```

---

### **示例4：团战站位激进**

#### **数据分析**
```rust
// 分析承伤和死亡关系
let avg_damage_taken = calculate_avg_damage_taken(games);
let avg_deaths_in_teamfight = calculate_teamfight_deaths(games);

if avg_damage_taken > role_threshold && avg_deaths_in_teamfight >= 1.5 {
    advice.push(GameAdvice {
        title: "团战站位过于激进".to_string(),
        problem: format!(
            "团战平均死亡{:.1}次，承伤过高",
            avg_deaths_in_teamfight
        ),
        evidence: format!(
            "平均承伤{:.0}，远超位置标准（{}）",
            avg_damage_taken,
            role_threshold as i32
        ),
        suggestions: vec![
            "📍 改善站位：保持与前排的安全距离".to_string(),
            "👀 观察技能：等对方关键控制技能交出后再进场".to_string(),
            "🛡️ 出装调整：考虑水银鞋、复活甲等保命装".to_string(),
            "⏰ 把握时机：不要一开始就冲，等残血再收割".to_string(),
            "🎯 目标选择：优先攻击安全范围内的目标".to_string(),
        ],
        priority: 4,
        category: AdviceCategory::Positioning,
        affected_role: Some(role.to_string()),
    });
}
```

---

### **示例5：视野控制不足**

#### **数据分析**
```rust
// 视野得分分析
if stats.vspm < thresholds::vision::for_role(role).1 {
    let (high, low) = thresholds::vision::for_role(role);

    advice.push(GameAdvice {
        title: "视野控制需要加强".to_string(),
        problem: format!(
            "视野得分仅{:.1}/分钟，低于{}位置标准（{:.1}）",
            stats.vspm, role, low
        ),
        evidence: format!(
            "平均每场仅放置{}个眼位，排眼{}个",
            avg_wards_placed, avg_wards_killed
        ),
        suggestions: vec![
            "🔍 养成买眼习惯：每次回城都购买控制守卫".to_string(),
            "📍 学习关键眼位：龙坑、野区入口、河道草丛".to_string(),
            "⏰ 提前做视野：打龙前1分钟就要布置视野".to_string(),
            "👁️ 注意排眼：使用扫描清理关键位置".to_string(),
            "🎯 购买真眼：身上始终保持1个真眼".to_string(),
            if role == "辅助" {
                "💡 辅助专属：优先升级辅助装备，解锁更多眼位".to_string()
            } else {
                "💡 非辅助：可以考虑出个幽梦或黑切提供视野".to_string()
            },
        ],
        priority: 3,
        category: AdviceCategory::Vision,
        affected_role: Some(role.to_string()),
    });
}
```

---

## 🏗️ **完整实现架构**

### **文件结构**
```
src-tauri/src/lcu/player_stats_analyzer/
├── advice/
│   ├── mod.rs                    # 导出
│   ├── advice_analyzer.rs        # 主建议生成器
│   ├── laning_advice.rs          # 对线期建议
│   ├── farming_advice.rs         # 发育建议
│   ├── teamfight_advice.rs       # 团战建议
│   ├── vision_advice.rs          # 视野建议
│   └── champion_advice.rs        # 英雄池建议
└── ...
```

### **主建议生成器**

```rust
// advice/advice_analyzer.rs

use crate::lcu::types::{PlayerMatchStats, GameAdvice};
use super::parser::ParsedGame;

/// 生成智能建议
pub fn generate_advice(
    stats: &PlayerMatchStats,
    games: &[ParsedGame],
    role: &str,
    strategy: &AnalysisStrategy,
) -> Vec<GameAdvice> {
    let mut advice = Vec::new();

    // 只在排位模式下生成详细建议
    if !strategy.enable_advanced_analysis() {
        return advice;
    }

    // 1. 对线期建议
    advice.extend(analyze_laning_advice(stats, games, role));

    // 2. 发育建议
    advice.extend(analyze_farming_advice(stats, games, role));

    // 3. 团战建议
    advice.extend(analyze_teamfight_advice(stats, games, role));

    // 4. 视野建议
    advice.extend(analyze_vision_advice(stats, role));

    // 5. 英雄池建议
    advice.extend(analyze_champion_pool_advice(stats));

    // 按优先级排序，只保留前5条最重要的
    advice.sort_by_key(|a| std::cmp::Reverse(a.priority));
    advice.truncate(5);

    advice
}
```

---

## 🎨 **前端显示设计**

### **建议卡片组件**

```vue
<!-- AdviceCard.vue -->
<template>
  <div class="advice-card" :class="`priority-${advice.priority}`">
    <div class="advice-header">
      <span class="advice-icon">{{ getCategoryIcon(advice.category) }}</span>
      <h3 class="advice-title">{{ advice.title }}</h3>
      <span class="priority-badge">{{ getPriorityLabel(advice.priority) }}</span>
    </div>

    <div class="advice-problem">
      <h4>📊 问题</h4>
      <p>{{ advice.problem }}</p>
    </div>

    <div class="advice-evidence">
      <h4>📈 数据</h4>
      <p>{{ advice.evidence }}</p>
    </div>

    <div class="advice-suggestions">
      <h4>💡 建议</h4>
      <ul>
        <li v-for="(suggestion, index) in advice.suggestions" :key="index">
          {{ suggestion }}
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { GameAdvice } from '@/types/generated/GameAdvice'

const props = defineProps<{
  advice: GameAdvice
}>()

const getCategoryIcon = (category: string) => {
  const icons = {
    Laning: '⚔️',
    Farming: '💰',
    Teamfight: '🤝',
    Vision: '👁️',
    Positioning: '📍',
    Decision: '🧠',
    Champion: '🎮'
  }
  return icons[category] || '💡'
}

const getPriorityLabel = (priority: number) => {
  if (priority >= 4) return '高优先级'
  if (priority >= 2) return '中优先级'
  return '低优先级'
}
</script>
```

### **建议面板**

```vue
<!-- AdvicePanel.vue -->
<template>
  <div class="advice-panel">
    <div class="panel-header">
      <h2>🎯 智能建议</h2>
      <p class="subtitle">基于近20场数据的针对性建议</p>
    </div>

    <div v-if="advice.length === 0" class="no-advice">
      <span class="icon">🎉</span>
      <h3>表现优秀！</h3>
      <p>暂时没有需要改进的地方，继续保持！</p>
    </div>

    <div v-else class="advice-list">
      <AdviceCard
        v-for="(item, index) in advice"
        :key="index"
        :advice="item"
      />
    </div>
  </div>
</template>
```

---

## 📊 **建议优先级算法**

```rust
// 计算建议优先级
fn calculate_priority(
    severity: f64,      // 问题严重程度 (0.0-1.0)
    frequency: f64,     // 问题出现频率 (0.0-1.0)
    impact: f64,        // 对胜率的影响 (0.0-1.0)
) -> u8 {
    let score = severity * 0.4 + frequency * 0.3 + impact * 0.3;

    if score >= 0.8 {
        5  // 紧急
    } else if score >= 0.6 {
        4  // 高
    } else if score >= 0.4 {
        3  // 中
    } else if score >= 0.2 {
        2  // 低
    } else {
        1  // 可选
    }
}

// 示例：前期容易死
let priority = calculate_priority(
    0.9,  // 严重程度：前期死太多影响很大
    0.6,  // 频率：60%的对局都有这个问题
    0.85, // 影响：对胜率影响巨大
);
// 结果：5 (紧急)
```

---

## 🎯 **实现路线图**

### **阶段1：基础建议系统** 🔴 高优先级

```
实现内容：
✅ 添加 GameAdvice 数据结构
✅ 实现 5 种核心建议：
   1. 对线期建议（前期容易死、被压制）
   2. 发育建议（中期效率下降）
   3. 视野建议（视野控制不足）
   4. 团战建议（参团率低、站位激进）
   5. 英雄池建议（绝活哥、单一依赖）

工作量：4-6 小时
价值：⭐⭐⭐⭐⭐ 极高
```

### **阶段2：时间线深度建议** 🟡 中优先级

```
实现内容：
✅ 基于时间线数据的精细建议
✅ 对线期具体问题诊断
✅ 分阶段发育曲线建议
✅ 激进度/稳健度建议

工作量：3-4 小时
价值：⭐⭐⭐⭐ 高
```

### **阶段3：个性化建议** 🔵 低优先级

```
实现内容：
✅ 根据段位调整建议
✅ 根据英雄调整建议
✅ 历史改进追踪
✅ 建议效果评估

工作量：4-5 小时
价值：⭐⭐⭐ 中
```

---

## 🔥 **示例：完整的建议输出**

```json
{
  "total_games": 20,
  "wins": 12,
  "win_rate": 60.0,
  "traits": [
    "大爹（KDA 4.2）",
    "上单专精（胜率68%）",
    "稳定发挥"
  ],
  "advice": [
    {
      "title": "前期存活能力需提升",
      "problem": "近20场中有12场在15分钟前死亡2次以上（占比60%）",
      "evidence": "前期平均死亡2.3次，对线期压力大",
      "suggestions": [
        "🛡️ 加强视野控制：前期在河道和三角草做好眼位",
        "📊 学习兵线管理：控制兵线在塔下，减少被gank机会",
        "⚠️ 提升警惕性：对方打野消失时立即后撤",
        "🎯 练习换血技巧：避免无意义的消耗战",
        "💪 选择稳健英雄：考虑使用塞恩、奥恩等抗压型英雄"
      ],
      "priority": 5,
      "category": "Laning",
      "affected_role": "上单"
    },
    {
      "title": "视野控制需要加强",
      "problem": "视野得分仅1.2/分钟，低于上单位置标准（1.5）",
      "evidence": "平均每场仅放置8个眼位，排眼2个",
      "suggestions": [
        "🔍 养成买眼习惯：每次回城都购买控制守卫",
        "📍 学习关键眼位：龙坑、野区入口、河道草丛",
        "⏰ 提前做视野：打龙前1分钟就要布置视野",
        "👁️ 注意排眼：使用扫描清理关键位置",
        "🎯 购买真眼：身上始终保持1个真眼"
      ],
      "priority": 3,
      "category": "Vision",
      "affected_role": "上单"
    }
  ]
}
```

---

## 💡 **总结**

### **你的想法的价值**

| 维度 | 价值 | 说明 |
|------|------|------|
| **用户体验** | ⭐⭐⭐⭐⭐ | 不只是"告诉问题"，更是"给出方案" |
| **实用性** | ⭐⭐⭐⭐⭐ | 玩家可以直接按建议改进 |
| **差异化** | ⭐⭐⭐⭐⭐ | 市面上少有的真正"智能"分析 |
| **技术实现** | ⭐⭐⭐⭐ | 基于现有数据，可行性高 |

### **核心优势**

1. **针对性强**：基于真实数据，而非泛泛而谈
2. **可操作性**：每条建议都具体可执行
3. **优先级明确**：先改最重要的问题
4. **位置相关**：不同位置不同建议

### **实现建议**

**强烈推荐结合实现：**
1. ✅ 时间线分析（识别问题）
2. ✅ 智能建议（解决问题）

**顺序**：
```
第一步：实现基础时间线分析 (2-3小时)
第二步：实现核心建议系统 (4-6小时)
第三步：优化和扩展 (按需)

总计：6-9小时，价值极高！
```

---

**这个想法太棒了！这才是真正的"智能助手"，不只是数据展示工具！** 🚀

需要我帮你实现这个系统吗？我们可以先从最核心的几个建议开始！
