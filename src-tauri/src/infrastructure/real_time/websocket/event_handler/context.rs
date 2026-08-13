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
#[path = "context_tests.rs"]
mod tests;
