//! 确定性 EvidenceBundle 测试
//!
//! 覆盖：真实时间速率 / 阶段边界 / 短局与 remake 容错 / 缺帧与单帧 /
//! 打野 CS 单列 / 阶段末绝对差与归一化优势 / 对手识别（lane-role 与空间回退）/
//! 事件证据（含 killerId=0、建筑助攻、资源参与）/ 位置枚举 / 多局聚合。
//!
//! 全部基于合成 fixtures，不依赖运行中的英雄联盟客户端。
//!
//! 运行：`cargo test --test analysis_evidence`

use std::path::PathBuf;

use serde_json::{json, Value};

use nidalee_lib::analysis_evidence::{
    advantage_to_percent, build_evidence_bundle, build_process_insight, extract_match_evidence, is_aram_queue,
    laning_opponent_diff, position_from_role_lane, resolve_lane_opponent, ActivityContext, DeathCause,
    EvidenceConfidence, EvidenceEventKind, EventInvolvement, EvidenceIssue, EvidencePosition, EvidenceQuality,
    GamePhase, MatchEvidence, MatchEvidenceInput, OpponentMatchMethod, ADVANTAGE_PERCENT_SCALE,
    MIN_SAMPLE_FOR_CONCLUSION,
};

// === 夹具加载 ===

const TARGET_TOP: &str = "00000000-0000-4000-8000-000000000001";
const TARGET_JUNGLE: &str = "00000000-0000-4000-8000-000000000002";
const TARGET_MID: &str = "00000000-0000-4000-8000-000000000003";
const TARGET_ADC: &str = "00000000-0000-4000-8000-000000000004";
const TARGET_SUPPORT: &str = "00000000-0000-4000-8000-000000000005";

fn fixture(name: &str) -> Value {
    let path: PathBuf = ["tests", "fixtures", "match_analysis", name].iter().collect();
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读取 fixture {} 失败: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("解析 fixture {} 失败: {e}", path.display()))
}

fn ten_players() -> Value {
    fixture("ranked_440_ten_players.json")
}

fn timeline_30min() -> Value {
    fixture("timeline_440_ten_players_30min.json")
}

fn approx(actual: Option<f64>, expected: f64, what: &str) {
    let actual = actual.unwrap_or_else(|| panic!("{what} 应有值，实际为 None"));
    assert!(
        (actual - expected).abs() < 1e-6,
        "{what}: 期望 {expected}，实际 {actual}"
    );
}

fn phase<'a>(evidence: &'a MatchEvidence, phase: GamePhase) -> &'a nidalee_lib::analysis_evidence::PhaseEvidence {
    evidence
        .phase(phase)
        .unwrap_or_else(|| panic!("缺少阶段 {phase:?}，实际阶段: {:?}", evidence.phases))
}

fn top_evidence() -> MatchEvidence {
    let game = ten_players();
    let timeline = timeline_30min();
    extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取 TOP 证据失败")
}

// === 1. 时间线：真实经过时间作分母 ===

#[test]
fn test_thirty_minute_timeline_uses_real_elapsed_time() {
    let evidence = top_evidence();

    let early = phase(&evidence, GamePhase::Early);
    approx(Some(early.duration_minutes), 10.0, "早期真实时长(分钟)");
    approx(early.cs_per_min, 8.0, "早期 CS/min");
    approx(early.gold_per_min, 480.0, "早期 Gold/min");
    approx(early.xp_per_min, 600.0, "早期 XP/min");

    let mid = phase(&evidence, GamePhase::Mid);
    approx(Some(mid.duration_minutes), 10.0, "中期真实时长(分钟)");
    approx(mid.cs_per_min, 9.0, "中期 CS/min");
    approx(mid.gold_per_min, 600.0, "中期 Gold/min");
    approx(mid.xp_per_min, 800.0, "中期 XP/min");

    let late = phase(&evidence, GamePhase::Late);
    approx(Some(late.duration_minutes), 10.0, "后期真实时长(分钟)");
    approx(late.cs_per_min, 8.0, "后期 CS/min");

    assert_eq!(evidence.quality, EvidenceQuality::Full, "数据齐全应为 Full");
}

#[test]
fn test_phase_boundaries_are_explicit_and_shared() {
    let evidence = top_evidence();

    let early = phase(&evidence, GamePhase::Early);
    let mid = phase(&evidence, GamePhase::Mid);
    let late = phase(&evidence, GamePhase::Late);

    assert_eq!(early.start_ms, 0);
    assert_eq!(early.end_ms, 600_000, "早期以 10 分钟帧收尾");
    assert_eq!(mid.start_ms, 600_000, "中期以 10 分钟帧为锚点起算");
    assert_eq!(mid.end_ms, 1_200_000);
    assert_eq!(late.start_ms, 1_200_000, "后期以 20 分钟帧为锚点起算");
    assert_eq!(late.end_ms, 1_800_000);

    assert_eq!(
        evidence.phases.iter().map(|p| p.phase).collect::<Vec<_>>(),
        vec![GamePhase::Early, GamePhase::Mid, GamePhase::Late],
        "阶段必须按固定顺序输出"
    );
}

#[test]
fn test_jungle_cs_is_separate_and_total_includes_lane_and_jungle() {
    let game = ten_players();
    let timeline = timeline_30min();
    let jungler = extract_match_evidence(&game, Some(&timeline), TARGET_JUNGLE).expect("提取打野证据失败");

    assert_eq!(jungler.position, EvidencePosition::Jungle);

    let early = phase(&jungler, GamePhase::Early);
    assert_eq!(early.lane_cs, 10, "线上兵线单列");
    assert_eq!(early.jungle_cs, 60, "打野野怪单列");
    assert_eq!(early.total_cs, 70, "total = lane + jungle");
    approx(early.lane_cs_per_min, 1.0, "打野线上 CS/min");
    approx(early.jungle_cs_per_min, 6.0, "打野野怪 CS/min");
    approx(early.cs_per_min, 7.0, "打野 total CS/min");

    // 上单也必须有独立的打野 CS 列（中期开始吃野）
    let top = top_evidence();
    let top_mid = phase(&top, GamePhase::Mid);
    assert_eq!(top_mid.lane_cs, 80);
    assert_eq!(top_mid.jungle_cs, 10);
    assert_eq!(top_mid.total_cs, 90);
}

// === 2. 短局 / remake / 缺帧 / 单帧 ===

#[test]
fn test_short_game_has_no_late_phase() {
    let game = ten_players();
    let timeline = fixture("timeline_440_short_15min.json");
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("短局提取失败");

    let early = phase(&evidence, GamePhase::Early);
    approx(early.cs_per_min, 7.0, "短局早期 CS/min");

    let mid = phase(&evidence, GamePhase::Mid);
    approx(Some(mid.duration_minutes), 5.0, "短局中期真实时长");
    approx(mid.cs_per_min, 7.0, "短局中期 CS/min");
    approx(mid.gold_per_min, 500.0, "短局中期 Gold/min");

    assert!(evidence.phase(GamePhase::Late).is_none(), "15 分钟局不应有后期阶段");
}

