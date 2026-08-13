//! Runtime logging policy.
//!
//! Business modules are grouped under stable, human-readable targets so log
//! output does not change merely because an implementation file is moved.
//! Development defaults to `info`; release builds default to `warn`. Set
//! `NIDALEE_LOG=debug` or `NIDALEE_LOG=trace` before launching the app to opt
//! into verbose output.

use std::borrow::Cow;

use log::LevelFilter;
use tauri::App;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

const LOG_LEVEL_ENV: &str = "NIDALEE_LOG";

pub(crate) fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let level = configured_level();
    let targets = if cfg!(debug_assertions) {
        vec![
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some("nidalee".into()),
            }),
        ]
    } else {
        vec![Target::new(TargetKind::LogDir {
            file_name: Some("nidalee".into()),
        })]
    };

    app.handle().plugin(
        tauri_plugin_log::Builder::default()
            .targets(targets)
            .level(level)
            .level_for("hyper", LevelFilter::Warn)
            .level_for("reqwest", LevelFilter::Warn)
            .level_for("tungstenite", LevelFilter::Warn)
            .level_for("tokio_tungstenite", LevelFilter::Warn)
            .max_file_size(2_000_000)
            .rotation_strategy(RotationStrategy::KeepSome(4))
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{} {:>5} {:<20} {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    record.level(),
                    stable_target(record.target()),
                    message
                ))
            })
            .build(),
    )?;

    log::info!(
        target: "app::logging",
        "runtime logging initialized level={level} ({LOG_LEVEL_ENV}=trace|debug|info|warn|error)"
    );
    Ok(())
}

fn configured_level() -> LevelFilter {
    std::env::var(LOG_LEVEL_ENV)
        .ok()
        .and_then(|value| parse_level(&value))
        .unwrap_or(default_level())
}

const fn default_level() -> LevelFilter {
    if cfg!(debug_assertions) {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    }
}

fn parse_level(value: &str) -> Option<LevelFilter> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" => Some(LevelFilter::Off),
        "error" => Some(LevelFilter::Error),
        "warn" | "warning" => Some(LevelFilter::Warn),
        "info" => Some(LevelFilter::Info),
        "debug" => Some(LevelFilter::Debug),
        "trace" => Some(LevelFilter::Trace),
        _ => None,
    }
}

fn stable_target(target: &str) -> Cow<'_, str> {
    let local = target
        .strip_prefix("nidalee_lib::")
        .or_else(|| target.strip_prefix("nidalee::"))
        .unwrap_or(target);

    let label = if local == "app::logging" {
        Some("app::logging")
    } else if local == "app" {
        Some("app::lifecycle")
    } else if local == "initialization" {
        Some("app::init")
    } else if local == "tray" {
        Some("app::tray")
    } else if local.starts_with("common::commands::game") {
        Some("app::launcher")
    } else if local.starts_with("common::commands::export") {
        Some("app::export")
    } else if local.starts_with("shared::request") {
        Some("lcu::http")
    } else if local.starts_with("shared::cache") {
        Some("lcu::cache")
    } else if local.starts_with("infrastructure::game_session::auth") {
        Some("lcu::auth")
    } else if local.starts_with("infrastructure::game_session::connection") {
        Some("lcu::connection")
    } else if local.starts_with("infrastructure::game_session::gameflow") {
        Some("game::flow")
    } else if local.starts_with("infrastructure::game_session::lobby") {
        Some("game::lobby")
    } else if local.starts_with("infrastructure::match_management::matchmaking") {
        Some("game::matchmaking")
    } else if local.starts_with("infrastructure::champion_selection::champ_select") {
        Some("game::champ-select")
    } else if local.starts_with("infrastructure::champion_selection::perks") {
        Some("build::runes")
    } else if local.starts_with("infrastructure::real_time::liveclient") {
        Some("lcu::liveclient")
    } else if local.contains("websocket::event_handler::enrichment::in_game_backfill") {
        Some("ws::backfill")
    } else if local.contains("websocket::event_handler::enrichment::recovery") {
        Some("ws::recovery")
    } else if local.contains("websocket::event_handler::enrichment::player_analysis") {
        Some("analysis::player")
    } else if local.contains("websocket::event_handler") {
        Some("ws::event")
    } else if local.contains("websocket::service") {
        Some("ws::supervisor")
    } else if local.contains("websocket::fallback") {
        Some("ws::snapshot")
    } else if local.contains("websocket::transport") || local.contains("websocket::commands") {
        Some("ws::transport")
    } else if local.starts_with("infrastructure::match_management::analysis_data") {
        Some("analysis::team")
    } else if local.contains("matches::service::process_review") {
        Some("match::review")
    } else if local.contains("matches::analysis_service") {
        Some("match::analysis")
    } else if local.contains("matches::fetcher") {
        Some("match::fetch")
    } else if local.starts_with("infrastructure::match_management::matches") {
        Some("match::history")
    } else if local.starts_with("infrastructure::data_services::summoner") {
        Some("summoner::profile")
    } else if local.starts_with("infrastructure::data_services::static_catalog") {
        Some("static::catalog")
    } else if local.starts_with("infrastructure::data_services::champion_data") {
        Some("static::champion")
    } else if local.starts_with("infrastructure::champion_selection::summoner_spells") {
        Some("static::spells")
    } else if local.starts_with("infrastructure::data_services::external::opgg") {
        Some("provider::opgg")
    } else if local.starts_with("infrastructure::data_services::external::hextech") {
        Some("provider::hextech")
    } else if local.starts_with("infrastructure::data_services::external::ai") {
        Some("provider::ai")
    } else if local.starts_with("domains::analysis") {
        Some("analysis::domain")
    } else {
        None
    };

    label.map_or_else(|| Cow::Borrowed(local), Cow::Borrowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_level_accepts_supported_names_case_insensitively() {
        assert_eq!(parse_level(" DEBUG "), Some(LevelFilter::Debug));
        assert_eq!(parse_level("warning"), Some(LevelFilter::Warn));
        assert_eq!(parse_level("TRACE"), Some(LevelFilter::Trace));
    }

    #[test]
    fn parse_level_rejects_unknown_names() {
        assert_eq!(parse_level("verbose"), None);
        assert_eq!(parse_level(""), None);
    }

    #[test]
    fn stable_target_groups_realtime_modules_by_responsibility() {
        assert_eq!(
            stable_target(
                "nidalee_lib::infrastructure::real_time::websocket::event_handler::enrichment::in_game_backfill"
            ),
            "ws::backfill"
        );
        assert_eq!(
            stable_target("nidalee_lib::infrastructure::real_time::websocket::event_handler::phase_session"),
            "ws::event"
        );
        assert_eq!(
            stable_target("nidalee_lib::infrastructure::real_time::websocket::service"),
            "ws::supervisor"
        );
    }

    #[test]
    fn stable_target_groups_core_data_flows() {
        assert_eq!(stable_target("nidalee_lib::shared::request"), "lcu::http");
        assert_eq!(
            stable_target("nidalee_lib::infrastructure::match_management::analysis_data::service"),
            "analysis::team"
        );
        assert_eq!(
            stable_target("nidalee_lib::infrastructure::data_services::static_catalog::service"),
            "static::catalog"
        );
    }

    #[test]
    fn stable_target_preserves_unknown_local_path_instead_of_hiding_it() {
        assert_eq!(
            stable_target("nidalee_lib::future::new_domain::worker"),
            "future::new_domain::worker"
        );
    }
}
