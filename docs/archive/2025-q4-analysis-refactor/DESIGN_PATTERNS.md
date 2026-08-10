# 设计模式架构方案

## 🎯 **核心设计模式组合**

基于你的需求，我推荐使用以下设计模式组合：

```
1. 责任链模式 (Chain of Responsibility) - 建议生成
2. 策略模式 (Strategy) - 视角切换
3. 工厂模式 (Factory) - 建议创建
4. 模板方法模式 (Template Method) - 分析流程
5. 观察者模式 (Observer) - 可选，用于扩展
```

---

## 📊 **设计模式1：责任链模式 - 建议生成器**

### **问题**
```
问题识别 → 需要生成建议

有5大类问题：
1. 对线期问题
2. 发育问题
3. 团战问题
4. 视野问题
5. 英雄池问题

每个分析器独立判断是否能生成建议
```

### **解决方案：责任链**

```rust
/// 建议分析器 trait（责任链节点）
pub trait AdviceAnalyzer: Send + Sync {
    /// 分析并生成建议（如果有）
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice>;

    /// 获取分析器名称
    fn name(&self) -> &str;

    /// 是否启用（根据策略）
    fn is_enabled(&self, strategy: &AnalysisStrategy) -> bool {
        true  // 默认启用
    }
}

/// 建议分析上下文
pub struct AdviceContext {
    pub stats: PlayerMatchStats,
    pub games: Vec<ParsedGame>,
    pub role: String,
    pub perspective: AdvicePerspective,
    pub target_name: Option<String>,
}

/// 建议生成器（责任链）
pub struct AdviceChain {
    analyzers: Vec<Box<dyn AdviceAnalyzer>>,
}

impl AdviceChain {
    pub fn new() -> Self {
        Self {
            analyzers: vec![
                Box::new(LaningAdviceAnalyzer),
                Box::new(FarmingAdviceAnalyzer),
                Box::new(TeamfightAdviceAnalyzer),
                Box::new(VisionAdviceAnalyzer),
                Box::new(ChampionAdviceAnalyzer),
            ],
        }
    }

    /// 执行责任链，收集所有建议
    pub fn generate(&self, context: &AdviceContext, strategy: &AnalysisStrategy) -> Vec<GameAdvice> {
        let mut all_advice = Vec::new();

        for analyzer in &self.analyzers {
            if analyzer.is_enabled(strategy) {
                let advice = analyzer.analyze(context);
                println!("📊 {}: 生成了 {} 条建议", analyzer.name(), advice.len());
                all_advice.extend(advice);
            }
        }

        // 按优先级排序
        all_advice.sort_by_key(|a| std::cmp::Reverse(a.priority));

        // 限制数量
        all_advice.truncate(5);

        all_advice
    }
}
```

### **具体实现示例**

