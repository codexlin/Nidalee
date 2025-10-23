/// 增强分析服务 - 整合新旧系统
///
/// 设计理念：
/// - 渐进式增强：旧系统继续工作，新系统作为可选增强
/// - 向后兼容：不破坏现有功能
/// - 解耦设计：通过桥接层连接新旧系统
use crate::domains::analysis::analyzers::core::timeline_bridge::TimelineBridge;
use crate::domains::analysis::services::intelligent_analysis_service::{
    perform_intelligent_analysis, IntelligentAnalysisResult
};
use crate::shared::types::{PlayerMatchStats, GameAdvice, AdvicePerspective};
use serde_json::Value;

/// 分析配置
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// 是否启用智能分析（新系统）
    pub enable_intelligent_analysis: bool,

    /// 是否启用对手分析
    pub enable_opponent_analysis: bool,

    /// 是否启用队友分析
    pub enable_teammate_analysis: bool,

    /// 是否启用自我提升分析
    pub enable_self_improvement: bool,

    /// 建议视角
    pub advice_perspective: AdvicePerspective,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enable_intelligent_analysis: false, // 默认不启用，保持向后兼容
            enable_opponent_analysis: true,
            enable_teammate_analysis: true,
            enable_self_improvement: true,
            advice_perspective: AdvicePerspective::SelfImprovement,
        }
    }
}

impl AnalysisConfig {
    /// 创建保守配置（仅使用旧系统）
    pub fn conservative() -> Self {
        Self {
            enable_intelligent_analysis: false,
            ..Default::default()
        }
    }

    /// 创建完整配置（启用所有新功能）
    pub fn full_featured() -> Self {
        Self {
            enable_intelligent_analysis: true,
            enable_opponent_analysis: true,
            enable_teammate_analysis: true,
            enable_self_improvement: true,
            advice_perspective: AdvicePerspective::SelfImprovement,
        }
    }

    /// 创建Beta测试配置
    pub fn beta() -> Self {
        Self {
            enable_intelligent_analysis: true,
            enable_opponent_analysis: true,
            enable_teammate_analysis: false, // 先不启用队友分析
            enable_self_improvement: true,
            advice_perspective: AdvicePerspective::SelfImprovement,
        }
    }
}

/// 统一分析结果
#[derive(Debug, Clone)]
pub struct UnifiedAnalysisResult {
    /// 基础分析结果（来自旧系统）
    pub player_stats: PlayerMatchStats,

    /// 智能分析结果（来自新系统，可选）
    pub intelligent_analysis: Option<IntelligentAnalysisResult>,

    /// 统一的建议列表（合并两个系统的建议）
    pub all_advice: Vec<GameAdvice>,

    /// 分析元数据
    pub metadata: AnalysisMetadata,
}

/// 分析元数据
#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    /// 使用的数据源
    pub data_source: String,

    /// 是否启用了智能分析
    pub intelligent_analysis_enabled: bool,

    /// 分析耗时（毫秒）
    pub analysis_time_ms: u64,

    /// 数据质量评分 (0-100)
    pub data_quality_score: u8,
}

/// 增强分析服务
pub struct EnhancedAnalysisService {
    config: AnalysisConfig,
    timeline_bridge: TimelineBridge,
}