#[test]
fn test_remake_game_has_only_early_phase() {
    let game = ten_players();
    let timeline = fixture("timeline_440_remake_5min.json");
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("remake 提取失败");

    assert_eq!(evidence.phases.len(), 1, "remake 只应有早期阶段");
    let early = phase(&evidence, GamePhase::Early);
    approx(Some(early.duration_minutes), 5.0, "remake 真实时长");
    approx(early.cs_per_min, 6.4, "remake CS/min = 32/5");

    assert!(evidence.phase(GamePhase::Mid).is_none());
    assert!(evidence.phase(GamePhase::Late).is_none());
}

#[test]
fn test_single_frame_does_not_divide_by_zero() {
    let game = ten_players();
    let timeline = json!({
        "frames": [{
            "timestamp": 0,
            "events": [],
            "participantFrames": {
                "1": { "participantId": 1, "currentGold": 500, "totalGold": 500, "level": 1, "xp": 0,
                       "minionsKilled": 0, "jungleMinionsKilled": 0, "position": { "x": 1000, "y": 1000 } },
                "6": { "participantId": 6, "currentGold": 500, "totalGold": 500, "level": 1, "xp": 0,
                       "minionsKilled": 0, "jungleMinionsKilled": 0, "position": { "x": 1100, "y": 1100 } }
            }
        }]
    });

    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("单帧提取失败");
    let early = phase(&evidence, GamePhase::Early);

    assert_eq!(early.frame_count, 1);
    assert_eq!(early.duration_minutes, 0.0);
    assert!(early.cs_per_min.is_none(), "单帧不得产出速率");
    assert!(early.gold_per_min.is_none());
    assert!(early.xp_per_min.is_none());
    assert!(evidence.has_issue(EvidenceIssue::SingleFrameNoRate));
    assert_eq!(evidence.quality, EvidenceQuality::TimelinePartial);
}

#[test]
fn test_missing_participant_frames_are_tolerated() {
    let game = ten_players();
    let timeline = fixture("timeline_440_missing_frames.json");
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("缺帧提取失败");

    let early = phase(&evidence, GamePhase::Early);
    approx(early.cs_per_min, 8.0, "缺帧前的早期速率仍可计算");

    let mid = phase(&evidence, GamePhase::Mid);
    assert_eq!(mid.frame_count, 1, "中期只剩边界锚点帧");
    assert!(mid.cs_per_min.is_none(), "缺帧阶段不得产出速率");

    assert!(evidence.phase(GamePhase::Late).is_none());
    assert!(evidence.has_issue(EvidenceIssue::TargetParticipantFramesMissing));
    assert_eq!(evidence.quality, EvidenceQuality::TimelinePartial);
}

#[test]
fn test_missing_timeline_degrades_without_panic() {
    let game = ten_players();
    let evidence = extract_match_evidence(&game, None, TARGET_TOP).expect("无时间线也应产出证据");

    assert_eq!(evidence.quality, EvidenceQuality::TimelineMissing);
    assert!(evidence.phases.is_empty());
    assert!(evidence.events.is_none());
    assert!(evidence.has_issue(EvidenceIssue::TimelineMissing));

    // 位置与对手不依赖时间线
    assert_eq!(evidence.position, EvidencePosition::Top);
    let opponent = evidence.opponent.as_ref().expect("lane-role 对手不依赖时间线");
    assert_eq!(opponent.participant_id, 6);
}

#[test]
fn test_empty_frames_timeline_is_reported() {
    let game = ten_players();
    let timeline = json!({ "frames": [] });
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("空帧不应报错");

    assert!(evidence.phases.is_empty());
    assert!(evidence.has_issue(EvidenceIssue::TimelineFramesEmpty));
}

#[test]
fn test_missing_target_participant_returns_explainable_error() {
    let game = ten_players();
    let error = extract_match_evidence(&game, None, "not-in-this-game").expect_err("目标不在本局应返回可解释错误");
    assert_eq!(
        error,
        nidalee_lib::analysis_evidence::EvidenceExtractionError::TargetNotFound
    );

    let broken = json!({ "gameId": 1, "queueId": 440 });
    let error = extract_match_evidence(&broken, None, TARGET_TOP).expect_err("缺 participantIdentities 应返回错误");
    assert_eq!(
        error,
        nidalee_lib::analysis_evidence::EvidenceExtractionError::MissingParticipantIdentities
    );
}

// === 3. 阶段末绝对差与归一化优势 ===

#[test]
fn test_phase_diff_is_absolute_end_of_phase_difference() {
    let evidence = top_evidence();

    let early_diff = phase(&evidence, GamePhase::Early)
        .opponent_diff
        .as_ref()
        .expect("早期应有对手差");
    assert_eq!(early_diff.opponent_participant_id, 6);
    assert_eq!(early_diff.cs_diff, 20, "阶段末 total CS 绝对差 = 80 - 60");
    assert_eq!(early_diff.gold_diff, 800, "阶段末金币绝对差 = 5300 - 4500");
    assert_eq!(early_diff.xp_diff, 1000, "阶段末经验绝对差 = 6000 - 5000");
    assert_eq!(early_diff.level_diff, 1);

    let mid_diff = phase(&evidence, GamePhase::Mid)
        .opponent_diff
        .as_ref()
        .expect("中期应有对手差");
    assert_eq!(mid_diff.cs_diff, 40, "中期末 total CS 差 = 170 - 130（含打野）");
    assert_eq!(mid_diff.gold_diff, 1800);
    assert_eq!(mid_diff.xp_diff, 2000);
}

#[test]
fn test_advantage_is_normalized_before_combination() {
    let evidence = top_evidence();
    let diff = phase(&evidence, GamePhase::Early)
        .opponent_diff
        .as_ref()
        .expect("早期应有对手差");

    // 归一化 = 份额差 (a-b)/(a+b)，无量纲且有界
    approx(Some(diff.normalized_cs_advantage), 20.0 / 140.0, "CS 归一化优势");
    approx(Some(diff.normalized_gold_advantage), 800.0 / 9800.0, "金币归一化优势");
    approx(Some(diff.normalized_xp_advantage), 1000.0 / 11000.0, "经验归一化优势");

    assert!(
        diff.overall_advantage.abs() <= 1.0,
        "归一化组合必须有界，实际 {}",
        diff.overall_advantage
    );
    // 若直接加权 raw 值，overall 会是数百量级
    assert!(
        diff.overall_advantage < 1.0,
        "禁止把 raw XP/Gold 直接加权，实际 {}",
        diff.overall_advantage
    );
}

