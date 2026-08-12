//! 统一分析契约 - 策略矩阵测试
//!
//! 覆盖：420 / 440 / 混合排位 / 全部模式 / 450 / 2400 / enabled=false /
//! 功能开关关闭（timeline、opponent、teammate、selfImprovement）/ 性能上限 /
//! 本地 AI 能力收敛。

use nidalee_lib::analysis_contract::{
    is_ranked_queue, resolve_analysis_policy, AnalysisCapabilities, AnalysisDegradationCode, AnalysisDepth,
    AnalysisFeature, AnalysisFeatureFlags, AnalysisMode, AnalysisQueueScope, MatchAnalysisRequest,
};

fn request(mode: AnalysisMode, depth: AnalysisDepth) -> MatchAnalysisRequest {
    MatchAnalysisRequest::new(mode, depth)
}

// === enabled = false ===

#[test]
fn test_disabled_yields_basic_only() {
    let mut req = request(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    req.features = AnalysisFeatureFlags::all_disabled();

    let policy = resolve_analysis_policy(&req);

    assert!(policy.basic_only, "总开关关闭时必须只做基础统计");
    assert_eq!(policy.effective_depth, AnalysisDepth::Simple);
    assert_eq!(policy.requested_depth, AnalysisDepth::Deep);
    assert!(!policy.enable_timeline);
    assert!(!policy.enable_opponent);
    assert!(!policy.enable_teammate);
    assert!(!policy.enable_self_improvement);
    assert!(!policy.per_game_depth);
    assert!(!policy.local_ai_eligible);
    assert!(policy.has_diagnostic(AnalysisDegradationCode::AnalysisDisabled));

    let caps = AnalysisCapabilities::from_policy(&policy);
    assert!(caps.basic_stats);
    assert!(!caps.deep_analysis);
    assert!(!caps.local_ai);
}

#[test]
fn test_disabled_but_feature_flags_on_still_basic_only() {
    let mut req = request(AnalysisMode::MixedRanked, AnalysisDepth::Deep);
    req.features = AnalysisFeatureFlags {
        enabled: false,
        ..AnalysisFeatureFlags::default()
    };

    let policy = resolve_analysis_policy(&req);

    assert!(policy.basic_only);
    assert!(!policy.enable_timeline);
    assert!(!policy.enable_opponent);
}

// === Simple：所有模式仅基础且无时间线 ===

#[test]
fn test_simple_depth_is_basic_for_all_modes() {
    let modes = [
        AnalysisMode::SoloRanked,
        AnalysisMode::FlexRanked,
        AnalysisMode::MixedRanked,
        AnalysisMode::Aram,
        AnalysisMode::Normals,
        AnalysisMode::AllModes,
    ];

    for mode in modes {
        let policy = resolve_analysis_policy(&request(mode, AnalysisDepth::Simple));

        assert!(policy.basic_only, "{:?} Simple 应只做基础统计", mode);
        assert_eq!(policy.effective_depth, AnalysisDepth::Simple);
        assert!(!policy.enable_timeline, "{:?} Simple 不应做时间线", mode);
        assert!(!policy.per_game_depth);
        assert!(!policy.local_ai_eligible, "{:?} Simple 无排位深度证据", mode);
        assert!(policy.has_diagnostic(AnalysisDegradationCode::SimpleDepthRequested));
    }
}

#[test]
fn test_simple_depth_keeps_queue_filter() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::SoloRanked, AnalysisDepth::Simple));

    assert_eq!(policy.selected_queue_ids, vec![420]);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::RankedOnly);
    assert_eq!(policy.depth_for_queue(420), AnalysisDepth::Simple);
}

// === Deep + 排位 ===

#[test]
fn test_deep_solo_ranked_420() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::SoloRanked, AnalysisDepth::Deep));

    assert_eq!(policy.selected_queue_ids, vec![420]);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::RankedOnly);
    assert_eq!(policy.effective_depth, AnalysisDepth::Deep);
    assert!(!policy.basic_only);
    assert!(!policy.per_game_depth, "纯排位无需按局降级");
    assert!(policy.deep_conclusions_ranked_only);
    assert!(policy.enable_timeline);
    assert!(policy.enable_opponent);
    assert!(policy.enable_teammate);
    assert!(policy.enable_self_improvement);
    assert!(policy.local_ai_eligible);
    assert!(
        policy.diagnostics.is_empty(),
        "默认排位深度分析不应产生降级诊断，实际: {:?}",
        policy.diagnostics
    );

    assert_eq!(policy.depth_for_queue(420), AnalysisDepth::Deep);
    assert!(policy.includes_queue(420));
    assert!(!policy.includes_queue(450));
}

