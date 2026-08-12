# 多位置分组分析设计方案

## 🎯 **核心需求**

用户查询排位赛时，如果玩了多个位置，应该：
1. **按位置分组**统计数据
2. **分别分析**每个位置的表现
3. **独立生成**每个位置的建议
4. **主位置优先**展示

---

## 📊 **数据结构设计**

### **新增：按位置分组的统计结果**

```rust
/// 多位置分析结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiPositionAnalysis {
    /// 所有位置的统计
    pub position_stats: Vec<PositionStats>,

    /// 主要位置（场次最多的）
    pub main_position: String,

    /// 总览数据（所有位置合计）
    pub overall_stats: PlayerMatchStats,
}

/// 单个位置的统计
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionStats {
    /// 位置名称（打野、上单、中单、ADC、辅助）
    pub position: String,

    /// 该位置的场次
    pub games: u32,

    /// 该位置的统计数据
    pub stats: PlayerMatchStats,

    /// 该位置的智能建议
    pub advice: Vec<GameAdvice>,

    /// 该位置的技能评分（仅排位赛）
    pub skill_assessment: Option<SkillAssessment>,
}
```

---

## 🔧 **实现步骤**

### **Step 1: 过滤并分组**

```rust
pub fn analyze_match_list_data_with_position_grouping(
    match_list_data: Value,
    current_puuid: &str,
    queue_id: Option<i32>,
) -> Result<MultiPositionAnalysis, String> {

    // 1. 解析所有对局
    let games = match_list_data.get("games")...;
    let parsed_games = parse_games(games, current_puuid);

    // 2. 按 queue_id 过滤（如果指定了）
    let filtered_games: Vec<_> = if let Some(qid) = queue_id {
        parsed_games.iter()
            .filter(|g| g.queue_id == qid as i64)
            .cloned()
            .collect()
    } else {
        parsed_games
    };

    // 3. 按位置分组
    let mut position_groups: HashMap<String, Vec<ParsedGame>> = HashMap::new();

    for game in &filtered_games {
        let position = identify_position_from_game(
            &game.player_data.role,
            &game.player_data.lane,
            game.queue_id
        );

        position_groups.entry(position)
            .or_insert_with(Vec::new)
            .push(game.clone());
    }

    // 4. 分析每个位置
    let mut position_stats_list = Vec::new();

    for (position, games) in position_groups {
        // 跳过"灵活"和"未知"，或者场次太少的位置
        if position == "灵活" || position == "未知" || games.len() < 3 {
            continue;
        }

        // 分析该位置的数据
        let stats = analyze_player_stats(&games, current_puuid, AnalysisContext::new());

        // 生成该位置的建议
        let advice = if matches!(strategy, AnalysisStrategy::Ranked) {
            generate_advice(
                &stats,
                &games,
                &position,  // ← 使用实际位置
                AdvicePerspective::SelfImprovement,
                None,
                &strategy,
            )
        } else {
            Vec::new()
        };

        position_stats_list.push(PositionStats {
            position: position.clone(),
            games: games.len() as u32,
            stats,
            advice,
            skill_assessment: None, // TODO: 计算技能评分
        });
    }

    // 5. 按场次排序，主位置排第一
    position_stats_list.sort_by(|a, b| b.games.cmp(&a.games));

    let main_position = position_stats_list.first()
        .map(|p| p.position.clone())
        .unwrap_or_else(|| "未知".to_string());

    // 6. 计算总览数据
    let overall_stats = analyze_player_stats(&filtered_games, current_puuid, AnalysisContext::new());

    Ok(MultiPositionAnalysis {
        position_stats: position_stats_list,
        main_position,
        overall_stats,
    })
}
```

---

## 🎨 **前端展示方案**

### **选项卡式展示**

```vue
<Tabs default-value="overview">
  <TabsList>
    <TabsTrigger value="overview">总览</TabsTrigger>
    <TabsTrigger value="jungle" v-if="hasPosition('打野')">
      打野 (5场)
    </TabsTrigger>
    <TabsTrigger value="top" v-if="hasPosition('上单')">
      上单 (3场)
    </TabsTrigger>
    <TabsTrigger value="mid" v-if="hasPosition('中单')">
      中单 (2场)
    </TabsTrigger>
  </TabsList>

  <!-- 总览标签 -->
  <TabsContent value="overview">
    <Card>
      <CardHeader>
        <CardTitle>排位赛总览 (10场)</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-3 gap-4">
          <StatCard title="总胜率" value="40%" />
          <StatCard title="平均KDA" value="2.3" />
          <StatCard title="主要位置" value="打野 (5场)" />
        </div>

        <!-- 位置分布 -->
        <div class="mt-4">
          <h4>位置分布</h4>
          <div class="space-y-2">
            <PositionBar position="打野" games="5" percentage="50%" />
            <PositionBar position="上单" games="3" percentage="30%" />
            <PositionBar position="中单" games="2" percentage="20%" />
          </div>
        </div>
      </CardContent>
    </Card>
  </TabsContent>

  <!-- 打野标签 -->
  <TabsContent value="jungle">
    <Card>
      <CardHeader>
        <CardTitle>打野位置分析 (5场)</CardTitle>
      </CardHeader>
      <CardContent>
        <!-- 打野专属数据 -->
        <GameStats :match-statistics="jungleStats" />

        <!-- 打野技能雷达图 -->
        <SkillRadarChart :skill-assessment="jungleSkills" />

        <!-- 打野建议 -->
        <div class="space-y-2">
          <h4>改进建议</h4>
          <AdviceCard
            v-for="advice in jungleAdvice"
            :key="advice.id"
            :advice="advice"
          />
        </div>
      </CardContent>
    </Card>
  </TabsContent>

  <!-- 其他位置同理 -->
</Tabs>
```

---

## 📋 **实施优先级**

### **MVP版本（最小可行产品）**
1. ✅ **位置分组统计** - 基础数据按位置分开
2. ✅ **主位置识别** - 找出玩得最多的位置
3. ✅ **分位置展示** - 前端用选项卡展示

### **完整版本**
4. ⏳ **分位置建议** - 每个位置独立生成建议
5. ⏳ **分位置评分** - 每个位置独立计算技能评分
6. ⏳ **位置对比** - 横向对比不同位置的表现

---

## ⚡ **性能考虑**

### **场次过滤**
- 位置场次 < 3场 → 不单独分析，归入"其他"
- 位置场次 >= 3场 → 独立分析

### **缓存策略**
- 按 `(puuid, queue_id, position)` 缓存分析结果
- 避免重复计算

---

## 🎯 **预期效果**

### **修复前**
```
查询: 440排位赛 30场
结果: 10场排位赛
分析: 混合所有位置的数据
建议: "你的打野生存能力弱" (但可能包含上单数据)
```

### **修复后**
```
查询: 440排位赛 30场
结果: 10场排位赛

总览:
- 总场次: 10
- 总胜率: 40%
- 主要位置: 打野 (5场)

打野位置 (5场):
- 胜率: 60%
- KDA: 2.8
- 建议: 针对打野的建议

上单位置 (3场):
- 胜率: 33%
- KDA: 1.5
- 建议: 针对上单的建议

中单位置 (2场):
- 胜率: 0%
- KDA: 2.0
- 建议: 针对中单的建议
```

---

## 🚀 **立即可做的优化**

### **后端改动**
1. 修改 `analyze_match_list_data` 函数
2. 按位置分组统计
3. 返回多位置数据结构

### **前端改动**
1. 添加选项卡组件
2. 展示位置分布
3. 切换不同位置查看详情

---

**要立即实现这个方案吗？** 这是一个重要的功能改进！