```rust
// laning_advice_analyzer.rs

pub struct LaningAdviceAnalyzer;

impl AdviceAnalyzer for LaningAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        // 分析：前期容易死
        if let Some(advice_item) = self.analyze_early_deaths(context) {
            advice.push(advice_item);
        }

        // 分析：对线被压制
        if let Some(advice_item) = self.analyze_laning_weakness(context) {
            advice.push(advice_item);
        }

        advice
    }

    fn name(&self) -> &str {
        "对线期分析器"
    }

    fn is_enabled(&self, strategy: &AnalysisStrategy) -> bool {
        // 只在排位模式下启用
        matches!(strategy, AnalysisStrategy::Ranked)
    }
}

impl LaningAdviceAnalyzer {
    fn analyze_early_deaths(&self, context: &AdviceContext) -> Option<GameAdvice> {
        let early_death_rate = calculate_early_death_rate(&context.games);

        if early_death_rate < 0.6 {
            return None;  // 没有问题
        }

        // 根据视角生成不同的建议
        Some(match context.perspective {
            AdvicePerspective::SelfImprovement => self.create_self_improvement_advice(early_death_rate),
            AdvicePerspective::Targeting => self.create_targeting_advice(early_death_rate, &context.target_name),
            AdvicePerspective::Collaboration => self.create_collaboration_advice(early_death_rate, &context.role),
        })
    }

    fn create_self_improvement_advice(&self, rate: f64) -> GameAdvice {
        GameAdvice {
            title: "前期存活能力需提升".to_string(),
            problem: format!("近期有{:.0}%的对局前期死亡过多", rate * 100.0),
            evidence: "前期平均死亡2.3次".to_string(),
            suggestions: vec![
                "🛡️ 加强视野控制".to_string(),
                "📊 学习兵线管理".to_string(),
                "⚠️ 提升警惕性".to_string(),
            ],
            priority: 5,
            category: AdviceCategory::Laning,
            perspective: AdvicePerspective::SelfImprovement,
            affected_role: None,
            target_player: None,
        }
    }

    fn create_targeting_advice(&self, rate: f64, target: &Option<String>) -> GameAdvice {
        GameAdvice {
            title: "可针对：前期容易被击杀".to_string(),
            problem: format!("对手有{:.0}%的对局前期死亡过多", rate * 100.0),
            evidence: "对手前期平均死亡2.3次，抗压能力弱".to_string(),
            suggestions: vec![
                "🎯 打野重点gank该路".to_string(),
                "📊 选择压制型英雄".to_string(),
                "⏰ 3级/6级抓一波".to_string(),
            ],
            priority: 5,
            category: AdviceCategory::Laning,
            perspective: AdvicePerspective::Targeting,
            affected_role: None,
            target_player: target.clone(),
        }
    }

    fn create_collaboration_advice(&self, rate: f64, role: &str) -> GameAdvice {
        GameAdvice {
            title: format!("队友{}前期需要保护", role),
            problem: format!("队友前期容易被击杀（{:.0}%对局）", rate * 100.0),
            evidence: "该队友前期压力大，容易被针对".to_string(),
            suggestions: vec![
                format!("🛡️ 打野：多去{}路反蹲", role),
                format!("👁️ 辅助：帮{}路做视野", role),
                "⚠️ 全队：注意支援信号".to_string(),
            ],
            priority: 4,
            category: AdviceCategory::Laning,
            perspective: AdvicePerspective::Collaboration,
            affected_role: Some(role.to_string()),
            target_player: None,
        }
    }
}
```

**优势**：
- ✅ 每个分析器独立，易于添加新分析器
- ✅ 自动过滤（根据策略启用/禁用）
- ✅ 统一接口，易于维护

---

## 🎯 **设计模式2：策略模式 - 视角切换**

### **问题**
```
同样的数据 → 三种不同视角的建议

1. SelfImprovement：改进建议（对自己）
2. Targeting：针对建议（对敌人）
3. Collaboration：协作建议（对队友）⭐
```

### **解决方案：策略模式**

```rust
/// 建议生成策略 trait
pub trait AdviceStrategy: Send + Sync {
    fn generate_advice(
        &self,
        problem_type: ProblemType,
        data: &ProblemData,
    ) -> GameAdvice;
}

/// 改进建议策略
pub struct SelfImprovementStrategy;

impl AdviceStrategy for SelfImprovementStrategy {
    fn generate_advice(&self, problem_type: ProblemType, data: &ProblemData) -> GameAdvice {
        match problem_type {
            ProblemType::EarlyDeaths => GameAdvice {
                title: "前期存活能力需提升".to_string(),
                suggestions: vec![
                    "🛡️ 加强视野控制".to_string(),
                    "📊 学习兵线管理".to_string(),
                ],
                perspective: AdvicePerspective::SelfImprovement,
                // ...
            },
            // ... 其他问题类型
        }
    }
}

/// 针对建议策略
pub struct TargetingStrategy;

impl AdviceStrategy for TargetingStrategy {
    fn generate_advice(&self, problem_type: ProblemType, data: &ProblemData) -> GameAdvice {
        match problem_type {
            ProblemType::EarlyDeaths => GameAdvice {
                title: "可针对：前期容易被击杀".to_string(),
                suggestions: vec![
                    "🎯 打野重点gank该路".to_string(),
                    "📊 选择压制型英雄".to_string(),
                ],
                perspective: AdvicePerspective::Targeting,
                // ...
            },
            // ... 其他问题类型
        }
    }
}

/// 协作建议策略 ⭐
pub struct CollaborationStrategy;

impl AdviceStrategy for CollaborationStrategy {
    fn generate_advice(&self, problem_type: ProblemType, data: &ProblemData) -> GameAdvice {
        match problem_type {
            ProblemType::EarlyDeaths => GameAdvice {
                title: format!("队友{}需要保护", data.role),
                suggestions: vec![
                    format!("🛡️ 打野多去{}路反蹲", data.role),
                    "👁️ 注意支援信号".to_string(),
                ],
                perspective: AdvicePerspective::Collaboration,
                // ...
            },
            // ... 其他问题类型
        }
    }
}

/// 策略工厂
pub struct AdviceStrategyFactory;

impl AdviceStrategyFactory {
    pub fn create(perspective: AdvicePerspective) -> Box<dyn AdviceStrategy> {
        match perspective {
            AdvicePerspective::SelfImprovement => Box::new(SelfImprovementStrategy),
            AdvicePerspective::Targeting => Box::new(TargetingStrategy),
            AdvicePerspective::Collaboration => Box::new(CollaborationStrategy),
        }
    }
}
```

