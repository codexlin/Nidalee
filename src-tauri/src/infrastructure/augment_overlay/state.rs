//! 海克斯大乱斗推荐侧栏的运行时状态与生命周期。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use once_cell::sync::OnceCell;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use super::types::{AugmentClearedPayload, AugmentDetectedPayload, OverlayGuideAugment, OverlayGuideTrio};
use super::window;
use crate::infrastructure::data_services::champion_data;
use crate::infrastructure::data_services::external::hextech::service;
use crate::infrastructure::data_services::external::hextech::types::{HextechAugmentStat, HextechChampionDetail};
use crate::infrastructure::real_time::liveclient;

const GUIDE_TRIOS: usize = 5;
const HEXTECH_QUEUE_ID: i64 = 2400;
const TOGGLE_DEBOUNCE_MS: u64 = 280;

#[derive(Clone)]
struct ChampionGuide {
    champion_id: i32,
    name: String,
    recommended_augments: Vec<OverlayGuideAugment>,
    recommended_trios: Vec<OverlayGuideTrio>,
}

struct OverlayController {
    enabled: bool,
    visible: bool,
    in_game: bool,
    revealing: bool,
    queue_id: Option<i64>,
    expected_champion_id: Option<i32>,
    prefetching: Option<i32>,
    guide: Option<ChampionGuide>,
    last_payload: Option<AugmentDetectedPayload>,
    lifecycle_generation: u64,
    visibility_generation: u64,
}

impl OverlayController {
    fn new() -> Self {
        Self {
            enabled: true,
            visible: false,
            in_game: false,
            revealing: false,
            queue_id: None,
            expected_champion_id: None,
            prefetching: None,
            guide: None,
            last_payload: None,
            lifecycle_generation: 0,
            visibility_generation: 0,
        }
    }

    fn mark_in_game_and_should_reveal(&mut self) -> bool {
        let first_enter = !self.in_game;
        self.in_game = true;
        self.enabled && first_enter
    }

    fn cached_champion_id(&self) -> Option<i32> {
        self.expected_champion_id
            .filter(|id| *id > 0)
            .or_else(|| self.guide.as_ref().map(|guide| guide.champion_id))
    }

    fn has_guide_for(&self, champion_id: Option<i32>) -> bool {
        self.guide.as_ref().is_some_and(|guide| {
            champion_id
                .filter(|id| *id > 0)
                .is_none_or(|id| guide.champion_id == id)
        })
    }

    fn remember_queue(&mut self, queue_id: Option<i64>) {
        if let Some(id) = queue_id.filter(|id| *id > 0) {
            self.queue_id = Some(id);
        }
    }

    fn active_queue(&self, known: Option<i64>) -> Option<i64> {
        known.filter(|id| *id > 0).or(self.queue_id)
    }

    fn can_manual_show(&self) -> bool {
        is_hextech_queue(self.queue_id) && (self.in_game || self.guide.is_some() || self.last_payload.is_some())
    }
}

fn is_hextech_queue(queue_id: Option<i64>) -> bool {
    queue_id == Some(HEXTECH_QUEUE_ID)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToggleAction {
    Hide,
    Show,
    Ignore,
}

fn toggle_action(enabled: bool, visible: bool, can_show: bool) -> ToggleAction {
    if !enabled {
        ToggleAction::Ignore
    } else if visible {
        ToggleAction::Hide
    } else if can_show {
        ToggleAction::Show
    } else {
        ToggleAction::Ignore
    }
}

fn is_in_game_phase(phase: Option<&str>) -> bool {
    matches!(phase, Some("GameStart" | "InProgress" | "Reconnect"))
}

fn should_end_overlay(phase: Option<&str>, in_game: bool) -> bool {
    match phase {
        Some("GameStart" | "InProgress" | "Reconnect" | "ChampSelect") => false,
        None | Some("None") => !in_game,
        _ => true,
    }
}

static CONTROLLER: OnceCell<Mutex<OverlayController>> = OnceCell::new();
static LAST_TOGGLE_MS: AtomicU64 = AtomicU64::new(0);

fn controller() -> &'static Mutex<OverlayController> {
    CONTROLLER.get_or_init(|| Mutex::new(OverlayController::new()))
}