#[test]
fn test_raw_xp_lead_does_not_dominate_normalized_advantage() {
    // A：CS 份额领先明显，但 raw 差值很小
    let a = single_phase_advantage(60, 40, 3000, 3000, 3000, 3000);
    // B：raw 经验领先 5000，但份额优势很小
    let b = single_phase_advantage(50, 50, 3000, 3000, 105_000, 100_000);

    assert!(a > b, "份额优势应主导，raw 经验差不得压倒一切：a={a}, b={b}");
    assert!(b.abs() < 0.1, "微小份额差不应放大成强结论，实际 {b}");
}

/// 构造只有一个阶段的时间线并返回归一化总优势
fn single_phase_advantage(
    target_cs: i64,
    opponent_cs: i64,
    target_gold: i64,
    opponent_gold: i64,
    target_xp: i64,
    opponent_xp: i64,
) -> f64 {
    let game = ten_players();
    let timeline = json!({
        "frames": [
            {
                "timestamp": 0,
                "events": [],
                "participantFrames": {
                    "1": { "participantId": 1, "totalGold": 0, "currentGold": 0, "level": 1, "xp": 0,
                           "minionsKilled": 0, "jungleMinionsKilled": 0, "position": { "x": 1000, "y": 1000 } },
                    "6": { "participantId": 6, "totalGold": 0, "currentGold": 0, "level": 1, "xp": 0,
                           "minionsKilled": 0, "jungleMinionsKilled": 0, "position": { "x": 1100, "y": 1100 } }
                }
            },
            {
                "timestamp": 600000,
                "events": [],
                "participantFrames": {
                    "1": { "participantId": 1, "totalGold": target_gold, "currentGold": 0, "level": 9, "xp": target_xp,
                           "minionsKilled": target_cs, "jungleMinionsKilled": 0, "position": { "x": 1500, "y": 1500 } },
                    "6": { "participantId": 6, "totalGold": opponent_gold, "currentGold": 0, "level": 9, "xp": opponent_xp,
                           "minionsKilled": opponent_cs, "jungleMinionsKilled": 0, "position": { "x": 1600, "y": 1600 } }
                }
            }
        ]
    });

    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取失败");
    evidence
        .phase(GamePhase::Early)
        .and_then(|p| p.opponent_diff.as_ref())
        .expect("应有对手差")
        .overall_advantage
}

// === 4. 对手识别 ===

#[test]
fn test_opponent_identified_by_lane_role_for_standard_positions() {
    let game = ten_players();
    let timeline = timeline_30min();

    let cases = [
        (TARGET_TOP, EvidencePosition::Top, 6),
        (TARGET_JUNGLE, EvidencePosition::Jungle, 7),
        (TARGET_MID, EvidencePosition::Mid, 8),
        (TARGET_ADC, EvidencePosition::Adc, 9),
        (TARGET_SUPPORT, EvidencePosition::Support, 10),
    ];

    for (puuid, expected_position, expected_opponent) in cases {
        let evidence = extract_match_evidence(&game, Some(&timeline), puuid).expect("提取失败");
        assert_eq!(evidence.position, expected_position, "位置识别错误: {puuid}");

        let opponent = evidence
            .opponent
            .as_ref()
            .unwrap_or_else(|| panic!("{expected_position:?} 应能识别对线对手"));
        assert_eq!(opponent.participant_id, expected_opponent, "对手错误: {puuid}");
        assert_eq!(opponent.method, OpponentMatchMethod::LaneRole);
        assert!(opponent.confidence >= 0.9, "lane-role 匹配应高置信");
        assert_eq!(opponent.position, expected_position, "对手位置应与自己相同");
    }
}

/// 抹掉**敌方**的 role/lane：目标仍是真实分路，敌方数据不可用 → 允许空间回退
fn game_with_unusable_enemy_lane_data() -> Value {
    let mut game = ten_players();
    for participant in game["participants"].as_array_mut().expect("participants") {
        if participant["teamId"] == json!(200) {
            participant["timeline"] = json!({ "role": "NONE", "lane": "NONE" });
        }
    }
    game
}

/// 上路对线站位：目标与敌方上单贴在一起，其余敌人分散在别的路
///
/// 空间回退需要足够多的对线期采样点，因此这里用每分钟一帧的时间线，
/// 而不是只有 0/10/20 分钟三帧的粗粒度夹具。
fn top_lane_proximity_timeline() -> Value {
    per_minute_timeline(&[1, 2, 6, 7, 9], |id, _| match id {
        1 => (2000.0, 11500.0), // 目标：上路
        2 => (7000.0, 7000.0),  // 队友中单
        6 => (2400.0, 11800.0), // 敌方上单：真正的对位
        7 => (6000.0, 8500.0),  // 敌方打野：河道游走
        _ => (11500.0, 2000.0), // 敌方下路
    })
}

#[test]
fn test_opponent_falls_back_to_spatial_proximity_when_lane_role_unusable() {
    let game = game_with_unusable_enemy_lane_data();
    let timeline = top_lane_proximity_timeline();

    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取失败");
    let opponent = evidence.opponent.as_ref().expect("应回退到空间邻近");

    assert_eq!(opponent.participant_id, 6, "空间邻近应命中同路敌人");
    assert_eq!(opponent.method, OpponentMatchMethod::SpatialProximity);
    assert!(
        opponent.confidence < 0.9,
        "空间回退置信度必须低于 lane-role，实际 {}",
        opponent.confidence
    );
    assert!(evidence.has_issue(EvidenceIssue::OpponentFromSpatialProximity));
}

#[test]
fn test_spatial_fallback_only_compares_enemy_team() {
    let game = game_with_unusable_enemy_lane_data();
    // 队友 2 全程和目标重叠（双人上路）：仍然不允许把队友当对手
    let timeline = per_minute_timeline(&[1, 2, 6], |id, _| match id {
        1 | 2 => (2000.0, 11500.0),
        _ => (2400.0, 11800.0),
    });

    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取失败");
    let opponent = evidence.opponent.as_ref().expect("应有对手");
    assert_eq!(opponent.participant_id, 6, "只能在敌方队伍里挑对手");
    assert_eq!(opponent.team_id, 200);
}

#[test]
fn test_spatial_fallback_needs_enough_valid_laning_samples() {
    let game = game_with_unusable_enemy_lane_data();
    // 只有 0 / 10 / 20 分钟三帧：剔除 t0 后对线期只剩一个采样点，不足以判定
    let coarse = timeline_30min();

    let evidence = extract_match_evidence(&game, Some(&coarse), TARGET_TOP).expect("提取失败");
    assert!(
        evidence.opponent.is_none(),
        "对线期有效采样不足时必须返回 None，而不是拿单帧硬猜"
    );
    assert!(evidence.has_issue(EvidenceIssue::OpponentUnidentified));
}