**优势**：
- ✅ 视角切换简单
- ✅ 易于添加新视角
- ✅ 逻辑清晰

---

## 🏗️ **设计模式3：建造者模式 - 复杂建议构建**

### **问题**
```
建议的构建过程复杂：
- 需要分析数据
- 需要计算优先级
- 需要生成多条建议
- 需要格式化描述
```

### **解决方案：建造者模式**

```rust
/// 建议建造者
pub struct AdviceBuilder {
    title: Option<String>,
    problem: Option<String>,
    evidence: Option<String>,
    suggestions: Vec<String>,
    priority: u8,
    category: AdviceCategory,
    perspective: AdvicePerspective,
    affected_role: Option<String>,
    target_player: Option<String>,
}

impl AdviceBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            problem: None,
            evidence: None,
            suggestions: Vec::new(),
            priority: 1,
            category: AdviceCategory::Laning,
            perspective: AdvicePerspective::SelfImprovement,
            affected_role: None,
            target_player: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn problem(mut self, problem: impl Into<String>) -> Self {
        self.problem = Some(problem.into());
        self
    }

    pub fn evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }

    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn category(mut self, category: AdviceCategory) -> Self {
        self.category = category;
        self
    }

    pub fn perspective(mut self, perspective: AdvicePerspective) -> Self {
        self.perspective = perspective;
        self
    }

    pub fn affected_role(mut self, role: impl Into<String>) -> Self {
        self.affected_role = Some(role.into());
        self
    }

    pub fn target_player(mut self, name: impl Into<String>) -> Self {
        self.target_player = Some(name.into());
        self
    }

    pub fn build(self) -> Option<GameAdvice> {
        Some(GameAdvice {
            title: self.title?,
            problem: self.problem?,
            evidence: self.evidence?,
            suggestions: self.suggestions,
            priority: self.priority,
            category: self.category,
            perspective: self.perspective,
            affected_role: self.affected_role,
            target_player: self.target_player,
        })
    }
}

/// 使用示例
fn create_early_death_advice(rate: f64, perspective: AdvicePerspective) -> Option<GameAdvice> {
    match perspective {
        AdvicePerspective::SelfImprovement => {
            AdviceBuilder::new()
                .title("前期存活能力需提升")
                .problem(format!("近期有{:.0}%的对局前期死亡过多", rate * 100.0))
                .evidence("前期平均死亡2.3次")
                .suggestion("🛡️ 加强视野控制")
                .suggestion("📊 学习兵线管理")
                .suggestion("⚠️ 提升警惕性")
                .priority(5)
                .category(AdviceCategory::Laning)
                .perspective(perspective)
                .build()
        },
        AdvicePerspective::Targeting => {
            AdviceBuilder::new()
                .title("可针对：前期容易被击杀")
                .problem(format!("对手有{:.0}%的对局前期死亡过多", rate * 100.0))
                .evidence("对手前期抗压能力弱")
                .suggestion("🎯 打野重点gank该路")
                .suggestion("📊 选择压制型英雄")
                .priority(5)
                .category(AdviceCategory::Laning)
                .perspective(perspective)
                .build()
        },
        // ...
    }
}
```

**优势**：
- ✅ 构建过程清晰
- ✅ 链式调用，代码优雅
- ✅ 易于扩展新字段

---

## 🎨 **设计模式4：模板方法模式 - 分析流程**

### **问题**
```
所有建议分析器的流程类似：
1. 提取数据
2. 计算指标
3. 判断是否有问题
4. 根据视角生成建议
5. 计算优先级
```

