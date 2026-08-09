//! 唯一分析编排器 + `analyze_matches` 应用服务测试
//!
//! 覆盖：一次分析只请求一次战绩列表 / 展示场次不被 `maxAnalysisGames` 截断 /
//! 混合队列只对排位做深度证据 / 娱乐 Deep 降级诊断 / 缺时间线仍有基础结果 /
//! 单局证据失败只降级该局 / 确定性特征携带证据且小样本不下强结论 /
//! 建议视角透传 / 新旧入口在同一份数据上结论一致 / Tauri 命令 serde 契约。
//!
//! 全部通过可注入的 `MatchDataSource` 完成，不依赖运行中的英雄联盟客户端。
//!
//! 运行：`cargo test --test analysis_orchestrator`

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use nidalee_lib::analysis_contract::{
    resolve_analysis_policy, AdvicePerspective, AnalysisDegradationCode, AnalysisDepth, AnalysisMode,
    MatchAnalysisRequest, MatchAnalysisResult, TraitSentiment,
};
use nidalee_lib::match_analysis::{
    analyze_matches_with_fetcher, legacy_analysis_request, legacy_overview_request, tactical_advice_request,
    to_multi_position_analysis,
};
use nidalee_lib::match_fetching::{MatchDataSource, MatchFetcher};

const ME: &str = "00000000-0000-4000-8000-000000000001";
const ENEMY: &str = "00000000-0000-4000-8000-000000000006";

// === 夹具：既能被 parser 解析、也能被 evidence 提取的最小对局 ===

/// 单局的可调参数（只暴露测试真正关心的量）
#[derive(Debug, Clone, Copy)]
struct GameSpec {
    game_id: u64,
    queue_id: i64,
    win: bool,
    /// 对线期（0-10 分钟）补刀，用于控制 cs/min 与对线优势
    early_cs: i64,
    /// 目标玩家的死亡数
    deaths: u32,
}

impl GameSpec {
    fn ranked(game_id: u64) -> Self {
        Self {
            game_id,
            queue_id: 440,
            win: true,
            early_cs: 80,
            deaths: 2,
        }
    }

    fn queue(mut self, queue_id: i64) -> Self {
        self.queue_id = queue_id;
        self
    }

    fn early_cs(mut self, early_cs: i64) -> Self {
        self.early_cs = early_cs;
        self
    }

    fn deaths(mut self, deaths: u32) -> Self {
        self.deaths = deaths;
        self
    }
}

fn participant(participant_id: i32, team_id: i32, win: bool, role: &str, lane: &str) -> Value {
    json!({
        "participantId": participant_id,
        "championId": 60 + participant_id,
        "teamId": team_id,
        "stats": {
            "win": win,
            "kills": 6,
            "deaths": 3,
            "assists": 8,
            "totalDamageDealtToChampions": 20000,
            "totalDamageTaken": 22000,
            "goldEarned": 12000,
            "visionScore": 30,
            "wardsPlaced": 10,
            "wardsKilled": 3,
            "totalMinionsKilled": 180,
            "neutralMinionsKilled": 4,
        },
        "timeline": { "role": role, "lane": lane },
    })
}

fn game_with_role(spec: GameSpec, role: &str, lane: &str) -> Value {
    json!({
        "gameId": spec.game_id,
        "queueId": spec.queue_id,
        "gameDuration": 1800,
        "gameCreation": 1_700_000_000_000i64 + spec.game_id as i64,
        "participantIdentities": [
            { "participantId": 1, "player": { "puuid": ME } },
            { "participantId": 6, "player": { "puuid": ENEMY } },
        ],
        "participants": [
            participant(1, 100, spec.win, role, lane),
            participant(6, 200, !spec.win, role, lane),
        ],
    })
}

fn game(spec: GameSpec) -> Value {
    game_with_role(spec, "SOLO", "TOP")
}

/// 目标玩家不在 `participants` 里的畸形对局：证据提取必须失败但不拖垮整批
fn game_without_target_participant(game_id: u64) -> Value {
    json!({
        "gameId": game_id,
        "queueId": 440,
        "gameDuration": 1800,
        "gameCreation": 1_700_000_000_000i64,
        "participantIdentities": [
            { "participantId": 1, "player": { "puuid": ME } },
        ],
        "participants": [participant(6, 200, true, "SOLO", "TOP")],
    })
}