#[test]
fn test_no_opponent_when_unresolvable() {
    // 敌方没有同位置玩家，且没有时间线可做空间回退
    let game = json!({
        "gameId": 1000000099,
        "queueId": 440,
        "gameDuration": 1500,
        "participantIdentities": [
            { "participantId": 1, "player": { "puuid": TARGET_TOP } },
            { "participantId": 6, "player": { "puuid": "00000000-0000-4000-8000-000000000006" } }
        ],
        "participants": [
            { "participantId": 1, "championId": 86, "teamId": 100, "stats": { "win": true },
              "timeline": { "role": "SOLO", "lane": "TOP" } },
            { "participantId": 6, "championId": 60, "teamId": 200, "stats": { "win": false },
              "timeline": { "role": "NONE", "lane": "JUNGLE" } }
        ]
    });

    let evidence = extract_match_evidence(&game, None, TARGET_TOP).expect("提取失败");
    assert!(evidence.opponent.is_none(), "识别不出对手时必须是 None");
    assert!(evidence.has_issue(EvidenceIssue::OpponentUnidentified));
}

#[test]
fn test_never_picks_first_enemy_as_lane_opponent() {
    // 敌方队伍首位是打野（participantId 6），目标是 ADC：绝不能因为「第一个敌人」而配对
    let game = json!({
        "gameId": 1000000098,
        "queueId": 440,
        "gameDuration": 1500,
        "participantIdentities": [
            { "participantId": 1, "player": { "puuid": TARGET_TOP } },
            { "participantId": 6, "player": { "puuid": "00000000-0000-4000-8000-000000000006" } },
            { "participantId": 7, "player": { "puuid": "00000000-0000-4000-8000-000000000007" } }
        ],
        "participants": [
            { "participantId": 1, "championId": 22, "teamId": 100, "stats": { "win": true },
              "timeline": { "role": "DUO_CARRY", "lane": "BOTTOM" } },
            { "participantId": 6, "championId": 60, "teamId": 200, "stats": { "win": false },
              "timeline": { "role": "NONE", "lane": "JUNGLE" } },
            { "participantId": 7, "championId": 21, "teamId": 200, "stats": { "win": false },
              "timeline": { "role": "DUO_CARRY", "lane": "BOTTOM" } }
        ]
    });

    let opponent = resolve_lane_opponent(&game, None, 1, 440).expect("应识别到真正的对位 ADC");
    assert_eq!(opponent.participant_id, 7, "必须匹配同位置敌人，而不是第一个敌人");
}

#[test]
fn test_jungle_opponent_by_smite_when_lane_missing() {
    // 排位字段糊掉时：双方带惩戒仍应对上打野（含浮点 spellId）
    let game = json!({
        "gameId": 1000000201,
        "queueId": 420,
        "gameDuration": 1800,
        "participants": [
            {
                "participantId": 1, "championId": 76, "teamId": 100,
                "spell1Id": 4.0, "spell2Id": 11.0,
                "stats": { "neutralMinionsKilled": 120 },
                "timeline": { "role": "NONE", "lane": "NONE" }
            },
            {
                "participantId": 2, "championId": 86, "teamId": 100,
                "spell1Id": 4, "spell2Id": 12,
                "stats": { "neutralMinionsKilled": 8 },
                "timeline": { "role": "SOLO", "lane": "TOP" }
            },
            {
                "participantId": 6, "championId": 64, "teamId": 200,
                "spell1Id": 4.0, "spell2Id": 11.0,
                "stats": { "neutralMinionsKilled": 110 },
                "timeline": { "role": "NONE", "lane": "NONE" }
            },
            {
                "participantId": 7, "championId": 103, "teamId": 200,
                "spell1Id": 4, "spell2Id": 14,
                "stats": { "neutralMinionsKilled": 4 },
                "timeline": { "role": "SOLO", "lane": "MIDDLE" }
            }
        ]
    });

    let opponent = resolve_lane_opponent(&game, None, 1, 420).expect("打野应对上敌方惩戒");
    assert_eq!(opponent.participant_id, 6);
    assert_eq!(opponent.position, EvidencePosition::Jungle);
}

#[test]
fn test_jungle_opponent_disambiguates_dual_smite_by_jungle_cs() {
    // 敌方双惩戒（辅助偶发带惩戒）：野刀领先者才是对位打野
    let game = json!({
        "gameId": 1000000202,
        "queueId": 420,
        "gameDuration": 1800,
        "participants": [
            {
                "participantId": 1, "championId": 76, "teamId": 100,
                "spell1Id": 4, "spell2Id": 11,
                "stats": { "neutralMinionsKilled": 130 },
                "timeline": { "role": "NONE", "lane": "JUNGLE" }
            },
            {
                "participantId": 6, "championId": 64, "teamId": 200,
                "spell1Id": 4, "spell2Id": 11,
                "stats": { "neutralMinionsKilled": 125 },
                "timeline": { "role": "NONE", "lane": "JUNGLE" }
            },
            {
                "participantId": 7, "championId": 412, "teamId": 200,
                "spell1Id": 4, "spell2Id": 11,
                "stats": { "neutralMinionsKilled": 12 },
                "timeline": { "role": "DUO_SUPPORT", "lane": "BOTTOM" }
            }
        ]
    });

    let opponent = resolve_lane_opponent(&game, None, 1, 420).expect("双惩戒应靠野刀消歧");
    assert_eq!(opponent.participant_id, 6, "应对上野刀更高的敌方打野");
}

#[test]
fn test_matchup_skipped_for_normal_draft() {
    // 匹配/娱乐局不做对位与过程复盘
    let game = json!({
        "gameId": 1000000203,
        "queueId": 400,
        "gameDuration": 1600,
        "participants": [
            {
                "participantId": 1, "championId": 76, "teamId": 100,
                "spell1Id": 4, "spell2Id": 11,
                "timeline": { "role": "NONE", "lane": "JUNGLE" }
            },
            {
                "participantId": 6, "championId": 64, "teamId": 200,
                "spell1Id": 4, "spell2Id": 11,
                "timeline": { "role": "NONE", "lane": "JUNGLE" }
            }
        ]
    });

    assert!(
        resolve_lane_opponent(&game, None, 1, 400).is_none(),
        "非排位不得产生对位"
    );
}

// === 4.1 空间邻近的鲁棒性（泉水 / t0 / 死亡尖峰）===

/// 蓝方泉水中心（召唤师峡谷）
const BLUE_FOUNTAIN: (f64, f64) = (400.0, 400.0);
/// 红方泉水中心
const RED_FOUNTAIN: (f64, f64) = (14400.0, 14400.0);

