use serde_json::Value;

pub(super) fn champ_select_session_with_gameflow_context(
    champ_select: &Value,
    gameflow_session: Option<&Value>,
) -> Value {
    let Some(context) = gameflow_session.and_then(gameflow_build_context) else {
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
struct GameflowBuildContext {
    queue_id: i64,
    is_custom_game: bool,
}

fn gameflow_build_context(session: &Value) -> Option<GameflowBuildContext> {
    if session.get("phase")?.as_str()? != "ChampSelect" {
        return None;
    }
    let game_data = session.get("gameData")?;
    Some(GameflowBuildContext {
        queue_id: game_data.get("queue")?.get("id")?.as_i64()?,
        is_custom_game: game_data.get("isCustomGame")?.as_bool()?,
    })
}

#[cfg(test)]
mod tests {
    use super::{champ_select_session_with_gameflow_context, gameflow_build_context, GameflowBuildContext};
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
            gameflow_build_context(&json!({
                "phase": "ChampSelect",
                "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
            })),
            Some(GameflowBuildContext {
                queue_id: 450,
                is_custom_game: false
            })
        );
        assert_eq!(
            gameflow_build_context(&json!({
                "phase": "ChampSelect",
                "gameData": { "queue": { "id": 450 } }
            })),
            None
        );
        assert_eq!(
            gameflow_build_context(&json!({
                "phase": "InProgress",
                "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
            })),
            None
        );
    }
}