fn participant_frame(cs: i64, gold: i64, xp: i64, level: i32, x: f64) -> Value {
    json!({
        "minionsKilled": cs,
        "jungleMinionsKilled": 0,
        "totalGold": gold,
        "xp": xp,
        "level": level,
        "position": { "x": x, "y": x },
    })
}

/// 0 / 10 / 20 / 30 分钟四帧的时间线，目标玩家对线期领先
fn timeline(spec: GameSpec) -> Value {
    let deaths: Vec<Value> = (0..spec.deaths)
        .map(|index| {
            json!({
                "type": "CHAMPION_KILL",
                "timestamp": 300_000 + index as i64 * 60_000,
                "killerId": 6,
                "victimId": 1,
            })
        })
        .collect();

    json!({
        "frames": [
            {
                "timestamp": 0,
                "events": [],
                "participantFrames": {
                    "1": participant_frame(0, 500, 0, 1, 500.0),
                    "6": participant_frame(0, 500, 0, 1, 600.0),
                },
            },
            {
                "timestamp": 600_000,
                "events": deaths,
                "participantFrames": {
                    "1": participant_frame(spec.early_cs, 5000, 6000, 9, 5000.0),
                    "6": participant_frame(50, 4000, 5000, 8, 5100.0),
                },
            },
            {
                "timestamp": 1_200_000,
                "events": [],
                "participantFrames": {
                    "1": participant_frame(spec.early_cs + 90, 9000, 11000, 13, 7000.0),
                    "6": participant_frame(140, 7500, 9500, 12, 7100.0),
                },
            },
            {
                "timestamp": 1_800_000,
                "events": [],
                "participantFrames": {
                    "1": participant_frame(spec.early_cs + 170, 13000, 16000, 16, 9000.0),
                    "6": participant_frame(220, 11000, 14000, 15, 9100.0),
                },
            },
        ]
    })
}

fn list_response(games: Vec<Value>) -> Value {
    json!({
        "games": {
            "gameCount": games.len(),
            "gameIndexBegin": 0,
            "games": games,
        }
    })
}

// === 可注入数据源 ===

#[derive(Default, Debug)]
struct Calls {
    list: Vec<usize>,
    detail: Vec<u64>,
    timeline: Vec<u64>,
}

#[derive(Default)]
struct MockSource {
    list: Value,
    timelines: HashMap<u64, Value>,
    failing_timelines: HashSet<u64>,
    calls: Arc<Mutex<Calls>>,
}

impl MockSource {
    /// 由若干 spec 生成「战绩列表 + 对应时间线」
    fn from_specs(specs: &[GameSpec]) -> Self {
        let mut timelines = HashMap::new();
        let mut games = Vec::new();
        for spec in specs {
            games.push(game(*spec));
            timelines.insert(spec.game_id, timeline(*spec));
        }

        Self {
            list: list_response(games),
            timelines,
            ..Default::default()
        }
    }

    fn with_extra_game(mut self, extra: Value) -> Self {
        self.list["games"]["games"]
            .as_array_mut()
            .expect("列表缺少 games.games")
            .push(extra);
        self
    }

    fn failing_timelines(mut self, ids: &[u64]) -> Self {
        self.failing_timelines = ids.iter().copied().collect();
        self
    }

    fn calls(&self) -> Arc<Mutex<Calls>> {
        self.calls.clone()
    }
}

impl MatchDataSource for MockSource {
    fn fetch_match_list_raw(
        &self,
        _puuid: &str,
        end_index: usize,
    ) -> impl Future<Output = Result<Value, String>> + Send {
        self.calls.lock().unwrap().list.push(end_index);
        let result = Ok(self.list.clone());
        async move { result }
    }

    fn fetch_game_detail_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send {
        self.calls.lock().unwrap().detail.push(game_id);
        async move { Err(format!("测试数据源没有单局详情: {game_id}")) }
    }

    fn fetch_game_timeline_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send {
        self.calls.lock().unwrap().timeline.push(game_id);
        let result = if self.failing_timelines.contains(&game_id) {
            Err(format!("时间线不可用: {game_id}"))
        } else {
            self.timelines
                .get(&game_id)
                .cloned()
                .ok_or_else(|| format!("测试数据源没有时间线: {game_id}"))
        };
        async move { result }
    }
}

async fn analyze(source: MockSource, request: &MatchAnalysisRequest) -> (MatchAnalysisResult, Arc<Mutex<Calls>>) {
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);
    let result = analyze_matches_with_fetcher(&fetcher, ME, request)
        .await
        .expect("分析失败");
    (result, calls)
}