/// 程序化生成「每分钟一帧、0..=30 分钟」的时间线（31 帧）
///
/// `position(participant_id, minute) -> (x, y)`；CS / 金币 / 经验按分钟线性增长，
/// 让阶段速率保持可预期。
fn per_minute_timeline(participant_ids: &[i32], position: impl Fn(i32, i64) -> (f64, f64)) -> Value {
    let frames: Vec<Value> = (0..=30i64)
        .map(|minute| {
            let mut participant_frames = serde_json::Map::new();
            for &id in participant_ids {
                let (x, y) = position(id, minute);
                participant_frames.insert(
                    id.to_string(),
                    json!({
                        "participantId": id,
                        "currentGold": 500,
                        "totalGold": 500 + minute * 400,
                        "level": 1 + minute / 2,
                        "xp": minute * 500,
                        "minionsKilled": minute * 7,
                        "jungleMinionsKilled": 0,
                        "position": { "x": x, "y": y }
                    }),
                );
            }
            json!({
                "timestamp": minute * 60_000,
                "events": [],
                "participantFrames": Value::Object(participant_frames)
            })
        })
        .collect();

    json!({ "frames": frames })
}

/// 真实对局里 t0 全员在泉水、死亡后坐标被钉在泉水。
///
/// 这些帧必须被剔除，否则「碰巧在自家泉水附近推线的敌人」会被误判成对线对手。
/// 本场景下，若对所有帧取平均距离，胜出的是错误的敌人 9。
#[test]
fn test_spatial_fallback_skips_fountain_and_death_frames() {
    let game = game_with_unusable_enemy_lane_data();

    let timeline = per_minute_timeline(&[1, 6, 9], |id, minute| {
        // 目标：t0 在泉水，1-7 分钟连续阵亡被钉在泉水，8 分钟后才真正在上路
        let target_in_fountain = minute <= 7;
        match id {
            1 if target_in_fountain => BLUE_FOUNTAIN,
            1 => (2000.0, 11500.0),
            // 敌方上单（正确对手）：t0 在自家泉水，之后一直在上路
            6 if minute == 0 => RED_FOUNTAIN,
            6 => (2400.0, 11800.0),
            // 敌方 ADC：一直在蓝方下路区域（靠近蓝色泉水但不在泉水内）
            9 if minute == 0 => RED_FOUNTAIN,
            _ => (2200.0, 2200.0),
        }
    });

    let opponent = resolve_lane_opponent(&game, Some(&timeline), 1, 440).expect("应能回退到空间邻近");

    assert_eq!(
        opponent.participant_id, 6,
        "泉水/死亡帧必须被剔除：否则会误判成靠近蓝方泉水的敌方 ADC 9"
    );
    assert_eq!(opponent.method, OpponentMatchMethod::SpatialProximity);
}

/// 单帧极端离群（一次远程支援）不得改变对手判定 —— 中位数而非均值
#[test]
fn test_spatial_fallback_is_robust_to_single_outlier_frame() {
    let game = game_with_unusable_enemy_lane_data();

    let timeline = per_minute_timeline(&[1, 6, 9], |id, minute| match id {
        // 目标常驻上路，第 5 分钟一次性传送到地图另一端
        1 if minute == 5 => (13000.0, 2000.0),
        1 => (2000.0, 11500.0),
        6 => (2400.0, 11800.0),
        // 敌方 ADC 常驻目标那次支援点附近
        _ => (12800.0, 2200.0),
    });

    let opponent = resolve_lane_opponent(&game, Some(&timeline), 1, 440).expect("应能回退到空间邻近");
    assert_eq!(opponent.participant_id, 6, "单帧离群不得污染中位数判定");
}

// === 4.2 非分路位置禁止空间回退 ===

#[test]
fn test_aram_never_produces_lane_opponent_or_diff() {
    let game = top_game_with_id(1000000450, 450);
    let timeline = timeline_30min();
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("大乱斗提取失败");

    assert_eq!(evidence.position, EvidencePosition::Aram, "450 必须归为 ARAM");
    assert!(evidence.opponent.is_none(), "大乱斗没有对线对手");
    assert!(evidence.has_issue(EvidenceIssue::OpponentUnidentified));
    assert!(
        evidence.phases.iter().all(|p| p.opponent_diff.is_none()),
        "没有对线对手就不得产出任何阶段末对手差"
    );
    assert!(
        !evidence.has_issue(EvidenceIssue::OpponentFromSpatialProximity),
        "非分路位置不允许走空间回退"
    );
}

#[test]
fn test_jungle_and_unknown_positions_do_not_use_spatial_fallback() {
    let game = game_with_unusable_enemy_lane_data();
    let timeline = timeline_30min();

    // 打野满地图游走，空间邻近对它没有对线语义
    let jungler = extract_match_evidence(&game, Some(&timeline), TARGET_JUNGLE).expect("打野提取失败");
    assert_eq!(jungler.position, EvidencePosition::Jungle);
    assert!(
        jungler
            .opponent
            .as_ref()
            .is_none_or(|opponent| opponent.method != OpponentMatchMethod::SpatialProximity),
        "打野不得用空间邻近编出对线对手"
    );

    // 目标自己的 role/lane 也不可用 → 位置 UNKNOWN → 同样禁止回退
    let mut unknown_game = game.clone();
    for participant in unknown_game["participants"].as_array_mut().expect("participants") {
        participant["timeline"] = json!({ "role": "NONE", "lane": "NONE" });
    }
    let unknown = extract_match_evidence(&unknown_game, Some(&timeline), TARGET_TOP).expect("提取失败");
    assert_eq!(unknown.position, EvidencePosition::Unknown);
    assert!(unknown.opponent.is_none(), "UNKNOWN 位置不得用空间邻近编出对线对手");
}

// === 4.3 ARAM 判定优先于 lane/role，且来源唯一 ===

#[test]
fn test_aram_queue_wins_over_lane_role_hints() {
    // 大乱斗的 role/lane 有时被填成 SOLO/MIDDLE，绝不能因此判成中单
    assert_eq!(position_from_role_lane("SOLO", "MIDDLE", 450), EvidencePosition::Aram);
    assert_eq!(
        position_from_role_lane("DUO_CARRY", "BOTTOM", 450),
        EvidencePosition::Aram
    );
    assert_eq!(position_from_role_lane("NONE", "JUNGLE", 450), EvidencePosition::Aram);

    // 峡谷排位不受影响
    assert_eq!(position_from_role_lane("SOLO", "MIDDLE", 440), EvidencePosition::Mid);
}

