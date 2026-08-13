use serde_json::Value;

pub(super) fn champ_select_session_with_gameflow_context(
    champ_select: &Value,
    gameflow_session: Option<&Value>,
) -> Value {
    let Some(context) = gameflow_session.and_then(champ_select_gameflow_context) else {
        return champ_select.clone();
    };
    let Some(session) = champ_select.as_object() else {
        return champ_select.clone();
    };

    let mut enriched = session.clone();
    enriched.insert("queueId".to_string(), Value::from(context.queue_id));
    enriched.insert("isCustomGame".to_string(), Value::from(context.is_custom_game));
    Value::Object(enriched)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GameflowContext {
    pub(super) queue_id: i64,
    pub(super) is_custom_game: bool,
}

fn gameflow_context_for_phase(session: &Value, expected_phase: &str) -> Option<GameflowContext> {
    if session.get("phase")?.as_str()? != expected_phase {
        return None;
    }
    let game_data = session.get("gameData")?;
    Some(GameflowContext {
        queue_id: game_data.get("queue")?.get("id")?.as_i64()?,
        is_custom_game: game_data.get("isCustomGame")?.as_bool()?,
    })
}

fn champ_select_gameflow_context(session: &Value) -> Option<GameflowContext> {
    gameflow_context_for_phase(session, "ChampSelect")
}

pub(super) fn in_progress_gameflow_context(
    cached_session: Option<&Value>,
    fetched_session: Option<&Value>,
) -> Option<GameflowContext> {
    fetched_session
        .and_then(|session| gameflow_context_for_phase(session, "InProgress"))
        .or_else(|| cached_session.and_then(|session| gameflow_context_for_phase(session, "InProgress")))
}

#[cfg(test)]
mod tests {
    use super::{
        champ_select_gameflow_context, champ_select_session_with_gameflow_context, in_progress_gameflow_context,
        GameflowContext,
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
}