fn ranked_request(count: u32) -> MatchAnalysisRequest {
    let mut request = MatchAnalysisRequest::new(AnalysisMode::FlexRanked, AnalysisDepth::Deep);
    request.count = count;
    request
}

// === 1. 一次分析只请求一次战绩列表 ===

#[tokio::test]
async fn test_analysis_issues_exactly_one_match_list_request() {
    let specs: Vec<GameSpec> = (1..=6).map(GameSpec::ranked).collect();
    let (_, calls) = analyze(MockSource::from_specs(&specs), &ranked_request(6)).await;

    let calls = calls.lock().unwrap();
    assert_eq!(
        calls.list.len(),
        1,
        "一次分析只能请求一次战绩列表，实际 {:?}",
        calls.list
    );
    assert!(calls.detail.is_empty(), "列表已含完整对局时不得再请求单局详情");
}

// === 2. 展示场次不被 maxAnalysisGames 截断 ===

#[tokio::test]
async fn test_display_games_are_not_capped_by_max_analysis_games() {
    let specs: Vec<GameSpec> = (1..=20).map(GameSpec::ranked).collect();
    let mut request = ranked_request(20);
    request.max_analysis_games = Some(5);

    let (result, calls) = analyze(MockSource::from_specs(&specs), &request).await;

    assert_eq!(
        result.display_games, 20,
        "展示场数必须是 count，而不是 maxAnalysisGames"
    );
    assert_eq!(result.overall_stats.total_games, 20, "基础统计必须覆盖全部展示场次");
    assert_eq!(result.matches.len(), 20, "返回的战绩列表必须是展示场次");

    let evidence = result.evidence.as_ref().expect("排位深度分析必须有证据");
    assert_eq!(evidence.match_count, 5, "深度证据受 maxAnalysisGames 约束");
    assert_eq!(result.analyzed_games, 5, "analyzedGames 是深度证据场数");
    assert_eq!(
        calls.lock().unwrap().timeline.len(),
        5,
        "时间线请求数必须受 maxAnalysisGames 约束"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == AnalysisDegradationCode::GameCountCapped),
        "被性能上限截断必须留下诊断"
    );
}

// === 3. 全部模式：深度证据只来自排位 ===

#[tokio::test]
async fn test_all_modes_keeps_fun_games_in_stats_but_deep_evidence_is_ranked_only() {
    let specs = [
        GameSpec::ranked(1).queue(420),
        GameSpec::ranked(2).queue(450),
        GameSpec::ranked(3).queue(440),
        GameSpec::ranked(4).queue(2400),
    ];
    let mut request = MatchAnalysisRequest::new(AnalysisMode::AllModes, AnalysisDepth::Deep);
    request.count = 4;

    let (result, calls) = analyze(MockSource::from_specs(&specs), &request).await;

    assert_eq!(result.overall_stats.total_games, 4, "全部模式的基础统计包含娱乐局");
    let evidence = result.evidence.as_ref().expect("混合模式仍有排位深度证据");
    let queues: Vec<i64> = evidence.matches.iter().map(|m| m.queue_id).collect();
    assert_eq!(queues, vec![420, 440], "深度证据只允许来自排位队列");
    assert_eq!(
        calls.lock().unwrap().timeline.clone(),
        vec![1, 3],
        "娱乐局不得请求时间线"
    );
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == AnalysisDegradationCode::PerGameDepthApplied));
}

// === 4. 娱乐模式 Deep：降级并给出诊断 ===

#[tokio::test]
async fn test_fun_queue_deep_request_degrades_with_diagnostic() {
    let specs = [
        GameSpec::ranked(1).queue(450),
        GameSpec::ranked(2).queue(450),
        GameSpec::ranked(3).queue(450),
    ];
    let mut request = MatchAnalysisRequest::new(AnalysisMode::Aram, AnalysisDepth::Deep);
    request.count = 3;

    let (result, calls) = analyze(MockSource::from_specs(&specs), &request).await;

    assert_eq!(result.overall_stats.total_games, 3, "降级后仍要给出基础统计");
    assert_eq!(result.display_games, 3);
    assert!(result.evidence.is_none(), "仅基础统计的策略不产出深度证据");
    assert!(!result.capabilities.deep_analysis);
    assert!(!result.capabilities.local_ai);
    assert!(calls.lock().unwrap().timeline.is_empty(), "娱乐模式零时间线请求");
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == AnalysisDegradationCode::FunModeDeepUnsupported));
    assert!(
        result.traits.iter().any(|t| t.key.starts_with("mode_affinity")),
        "娱乐局应有模式身份特征，实际 {:?}",
        result.traits.iter().map(|t| t.key.clone()).collect::<Vec<_>>()
    );
    assert!(
        result.traits.iter().all(|t| !t.key.starts_with("laning")),
        "无 Evidence 时不得出现对线类特征"
    );
    assert!(result.advice.is_empty(), "娱乐身份/表现特征不产出排位建议");
}

