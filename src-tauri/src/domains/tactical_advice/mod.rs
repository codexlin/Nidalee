/// 智能建议系统（v3.0）
///
/// 架构：
/// - types: 数据类型定义
/// - context: 上下文对象
/// - builder: 建造者模式
/// - chain: 责任链模式
/// - factory: 工厂模式
/// - strategies: 策略模式（3种视角）
/// - analyzers: 分析器（5个责任链节点）
///
/// 核心流程：
/// 1. 构建上下文（Context）
/// 2. 创建责任链（Chain）
/// 3. 添加分析器（Analyzers）
/// 4. 执行分析，生成建议

pub mod types;
pub mod context;
pub mod builder;
pub mod chain;
pub mod factory;
pub mod strategies;
pub mod analyzers;

pub use types::{GameAdvice, AdvicePerspective};
pub use context::AdviceContext;
pub use chain::AdviceChain;
  // 从 analyzers::base 导出

use crate::shared::types::PlayerMatchStats;
use crate::domains::analysis::{ParsedGame, AnalysisStrategy};

/// 主入口：生成智能建议
///
/// 参数：
/// - stats: 玩家统计数据
/// - games: 解析后的对局数据（含时间线）
/// - role: 主要位置
/// - perspective: 建议视角（自己/敌人/队友）
/// - target_name: 目标玩家名称（针对/协作时使用）
/// - strategy: 分析策略（决定是否生成建议）
///
/// 返回：
/// - Vec<GameAdvice>: 最多5条优先级建议
pub fn generate_advice(
    stats: &PlayerMatchStats,
    games: &[ParsedGame],
    role: &str,
    perspective: AdvicePerspective,
    target_name: Option<String>,
    strategy: &AnalysisStrategy,
) -> Vec<GameAdvice> {
    // 只在排位模式下生成建议
    if !matches!(strategy, AnalysisStrategy::Ranked) {
        println!("⏭️  建议系统：非排位模式，跳过");
        return Vec::new();
    }

    println!("🎯 开始生成智能建议...");
    println!("   视角：{}", perspective.description());
    println!("   位置：{}", role);

    // 1. 构建上下文
    let context = AdviceContext::new(
        stats.clone(),
        games.to_vec(),
        role.to_string(),
        perspective,
        target_name,
    );

    // 2. 创建责任链并添加分析器
    let chain = AdviceChain::new()
        .add_analyzer(Box::new(analyzers::LaningAdviceAnalyzer))
        .add_analyzer(Box::new(analyzers::FarmingAdviceAnalyzer))
        .add_analyzer(Box::new(analyzers::TeamfightAdviceAnalyzer))
        .add_analyzer(Box::new(analyzers::VisionAdviceAnalyzer))
        .add_analyzer(Box::new(analyzers::ChampionAdviceAnalyzer));

    // 3. 执行责任链，生成建议
    let advice = chain.generate(&context, strategy);

    println!("✅ 建议生成完成：共 {} 条", advice.len());

    advice
}