#[test]
fn test_non_ranked_never_uses_lane_role_hints() {
    // 海克斯 / 匹配常残留峡谷字段，不得拆成五分路
    assert_eq!(
        position_from_role_lane("DUO_SUPPORT", "BOTTOM", 2400),
        EvidencePosition::Flex
    );
    assert_eq!(position_from_role_lane("SOLO", "TOP", 430), EvidencePosition::Flex);
    assert_eq!(position_from_role_lane("JUNGLE", "NONE", 1700), EvidencePosition::Flex);
    assert_eq!(position_from_role_lane("SOLO", "MID", 900), EvidencePosition::Flex);

    // 排位仍按 role/lane
    assert_eq!(
        position_from_role_lane("DUO_SUPPORT", "BOTTOM", 420),
        EvidencePosition::Support
    );
}

#[test]
fn test_aram_queue_definition_comes_from_queue_config() {
    assert!(is_aram_queue(450), "450 是唯一有明确语义的大乱斗队列");
    assert!(!is_aram_queue(440));
    assert!(!is_aram_queue(420));
    // 证据层不得私设队列白名单：没有 catalog 语义的队列一律不算 ARAM
    assert!(!is_aram_queue(100), "证据层不得私设 ARAM 队列白名单");
    assert!(!is_aram_queue(930), "证据层不得私设 ARAM 队列白名单");
}

// === 4.4 对线比较取 Early 阶段 ===

#[test]
fn test_laning_comparison_uses_early_phase_not_last_phase() {
    let evidence = top_evidence();

    let laning = laning_opponent_diff(&evidence.phases).expect("应有对线期对手差");
    let early = phase(&evidence, GamePhase::Early)
        .opponent_diff
        .as_ref()
        .expect("早期应有对手差");
    let late = phase(&evidence, GamePhase::Late)
        .opponent_diff
        .as_ref()
        .expect("后期应有对手差");

    assert_eq!(laning, early, "对线比较必须取 Early 阶段");
    assert_ne!(
        laning.cs_diff, late.cs_diff,
        "夹具需保证前后期差值不同，否则该测试无意义"
    );
}

#[test]
fn test_advantage_percent_scale_semantics() {
    // 归一化份额差 [-1, 1] → 百分比 [-100, 100]，这是旧阈值消费的唯一语义
    assert_eq!(ADVANTAGE_PERCENT_SCALE, 100.0);
    approx(Some(advantage_to_percent(0.0)), 0.0, "无优势");
    approx(Some(advantage_to_percent(0.25)), 25.0, "25% 份额优势");
    approx(Some(advantage_to_percent(-1.0)), -100.0, "完全劣势");
    // 输入越界时仍然收敛到 [-100, 100]
    approx(Some(advantage_to_percent(3.0)), 100.0, "越界输入必须收敛");
}

// === 4.5 阶段边界帧与诊断去重 ===

#[test]
fn test_phase_boundary_frame_is_counted_in_both_phases_by_design() {
    let game = ten_players();
    let timeline = per_minute_timeline(&[1, 6], |_, _| (2000.0, 11500.0));
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取失败");

    let early = phase(&evidence, GamePhase::Early);
    let mid = phase(&evidence, GamePhase::Mid);
    let late = phase(&evidence, GamePhase::Late);

    // 10 / 20 分钟帧同时是上一阶段的终点与下一阶段的锚点：这是速率正确的前提
    assert_eq!(early.frame_count, 11, "0..=10 分钟共 11 帧");
    assert_eq!(mid.frame_count, 11, "10..=20 分钟共 11 帧（含边界锚点）");
    assert_eq!(late.frame_count, 11, "20..=30 分钟共 11 帧（含边界锚点）");
    assert_eq!(
        early.frame_count + mid.frame_count + late.frame_count,
        31 + 2,
        "两个边界帧被双计，总数比真实帧数多 2"
    );

    // 双计不影响速率：分母始终是真实经过时间
    approx(Some(early.duration_minutes), 10.0, "早期真实时长");
    approx(early.cs_per_min, 7.0, "边界双计不得改变速率");
    approx(mid.cs_per_min, 7.0, "边界双计不得改变速率");
}

#[test]
fn test_diagnostics_are_deduplicated_per_issue_code() {
    let game = ten_players();
    // 时间线里完全没有目标玩家的帧：既「缺帧」又「没有任何阶段」
    let timeline = per_minute_timeline(&[6, 7], |_, _| (2000.0, 11500.0));
    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取失败");

    assert!(evidence.phases.is_empty());
    let missing_frames = evidence
        .diagnostics
        .iter()
        .filter(|d| d.code == EvidenceIssue::TargetParticipantFramesMissing)
        .count();
    assert_eq!(missing_frames, 1, "同一缺陷码只能出现一次，实际 {missing_frames}");

    let codes: Vec<_> = evidence.diagnostics.iter().map(|d| d.code).collect();
    let mut deduped = codes.clone();
    deduped.dedup();
    assert_eq!(codes.len(), deduped.len(), "诊断列表不得出现重复缺陷码: {codes:?}");
}

// === 5. 事件证据 ===

#[test]
fn test_event_evidence_counts_takedowns_and_objectives() {
    let evidence = top_evidence();
    let events = evidence.events.as_ref().expect("应有事件证据");

    assert_eq!(events.kills, 1, "只有 killerId == 目标 的击杀计入");
    assert_eq!(events.deaths, 2, "被 6 击杀 + 被 killerId=0 处决");
    assert_eq!(events.assists, 1);

    assert_eq!(events.dragon_takedowns, 1);
    assert_eq!(events.horde_takedowns, 1, "虚空幼虫（HORDE）参与");
    assert_eq!(events.baron_takedowns, 1);
    assert_eq!(events.herald_takedowns, 0, "敌方与无关先锋不得计入");
    assert_eq!(events.building_takedowns, 2, "建筑击杀 + 建筑助攻");

    // 关键事件保留时间与证据
    let first_kill = events
        .key_events
        .iter()
        .find(|e| e.kind == EvidenceEventKind::ChampionKill)
        .expect("应保留击杀事件");
    assert_eq!(first_kill.timestamp_ms, 300_000);

    let baron = events
        .key_events
        .iter()
        .find(|e| e.kind == EvidenceEventKind::Baron)
        .expect("应保留大龙事件");
    assert_eq!(baron.timestamp_ms, 1_500_000);

    assert!(
        events
            .key_events
            .windows(2)
            .all(|w| w[0].timestamp_ms <= w[1].timestamp_ms),
        "关键事件必须按时间升序（确定性）"
    );
}

