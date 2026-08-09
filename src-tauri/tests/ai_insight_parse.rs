//! AI 结构化响应解析测试（不联网）
//!
//! 运行：`cargo test --test ai_insight_parse`

use nidalee_lib::ai_contract::parse_ai_insight_response;

#[test]
fn test_parses_bare_json_insight() {
    let raw = r#"{
      "summary": "对线期压制明显",
      "confidence": 0.8,
      "findings": [
        { "title": "补刀高效", "detail": "对线期 7.5 CS/min", "evidenceGameIds": [1,2], "confidence": 0.75 }
      ],
      "suggestions": [
        { "title": "保持节奏", "actions": ["资源刷新前提前清线"], "evidenceGameIds": [1], "priority": 5 }
      ]
    }"#;
    let insight = parse_ai_insight_response(raw).expect("应解析成功");
    assert_eq!(insight.summary, "对线期压制明显");
    assert_eq!(insight.findings.len(), 1);
    assert_eq!(insight.findings[0].evidence_game_ids, vec![1, 2]);
    assert_eq!(insight.suggestions[0].actions.len(), 1);
}

#[test]
fn test_parses_fenced_json() {
    let raw = "```json\n{\"summary\":\"ok\",\"confidence\":0.5,\"findings\":[],\"suggestions\":[]}\n```";
    let insight = parse_ai_insight_response(raw).expect("应解析围栏 JSON");
    assert_eq!(insight.summary, "ok");
    assert!((insight.confidence - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_rejects_invalid_confidence() {
    let raw = r#"{"summary":"x","confidence":1.5,"findings":[],"suggestions":[]}"#;
    assert!(parse_ai_insight_response(raw).is_err());
}

#[test]
fn test_rejects_empty_summary() {
    let raw = r#"{"summary":"  ","confidence":0.2,"findings":[],"suggestions":[]}"#;
    assert!(parse_ai_insight_response(raw).is_err());
}

#[test]
fn test_rejects_non_json() {
    assert!(parse_ai_insight_response("not json at all").is_err());
}