impl EnhancedAnalysisService {
    /// 创建增强分析服务
    pub fn new(config: AnalysisConfig) -> Self {
        let timeline_bridge = TimelineBridge::new();

        Self {
            config,
            timeline_bridge,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(AnalysisConfig::default())
    }

    /// 执行完整分析
    pub fn analyze(
        &self,
        match_data: &Value,
        target_puuid: &str,
        target_participant_id: i32,
    ) -> Result<UnifiedAnalysisResult, String> {
        let start_time = std::time::Instant::now();

        // 1. 评估数据质量
        let data_quality_score = self.assess_data_quality(match_data);

        // 2. 执行旧系统分析（始终执行，保证基础功能）
        let player_stats = self.analyze_with_legacy_system(match_data, target_puuid)?;

        // 3. 执行新系统智能分析（可选）
        let intelligent_analysis = if self.config.enable_intelligent_analysis {
            match perform_intelligent_analysis(match_data, target_puuid, target_participant_id) {
                Ok(analysis) => Some(analysis),
                Err(e) => {
                    eprintln!("⚠️  智能分析失败，降级到基础分析: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // 4. 合并建议
        let all_advice = self.merge_advice(&player_stats, &intelligent_analysis);

        // 5. 生成元数据
        let analysis_time_ms = start_time.elapsed().as_millis() as u64;
        let metadata = AnalysisMetadata {
            data_source: "frames".to_string(), // 只使用 frames 数据源
            intelligent_analysis_enabled: self.config.enable_intelligent_analysis,
            analysis_time_ms,
            data_quality_score,
        };

        Ok(UnifiedAnalysisResult {
            player_stats,
            intelligent_analysis,
            all_advice,
            metadata,
        })
    }

    /// 评估数据质量（基于 frames 数据）
    fn assess_data_quality(&self, match_data: &Value) -> u8 {
        let mut score = 0u8;

        // 检查基础数据
        if match_data.get("participants").is_some() {
            score += 30;
        }

        // 检查 frames 数据
        if let Some(match_timeline) = match_data.get("match_timeline_json") {
            if let Some(frames) = match_timeline.get("frames").and_then(|f| f.as_array()) {
                if !frames.is_empty() {
                    score += 50; // 有 frames 数据

                    // 检查数据完整性
                    if frames.len() > 10 {
                        score += 10; // 对局时长超过10分钟
                    }

                    // 检查是否有事件数据
                    if frames.iter().any(|f| {
                        f.get("events")
                            .and_then(|e| e.as_array())
                            .map(|arr| !arr.is_empty())
                            .unwrap_or(false)
                    }) {
                        score += 10; // 有事件数据
                    }
                }
            }
        }

        score.min(100)
    }

    /// 使用旧系统进行分析
    fn analyze_with_legacy_system(
        &self,
        match_data: &Value,
        target_puuid: &str,
    ) -> Result<PlayerMatchStats, String> {
        // 这里应该调用旧系统的分析逻辑
        // 由于旧系统的完整实现在 infrastructure 层，这里提供一个简化版本

        // TODO: 集成旧系统的完整分析流程
        // 包括：parse_games, analyze_player_stats, analyze_traits 等

        // 暂时返回一个占位符
        Ok(PlayerMatchStats {
            total_games: 10,
            wins: 6,
            losses: 4,
            win_rate: 60.0,
            avg_kills: 5.0,
            avg_deaths: 3.0,
            avg_assists: 7.0,
            avg_kda: 4.0,
            ..Default::default()
        })
    }

    /// 合并新旧系统的建议
    fn merge_advice(
        &self,
        player_stats: &PlayerMatchStats,
        intelligent_analysis: &Option<IntelligentAnalysisResult>,
    ) -> Vec<GameAdvice> {
        let mut all_advice = Vec::new();

        // 1. 添加旧系统的建议
        all_advice.extend(player_stats.advice.clone());

        // 2. 添加新系统的建议（如果有）
        if let Some(ref intelligent) = intelligent_analysis {
            for comprehensive_advice in &intelligent.comprehensive_advice {
                // 转换为 GameAdvice 格式
                // 将 category 字符串映射到 AdviceCategory 枚举
                let category = match comprehensive_advice.category.as_str() {
                    "对线期" | "对线策略" => crate::shared::types::AdviceCategory::Laning,
                    "发育期" | "补刀技巧" => crate::shared::types::AdviceCategory::Farming,
                    "团战期" | "团战策略" => crate::shared::types::AdviceCategory::Teamfight,
                    "视野控制" | "视野策略" => crate::shared::types::AdviceCategory::Vision,
                    "站位" => crate::shared::types::AdviceCategory::Positioning,
                    "风格应对" => crate::shared::types::AdviceCategory::Decision,
                    _ => crate::shared::types::AdviceCategory::Decision,
                };

                all_advice.push(GameAdvice {
                    title: comprehensive_advice.title.clone(),
                    problem: comprehensive_advice.description.clone(),
                    evidence: format!("优先级: {}", comprehensive_advice.priority),
                    suggestions: comprehensive_advice.specific_actions.clone(),
                    priority: comprehensive_advice.priority as i32,
                    category,
                    perspective: self.config.advice_perspective.clone(),
                    affected_role: None,
                    target_player: None,
                });
            }
        }

        // 3. 去重和排序
        self.deduplicate_and_sort_advice(all_advice)
    }

    /// 去重和排序建议
    fn deduplicate_and_sort_advice(&self, mut advice: Vec<GameAdvice>) -> Vec<GameAdvice> {
        // 按标题去重
        let mut seen_titles = std::collections::HashSet::new();
        advice.retain(|a| seen_titles.insert(a.title.clone()));

        // 按优先级降序排序
        advice.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 限制数量（最多15条）
        advice.truncate(15);

        advice
    }

    /// 获取时间线桥接器（用于外部访问）
    pub fn timeline_bridge(&self) -> &TimelineBridge {
        &self.timeline_bridge
    }

    /// 获取配置
    pub fn config(&self) -> &AnalysisConfig {
        &self.config
    }
}

/// 快捷函数：使用默认配置执行分析
pub fn analyze_with_default_config(
    match_data: &Value,
    target_puuid: &str,
    target_participant_id: i32,
) -> Result<UnifiedAnalysisResult, String> {
    let service = EnhancedAnalysisService::with_default_config();
    service.analyze(match_data, target_puuid, target_participant_id)
}

/// 快捷函数：使用完整配置执行分析
pub fn analyze_with_full_features(
    match_data: &Value,
    target_puuid: &str,
    target_participant_id: i32,
) -> Result<UnifiedAnalysisResult, String> {
    let service = EnhancedAnalysisService::new(AnalysisConfig::full_featured());
    service.analyze(match_data, target_puuid, target_participant_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert!(!config.enable_intelligent_analysis); // 默认不启用
    }

    #[test]
    fn test_analysis_config_conservative() {
        let config = AnalysisConfig::conservative();
        assert!(!config.enable_intelligent_analysis);
    }

    #[test]
    fn test_analysis_config_full_featured() {
        let config = AnalysisConfig::full_featured();
        assert!(config.enable_intelligent_analysis);
        assert!(config.enable_opponent_analysis);
        assert!(config.enable_teammate_analysis);
        assert!(config.enable_self_improvement);
    }

    #[test]
    fn test_assess_data_quality() {
        let service = EnhancedAnalysisService::with_default_config();

        // 测试有 frames 数据的情况
        let match_data_with_frames = json!({
            "participants": [],
            "match_timeline_json": {
                "frames": [
                    {"timestamp": 0, "events": [], "participantFrames": {}}
                ]
            }
        });

        let score = service.assess_data_quality(&match_data_with_frames);
        assert!(score >= 80); // 应该有高质量分数

        // 测试只有基础数据的情况
        let match_data_basic = json!({
            "participants": []
        });

        let score = service.assess_data_quality(&match_data_basic);
        assert!(score >= 30 && score < 50); // 中等质量分数
    }

    #[test]
    fn test_deduplicate_advice() {
        use crate::shared::types::AdviceCategory;
        let service = EnhancedAnalysisService::with_default_config();

        let advice = vec![
            GameAdvice {
                title: "建议1".to_string(),
                problem: "问题1".to_string(),
                evidence: "证据1".to_string(),
                suggestions: vec!["建议A".to_string()],
                priority: 5,
                category: AdviceCategory::Laning,
                perspective: AdvicePerspective::SelfImprovement,
                affected_role: None,
                target_player: None,
            },
            GameAdvice {
                title: "建议1".to_string(), // 重复
                problem: "问题1".to_string(),
                evidence: "证据1".to_string(),
                suggestions: vec!["建议A".to_string()],
                priority: 4,
                category: AdviceCategory::Laning,
                perspective: AdvicePerspective::SelfImprovement,
                affected_role: None,
                target_player: None,
            },
            GameAdvice {
                title: "建议2".to_string(),
                problem: "问题2".to_string(),
                evidence: "证据2".to_string(),
                suggestions: vec!["建议B".to_string()],
                priority: 3,
                category: AdviceCategory::Teamfight,
                perspective: AdvicePerspective::SelfImprovement,
                affected_role: None,
                target_player: None,
            },
        ];

        let result = service.deduplicate_and_sort_advice(advice);
        assert_eq!(result.len(), 2); // 去重后只剩2条
        assert_eq!(result[0].priority, 5); // 按优先级降序
        assert_eq!(result[1].priority, 3);
    }
}