### **解决方案：模板方法**

```rust
/// 抽象建议分析器（模板）
pub trait AdviceAnalyzerTemplate {
    /// 模板方法：定义分析流程
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        // 1. 提取相关数据（子类实现）
        let data = self.extract_data(context);

        // 2. 计算指标（子类实现）
        let metrics = self.calculate_metrics(&data);

        // 3. 识别问题（子类实现）
        let problems = self.identify_problems(&metrics);

        // 4. 为每个问题生成建议
        for problem in problems {
            // 5. 根据视角生成建议（子类实现）
            if let Some(advice_item) = self.generate_advice_for_problem(&problem, context) {
                advice.push(advice_item);
            }
        }

        advice
    }

    // 钩子方法（子类实现）
    fn extract_data(&self, context: &AdviceContext) -> Self::Data;
    fn calculate_metrics(&self, data: &Self::Data) -> Self::Metrics;
    fn identify_problems(&self, metrics: &Self::Metrics) -> Vec<Self::Problem>;
    fn generate_advice_for_problem(&self, problem: &Self::Problem, context: &AdviceContext) -> Option<GameAdvice>;

    // 关联类型
    type Data;
    type Metrics;
    type Problem;
}

/// 具体实现：对线期分析
pub struct LaningAnalyzer;

impl AdviceAnalyzerTemplate for LaningAnalyzer {
    type Data = LaningData;
    type Metrics = LaningMetrics;
    type Problem = LaningProblem;

    fn extract_data(&self, context: &AdviceContext) -> LaningData {
        LaningData {
            games: context.games.clone(),
            role: context.role.clone(),
        }
    }

    fn calculate_metrics(&self, data: &LaningData) -> LaningMetrics {
        LaningMetrics {
            early_death_rate: calculate_early_death_rate(&data.games),
            avg_cs_diff: calculate_avg_cs_diff(&data.games),
            avg_xp_diff: calculate_avg_xp_diff(&data.games),
        }
    }

    fn identify_problems(&self, metrics: &LaningMetrics) -> Vec<LaningProblem> {
        let mut problems = Vec::new();

        if metrics.early_death_rate >= 0.6 {
            problems.push(LaningProblem::EarlyDeaths(metrics.early_death_rate));
        }

        if metrics.avg_cs_diff <= -15.0 {
            problems.push(LaningProblem::CsDeficit(metrics.avg_cs_diff));
        }

        problems
    }

    fn generate_advice_for_problem(&self, problem: &LaningProblem, context: &AdviceContext) -> Option<GameAdvice> {
        match problem {
            LaningProblem::EarlyDeaths(rate) => {
                self.create_early_death_advice(*rate, &context.perspective)
            },
            LaningProblem::CsDeficit(diff) => {
                self.create_cs_deficit_advice(*diff, &context.perspective)
            },
        }
    }
}
```

**优势**：
- ✅ 流程统一，代码复用
- ✅ 每个分析器只实现特定逻辑
- ✅ 易于维护

---

## 🏭 **设计模式5：工厂模式 - 建议创建**

### **问题**
```
根据不同的问题类型和视角
创建不同的建议对象
```

### **解决方案：抽象工厂**

