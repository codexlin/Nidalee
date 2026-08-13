use super::{
    classify_incoming, phase_health_requires_full_snapshot, select_effective_phase, IncomingMessage, SnapshotMergeState,
};
use crate::infrastructure::real_time::websocket::fallback::{SnapshotBatch, SnapshotEntry, PHASE_URI};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

#[test]
fn phase_health_only_recovers_without_newer_ws_phase() {
    assert!(phase_health_requires_full_snapshot(None, Some("None"), Some("Lobby")));
    assert!(!phase_health_requires_full_snapshot(None, Some("Lobby"), Some("Lobby")));
    assert!(!phase_health_requires_full_snapshot(
        Some("ChampSelect"),
        Some("Lobby"),
        Some("None")
    ));
}

#[test]
fn eof_and_close_are_both_terminal() {
    assert!(matches!(classify_incoming(None), Ok(IncomingMessage::Closed)));
    assert!(matches!(
        classify_incoming(Some(Ok(Message::Close(None)))),
        Ok(IncomingMessage::Closed)
    ));
}

#[test]
fn snapshot_merge_tracks_text_while_ping_and_close_remain_classifiable() {
    let mut merge = SnapshotMergeState::default();
    let text =
        r#"[8,"OnJsonApiEvent",{"uri":"/lol-gameflow/v1/gameflow-phase","eventType":"Update","data":"ChampSelect"}]"#;
    match classify_incoming(Some(Ok(Message::Text(text.to_string())))).unwrap() {
        IncomingMessage::Text(text) => merge.observe_text(&text),
        _ => panic!("expected text"),
    }
    assert!(matches!(
        classify_incoming(Some(Ok(Message::Ping(vec![1, 2])))),
        Ok(IncomingMessage::Ping(_))
    ));

    let skipped = merge.skipped_uris(&SnapshotBatch::with_phase("Lobby"));
    assert!(skipped.contains(PHASE_URI));
    assert_eq!(merge.observed_phase.as_deref(), Some("ChampSelect"));
}

#[test]
fn same_phase_snapshot_skips_only_uris_already_seen_on_ws() {
    let mut merge = SnapshotMergeState::default();
    merge.observe_text(
        r#"[8,"OnJsonApiEvent",{"uri":"/lol-gameflow/v1/gameflow-phase","eventType":"Update","data":"ChampSelect"}]"#,
    );
    let mut snapshot = SnapshotBatch::with_phase("ChampSelect");
    snapshot.entries.push(SnapshotEntry {
        uri: "/lol-gameflow/v1/session",
        data: Value::Null,
    });

    let skipped = merge.skipped_uris(&snapshot);

    assert!(skipped.contains(PHASE_URI));
    assert!(!skipped.contains("/lol-gameflow/v1/session"));
}

#[test]
fn observed_ws_phase_rejects_snapshot_when_http_phase_is_unknown() {
    let mut merge = SnapshotMergeState::default();
    merge.observe_text(
        r#"[8,"OnJsonApiEvent",{"uri":"/lol-gameflow/v1/gameflow-phase","eventType":"Update","data":"ChampSelect"}]"#,
    );
    let snapshot = SnapshotBatch {
        phase: None,
        entries: vec![SnapshotEntry {
            uri: "/lol-gameflow/v1/session",
            data: Value::Null,
        }],
    };

    let skipped = merge.skipped_uris(&snapshot);

    assert!(skipped.contains("/lol-gameflow/v1/session"));
}

#[test]
fn effective_phase_prefers_observed_ws_then_http_then_previous_state() {
    assert_eq!(
        select_effective_phase(
            Some("ChampSelect".to_string()),
            Some("Lobby".to_string()),
            Some("None".to_string()),
        )
        .as_deref(),
        Some("ChampSelect")
    );
    assert_eq!(
        select_effective_phase(None, Some("Lobby".to_string()), Some("None".to_string())).as_deref(),
        Some("Lobby")
    );
    assert_eq!(
        select_effective_phase(None, None, Some("None".to_string())).as_deref(),
        Some("None")
    );
}