#[test]
fn test_deep_flex_ranked_440() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::FlexRanked, AnalysisDepth::Deep));

    assert_eq!(policy.selected_queue_ids, vec![440]);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::RankedOnly);
    assert_eq!(policy.effective_depth, AnalysisDepth::Deep);
    assert!(policy.local_ai_eligible);
    assert_eq!(policy.depth_for_queue(440), AnalysisDepth::Deep);
    assert!(policy.timeline_enabled_for_queue(440));
}

#[test]
fn test_deep_mixed_ranked_420_440() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::MixedRanked, AnalysisDepth::Deep));

    assert_eq!(policy.selected_queue_ids, vec![420, 440]);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::RankedOnly);
    assert_eq!(policy.effective_depth, AnalysisDepth::Deep);
    assert!(!policy.per_game_depth, "混合排位全部是排位，不需要按局策略");
    assert!(policy.deep_conclusions_ranked_only);
    assert!(policy.local_ai_eligible);
    assert_eq!(policy.depth_for_queue(420), AnalysisDepth::Deep);
    assert_eq!(policy.depth_for_queue(440), AnalysisDepth::Deep);
}

// === Deep + 娱乐模式降级 ===

#[test]
fn test_deep_normals_excludes_ranked_and_downgrades() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::Normals, AnalysisDepth::Deep));

    assert!(policy.selected_queue_ids.is_empty());
    assert!(policy.exclude_ranked);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::NonRankedOnly);
    assert_eq!(policy.effective_depth, AnalysisDepth::Simple);
    assert!(policy.basic_only);
    assert!(policy.includes_queue(430));
    assert!(policy.includes_queue(450));
    assert!(!policy.includes_queue(420));
    assert!(!policy.includes_queue(440));
    assert!(policy.has_diagnostic(AnalysisDegradationCode::FunModeDeepUnsupported));
}

#[test]
fn test_deep_aram_450_downgrades_to_simple() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::Aram, AnalysisDepth::Deep));

    assert_eq!(policy.selected_queue_ids, vec![450]);
    assert!(!policy.exclude_ranked);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::NonRankedOnly);
    assert_eq!(policy.requested_depth, AnalysisDepth::Deep);
    assert_eq!(
        policy.effective_depth,
        AnalysisDepth::Simple,
        "娱乐模式必须降级为简单分析"
    );
    assert!(policy.basic_only);
    assert!(!policy.enable_timeline);
    assert!(!policy.per_game_depth);
    assert!(!policy.deep_conclusions_ranked_only);
    assert!(!policy.local_ai_eligible);
    assert!(policy.has_diagnostic(AnalysisDegradationCode::FunModeDeepUnsupported));
    assert!(policy.has_diagnostic(AnalysisDegradationCode::LocalAiRequiresRankedDeepEvidence));

    let reason = policy
        .diagnostics
        .iter()
        .find(|d| d.code == AnalysisDegradationCode::FunModeDeepUnsupported)
        .expect("必须给出降级原因");
    assert!(!reason.message.is_empty(), "降级原因不能为空");
    assert_eq!(reason.feature, Some(AnalysisFeature::DeepAnalysis));

    let caps = AnalysisCapabilities::from_policy(&policy);
    assert!(!caps.deep_analysis);
    assert!(!caps.ranked_deep_evidence);
    assert!(!caps.local_ai);
}

#[test]
fn test_deep_queue_override_2400_downgrades_to_simple() {
    let mut req = request(AnalysisMode::AllModes, AnalysisDepth::Deep);
    req.queue_id = Some(2400);

    let policy = resolve_analysis_policy(&req);

    assert_eq!(policy.selected_queue_ids, vec![2400]);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::NonRankedOnly);
    assert_eq!(policy.effective_depth, AnalysisDepth::Simple);
    assert!(policy.basic_only);
    assert!(!policy.per_game_depth);
    assert!(!policy.local_ai_eligible);
    assert!(policy.has_diagnostic(AnalysisDegradationCode::FunModeDeepUnsupported));
}