```rust
/// 建议工厂 trait
pub trait AdviceFactory {
    fn create_laning_advice(&self, problem: LaningProblem, data: &ProblemData) -> GameAdvice;
    fn create_farming_advice(&self, problem: FarmingProblem, data: &ProblemData) -> GameAdvice;
    fn create_teamfight_advice(&self, problem: TeamfightProblem, data: &ProblemData) -> GameAdvice;
    fn create_vision_advice(&self, problem: VisionProblem, data: &ProblemData) -> GameAdvice;
}

/// 改进建议工厂
pub struct SelfImprovementFactory;

impl AdviceFactory for SelfImprovementFactory {
    fn create_laning_advice(&self, problem: LaningProblem, data: &ProblemData) -> GameAdvice {
        match problem {
            LaningProblem::EarlyDeaths(rate) => {
                AdviceBuilder::new()
                    .title("前期存活能力需提升")
                    .problem(format!("你在近期有{:.0}%的对局前期死亡过多", rate * 100.0))
                    .suggestion("🛡️ 加强视野控制：前期在河道和三角草做好眼位")
                    .suggestion("📊 学习兵线管理：控制兵线在塔下")
                    .suggestion("⚠️ 提升警惕性：打野消失时后撤")
                    .priority(5)
                    .category(AdviceCategory::Laning)
                    .perspective(AdvicePerspective::SelfImprovement)
                    .build()
                    .unwrap()
            },
            // ...
        }
    }
    // ...
}

/// 针对建议工厂
pub struct TargetingFactory;

impl AdviceFactory for TargetingFactory {
    fn create_laning_advice(&self, problem: LaningProblem, data: &ProblemData) -> GameAdvice {
        match problem {
            LaningProblem::EarlyDeaths(rate) => {
                AdviceBuilder::new()
                    .title("可针对的弱点：前期容易被击杀")
                    .problem(format!("对手有{:.0}%的对局前期死亡过多", rate * 100.0))
                    .suggestion("🎯 打野优先级：前期重点照顾该路")
                    .suggestion("📊 英雄选择：选择前期强势的压制型英雄")
                    .suggestion("⏰ 时机把握：3级/6级抓一波")
                    .priority(5)
                    .category(AdviceCategory::Laning)
                    .perspective(AdvicePerspective::Targeting)
                    .target_player(data.target_name.clone())
                    .build()
                    .unwrap()
            },
            // ...
        }
    }
    // ...
}

/// 协作建议工厂 ⭐
pub struct CollaborationFactory;

impl AdviceFactory for CollaborationFactory {
    fn create_laning_advice(&self, problem: LaningProblem, data: &ProblemData) -> GameAdvice {
        match problem {
            LaningProblem::EarlyDeaths(rate) => {
                AdviceBuilder::new()
                    .title(format!("队友{}前期需要保护", data.role))
                    .problem(format!("该队友有{:.0}%的对局前期容易被击杀", rate * 100.0))
                    .suggestion(format!("🛡️ 打野：多去{}路反蹲，保护不崩", data.role))
                    .suggestion(format!("👁️ 全队：帮{}路做视野", data.role))
                    .suggestion("⚠️ 不要过度依赖该路carry")
                    .priority(4)
                    .category(AdviceCategory::Laning)
                    .perspective(AdvicePerspective::Collaboration)
                    .affected_role(Some(data.role.clone()))
                    .build()
                    .unwrap()
            },
            // ...
        }
    }
    // ...
}
```

**优势**：
- ✅ 创建逻辑集中
- ✅ 易于统一风格
- ✅ 便于测试

---

## 🔄 **设计模式6：组合模式 - 团队战术**

### **问题**
```
团队战术是层次化的：
- 整体战术
  ├─ 上路战术
  ├─ 打野战术
  ├─ 中路战术
  ├─ ADC战术
  └─ 辅助战术
```

### **解决方案：组合模式**

```rust
/// 战术节点 trait
pub trait TacticalNode {
    fn get_advice(&self) -> Vec<String>;
    fn get_priority(&self) -> String;
    fn get_target_role(&self) -> String;
}

/// 叶子节点：单个位置的战术
pub struct RoleTactical {
    role: String,
    priority: String,
    suggestions: Vec<String>,
}

impl TacticalNode for RoleTactical {
    fn get_advice(&self) -> Vec<String> {
        self.suggestions.clone()
    }

    fn get_priority(&self) -> String {
        self.priority.clone()
    }

    fn get_target_role(&self) -> String {
        self.role.clone()
    }
}

/// 组合节点：整体战术
pub struct TeamTactical {
    core_strategy: String,
    strategy_reason: String,
    role_tactics: Vec<Box<dyn TacticalNode>>,
}

impl TeamTactical {
    pub fn new(strategy: String, reason: String) -> Self {
        Self {
            core_strategy: strategy,
            strategy_reason: reason,
            role_tactics: Vec::new(),
        }
    }

    pub fn add_role_tactical(&mut self, tactical: Box<dyn TacticalNode>) {
        self.role_tactics.push(tactical);
    }

    pub fn get_all_advice(&self) -> Vec<TacticalAdvice> {
        self.role_tactics
            .iter()
            .map(|t| TacticalAdvice {
                target_role: t.get_target_role(),
                priority: t.get_priority(),
                suggestions: t.get_advice(),
                importance: 5,
                icon: "🎯".to_string(),
            })
            .collect()
    }
}
```