// === 5. 时间线缺失：仍有基础结果 ===

#[tokio::test]
async fn test_missing_timeline_still_produces_basic_result() {
    let specs: Vec<GameSpec> = (1..=3).map(GameSpec::ranked).collect();
    let source = MockSource::from_specs(&specs).failing_timelines(&[1, 2, 3]);

    let (result, _) = analyze(source, &ranked_request(3)).await;

    assert_eq!(result.overall_stats.total_games, 3, "时间线失败不得影响基础统计");
    assert_eq!(result.matches.len(), 3);
    let evidence = result.evidence.as_ref().expect("没有时间线也要有详情级证据");
    assert_eq!(evidence.match_count, 3);
    assert!(
        evidence.matches.iter().all(|m| m.phases.is_empty()),
        "没有时间线就不能编出分阶段速率"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == AnalysisDegradationCode::TimelineDataUnavailable),
        "时间线失败必须留下诊断，实际 {:?}",
        result.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

// === 6. 单局证据失败只降级该局 ===

#[tokio::test]
async fn test_single_evidence_failure_does_not_fail_the_batch() {
    let specs: Vec<GameSpec> = (1..=2).map(GameSpec::ranked).collect();
    let source = MockSource::from_specs(&specs).with_extra_game(game_without_target_participant(99));

    let (result, _) = analyze(source, &ranked_request(3)).await;

    let evidence = result.evidence.as_ref().expect("其余对局仍要有证据");
    assert_eq!(evidence.match_count, 2, "只跳过提取失败的那一局");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == AnalysisDegradationCode::EvidenceExtractionFailed),
        "单局提取失败必须留下诊断"
    );
}

// === 7. 特征携带证据；小样本不下强结论 ===

#[tokio::test]
async fn test_traits_carry_evidence_and_sufficient_sample_supports_conclusion() {
    let specs: Vec<GameSpec> = (1..=6).map(GameSpec::ranked).collect();
    let (result, _) = analyze(MockSource::from_specs(&specs), &ranked_request(6)).await;

    assert!(!result.traits.is_empty(), "排位深度分析必须产出确定性特征");
    for item in &result.traits {
        assert_eq!(
            item.sample_count as usize,
            item.evidence_game_ids.len(),
            "特征 {} 的样本量必须与证据对局一一对应",
            item.key
        );
        assert!(!item.evidence_game_ids.is_empty(), "特征 {} 缺少证据对局", item.key);
        assert!((0.0..=1.0).contains(&item.frequency), "频率必须是 0..1 的占比");
    }

    assert!(
        result.traits.iter().all(|t| t.key != "farming_efficiency"),
        "全局特征不得包含位置敏感的补刀指标"
    );
    let top = result
        .position_stats
        .iter()
        .find(|p| p.position == "TOP")
        .expect("TOP 位置分组必须存在");
    let farming = top
        .stats
        .traits
        .iter()
        .find(|t| t.name.contains("补刀") || t.description.contains("补刀"))
        .or_else(|| {
            // DeterministicTrait 映射后 name 为「补刀高效」等；若旧映射未带 key，用 description
            top.stats.traits.first()
        });
    assert!(
        top.stats.traits.iter().any(|t| t.description.contains("补刀") || t.name.contains("补刀")),
        "TOP 位置维度必须产出补刀特征，实际 {:?}",
        top.stats.traits
    );
    let _ = farming;
    assert!(
        !result.overall_stats.traits.is_empty(),
        "旧返回类型也要拿到映射后的特征"
    );
}