// === Deep + 全部模式：按局策略 ===

#[test]
fn test_deep_all_modes_uses_per_game_depth() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::AllModes, AnalysisDepth::Deep));

    assert!(policy.selected_queue_ids.is_empty(), "全部模式不过滤队列");
    assert_eq!(policy.queue_scope, AnalysisQueueScope::Unfiltered);
    assert_eq!(policy.effective_depth, AnalysisDepth::Deep);
    assert!(!policy.basic_only);
    assert!(policy.per_game_depth, "全部模式应按局决定深度");
    assert!(policy.deep_conclusions_ranked_only, "深度结论仅来自排位");
    assert!(policy.has_diagnostic(AnalysisDegradationCode::PerGameDepthApplied));

    assert_eq!(policy.depth_for_queue(420), AnalysisDepth::Deep);
    assert_eq!(policy.depth_for_queue(440), AnalysisDepth::Deep);
    assert_eq!(policy.depth_for_queue(450), AnalysisDepth::Simple);
    assert_eq!(policy.depth_for_queue(2400), AnalysisDepth::Simple);

    assert!(policy.timeline_enabled_for_queue(420));
    assert!(!policy.timeline_enabled_for_queue(450));

    assert!(policy.includes_queue(450), "不过滤时任何队列都参与");
    assert!(policy.local_ai_eligible);
}

#[test]
fn test_deep_mixed_override_ranked_and_fun_uses_per_game_depth() {
    let mut req = request(AnalysisMode::AllModes, AnalysisDepth::Deep);
    req.queue_ids = Some(vec![420, 450]);

    let policy = resolve_analysis_policy(&req);

    assert_eq!(policy.queue_scope, AnalysisQueueScope::Mixed);
    assert!(policy.per_game_depth);
    assert_eq!(policy.effective_depth, AnalysisDepth::Deep);
    assert!(policy.deep_conclusions_ranked_only);
    assert_eq!(policy.depth_for_queue(420), AnalysisDepth::Deep);
    assert_eq!(policy.depth_for_queue(450), AnalysisDepth::Simple);
    assert!(policy.includes_queue(450));
    assert!(!policy.includes_queue(440));
}

// === 队列覆盖优先级 ===