pub fn on_gameflow_phase(app: AppHandle, phase: Option<String>, champion_id: Option<i32>, queue_id: Option<i64>) {
    tokio::spawn(async move {
        let queue = {
            let mut ctrl = controller().lock().await;
            ctrl.remember_queue(queue_id);
            ctrl.active_queue(queue_id)
        };

        if is_in_game_phase(phase.as_deref()) {
            let Some(queue) = queue else {
                log::debug!(target: "augment::overlay", "等待 gameflow session 提供队列信息");
                return;
            };
            if !is_hextech_queue(Some(queue)) {
                end_session(&app, "non-hextech").await;
                return;
            }
            let should_reveal = controller().lock().await.mark_in_game_and_should_reveal();
            if should_reveal {
                reveal_panel(app, champion_id, false).await;
            }
            return;
        }

        if phase.as_deref() == Some("ChampSelect") {
            let hextech = {
                let mut ctrl = controller().lock().await;
                ctrl.in_game = false;
                is_hextech_queue(ctrl.active_queue(queue_id))
            };
            if hextech {
                let _ = prefetch_guide(champion_id).await;
            }
            return;
        }

        let in_game = controller().lock().await.in_game;
        if should_end_overlay(phase.as_deref(), in_game) {
            end_session(&app, "gameflow-phase").await;
        }
    });
}

pub fn on_champion_ready(champion_id: Option<i32>, queue_id: Option<i64>) {
    tokio::spawn(async move {
        let hextech = {
            let mut ctrl = controller().lock().await;
            ctrl.remember_queue(queue_id);
            is_hextech_queue(ctrl.active_queue(queue_id))
        };
        if hextech {
            let _ = prefetch_guide(champion_id).await;
        }
    });
}

pub fn on_in_game_champion(app: AppHandle, champion_id: Option<i32>, queue_id: Option<i64>) {
    tokio::spawn(async move {
        let (enabled, in_game, visible, hextech) = {
            let mut ctrl = controller().lock().await;
            ctrl.remember_queue(queue_id);
            (
                ctrl.enabled,
                ctrl.in_game,
                ctrl.visible,
                is_hextech_queue(ctrl.active_queue(queue_id)),
            )
        };
        if !hextech {
            end_session(&app, "non-hextech").await;
            return;
        }
        if !enabled || !in_game {
            return;
        }
        if let Some(guide) = prefetch_guide(champion_id).await.filter(|_| visible) {
            emit_guide(&app, &guide).await;
        }
    });
}

pub async fn toggle_side_panel(app: AppHandle) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();
    let last = LAST_TOGGLE_MS.swap(now, Ordering::Relaxed);
    if now.saturating_sub(last) < TOGGLE_DEBOUNCE_MS {
        return;
    }

    let action = {
        let ctrl = controller().lock().await;
        toggle_action(ctrl.enabled, ctrl.visible, ctrl.can_manual_show())
    };
    match action {
        ToggleAction::Hide => hide_side_panel(app).await,
        ToggleAction::Show => reveal_panel(app, None, true).await,
        ToggleAction::Ignore => log::debug!(target: "augment::overlay", "当前没有可显示的海克斯推荐"),
    }
}

pub async fn is_visible() -> bool {
    controller().lock().await.visible
}

/// 手动隐藏只改变可见性，保留本局队列、英雄与推荐快照，以便快捷键再次显示。
pub async fn hide_side_panel(app: AppHandle) {
    let mut ctrl = controller().lock().await;
    ctrl.visible = false;
    ctrl.revealing = false;
    ctrl.visibility_generation = ctrl.visibility_generation.wrapping_add(1);
    drop(ctrl);
    window::conceal_side_panel(&app);
    emit_visibility(&app, false);
}

pub async fn snapshot() -> Option<AugmentDetectedPayload> {
    controller().lock().await.last_payload.clone()
}

async fn reveal_panel(app: AppHandle, champion_id: Option<i32>, force: bool) {
    let (cached, lifecycle_generation, visibility_generation) = {
        let mut ctrl = controller().lock().await;
        if !ctrl.enabled || (!force && ctrl.visible && ctrl.has_guide_for(champion_id)) || ctrl.revealing {
            return;
        }
        ctrl.revealing = true;
        ctrl.visibility_generation = ctrl.visibility_generation.wrapping_add(1);
        (
            ctrl.guide.clone().filter(|_| ctrl.has_guide_for(champion_id)),
            ctrl.lifecycle_generation,
            ctrl.visibility_generation,
        )
    };

    if let Err(error) = window::ensure_window(&app) {
        log::error!(target: "augment::overlay", "创建侧栏失败: {error}");
        finish_reveal(lifecycle_generation, visibility_generation, false).await;
        return;
    }
    if !is_current_reveal(lifecycle_generation, visibility_generation).await {
        return;
    }

    if let Some(guide) = &cached {
        emit_guide(&app, guide).await;
    } else {
        emit_pending(&app, champion_id).await;
    }
    if !is_current_reveal(lifecycle_generation, visibility_generation).await {
        return;
    }
    window::show_side_panel(&app);
    controller().lock().await.visible = true;
    emit_visibility(&app, true);

    let resolved = resolve_champion_id(champion_id).await;
    if !is_current_reveal(lifecycle_generation, visibility_generation).await {
        return;
    }
    match prefetch_guide(resolved).await {
        Some(guide) if is_current_reveal(lifecycle_generation, visibility_generation).await => {
            emit_guide(&app, &guide).await
        }
        None if cached.is_none() => emit_payload(&app, resolved, None, Vec::new(), Vec::new(), false).await,
        _ => {}
    }
    finish_reveal(lifecycle_generation, visibility_generation, true).await;
}