#[tokio::test]
async fn test_teammate_and_self_improvement_capabilities_stay_false_until_evidence_exists() {
    let specs: Vec<GameSpec> = (1..=6).map(GameSpec::ranked).collect();
    let (result, _) = analyze(MockSource::from_specs(&specs), &ranked_request(6)).await;

    assert!(result.capabilities.deep_analysis, "排位深度应可用");
    assert!(result.capabilities.local_ai, "排位深度证据应允许本地 AI");
    assert!(
        !result.capabilities.teammate,
        "Evidence 尚未产出队友维度时不得声明 teammate 可用"
    );
    assert!(
        !result.capabilities.self_improvement,
        "Evidence 尚未产出自我提升维度时不得声明 selfImprovement 可用"
    );
}

#[tokio::test]
async fn test_single_game_sample_never_produces_strong_conclusion() {
    let specs = [GameSpec::ranked(1)];
    let (result, _) = analyze(MockSource::from_specs(&specs), &ranked_request(1)).await;

    assert!(!result.traits.is_empty(), "单场仍可描述事实");
    assert!(
        result.traits.iter().all(|t| !t.supports_conclusion),
        "单场样本不得支撑结论"
    );
    assert!(
        result.traits.iter().all(|t| t.sentiment == TraitSentiment::Neutral),
        "样本不足时不得给出好/坏定性"
    );
    assert!(result.advice.is_empty(), "样本不足时不得产出建议");
    assert!(result.overall_stats.traits.is_empty(), "旧返回类型只接收可下结论的特征");
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == AnalysisDegradationCode::InsufficientEvidenceSample));
}

// === 8. 建议：基于证据 + 视角透传 ===

#[tokio::test]
async fn test_advice_is_evidence_backed_and_respects_perspective() {
    // 对线期只有 3.0 CS/min，且死亡偏多：必然产出可下结论的负向建议
    let specs: Vec<GameSpec> = (1..=6).map(|id| GameSpec::ranked(id).early_cs(30).deaths(8)).collect();
    let mut request = ranked_request(6);
    request.perspective = Some(AdvicePerspective::Targeting);
    request.target_player = Some("对手".to_string());

    let (result, _) = analyze(MockSource::from_specs(&specs), &request).await;

    assert!(
        result.advice.iter().any(|a| a.key == "death_control"),
        "全局建议应包含阵亡类（非位置敏感），实际 {:?}",
        result.advice.iter().map(|a| a.key.clone()).collect::<Vec<_>>()
    );
    for advice in &result.advice {
        assert_eq!(advice.perspective, AdvicePerspective::Targeting, "视角必须透传");
        assert_eq!(advice.target_player.as_deref(), Some("对手"));
        assert!(advice.sample_count >= 3, "建议只能建立在足量样本上");
        assert!(!advice.evidence_game_ids.is_empty(), "建议必须能追溯到证据对局");
        assert!(!advice.evidence.is_empty(), "建议必须写明证据");
        assert_ne!(advice.key, "farming_efficiency", "全局建议不得包含位置敏感的补刀项");
    }
    let top = result
        .position_stats
        .iter()
        .find(|p| p.position == "TOP")
        .expect("TOP 位置分组必须存在");
    assert!(
        top.stats.advice.iter().any(|a| a.title.contains("补刀") || a.problem.contains("补刀")),
        "TOP 位置维度必须产出补刀建议，实际 {:?}",
        top.stats.advice
    );
}

// === 9. 新旧入口：同一份数据结论一致 ===

#[tokio::test]
async fn test_legacy_views_are_lossless_projections_of_the_single_result() {
    let specs: Vec<GameSpec> = (1..=6).map(GameSpec::ranked).collect();
    let (result, _) = analyze(MockSource::from_specs(&specs), &ranked_request(6)).await;

    let multi_position = to_multi_position_analysis(&result);

    assert_eq!(
        multi_position.overall_stats.total_games,
        result.overall_stats.total_games
    );
    assert_eq!(multi_position.overall_stats.win_rate, result.overall_stats.win_rate);
    assert_eq!(multi_position.main_position, result.main_position);
    assert_eq!(multi_position.position_stats.len(), result.position_stats.len());
    assert_eq!(
        multi_position.overall_stats.recent_performance.len(),
        result.matches.len(),
        "旧的 recent_performance 与新的 matches 是同一批展示对局"
    );
}