**优势**：
- ✅ 层次清晰
- ✅ 易于添加新节点
- ✅ 统一处理

---

## 🎯 **完整架构整合**

### **整体架构图**

```rust
┌──────────────────────────────────────────────────────────────┐
│ 入口：generate_comprehensive_advice()                         │
└────────────────┬─────────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ 1. 构建上下文 (AdviceContext)                                 │
│    - stats, games, role, perspective                         │
└────────────────┬─────────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ 2. 创建策略工厂 (AdviceStrategyFactory)                      │
│    - 根据 perspective 创建对应策略                            │
└────────────────┬─────────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ 3. 责任链执行 (AdviceChain)                                  │
│    ┌────────────────────────────────────────────┐            │
│    │ LaningAdviceAnalyzer                       │            │
│    │ → 提取数据 → 计算指标 → 识别问题          │            │
│    │ → 使用策略生成建议 → 使用建造者构建       │            │
│    └────────────────────────────────────────────┘            │
│    ┌────────────────────────────────────────────┐            │
│    │ FarmingAdviceAnalyzer                      │            │
│    │ → ...                                      │            │
│    └────────────────────────────────────────────┘            │
│    ┌────────────────────────────────────────────┐            │
│    │ TeamfightAdviceAnalyzer                    │            │
│    │ → ...                                      │            │
│    └────────────────────────────────────────────┘            │
│    ... 其他分析器                                            │
└────────────────┬─────────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ 4. 汇总和优化                                                │
│    - 按优先级排序                                            │
│    - 限制数量（5条）                                         │
└────────────────┬─────────────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ 5. 返回：Vec<GameAdvice>                                     │
└──────────────────────────────────────────────────────────────┘
```

---

## 📂 **完整文件结构**

```
src-tauri/src/lcu/player_stats_analyzer/
├── mod.rs
│
├── 📊 数据层（已完成）
│   ├── parser.rs              # Parser 模式
│   ├── strategy.rs            # Strategy 模式
│   └── thresholds.rs          # 配置
│
├── 🔬 分析层（已完成）
│   ├── analyzer.rs
│   ├── traits_analyzer.rs
│   ├── advanced_analyzer.rs
│   ├── role_analyzer.rs
│   ├── distribution_analyzer.rs
│   └── trait_merger.rs
│
├── ⏱️ 时间线层（计划）
│   └── timeline_analyzer.rs   # 时间线分析
│
├── 💡 建议层（计划）
│   └── advice/
│       ├── mod.rs
│       ├── chain.rs           # 责任链模式 ⭐
│       ├── context.rs         # 上下文
│       ├── factory.rs         # 工厂模式 ⭐
│       ├── builder.rs         # 建造者模式 ⭐
│       ├── analyzers/
│       │   ├── mod.rs
│       │   ├── base.rs        # 模板方法 ⭐
│       │   ├── laning.rs      # 对线期
│       │   ├── farming.rs     # 发育
│       │   ├── teamfight.rs   # 团战
│       │   ├── vision.rs      # 视野
│       │   └── champion.rs    # 英雄池
│       └── strategies/
│           ├── self_improvement.rs  # 改进策略
│           ├── targeting.rs         # 针对策略
│           └── collaboration.rs     # 协作策略 ⭐
│
└── 🎯 战术层（计划）
    └── team_tactical/
        ├── mod.rs
        ├── power_evaluator.rs    # 实力评估
        ├── tactical_decision.rs  # 战术决策树
        └── tactical_generator.rs # 战术生成（组合模式 ⭐）
```

---

## 🎯 **完整代码示例**

### **主入口函数**