async fn finish_reveal(lifecycle_generation: u64, visibility_generation: u64, visible: bool) {
    let mut ctrl = controller().lock().await;
    if ctrl.lifecycle_generation == lifecycle_generation && ctrl.visibility_generation == visibility_generation {
        ctrl.revealing = false;
        ctrl.visible = visible;
    }
}

async fn is_current_reveal(lifecycle_generation: u64, visibility_generation: u64) -> bool {
    let ctrl = controller().lock().await;
    ctrl.lifecycle_generation == lifecycle_generation && ctrl.visibility_generation == visibility_generation
}

async fn resolve_champion_id(known: Option<i32>) -> Option<i32> {
    if let Some(id) = known.filter(|id| *id > 0) {
        return Some(id);
    }
    if let Some(id) = controller().lock().await.cached_champion_id() {
        return Some(id);
    }
    for _ in 0..10 {
        if let Some(id) = liveclient_champion_id().await {
            return Some(id);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    None
}

fn live_champion_alias(raw_name: &str) -> &str {
    raw_name
        .rsplit(['_', ' '])
        .find(|part| !part.is_empty())
        .unwrap_or(raw_name)
}

async fn liveclient_champion_id() -> Option<i32> {
    let player = liveclient::service::get_local_live_player().await.ok()?;
    champion_data::get_champion_id_by_name(&player.champion_name)
        .or_else(|| champion_data::get_champion_id_by_name(live_champion_alias(&player.raw_champion_name)))
}

async fn end_session(app: &AppHandle, reason: &str) {
    let was_active = {
        let mut ctrl = controller().lock().await;
        let was_active = ctrl.visible || ctrl.in_game || ctrl.guide.is_some() || ctrl.last_payload.is_some();
        ctrl.visible = false;
        ctrl.in_game = false;
        ctrl.revealing = false;
        ctrl.queue_id = None;
        ctrl.expected_champion_id = None;
        ctrl.prefetching = None;
        ctrl.guide = None;
        ctrl.last_payload = None;
        ctrl.lifecycle_generation = ctrl.lifecycle_generation.wrapping_add(1);
        ctrl.visibility_generation = ctrl.visibility_generation.wrapping_add(1);
        was_active
    };
    window::hide_side_panel(app);
    if was_active {
        emit_visibility(app, false);
        emit_cleared(app, reason);
    }
}

async fn prefetch_guide(champion_id: Option<i32>) -> Option<ChampionGuide> {
    let champion_id = champion_id.filter(|id| *id > 0)?;
    {
        let mut ctrl = controller().lock().await;
        if let Some(guide) = ctrl.guide.as_ref().filter(|guide| guide.champion_id == champion_id) {
            return Some(guide.clone());
        }
        if ctrl.prefetching == Some(champion_id) {
            drop(ctrl);
            return wait_for_guide(champion_id).await;
        }
        ctrl.expected_champion_id = Some(champion_id);
        ctrl.prefetching = Some(champion_id);
    }

    let loaded = service::get_champion_detail(champion_id).await.ok().map(build_guide);
    let mut ctrl = controller().lock().await;
    ctrl.prefetching = None;
    match loaded {
        Some(guide) if ctrl.expected_champion_id == Some(champion_id) => {
            ctrl.guide = Some(guide.clone());
            Some(guide)
        }
        _ => None,
    }
}

async fn wait_for_guide(champion_id: i32) -> Option<ChampionGuide> {
    for _ in 0..100 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let ctrl = controller().lock().await;
        if let Some(guide) = ctrl.guide.as_ref().filter(|guide| guide.champion_id == champion_id) {
            return Some(guide.clone());
        }
        if ctrl.prefetching != Some(champion_id) {
            return None;
        }
    }
    None
}

fn build_guide(detail: HextechChampionDetail) -> ChampionGuide {
    let recommended_augments = detail.augments.iter().map(to_guide_augment).collect();
    let by_id: HashMap<i32, &HextechAugmentStat> = detail.augments.iter().map(|item| (item.id, item)).collect();
    let recommended_trios = detail
        .augment_trios
        .iter()
        .take(GUIDE_TRIOS)
        .map(|trio| OverlayGuideTrio {
            augments: trio
                .augment_ids
                .iter()
                .map(|id| {
                    by_id
                        .get(id)
                        .map(|stat| to_guide_augment(stat))
                        .unwrap_or_else(|| missing_augment(*id))
                })
                .collect(),
            win_rate: trio.win_rate,
            pick_rate: trio.pick_rate,
            games: trio.games,
        })
        .collect();
    ChampionGuide {
        champion_id: detail.summary.champion_id,
        name: detail.summary.name,
        recommended_augments,
        recommended_trios,
    }
}

fn to_guide_augment(stat: &HextechAugmentStat) -> OverlayGuideAugment {
    OverlayGuideAugment {
        id: stat.id,
        name: stat.name.clone(),
        icon_url: stat.icon_url.clone(),
        rarity_name: stat.rarity_name.clone(),
        rarity_display_name: stat.rarity_display_name.clone(),
        win_rate: stat.win_rate,
        pick_rate: stat.pick_rate,
        games: stat.games,
        tier: stat.tier,
    }
}

fn missing_augment(id: i32) -> OverlayGuideAugment {
    OverlayGuideAugment {
        id,
        name: format!("#{id}"),
        icon_url: String::new(),
        rarity_name: String::new(),
        rarity_display_name: String::new(),
        win_rate: 0.0,
        pick_rate: 0.0,
        games: None,
        tier: None,
    }
}

async fn emit_pending(app: &AppHandle, champion_id: Option<i32>) {
    emit_payload(app, champion_id, None, Vec::new(), Vec::new(), true).await;
}

async fn emit_guide(app: &AppHandle, guide: &ChampionGuide) {
    emit_payload(
        app,
        Some(guide.champion_id),
        Some(guide.name.clone()),
        guide.recommended_augments.clone(),
        guide.recommended_trios.clone(),
        false,
    )
    .await;
}

async fn emit_payload(
    app: &AppHandle,
    champion_id: Option<i32>,
    champion_name: Option<String>,
    recommended_augments: Vec<OverlayGuideAugment>,
    recommended_trios: Vec<OverlayGuideTrio>,
    winrate_pending: bool,
) {
    let payload = AugmentDetectedPayload {
        success: true,
        game_phase: "augment-guide".to_string(),
        champion_id,
        champion_name,
        recommended_augments,
        recommended_trios,
        timestamp: chrono::Utc::now().timestamp_millis(),
        winrate_pending,
    };
    controller().lock().await.last_payload = Some(payload.clone());
    let _ = app.emit("augment-detected", &payload);
    let _ = app.emit_to(window::SIDE_PANEL_LABEL, "augment-detected", &payload);
}

fn emit_visibility(app: &AppHandle, visible: bool) {
    let _ = app.emit("augment-overlay-visibility", visible);
    let _ = app.emit_to(window::SIDE_PANEL_LABEL, "augment-overlay-visibility", visible);
}

fn emit_cleared(app: &AppHandle, reason: &str) {
    let payload = AugmentClearedPayload {
        success: true,
        game_phase: "augment-cleared".to_string(),
        reason: reason.to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    let _ = app.emit("augment-cleared", &payload);
    let _ = app.emit_to(window::SIDE_PANEL_LABEL, "augment-cleared", &payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_can_hide_then_show_without_clearing_session() {
        let mut ctrl = OverlayController::new();
        ctrl.queue_id = Some(HEXTECH_QUEUE_ID);
        ctrl.in_game = true;
        ctrl.visible = true;
        assert_eq!(
            toggle_action(ctrl.enabled, ctrl.visible, ctrl.can_manual_show()),
            ToggleAction::Hide
        );

        ctrl.visible = false;
        assert!(ctrl.in_game);
        assert_eq!(ctrl.queue_id, Some(HEXTECH_QUEUE_ID));
        assert_eq!(
            toggle_action(ctrl.enabled, ctrl.visible, ctrl.can_manual_show()),
            ToggleAction::Show
        );
    }

    #[test]
    fn overlay_is_limited_to_hextech_queue() {
        assert!(is_hextech_queue(Some(HEXTECH_QUEUE_ID)));
        assert!(!is_hextech_queue(Some(450)));
        assert!(!is_hextech_queue(None));
    }

    #[test]
    fn transient_none_does_not_end_active_game() {
        assert!(!should_end_overlay(None, true));
        assert!(should_end_overlay(None, false));
        assert!(!should_end_overlay(Some("InProgress"), true));
        assert!(should_end_overlay(Some("EndOfGame"), true));
    }
}
