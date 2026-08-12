use serde_json::Value;

const JSON_API_EVENT_NAME: &str = "OnJsonApiEvent";

#[derive(Debug, Clone, PartialEq)]
pub(super) struct JsonApiEvent {
    pub uri: String,
    pub event_type: String,
    pub data: Value,
}

impl JsonApiEvent {
    pub fn from_payload(payload: &Value) -> Option<Self> {
        Some(Self {
            uri: payload.get("uri")?.as_str()?.to_owned(),
            event_type: payload
                .get("eventType")
                .and_then(Value::as_str)
                .unwrap_or("Update")
                .to_owned(),
            data: payload.get("data").cloned().unwrap_or(Value::Null),
        })
    }
}

pub(super) fn decode_json_api_event(text: &str) -> serde_json::Result<Option<JsonApiEvent>> {
    let envelope = serde_json::from_str::<Value>(text)?;
    let Some(parts) = envelope.as_array() else {
        return Ok(None);
    };
    if parts.len() < 3 || parts[0].as_u64() != Some(8) || parts[1].as_str() != Some(JSON_API_EVENT_NAME) {
        return Ok(None);
    }
    Ok(JsonApiEvent::from_payload(&parts[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_json_api_event() {
        let event =
            decode_json_api_event(r#"[8,"OnJsonApiEvent",{"uri":"/phase","eventType":"Update","data":"Lobby"}]"#)
                .expect("valid JSON")
                .expect("valid JSON API event");
        assert_eq!(event.uri, "/phase");
        assert_eq!(event.event_type, "Update");
        assert_eq!(event.data, Value::String("Lobby".to_owned()));
    }

    #[test]
    fn ignores_unrelated_wamp_messages() {
        assert_eq!(decode_json_api_event(r#"[1,"other"]"#).expect("valid JSON"), None);
    }
}