#[test]
fn test_zero_killer_id_is_not_attributed_to_target() {
    let game = ten_players();
    let timeline = json!({
        "frames": [{
            "timestamp": 60000,
            "events": [
                { "type": "CHAMPION_KILL", "timestamp": 30000, "killerId": 0, "victimId": 6, "assistingParticipantIds": [] }
            ],
            "participantFrames": {
                "1": { "participantId": 1, "totalGold": 500, "currentGold": 500, "level": 1, "xp": 0,
                       "minionsKilled": 0, "jungleMinionsKilled": 0, "position": { "x": 1000, "y": 1000 } }
            }
        }]
    });

    let evidence = extract_match_evidence(&game, Some(&timeline), TARGET_TOP).expect("提取失败");
    let events = evidence.events.as_ref().expect("应有事件证据");
    assert_eq!(events.kills, 0, "killerId=0（处决/防御塔）不得归给目标");
}

// === 6. 位置枚举 ===

#[test]
fn test_position_enum_covers_lcu_role_lane_variants() {
    let ranked = 440;
    let cases = [
        ("SOLO", "TOP", ranked, EvidencePosition::Top),
        ("NONE", "TOP", ranked, EvidencePosition::Top),
        ("SOLO", "MIDDLE", ranked, EvidencePosition::Mid),
        ("SOLO", "MID", ranked, EvidencePosition::Mid),
        ("JUNGLE", "NONE", ranked, EvidencePosition::Jungle),
        ("NONE", "JUNGLE", ranked, EvidencePosition::Jungle),
        ("DUO_CARRY", "BOTTOM", ranked, EvidencePosition::Adc),
        ("CARRY", "BOTTOM", ranked, EvidencePosition::Adc),
        ("SOLO", "BOTTOM", ranked, EvidencePosition::Adc),
        ("DUO_SUPPORT", "BOTTOM", ranked, EvidencePosition::Support),
        ("SUPPORT", "BOT", ranked, EvidencePosition::Support),
        ("UTILITY", "BOTTOM", ranked, EvidencePosition::Support),
        ("NONE", "NONE", 450, EvidencePosition::Aram),
        ("NONE", "NONE", 430, EvidencePosition::Flex),
        ("SOLO", "TOP", 430, EvidencePosition::Flex),
        ("DUO_SUPPORT", "BOTTOM", 2400, EvidencePosition::Flex),
        ("NONE", "NONE", ranked, EvidencePosition::Unknown),
    ];

    for (role, lane, queue_id, expected) in cases {
        assert_eq!(
            position_from_role_lane(role, lane, queue_id),
            expected,
            "role={role}, lane={lane}, queue={queue_id}"
        );
    }
}

#[test]
fn test_position_fact_values_are_ascii_codes_not_chinese() {
    for position in [
        EvidencePosition::Top,
        EvidencePosition::Jungle,
        EvidencePosition::Mid,
        EvidencePosition::Adc,
        EvidencePosition::Support,
        EvidencePosition::Aram,
        EvidencePosition::Flex,
        EvidencePosition::Unknown,
    ] {
        let serialized = serde_json::to_string(&position).expect("序列化位置失败");
        assert!(serialized.is_ascii(), "后端事实值不得包含中文展示文案：{serialized}");
        assert_eq!(serialized, format!("\"{}\"", position.as_str()));
    }

    assert_eq!(EvidencePosition::Adc.as_str(), "ADC");
    assert_eq!(EvidencePosition::Support.as_str(), "SUPPORT");
}

// === 7. 多局聚合 ===

fn bundle_of(games: Vec<(Value, Option<Value>)>) -> nidalee_lib::analysis_evidence::EvidenceBundle {
    let inputs: Vec<MatchEvidenceInput<'_>> = games
        .iter()
        .map(|(game, timeline)| MatchEvidenceInput::new(game, timeline.as_ref()))
        .collect();
    build_evidence_bundle(TARGET_TOP, &inputs)
}

fn top_game_with_id(game_id: u64, queue_id: i64) -> Value {
    let mut game = ten_players();
    game["gameId"] = json!(game_id);
    game["queueId"] = json!(queue_id);
    game
}

#[test]
fn test_aggregate_groups_by_queue_and_position() {
    let timeline = timeline_30min();
    let games = vec![
        (top_game_with_id(1000000012, 440), Some(timeline.clone())),
        (top_game_with_id(1000000010, 440), Some(timeline.clone())),
        (top_game_with_id(1000000011, 440), Some(timeline.clone())),
        (top_game_with_id(1000000020, 420), Some(timeline.clone())),
    ];
    let bundle = bundle_of(games);

    assert_eq!(bundle.match_count, 4);
    assert_eq!(bundle.summaries.len(), 2, "应按 (queue, position) 分两组");

    assert_eq!(
        bundle.summaries.iter().map(|s| s.queue_id).collect::<Vec<_>>(),
        vec![420, 440],
        "聚合必须按队列升序输出（确定性）"
    );

    let flex = bundle
        .summaries
        .iter()
        .find(|s| s.queue_id == 440)
        .expect("缺少 440 分组");
    assert_eq!(flex.position, EvidencePosition::Top);
    assert_eq!(flex.sample_size, 3);
    assert_eq!(
        flex.evidence_game_ids,
        vec![1000000010, 1000000011, 1000000012],
        "证据 gameId 必须升序去重"
    );

    let early = flex.phase_average(GamePhase::Early).expect("缺少早期阶段均值");
    assert_eq!(early.sample_size, 3);
    approx(early.avg_cs_per_min, 8.0, "早期 CS/min 均值");
    approx(early.avg_gold_per_min, 480.0, "早期 Gold/min 均值");
    approx(early.avg_jungle_cs_per_min, 0.0, "早期打野 CS/min 均值");
    approx(early.avg_cs_diff, 20.0, "早期 CS 差均值");

    assert_eq!(flex.event_rates.sample_size, 3);
    approx(Some(flex.event_rates.kills_per_game), 1.0, "场均击杀");
    approx(Some(flex.event_rates.deaths_per_game), 2.0, "场均死亡");
    approx(Some(flex.event_rates.dragon_takedowns_per_game), 1.0, "场均小龙参与");
    approx(Some(flex.event_rates.building_takedowns_per_game), 2.0, "场均建筑参与");
}

#[test]
fn test_event_rates_only_count_games_with_valid_target_frames() {
    let good = timeline_30min();
    // 有 frames，但里面根本没有目标玩家 → 事件证据不可信，不能进场均分母
    let no_target_frames = per_minute_timeline(&[6, 7], |_, _| (2000.0, 11500.0));

    let bundle = bundle_of(vec![
        (top_game_with_id(1000000010, 440), Some(good)),
        (top_game_with_id(1000000011, 440), Some(no_target_frames)),
    ]);

    let summary = bundle.summaries.first().expect("应有一个分组");
    assert_eq!(summary.sample_size, 2, "两局都算进对局样本");
    assert_eq!(
        summary.event_rates.sample_size, 1,
        "只有存在目标帧的时间线才能作事件频率的分母"
    );
    approx(Some(summary.event_rates.kills_per_game), 1.0, "场均击杀不得被稀释");
    approx(Some(summary.event_rates.deaths_per_game), 2.0, "场均死亡不得被稀释");
}