```rust
// mod.rs

/// 生成完整的玩家分析（包含建议）
pub fn analyze_player_with_advice(
    games: &[Value],
    puuid: &str,
    queue_id: Option<i32>,
    perspective: AdvicePerspective,
    target_name: Option<String>,
) -> PlayerMatchStats {
    // 1. Parser 层
    let parsed_games = parse_games(games, puuid);

    // 2. Strategy 层
    let strategy = AnalysisStrategy::from_queue_id(queue_id.unwrap_or(0) as i64);

    // 3. Analyzer 层
    let mut context = AnalysisContext::new();
    if let Some(qid) = queue_id {
        context = context.with_queue_id(qid);
    }
    let mut player_stats = analyze_player_stats(&parsed_games, puuid, context);

    // 4. Traits 层（现有）
    let mut traits = Vec::new();
    traits.extend(analyze_traits(&player_stats));
    if strategy.enable_advanced_analysis() {
        traits.extend(analyze_advanced_traits(&player_stats, games, puuid));
        traits.extend(analyze_role_based_traits(&player_stats, &identify_player_roles(games, puuid)));
        traits.extend(analyze_distribution_traits(&player_stats.recent_performance));

        // 新增：时间线分析 ⭐
        traits.extend(analyze_timeline_traits(&parsed_games, &main_role));
    }
    traits.extend(analyze_win_loss_pattern(&player_stats.recent_performance));

    player_stats.traits = optimize_traits(traits);

    // 5. Advice 层（新增）⭐
    if strategy == AnalysisStrategy::Ranked {
        let advice_context = AdviceContext {
            stats: player_stats.clone(),
            games: parsed_games,
            role: identify_main_role(&parsed_games),
            perspective,
            target_name,
        };

        let chain = AdviceChain::new();
        player_stats.advice = chain.generate(&advice_context, &strategy);
    }

    player_stats
}
```

---

## 🎨 **设计模式总结**

| 设计模式 | 用途 | 位置 | 优势 |
|---------|------|------|------|
| **责任链模式** | 建议生成 | `advice/chain.rs` | 独立分析器，易扩展 |
| **策略模式** | 视角切换 | `advice/strategies/` | 三种视角统一接口 |
| **建造者模式** | 建议构建 | `advice/builder.rs` | 链式调用，优雅 |
| **模板方法模式** | 分析流程 | `advice/analyzers/base.rs` | 流程统一，代码复用 |
| **工厂模式** | 建议创建 | `advice/factory.rs` | 创建逻辑集中 |
| **组合模式** | 团队战术 | `team_tactical/` | 层次清晰 |

---

## 🚀 **实现优势**

### **代码质量**

```
✅ 高内聚低耦合：每个模式职责单一
✅ 开闭原则：易于扩展，无需修改现有代码
✅ 单一职责：每个类只做一件事
✅ 依赖倒置：依赖抽象，不依赖具体
✅ 接口隔离：接口精简，易于实现
```

### **可维护性**

```
✅ 新增建议类型：添加新的 Analyzer
✅ 新增视角：添加新的 Strategy
✅ 修改建议内容：修改对应 Factory
✅ 调整流程：修改 Template Method
```

### **可测试性**

```
✅ 每个模式都可独立测试
✅ Mock 对象容易创建
✅ 单元测试覆盖率高
```

---

## 📊 **实现示例预览**

```rust
// 使用示例：生成改进建议
let advice = generate_comprehensive_advice(
    &games,
    puuid,
    Some(420),  // 排位
    AdvicePerspective::SelfImprovement,  // 对自己
    None,
);

// 使用示例：生成针对建议
let advice = generate_comprehensive_advice(
    &enemy_games,
    enemy_puuid,
    Some(420),
    AdvicePerspective::Targeting,  // 对敌人
    Some("敌方上单".to_string()),
);

// 使用示例：生成协作建议
let advice = generate_comprehensive_advice(
    &teammate_games,
    teammate_puuid,
    Some(420),
    AdvicePerspective::Collaboration,  // 对队友 ⭐
    Some("队友上单".to_string()),
);
```

---

## 🎯 **完整流程（带设计模式）**

