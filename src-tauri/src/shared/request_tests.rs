use super::{execute_with_auth_retry, redact_lcu_path, LcuAttempt};
use crate::shared::types::LcuAuthInfo;
use reqwest::StatusCode;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn auth(token: &str, port: u16) -> LcuAuthInfo {
    LcuAuthInfo {
        riotclient_auth_token: format!("riot-{token}"),
        riotclient_app_port: port + 1,
        remoting_auth_token: token.to_string(),
        app_port: port,
    }
}

#[test]
fn lcu_log_path_redacts_uuid_segments() {
    let path = "/lol-ranked/v1/ranked-stats/c69b0edb-fbe6-5d0a-8eb0-8b36bb977a98";
    assert_eq!(redact_lcu_path(path), "/lol-ranked/v1/ranked-stats/<puuid>");
}

#[test]
fn lcu_log_path_preserves_non_identifier_query_values() {
    let path = concat!(
        "/lol-match-history/v1/",
        "products/",
        "lol/",
        "c69b0edb-fbe6-5d0a-8eb0-8b36bb977a98/matches?begIndex=0&",
        "endIndex=49"
    );
    assert_eq!(
        redact_lcu_path(path),
        concat!(
            "/lol-match-history/v1/",
            "products/",
            "lol/",
            "<puuid>/matches?begIndex=0&",
            "endIndex=49"
        )
    );
}

async fn run_sequence(statuses: Vec<StatusCode>) -> (Result<&'static str, String>, usize, Vec<String>) {
    let statuses = Arc::new(tokio::sync::Mutex::new(VecDeque::from(statuses)));
    let current_auth = Arc::new(tokio::sync::Mutex::new(auth("old", 2999)));
    let used_tokens = Arc::new(Mutex::new(Vec::new()));
    let refreshes = Arc::new(AtomicUsize::new(0));

    let result = execute_with_auth_retry(
        || {
            let statuses = statuses.clone();
            let current_auth = current_auth.clone();
            let used_tokens = used_tokens.clone();
            async move {
                let status = statuses
                    .lock()
                    .await
                    .pop_front()
                    .expect("test status sequence exhausted");
                let auth = current_auth.lock().await.clone();
                used_tokens.lock().unwrap().push(auth.remoting_auth_token.clone());
                Ok(LcuAttempt {
                    value: "ok",
                    status,
                    auth,
                })
            }
        },
        |_failed_auth| {
            let current_auth = current_auth.clone();
            let refreshes = refreshes.clone();
            async move {
                refreshes.fetch_add(1, Ordering::SeqCst);
                let refreshed = auth("new", 3001);
                *current_auth.lock().await = refreshed.clone();
                Ok(refreshed)
            }
        },
    )
    .await;

    let refresh_count = refreshes.load(Ordering::SeqCst);
    let tokens = used_tokens.lock().unwrap().clone();
    (result, refresh_count, tokens)
}

#[tokio::test]
async fn success_does_not_refresh() {
    let (result, refreshes, tokens) = run_sequence(vec![StatusCode::OK]).await;
    assert_eq!(result, Ok("ok"));
    assert_eq!(refreshes, 0);
    assert_eq!(tokens, vec!["old"]);
}

#[tokio::test]
async fn unauthorized_refreshes_then_retries_with_new_auth() {
    let (result, refreshes, tokens) = run_sequence(vec![StatusCode::UNAUTHORIZED, StatusCode::OK]).await;
    assert_eq!(result, Ok("ok"));
    assert_eq!(refreshes, 1);
    assert_eq!(tokens, vec!["old", "new"]);
}

#[tokio::test]
async fn second_unauthorized_fails_without_another_refresh() {
    let (result, refreshes, _) = run_sequence(vec![StatusCode::UNAUTHORIZED, StatusCode::UNAUTHORIZED]).await;
    assert!(result.unwrap_err().contains("多次认证失败"));
    assert_eq!(refreshes, 1);
}

#[tokio::test]
async fn server_error_does_not_refresh() {
    let (result, refreshes, _) = run_sequence(vec![StatusCode::INTERNAL_SERVER_ERROR]).await;
    assert!(result.unwrap_err().contains("500"));
    assert_eq!(refreshes, 0);
}
