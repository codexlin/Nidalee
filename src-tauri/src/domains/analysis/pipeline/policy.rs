/// 分析策略解析（唯一入口）
///
/// 规则矩阵：
/// | 条件 | 结果 |
/// |------|------|
/// | `enabled = false` | 仅基础统计，深度降级为 Simple，全部子功能关闭 |
/// | `Simple`（任意模式） | 仅基础统计，且不做时间线 |
/// | `Deep` + 仅排位队列（420/440） | 深度分析，子功能按开关生效 |
/// | `Deep` + 仅娱乐队列（如 450/2400） | 降级为 Simple，并给出降级原因 |
/// | `Deep` + 全部/混合队列 | 按局策略（排位 deep、娱乐 simple），深度结论仅排位 |
/// | 本地 AI | 只在存在排位深度证据时可用 |
///
/// 队列覆盖优先级：`queue_ids` > `queue_id` > 模式默认队列；空集合表示不过滤。
use super::types::{
    AnalysisDegradationCode, AnalysisDepth, AnalysisDiagnostic, AnalysisFeature, AnalysisPolicy, AnalysisQueueScope,
    MatchAnalysisRequest,
};

/// 解析分析策略（请求 → 策略）
pub fn resolve_analysis_policy(request: &MatchAnalysisRequest) -> AnalysisPolicy {
    let selected_queue_ids = request.requested_queue_ids();
    let queue_scope = AnalysisQueueScope::from_queue_ids(&selected_queue_ids);
    let effective_game_count = request.effective_game_count();

    let mut diagnostics = Vec::new();

    if effective_game_count < request.count {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::GameCountCapped,
            AnalysisFeature::GameCount,
            format!(
                "请求 {} 场，受性能上限限制实际分析 {} 场",
                request.count, effective_game_count
            ),
        ));
    }

    // 1. 总开关关闭：只保留基础统计
    if !request.features.enabled {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::AnalysisDisabled,
            AnalysisFeature::DeepAnalysis,
            "智能分析已关闭，仅提供基础统计",
        ));

        return AnalysisPolicy {
            mode: request.mode,
            requested_depth: request.depth,
            effective_depth: AnalysisDepth::Simple,
            selected_queue_ids,
            queue_scope,
            effective_game_count,
            basic_only: true,
            per_game_depth: false,
            deep_conclusions_ranked_only: false,
            enable_timeline: false,
            enable_opponent: false,
            enable_teammate: false,
            enable_self_improvement: false,
            local_ai_eligible: false,
            diagnostics,
        };
    }

    // 2. 功能开关关闭的诊断（与深度判定相互独立）
    push_feature_flag_diagnostics(request, &mut diagnostics);

    // 3. 深度判定
    let (effective_depth, basic_only, per_game_depth) = match request.depth {
        AnalysisDepth::Simple => {
            diagnostics.push(AnalysisDiagnostic::with_feature(
                AnalysisDegradationCode::SimpleDepthRequested,
                AnalysisFeature::DeepAnalysis,
                "已选择简单分析：仅基础统计，不做时间线与深度结论",
            ));
            (AnalysisDepth::Simple, true, false)
        }
        AnalysisDepth::Deep => match queue_scope {
            // 娱乐/匹配等非排位队列没有稳定的深度证据，降级为简单分析
            AnalysisQueueScope::NonRankedOnly => {
                diagnostics.push(AnalysisDiagnostic::with_feature(
                    AnalysisDegradationCode::FunModeDeepUnsupported,
                    AnalysisFeature::DeepAnalysis,
                    format!(
                        "队列 {:?} 属于娱乐/非排位模式，缺少可靠的深度分析依据，已降级为简单分析",
                        selected_queue_ids
                    ),
                ));
                (AnalysisDepth::Simple, true, false)
            }
            // 纯排位：完整深度分析
            AnalysisQueueScope::RankedOnly => (AnalysisDepth::Deep, false, false),
            // 全部/混合：按局决定深度
            AnalysisQueueScope::Unfiltered | AnalysisQueueScope::Mixed => {
                diagnostics.push(AnalysisDiagnostic::with_feature(
                    AnalysisDegradationCode::PerGameDepthApplied,
                    AnalysisFeature::DeepAnalysis,
                    "包含非排位对局：排位局使用深度分析、娱乐局使用简单分析，深度结论仅来自排位局",
                ));
                (AnalysisDepth::Deep, false, true)
            }
        },
    };

    let deep_analysis = !basic_only && effective_depth == AnalysisDepth::Deep;

    // 4. 子功能：深度层功能受深度约束，其余仅受开关约束
    let enable_timeline = deep_analysis && request.features.timeline;
    let enable_opponent = request.features.opponent;
    let enable_teammate = request.features.teammate;
    let enable_self_improvement = request.features.self_improvement;

    // 5. 本地 AI：必须具备排位深度证据（此处判定前提，运行期再由 capabilities 收敛）
    let local_ai_eligible = deep_analysis && queue_scope.may_contain_ranked();
    if request.depth == AnalysisDepth::Deep && !local_ai_eligible {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::LocalAiRequiresRankedDeepEvidence,
            AnalysisFeature::LocalAi,
            "本地 AI 能力需要排位深度证据，当前策略无法产生该证据",
        ));
    }

    AnalysisPolicy {
        mode: request.mode,
        requested_depth: request.depth,
        effective_depth,
        selected_queue_ids,
        queue_scope,
        effective_game_count,
        basic_only,
        per_game_depth,
        deep_conclusions_ranked_only: deep_analysis,
        enable_timeline,
        enable_opponent,
        enable_teammate,
        enable_self_improvement,
        local_ai_eligible,
        diagnostics,
    }
}

/// 记录被功能开关关闭的能力
fn push_feature_flag_diagnostics(request: &MatchAnalysisRequest, diagnostics: &mut Vec<AnalysisDiagnostic>) {
    let flags = &request.features;

    if !flags.timeline {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::TimelineDisabledByFeatureFlag,
            AnalysisFeature::Timeline,
            "时间线分析已被设置关闭",
        ));
    }
    if !flags.opponent {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::OpponentDisabledByFeatureFlag,
            AnalysisFeature::Opponent,
            "对手分析已被设置关闭",
        ));
    }
    if !flags.teammate {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::TeammateDisabledByFeatureFlag,
            AnalysisFeature::Teammate,
            "队友分析已被设置关闭",
        ));
    }
    if !flags.self_improvement {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::SelfImprovementDisabledByFeatureFlag,
            AnalysisFeature::SelfImprovement,
            "自我提升分析已被设置关闭",
        ));
    }
}