#[test]
fn test_single_game_cannot_support_strong_conclusion() {
    let timeline = timeline_30min();
    let bundle = bundle_of(vec![(top_game_with_id(1000000010, 440), Some(timeline))]);

    let summary = bundle.summaries.first().expect("应有一个分组");
    assert_eq!(summary.sample_size, 1);
    assert_eq!(summary.confidence, EvidenceConfidence::Insufficient);
    assert!(!summary.supports_conclusion, "单场不得触发全局强结论");
    assert!(MIN_SAMPLE_FOR_CONCLUSION >= 3, "最小样本阈值必须 >= 3");
}

#[test]
fn test_aggregation_is_deterministic() {
    let timeline = timeline_30min();
    let build = || {
        bundle_of(vec![
            (top_game_with_id(1000000012, 440), Some(timeline.clone())),
            (top_game_with_id(1000000010, 420), Some(timeline.clone())),
            (top_game_with_id(1000000011, 440), Some(timeline.clone())),
        ])
    };

    let first = serde_json::to_value(build()).expect("序列化失败");
    let second = serde_json::to_value(build()).expect("序列化失败");
    assert_eq!(first, second, "同样输入必须产出完全一致的证据包");
}

// === 8. Task1 fixtures 回归：没有 frames 时间线也不炸 ===

#[test]
fn test_task1_fixtures_still_produce_evidence() {
    let cases = [
        ("ranked_440_standard.json", EvidencePosition::Jungle),
        ("ranked_440_short_game.json", EvidencePosition::Mid),
        ("ranked_440_missing_timeline.json", EvidencePosition::Adc),
        ("ranked_440_empty_deltas.json", EvidencePosition::Support),
    ];

    for (name, expected_position) in cases {
        let game = fixture(name);
        let evidence =
            extract_match_evidence(&game, None, TARGET_TOP).unwrap_or_else(|e| panic!("{name} 提取失败: {e:?}"));
        assert_eq!(evidence.position, expected_position, "{name} 位置识别错误");
        assert_eq!(evidence.quality, EvidenceQuality::TimelineMissing, "{name}");
        assert_eq!(evidence.queue_id, 440, "{name}");

        // 2 人 fixture：敌方同位置唯一，必须能配对
        let opponent = evidence
            .opponent
            .as_ref()
            .unwrap_or_else(|| panic!("{name} 应识别对手"));
        assert_eq!(opponent.participant_id, 2, "{name}");
    }
}

// === 9. 时间线抽全 + 过程复盘 ===

fn timeline_process_full() -> Value {
    fixture("timeline_440_process_full.json")
}

#[test]
fn test_timeline_extracts_all_known_event_kinds() {
    let evidence =
        extract_match_evidence(&ten_players(), Some(&timeline_process_full()), TARGET_TOP).expect("提取失败");
    let events = evidence.events.as_ref().expect("应有事件");

    assert_eq!(events.deaths_solo, 1, "无助攻死亡=单杀");
    assert_eq!(events.deaths_gank_or_multi, 1, "有助攻死亡=被抓");
    assert_eq!(events.deaths, 2);

    assert_eq!(events.dragons_seen, 1);
    assert_eq!(events.dragons_missed, 1, "敌方独龙应记错过");
    assert_eq!(events.dragon_takedowns, 0);
    assert_eq!(events.herald_takedowns, 1, "己方先锋助攻计入参与");

    assert!(events.wards_placed >= 1);
    assert!(events.wards_killed >= 1);
    assert!(events.items_purchased >= 1);
    assert!(events.items_sold >= 1);
    assert!(events.items_destroyed >= 1);
    assert!(events.items_undo >= 1);
    assert!(events.skill_level_ups >= 1);
    assert!(events.level_ups >= 1);
    assert!(events.special_kills >= 1);
    assert!(events.turret_plates >= 1);
    assert!(events.dragon_souls >= 1);

    assert!(
        events
            .unknown_events
            .iter()
            .any(|u| u.raw_type == "CUSTOM_UNKNOWN_EVENT"),
        "未知 type 不得静默丢弃: {:?}",
        events.unknown_events
    );
    assert!(
        events
            .key_events
            .iter()
            .any(|e| e.kind == EvidenceEventKind::Unknown),
        "未知事件应进入 key_events"
    );

    let missed_dragon = events
        .key_events
        .iter()
        .find(|e| e.kind == EvidenceEventKind::Dragon && e.involvement == EventInvolvement::Uninvolved)
        .expect("应保留未参与的小龙事件");
    assert_eq!(missed_dragon.allied_side, Some(false));
    assert_eq!(
        missed_dragon.activity_context,
        Some(ActivityContext::OwnJungle),
        "敌方拿龙时目标在己方野区附近"
    );

    let solo = events
        .key_events
        .iter()
        .find(|e| e.kind == EvidenceEventKind::ChampionDeath && e.death_cause == Some(DeathCause::Solo))
        .expect("单杀死亡");
    assert_eq!(solo.assistant_count, 0);

    let gank = events
        .key_events
        .iter()
        .find(|e| e.kind == EvidenceEventKind::ChampionDeath && e.death_cause == Some(DeathCause::GankOrMulti))
        .expect("被抓死亡");
    assert!(gank.assistant_count >= 1);
}

#[test]
fn test_process_insight_death_and_objective_from_full_timeline() {
    let evidence =
        extract_match_evidence(&ten_players(), Some(&timeline_process_full()), TARGET_TOP).expect("提取失败");
    // 凑够结论样本量：同一证据复制 3 份
    let matches = vec![evidence.clone(), evidence.clone(), evidence.clone()];
    let insight = build_process_insight(&matches);

    assert!(insight.has_timeline);
    let death = insight.death_breakdown.expect("应有阵亡复盘");
    assert_eq!(death.solo, 3);
    assert_eq!(death.gank_or_multi, 3);
    assert!(death.summary.contains("单杀") || death.summary.contains("被抓") || death.summary.contains("分散"));

    let obj = insight.objective_process.expect("应有资源复盘");
    assert_eq!(obj.dragons_missed, 3);
    assert!(
        obj.missed_activity.iter().any(|b| b.activity == "ownJungle"),
        "错过活动应含己方野区: {:?}",
        obj.missed_activity
    );
}

#[test]
fn test_process_insight_degrades_without_timeline() {
    let evidence = extract_match_evidence(&ten_players(), None, TARGET_TOP).expect("提取失败");
    let insight = build_process_insight(&[evidence]);
    assert!(!insight.has_timeline);
    assert!(insight.degradation_message.is_some());
    assert!(insight.death_breakdown.is_none());
    assert!(insight.actions.is_empty());
}