#[test]
fn test_legacy_entry_points_map_to_the_same_request_contract() {
    // 旧 get_match_history(count, queueId=440)：不得再出现写死的 420 / SoloRanked
    let request = legacy_analysis_request(20, Some(440), None, None, None);
    assert_eq!(request.count, 20, "count 是展示场数");
    assert_eq!(request.mode, AnalysisMode::FlexRanked);
    assert!(
        request.max_analysis_games.is_some(),
        "旧入口必须给深度证据设上限，避免一次分析拉 20 份时间线"
    );

    // 旧 get_player_tactical_advice：视角透传，队列不再写死 420
    let advice_request = tactical_advice_request(AdvicePerspective::Targeting, Some("某玩家".to_string()));
    assert_eq!(advice_request.perspective, Some(AdvicePerspective::Targeting));
    assert_eq!(advice_request.target_player.as_deref(), Some("某玩家"));
    assert_ne!(advice_request.mode, AnalysisMode::SoloRanked, "不得写死单排");
    assert_eq!(advice_request.requested_queue_ids(), Vec::<i64>::new(), "不得写死 420");
}

// === 10. Tauri 命令 serde 契约 ===

#[test]
fn test_request_deserializes_from_camel_case_payload() {
    let payload = json!({
        "count": 20,
        "mode": "allModes",
        "depth": "deep",
        "queueIds": [420, 440],
        "maxAnalysisGames": 5,
        "perspective": "SelfImprovement",
        "features": {
            "enabled": true,
            "timeline": true,
            "opponent": true,
            "teammate": false,
            "selfImprovement": true,
        }
    });

    let request: MatchAnalysisRequest = serde_json::from_value(payload).expect("请求必须能从 camelCase 载荷反序列化");

    assert_eq!(request.count, 20);
    assert_eq!(request.mode, AnalysisMode::AllModes);
    assert_eq!(request.depth, AnalysisDepth::Deep);
    assert_eq!(request.queue_ids, Some(vec![420, 440]));
    assert_eq!(request.max_analysis_games, Some(5));
    assert!(!request.features.teammate);
}

#[test]
fn test_request_defaults_survive_minimal_payload() {
    let request: MatchAnalysisRequest =
        serde_json::from_value(json!({ "count": 10, "mode": "soloRanked", "depth": "simple" }))
            .expect("最小载荷必须可用");

    assert!(request.features.enabled, "features 缺省时必须取默认开关");
    assert_eq!(request.max_analysis_games, None);
    assert_eq!(request.perspective, None);
}

#[tokio::test]
async fn test_result_serializes_with_camel_case_keys() {
    let specs: Vec<GameSpec> = (1..=6).map(GameSpec::ranked).collect();
    let (result, _) = analyze(MockSource::from_specs(&specs), &ranked_request(6)).await;

    let value = serde_json::to_value(&result).expect("结果必须可序列化");

    for key in [
        "overallStats",
        "positionStats",
        "mainPosition",
        "analyzedGames",
        "displayGames",
        "matches",
        "traits",
        "advice",
        "policy",
        "capabilities",
        "diagnostics",
        "evidence",
    ] {
        assert!(value.get(key).is_some(), "结果缺少 camelCase 字段 {key}");
    }

    let first_trait = value["traits"][0].clone();
    for key in ["key", "sampleCount", "frequency", "confidence", "evidenceGameIds"] {
        assert!(first_trait.get(key).is_some(), "特征缺少 camelCase 字段 {key}");
    }
    assert_eq!(value["policy"]["effectiveGameCount"], json!(6));
}

// === 11. 队列过滤：策略之外的对局不进入任何结果 ===

#[tokio::test]
async fn test_policy_queue_filter_excludes_unselected_games() {
    let specs = [
        GameSpec::ranked(1).queue(420),
        GameSpec::ranked(2).queue(440),
        GameSpec::ranked(3).queue(450),
    ];
    let mut request = ranked_request(3);
    request.mode = AnalysisMode::SoloRanked;

    let (result, calls) = analyze(MockSource::from_specs(&specs), &request).await;

    assert_eq!(result.overall_stats.total_games, 1, "只保留 420");
    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].queue_id, Some(420));
    assert_eq!(calls.lock().unwrap().timeline.clone(), vec![1]);
    assert_eq!(resolve_analysis_policy(&request).selected_queue_ids, vec![420]);
}

// === 12. 位置词表统一：打野 / 辅助 / ARAM ===