#[test]
fn test_queue_ids_override_takes_precedence() {
    let mut req = request(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    req.queue_id = Some(450);
    req.queue_ids = Some(vec![440]);

    let policy = resolve_analysis_policy(&req);

    assert_eq!(policy.selected_queue_ids, vec![440]);
    assert_eq!(policy.queue_scope, AnalysisQueueScope::RankedOnly);
}

#[test]
fn test_empty_queue_ids_override_means_unfiltered() {
    let mut req = request(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    req.queue_ids = Some(vec![]);

    let policy = resolve_analysis_policy(&req);

    assert!(policy.selected_queue_ids.is_empty());
    assert_eq!(policy.queue_scope, AnalysisQueueScope::Unfiltered);
    assert!(policy.per_game_depth);
}

// === 功能开关 ===

#[test]
fn test_timeline_feature_flag_disabled() {
    let mut req = request(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    req.features.timeline = false;

    let policy = resolve_analysis_policy(&req);

    assert_eq!(policy.effective_depth, AnalysisDepth::Deep);
    assert!(!policy.enable_timeline);
    assert!(policy.enable_opponent, "关闭时间线不应影响其他功能");
    assert!(policy.has_diagnostic(AnalysisDegradationCode::TimelineDisabledByFeatureFlag));
    assert!(!policy.timeline_enabled_for_queue(420));
    assert!(policy.local_ai_eligible, "时间线关闭不影响排位深度证据");

    let caps = AnalysisCapabilities::from_policy(&policy);
    assert!(!caps.timeline);
    assert!(caps.deep_analysis);
}

#[test]
fn test_opponent_feature_flag_disabled() {
    let mut req = request(AnalysisMode::FlexRanked, AnalysisDepth::Deep);
    req.features.opponent = false;

    let policy = resolve_analysis_policy(&req);

    assert!(!policy.enable_opponent);
    assert!(policy.enable_timeline);
    assert!(policy.enable_teammate);
    assert!(policy.has_diagnostic(AnalysisDegradationCode::OpponentDisabledByFeatureFlag));
}

#[test]
fn test_teammate_and_self_improvement_flags_disabled() {
    let mut req = request(AnalysisMode::MixedRanked, AnalysisDepth::Deep);
    req.features.teammate = false;
    req.features.self_improvement = false;

    let policy = resolve_analysis_policy(&req);

    assert!(!policy.enable_teammate);
    assert!(!policy.enable_self_improvement);
    assert!(policy.has_diagnostic(AnalysisDegradationCode::TeammateDisabledByFeatureFlag));
    assert!(policy.has_diagnostic(AnalysisDegradationCode::SelfImprovementDisabledByFeatureFlag));

    let caps = AnalysisCapabilities::from_policy(&policy);
    assert!(!caps.teammate);
    assert!(!caps.self_improvement);
    assert!(caps.deep_analysis);
}

// === 性能上限 ===

#[test]
fn test_max_analysis_games_caps_effective_count() {
    let mut req = request(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    req.count = 50;
    req.max_analysis_games = Some(20);

    let policy = resolve_analysis_policy(&req);

    assert_eq!(policy.effective_game_count, 20);
    assert!(policy.has_diagnostic(AnalysisDegradationCode::GameCountCapped));
}

#[test]
fn test_max_analysis_games_above_count_is_not_capped() {
    let mut req = request(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    req.count = 10;
    req.max_analysis_games = Some(30);

    let policy = resolve_analysis_policy(&req);

    assert_eq!(policy.effective_game_count, 10);
    assert!(!policy.has_diagnostic(AnalysisDegradationCode::GameCountCapped));
}

// === 能力收敛：本地 AI 只对排位深度证据可用 ===

#[test]
fn test_local_ai_requires_observed_ranked_games() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::AllModes, AnalysisDepth::Deep));
    let potential = AnalysisCapabilities::from_policy(&policy);
    assert!(potential.local_ai, "全部模式深度下 AI 属于潜在可用");

    let only_fun = potential.refined_with_observed_queues(&[450, 2400]);
    assert!(!only_fun.ranked_deep_evidence);
    assert!(!only_fun.local_ai, "只有娱乐对局时本地 AI 不可用");

    let with_ranked = AnalysisCapabilities::from_policy(&policy).refined_with_observed_queues(&[450, 440]);
    assert!(with_ranked.ranked_deep_evidence);
    assert!(with_ranked.local_ai);

    let no_games = AnalysisCapabilities::from_policy(&policy).refined_with_observed_queues(&[]);
    assert!(!no_games.local_ai);
}

#[test]
fn test_local_ai_unavailable_when_simple_depth() {
    let policy = resolve_analysis_policy(&request(AnalysisMode::SoloRanked, AnalysisDepth::Simple));
    let caps = AnalysisCapabilities::from_policy(&policy).refined_with_observed_queues(&[420]);

    assert!(!caps.deep_analysis);
    assert!(!caps.ranked_deep_evidence, "简单深度不构成排位深度证据");
    assert!(!caps.local_ai);
}

// === 请求侧辅助函数 ===

#[test]
fn test_request_helpers() {
    let mut req = request(AnalysisMode::Aram, AnalysisDepth::Deep);
    assert_eq!(req.requested_queue_ids(), vec![450]);
    assert_eq!(req.effective_game_count(), req.count);

    req.max_analysis_games = Some(5);
    assert_eq!(req.effective_game_count(), 5);

    assert!(is_ranked_queue(420));
    assert!(is_ranked_queue(440));
    assert!(!is_ranked_queue(450));
    assert!(!is_ranked_queue(2400));
}

// === 空结果契约 ===

#[test]
fn test_empty_result_carries_policy_and_diagnostics() {
    use nidalee_lib::analysis_contract::MatchAnalysisResult;

    let policy = resolve_analysis_policy(&request(AnalysisMode::Aram, AnalysisDepth::Deep));
    let result = MatchAnalysisResult::empty(policy.clone());

    assert_eq!(result.analyzed_games, 0);
    assert!(result.position_stats.is_empty());
    assert_eq!(result.overall_stats.total_games, 0);
    assert_eq!(result.policy.effective_depth, AnalysisDepth::Simple);
    assert!(!result.capabilities.deep_analysis);
    assert!(!result.capabilities.local_ai);
    assert!(!result.diagnostics.is_empty(), "空结果也要解释降级原因");
    assert!(result.evidence.is_none());
    assert!(result.ai_insight.is_none());
}
