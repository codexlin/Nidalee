use super::{
    champ_select_gameflow_context, champ_select_session_with_gameflow_context, in_progress_gameflow_context,
    overlay_queue_id, GameflowContext,
};
use serde_json::json;

#[test]
fn champ_select_session_uses_authoritative_gameflow_context() {
    let raw = json!({ "localPlayerCellId": 4, "queueId": 0, "isCustomGame": true, "myTeam": [] });
    let gameflow = json!({
        "phase": "ChampSelect",
        "gameData": {
            "queue": { "id": 440 },
            "isCustomGame": false
        }
    });

    let session = champ_select_session_with_gameflow_context(&raw, Some(&gameflow));

    assert_eq!(session["queueId"], 440);
    assert_eq!(session["isCustomGame"], false);
    assert_eq!(session["localPlayerCellId"], 4);
}

#[test]
fn gameflow_context_requires_both_runtime_fields() {
    assert_eq!(
        champ_select_gameflow_context(&json!({
            "phase": "ChampSelect",
            "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
        })),
        Some(GameflowContext {
            queue_id: 450,
            is_custom_game: false
        })
    );
    assert_eq!(
        champ_select_gameflow_context(&json!({
            "phase": "ChampSelect",
            "gameData": { "queue": { "id": 450 } }
        })),
        None
    );
    assert_eq!(
        champ_select_gameflow_context(&json!({
            "phase": "InProgress",
            "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
        })),
        None
    );
}

#[test]
fn in_progress_context_prefers_fresh_runtime_state() {
    let cached = json!({
        "phase": "InProgress",
        "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
    });
    let fetched = json!({
        "phase": "InProgress",
        "gameData": { "queue": { "id": 420 }, "isCustomGame": false }
    });

    let context = in_progress_gameflow_context(Some(&cached), Some(&fetched));

    assert_eq!(
        context,
        Some(GameflowContext {
            queue_id: 420,
            is_custom_game: false
        })
    );
}

#[test]
fn in_progress_context_rejects_stale_or_incomplete_sessions() {
    let stale = json!({
        "phase": "ChampSelect",
        "gameData": { "queue": { "id": 440 }, "isCustomGame": false }
    });
    let incomplete = json!({
        "phase": "InProgress",
        "gameData": { "queue": { "id": 440 } }
    });

    let context = in_progress_gameflow_context(Some(&stale), Some(&incomplete));

    assert_eq!(context, None);
}

#[test]
fn overlay_queue_prefers_gameflow_then_champ_select() {
    let select = json!({ "queueId": 2400 });
    let flow = json!({ "gameData": { "queue": { "id": 420 } } });

    assert_eq!(overlay_queue_id(Some(&select), Some(&flow)), Some(420));
    assert_eq!(overlay_queue_id(Some(&select), None), Some(2400));
    assert_eq!(overlay_queue_id(None, None), None);
    assert_eq!(overlay_queue_id(Some(&json!({ "queueId": 0 })), None), None);
}