fn source_with_role(specs: &[GameSpec], role: &str, lane: &str) -> MockSource {
    let mut timelines = HashMap::new();
    let mut games = Vec::new();
    for spec in specs {
        games.push(game_with_role(*spec, role, lane));
        timelines.insert(spec.game_id, timeline(*spec));
    }
    MockSource {
        list: list_response(games),
        timelines,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_jungle_position_groups_as_jungle_and_joins_evidence() {
    let specs: Vec<GameSpec> = (1..=4).map(GameSpec::ranked).collect();
    let (result, _) = analyze(source_with_role(&specs, "NONE", "JUNGLE"), &ranked_request(4)).await;

    assert_eq!(result.matches[0].position, "JUNGLE");
    let jungle = result
        .position_stats
        .iter()
        .find(|p| p.position == "JUNGLE")
        .expect("打野必须分到 JUNGLE 而不是 UNKNOWN");
    assert_eq!(jungle.games, 4);
    assert!(
        result.position_stats.iter().all(|p| p.position != "UNKNOWN"),
        "打野不得落到 UNKNOWN"
    );
    let evidence = result.evidence.as_ref().expect("必须有证据");
    assert!(
        evidence.matches.iter().all(|m| m.position.as_str() == "JUNGLE"),
        "证据位置必须与分组一致"
    );
}

#[tokio::test]
async fn test_support_never_gets_farming_trait_or_advice() {
    let specs: Vec<GameSpec> = (1..=10).map(|id| GameSpec::ranked(id).early_cs(10).deaths(2)).collect();
    let (result, _) = analyze(source_with_role(&specs, "DUO_SUPPORT", "BOTTOM"), &ranked_request(10)).await;

    assert_eq!(result.matches[0].position, "SUPPORT");
    let support = result
        .position_stats
        .iter()
        .find(|p| p.position == "SUPPORT")
        .expect("辅助位置分组");
    assert!(
        support
            .stats
            .traits
            .iter()
            .all(|t| !t.name.contains("补刀") && !t.description.contains("补刀")),
        "辅助不得出现补刀特征，实际 {:?}",
        support.stats.traits
    );
    assert!(
        support
            .stats
            .advice
            .iter()
            .all(|a| !a.title.contains("补刀") && !a.problem.contains("补刀")),
        "辅助不得出现补刀建议"
    );
    assert!(
        result.traits.iter().all(|t| t.key != "farming_efficiency"),
        "全局也不得有补刀特征"
    );
}

#[tokio::test]
async fn test_aram_position_code_is_aram_but_no_position_stats_bucket() {
    let specs = [GameSpec::ranked(1).queue(450), GameSpec::ranked(2).queue(450)];
    let mut request = MatchAnalysisRequest::new(AnalysisMode::Aram, AnalysisDepth::Simple);
    request.count = 2;
    let (result, _) = analyze(source_with_role(&specs, "SOLO", "MIDDLE"), &request).await;

    // 单局字段仍标 ARAM；位置统计整段跳过（仅排位做五分路）
    assert!(result.matches.iter().all(|m| m.position == "ARAM"));
    assert!(result.position_stats.is_empty());
    assert!(!result.capabilities.position_breakdown);
    assert_eq!(result.main_position, "UNKNOWN");
}

#[tokio::test]
async fn test_hextech_simple_produces_affinity_and_fun_traits() {
    let specs: Vec<GameSpec> = (1..=5).map(|id| GameSpec::ranked(id).queue(2400)).collect();
    let mut request = MatchAnalysisRequest::new(AnalysisMode::AllModes, AnalysisDepth::Simple);
    request.count = 5;
    let (result, _) = analyze(source_with_role(&specs, "DUO_SUPPORT", "BOTTOM"), &request).await;

    assert!(result.evidence.is_none());
    assert!(
        result.traits.iter().any(|t| t.key == "mode_affinity_hextech"),
        "应识别海克斯常驻，实际 {:?}",
        result.traits.iter().map(|t| t.key.clone()).collect::<Vec<_>>()
    );
    assert!(
        result
            .overall_stats
            .traits
            .iter()
            .any(|t| t.name.contains("海克斯") || t.name.contains("乱斗")),
        "可下结论的身份标签应投影到 overallStats.traits"
    );
    assert!(
        result.traits.iter().all(|t| !t.key.contains("laning")),
        "海克斯不得套用对线特征"
    );
}

#[tokio::test]
async fn test_hextech_aram_skips_position_stats_despite_fake_lanes() {
    let specs: Vec<GameSpec> = (1..=5).map(|id| GameSpec::ranked(id).queue(2400)).collect();
    let mut request = MatchAnalysisRequest::new(AnalysisMode::AllModes, AnalysisDepth::Simple);
    request.count = 5;
    let (result, _) = analyze(source_with_role(&specs, "DUO_SUPPORT", "BOTTOM"), &request).await;

    assert!(result.matches.iter().all(|m| m.position == "FLEX"));
    assert!(
        result.position_stats.is_empty(),
        "海克斯不得产出假五分路统计，实际 {:?}",
        result.position_stats.iter().map(|p| &p.position).collect::<Vec<_>>()
    );
    assert!(!result.capabilities.position_breakdown);
}

#[tokio::test]
async fn test_mixed_modes_position_stats_only_count_ranked() {
    let specs = [
        GameSpec::ranked(1).queue(420),
        GameSpec::ranked(2).queue(2400),
        GameSpec::ranked(3).queue(420),
        GameSpec::ranked(4).queue(450),
    ];
    let mut request = MatchAnalysisRequest::new(AnalysisMode::AllModes, AnalysisDepth::Simple);
    request.count = 4;
    let (result, _) = analyze(source_with_role(&specs, "SOLO", "TOP"), &request).await;

    assert_eq!(result.display_games, 4);
    let total_position_games: u32 = result.position_stats.iter().map(|p| p.games).sum();
    assert_eq!(total_position_games, 2, "位置统计只计入两场排位");
    assert!(result.capabilities.position_breakdown);
    assert_eq!(result.main_position, "TOP");
    assert!(result.position_stats.iter().all(|p| p.position == "TOP" || p.position == "UNKNOWN"));
}

// === 13. 诊断去重 + capabilities 收敛 ===

#[tokio::test]
async fn test_timeline_failures_dedupe_to_single_diagnostic() {
    let specs: Vec<GameSpec> = (1..=3).map(GameSpec::ranked).collect();
    let source = MockSource::from_specs(&specs).failing_timelines(&[1, 2, 3]);
    let (result, _) = analyze(source, &ranked_request(3)).await;

    let timeline_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == AnalysisDegradationCode::TimelineDataUnavailable)
        .collect();
    assert_eq!(
        timeline_diags.len(),
        1,
        "3 局时间线失败只能有 1 条诊断，实际 {:?}",
        result.diagnostics
    );
    assert!(!result.capabilities.timeline, "无完整时间线时 timeline 能力必须为 false");
}

#[tokio::test]
async fn test_simple_depth_has_position_breakdown_but_no_deep_capabilities() {
    let specs: Vec<GameSpec> = (1..=4).map(GameSpec::ranked).collect();
    let mut request = MatchAnalysisRequest::new(AnalysisMode::FlexRanked, AnalysisDepth::Simple);
    request.count = 4;
    let (result, calls) = analyze(MockSource::from_specs(&specs), &request).await;

    assert!(result.capabilities.position_breakdown);
    assert!(!result.position_stats.is_empty());
    assert!(!result.capabilities.deep_analysis);
    assert!(!result.capabilities.timeline);
    assert!(!result.capabilities.opponent);
    assert!(result.evidence.is_none());
    assert!(
        result.traits.iter().any(|t| t.key == "mode_affinity_ranked"),
        "Simple 排位仍可有身份标签，实际 {:?}",
        result.traits.iter().map(|t| t.key.clone()).collect::<Vec<_>>()
    );
    assert!(
        result.traits.iter().all(|t| t.key.starts_with("mode_affinity")),
        "Simple 无 Evidence，不得出现排位深度特征"
    );
    assert!(calls.lock().unwrap().timeline.is_empty());
}

#[test]
fn test_overview_request_is_simple_with_timeline_disabled() {
    let request = legacy_overview_request(20, Some(420), None);
    let policy = resolve_analysis_policy(&request);
    assert_eq!(request.depth, AnalysisDepth::Simple);
    assert!(!request.features.timeline);
    assert!(policy.basic_only);
    assert!(!policy.enable_timeline);
}

// === 14. 领域层禁止依赖 infrastructure ===

#[test]
fn test_domains_analysis_must_not_import_infrastructure() {
    let roots = [
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domains"),
    ];
    let mut offenders = Vec::new();
    for root in roots {
        for entry in walkdir_rs(&root) {
            let content = std::fs::read_to_string(&entry).unwrap_or_default();
            if content.contains("use crate::infrastructure") || content.contains("crate::infrastructure::") {
                offenders.push(entry.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "domains/ 不得依赖 infrastructure，违规文件: {:?}",
        offenders
    );
}

fn walkdir_rs(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_rs(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    files
}