```
用户操作
    ↓
前端调用: invoke('get_match_history_with_advice', { perspective })
    ↓
┌────────────────────────────────────────────┐
│ Parser 模式：解析 LCU JSON                 │
│ → ParsedGame（含时间线数据）⭐              │
└──────────────┬─────────────────────────────┘
               ↓
┌────────────────────────────────────────────┐
│ Strategy 模式（已有）：选择分析深度        │
│ → Ranked: 完整分析                         │
│ → Other: 简化分析                          │
└──────────────┬─────────────────────────────┘
               ↓
┌────────────────────────────────────────────┐
│ Analyzer + Traits（已有）：生成特征        │
│ → 12/6 个精炼特征                          │
└──────────────┬─────────────────────────────┘
               ↓
┌────────────────────────────────────────────┐
│ 责任链模式：建议生成 ⭐                    │
│ ┌────────────────────────────────────────┐ │
│ │ LaningAnalyzer (模板方法) ⭐           │ │
│ │ → 识别问题：前期容易死                │ │
│ │ → 使用策略模式生成建议 ⭐              │ │
│ │   ├─ SelfImprovementStrategy: 改进建议│ │
│ │   ├─ TargetingStrategy: 针对建议      │ │
│ │   └─ CollaborationStrategy: 协作建议⭐│ │
│ │ → 使用建造者构建 ⭐                    │ │
│ └────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────┐ │
│ │ FarmingAnalyzer → ...                  │ │
│ └────────────────────────────────────────┘ │
│ ... 其他分析器                             │
└──────────────┬─────────────────────────────┘
               ↓
┌────────────────────────────────────────────┐
│ 汇总：5条优先级建议                        │
└──────────────┬─────────────────────────────┘
               ↓
┌────────────────────────────────────────────┐
│ 组合模式：团队战术（如果需要）⭐           │
│ → TeamTactical（整体战术）                 │
│   ├─ RoleTactical（上单）                  │
│   ├─ RoleTactical（打野）                  │
│   └─ ...                                   │
└──────────────┬─────────────────────────────┘
               ↓
返回：PlayerMatchStats + 建议 + 战术
```

---

## 💡 **设计模式的核心价值**

### **1. 责任链模式**

```
优势：
✅ 新增分析器 = 新增一个链节点
✅ 每个分析器独立，互不影响
✅ 可以动态启用/禁用分析器

示例：
想添加"装备分析"？
→ 创建 ItemAdviceAnalyzer
→ 加入责任链
→ 完成！
```

### **2. 策略模式（视角切换）**

```
优势：
✅ 三种视角统一接口
✅ 切换视角 = 切换策略
✅ 易于添加新视角

示例：
想添加"教练视角"？
→ 创建 CoachStrategy
→ 实现接口
→ 完成！
```

### **3. 建造者模式**

```
优势：
✅ 构建过程清晰
✅ 链式调用优雅
✅ 必填字段检查

示例：
AdviceBuilder::new()
    .title("...")
    .problem("...")
    .suggestion("...")
    .priority(5)
    .build()
```

### **4. 模板方法模式**

```
优势：
✅ 流程统一
✅ 代码复用
✅ 子类只实现差异部分

示例：
所有分析器都遵循：
提取数据 → 计算指标 → 识别问题 → 生成建议
```

---

## 🎯 **推荐实现方案**

### **方案：分层实现，逐步演进**

```
第1层：基础设施（1-2小时）
├─ AdviceContext（上下文）
├─ AdviceBuilder（建造者）
├─ AdviceChain（责任链）
└─ 基础 trait 定义

第2层：策略实现（2-3小时）
├─ SelfImprovementStrategy
├─ TargetingStrategy
└─ CollaborationStrategy ⭐

第3层：分析器实现（4-6小时）
├─ LaningAdviceAnalyzer
├─ FarmingAdviceAnalyzer
├─ TeamfightAdviceAnalyzer
├─ VisionAdviceAnalyzer
└─ ChampionAdviceAnalyzer

第4层：团队战术（可选，3-4小时）
└─ team_tactical/ 目录
```

**总计：10-15小时**

---

## 🎉 **总结**

### **设计模式的价值**

```
没有设计模式：
- 代码混乱
- 难以扩展
- 耦合度高
- 难以测试

使用设计模式：
✅ 代码清晰
✅ 易于扩展
✅ 低耦合
✅ 易于测试
✅ 专业优雅
```

### **推荐的设计模式组合**

```
核心：
1. 责任链模式 - 建议生成（最重要）⭐
2. 策略模式 - 视角切换
3. 建造者模式 - 建议构建

辅助：
4. 模板方法 - 流程统一
5. 工厂模式 - 对象创建
6. 组合模式 - 层次结构
```

### **实现建议**

**我建议采用这套设计模式！**

**原因**：
- ✅ 符合 SOLID 原则
- ✅ 易于扩展和维护
- ✅ 代码优雅专业
- ✅ 适合团队协作

---

**需要我帮你用这套设计模式开始实现吗？** 🚀

我可以先实现基础设施（责任链 + 建造者），然后逐步添加分析器！
