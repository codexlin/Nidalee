//! 海克斯推荐侧栏：选人预载，对局展示；OCR 顶栏默认关闭，由设置打开。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use once_cell::sync::OnceCell;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use super::scan;
use super::types::{
    AugmentClearedPayload, AugmentDetectedPayload, OverlayAugment, OverlayGuideAugment, OverlayGuideTrio,
};
use super::window;
use crate::infrastructure::data_services::champion_data;
use crate::infrastructure::data_services::external::hextech::service;
use crate::infrastructure::data_services::external::hextech::types::{
    HextechAugmentStat, HextechChampionDetail,
};
use crate::infrastructure::real_time::liveclient;

const GUIDE_TRIOS: usize = 5;
const HEXTECH_QUEUE_ID: i64 = 2400;

#[derive(Clone)]
struct ChampionGuide {
    champion_id: i32,
    name: String,
    recommended_augments: Vec<OverlayGuideAugment>,
    recommended_trios: Vec<OverlayGuideTrio>,
}

struct OverlayController {
    enabled: bool,
    ocr_enabled: bool,
    visible: bool,
    in_game: bool,
    revealing: bool,
    queue_id: Option<i64>,
    expected_champion_id: Option<i32>,
    prefetching: Option<i32>,
    guide: Option<ChampionGuide>,
    last_payload: Option<AugmentDetectedPayload>,
    current_offers: Vec<OverlayAugment>,
    cards_dismissed: bool,
    scanning: bool,
    scan_generation: u64,
}

impl OverlayController {
    fn new() -> Self {
        Self {
            enabled: true,
            ocr_enabled: false,
            visible: false,
            in_game: false,
            revealing: false,
            queue_id: None,
            expected_champion_id: None,
            prefetching: None,
            guide: None,
            last_payload: None,
            current_offers: Vec::new(),
            cards_dismissed: false,
            scanning: false,
            scan_generation: 0,
        }
    }

    fn can_scan(&self) -> bool {
        self.enabled && self.ocr_enabled && self.in_game && is_hextech_queue(self.queue_id)
    }

    /// 第一次进入对局才自动弹出；之后只由快捷键切换显隐。
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
        let Some(guide) = &self.guide else {
            return false;
        };
        match champion_id.filter(|id| *id > 0) {
            Some(id) => guide.champion_id == id,
            None => true,
        }
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
        is_hextech_queue(self.queue_id)
            && (self.in_game || self.guide.is_some() || self.last_payload.is_some())
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

fn toggle_action(enabled: bool, window_visible: bool, can_show: bool) -> ToggleAction {
    if !enabled {
        return ToggleAction::Ignore;
    }
    if window_visible {
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
        Some("WaitingForStats" | "PreEndOfGame" | "EndOfGame" | "Lobby" | "Matchmaking" | "ReadyCheck" | "Terminated" | "FailedToLaunch" | "TerminatedInError") => {
            true
        }
        _ => true,
    }
}

static CONTROLLER: OnceCell<Mutex<OverlayController>> = OnceCell::new();
static LAST_TOGGLE_MS: AtomicU64 = AtomicU64::new(0);

fn controller() -> &'static Mutex<OverlayController> {
    CONTROLLER.get_or_init(|| Mutex::new(OverlayController::new()))
}

pub fn on_gameflow_phase(
    app: AppHandle,
    phase: Option<String>,
    champion_id: Option<i32>,
    queue_id: Option<i64>,
) {
    tokio::spawn(async move {
        let queue = {
            let mut ctrl = controller().lock().await;
            ctrl.remember_queue(queue_id);
            ctrl.active_queue(queue_id)
        };

        if is_in_game_phase(phase.as_deref()) {
            if queue.is_none() {
                log::debug!("[augment-overlay] 对局阶段但还没有 queue，等待 session");
                return;
            }
            if !is_hextech_queue(queue) {
                log::info!("[augment-overlay] 非海克斯对局，不显示侧栏 queue={queue:?}");
                hide_all(app, "non-hextech").await;
                return;
            }
            let (should_reveal, ocr_enabled) = {
                let mut ctrl = controller().lock().await;
                let should_reveal = ctrl.mark_in_game_and_should_reveal();
                (should_reveal, ctrl.ocr_enabled)
            };
            if ocr_enabled {
                scan::start(app.clone());
            }
            if should_reveal {
                reveal_panel(app, champion_id, false).await;
            }
            return;
        }
        if phase.as_deref() == Some("ChampSelect") {
            let mut ctrl = controller().lock().await;
            ctrl.in_game = false;
            let hextech = is_hextech_queue(ctrl.active_queue(queue_id));
            drop(ctrl);
            if hextech {
                let _ = prefetch_guide(champion_id).await;
            }
            return;
        }
        let in_game = controller().lock().await.in_game;
        if should_end_overlay(phase.as_deref(), in_game) {
            hide_all(app, "gameflow-phase").await;
        } else {
            log::debug!("[augment-overlay] 忽略短暂的 gameflow 清空 phase={phase:?}");
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
        let (enabled, in_game, visible, hextech, guide) = {
            let mut ctrl = controller().lock().await;
            ctrl.remember_queue(queue_id);
            (
                ctrl.enabled,
                ctrl.in_game,
                ctrl.visible,
                is_hextech_queue(ctrl.active_queue(queue_id)),
                ctrl.guide.clone(),
            )
        };
        if !hextech {
            hide_all(app, "non-hextech").await;
            return;
        }
        let _ = prefetch_guide(champion_id).await;
        if !enabled || !in_game || !visible {
            return;
        }
        if let Some(guide) = guide {
            emit_guide(&app, &guide).await;
        }
    });
}

pub async fn toggle_side_panel(app: AppHandle) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let last = LAST_TOGGLE_MS.load(Ordering::Relaxed);
    if now.saturating_sub(last) < 280 {
        return;
    }
    LAST_TOGGLE_MS.store(now, Ordering::Relaxed);

    let (enabled, visible, can_show) = {
        let ctrl = controller().lock().await;
        (ctrl.enabled, ctrl.visible, ctrl.can_manual_show())
    };
    match toggle_action(enabled, visible, can_show) {
        ToggleAction::Hide => hide_side_panel(app).await,
        ToggleAction::Show => reveal_panel(app, None, true).await,
        ToggleAction::Ignore => {
            log::info!("[augment-overlay] 快捷键显示忽略：不是海克斯对局或没有推荐");
        }
    }
}

pub async fn is_visible() -> bool {
    controller().lock().await.visible
}

pub async fn set_enabled(app: AppHandle, enabled: bool) {
    let mut ctrl = controller().lock().await;
    ctrl.enabled = enabled;
    if !enabled {
        drop(ctrl);
        hide_all(app, "disabled").await;
    }
}

pub async fn is_ocr_enabled() -> bool {
    controller().lock().await.ocr_enabled
}

pub async fn set_ocr_enabled(app: AppHandle, enabled: bool) {
    let should_start = {
        let mut ctrl = controller().lock().await;
        if ctrl.ocr_enabled == enabled {
            return;
        }
        ctrl.ocr_enabled = enabled;
        if enabled {
            ctrl.can_scan()
        } else {
            ctrl.scanning = false;
            ctrl.scan_generation = ctrl.scan_generation.wrapping_add(1);
            false
        }
    };
    if enabled {
        log::info!("[augment-overlay] 已开启本轮三选识别");
        if should_start {
            scan::start(app);
        }
        return;
    }
    clear_current_offers(&app).await;
    log::info!("[augment-overlay] 已关闭本轮三选识别");
}

pub async fn hide_overlay(app: AppHandle) {
    hide_side_panel(app).await;
}

pub async fn hide_side_panel(app: AppHandle) {
    let mut ctrl = controller().lock().await;
    ctrl.visible = false;
    drop(ctrl);
    window::conceal_side_panel(&app);
    emit_visibility(&app, false);
    log::info!("[augment-overlay] 已隐藏推荐侧栏");
}

pub async fn snapshot() -> Option<AugmentDetectedPayload> {
    controller().lock().await.last_payload.clone()
}

pub async fn begin_scan() -> bool {
    let mut ctrl = controller().lock().await;
    if ctrl.scanning || !ctrl.can_scan() {
        return false;
    }
    ctrl.scanning = true;
    true
}

pub async fn scan_generation() -> u64 {
    controller().lock().await.scan_generation
}

pub async fn should_keep_scanning(generation: u64) -> bool {
    let ctrl = controller().lock().await;
    ctrl.scanning && ctrl.scan_generation == generation && ctrl.can_scan()
}

pub async fn end_scan(generation: u64) {
    let mut ctrl = controller().lock().await;
    if ctrl.scan_generation == generation {
        ctrl.scanning = false;
    }
}

pub async fn cached_guide_augments() -> Vec<OverlayGuideAugment> {
    controller()
        .lock()
        .await
        .guide
        .as_ref()
        .map(|guide| guide.recommended_augments.clone())
        .unwrap_or_default()
}

fn offer_ids(offers: &[OverlayAugment]) -> Vec<i32> {
    offers.iter().filter_map(|item| item.id).collect()
}

pub async fn publish_current_offers(app: &AppHandle, offers: Vec<OverlayAugment>) {
    let mut ctrl = controller().lock().await;
    if offer_ids(&offers) != offer_ids(&ctrl.current_offers) {
        ctrl.cards_dismissed = false;
    }
    ctrl.current_offers = offers.clone();
    let dismissed = ctrl.cards_dismissed;
    let Some(mut payload) = ctrl.last_payload.clone() else {
        return;
    };
    payload.augments = offers;
    payload.timestamp = chrono::Utc::now().timestamp_millis();
    ctrl.last_payload = Some(payload.clone());
    drop(ctrl);
    emit_detected(app, &payload);
    if !dismissed {
        window::show_card_window(app);
    }
}

pub async fn clear_current_offers(app: &AppHandle) {
    let mut ctrl = controller().lock().await;
    if ctrl.current_offers.is_empty() {
        return;
    }
    ctrl.current_offers.clear();
    ctrl.cards_dismissed = false;
    let Some(mut payload) = ctrl.last_payload.clone() else {
        window::hide_card_window(app);
        return;
    };
    payload.augments.clear();
    payload.timestamp = chrono::Utc::now().timestamp_millis();
    ctrl.last_payload = Some(payload.clone());
    drop(ctrl);
    emit_detected(app, &payload);
    window::hide_card_window(app);
}

pub async fn dismiss_card_overlay(app: AppHandle) {
    let mut ctrl = controller().lock().await;
    ctrl.cards_dismissed = true;
    drop(ctrl);
    window::hide_card_window(&app);
}

async fn reveal_panel(app: AppHandle, champion_id: Option<i32>, force: bool) {
    let cached_guide = {
        let mut ctrl = controller().lock().await;
        if !ctrl.enabled {
            return;
        }
        if !force && ctrl.visible && ctrl.has_guide_for(champion_id) {
            return;
        }
        if ctrl.revealing && !force {
            return;
        }
        ctrl.revealing = true;
        ctrl.guide.clone().filter(|_| ctrl.has_guide_for(champion_id))
    };

    if let Err(error) = window::ensure_window(&app) {
        log::error!("[augment-overlay] {error}");
        controller().lock().await.revealing = false;
        return;
    }

    if let Some(guide) = &cached_guide {
        emit_guide(&app, guide).await;
    } else {
        emit_pending(&app, champion_id).await;
    }
    window::show_side_panel(&app);
    {
        let mut ctrl = controller().lock().await;
        ctrl.visible = true;
    }
    emit_visibility(&app, true);

    let champion_id = resolve_champion_id(champion_id).await;
    match prefetch_guide(champion_id).await {
        Some(guide) => emit_guide(&app, &guide).await,
        None if cached_guide.is_some() => {}
        None => {
            log::warn!("[augment-overlay] 进入对局但没有英雄推荐数据 champion={champion_id:?}");
            emit_payload(&app, champion_id, None, Vec::new(), Vec::new(), false).await;
        }
    }

    let mut ctrl = controller().lock().await;
    ctrl.revealing = false;
    ctrl.visible = true;
    log::info!("[augment-overlay] 已显示推荐侧栏 champion={champion_id:?}");
}

async fn resolve_champion_id(known: Option<i32>) -> Option<i32> {
    if let Some(id) = known.filter(|id| *id > 0) {
        return Some(id);
    }
    if let Some(id) = controller().lock().await.cached_champion_id() {
        return Some(id);
    }
    for attempt in 1..=10 {
        match liveclient_champion_id().await {
            Some(id) => {
                log::info!("[augment-overlay] 从对局 LiveClient 恢复英雄 {id} (第 {attempt} 次)");
                return Some(id);
            }
            None => {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    None
}

fn champion_id_from_live_names(champion_name: &str, raw_champion_name: &str) -> Option<i32> {
    champion_data::get_champion_id_by_name(champion_name)
        .or_else(|| champion_data::get_champion_id_by_name(live_champion_alias(raw_champion_name)))
}

fn live_champion_alias(raw_champion_name: &str) -> &str {
    raw_champion_name
        .rsplit(['_', ' '])
        .find(|part| !part.is_empty())
        .unwrap_or(raw_champion_name)
}

async fn liveclient_champion_id() -> Option<i32> {
    match liveclient::service::get_local_live_player().await {
        Ok(player) => {
            if let Some(id) = champion_id_from_live_names(&player.champion_name, &player.raw_champion_name) {
                return Some(id);
            }
            log::warn!(
                "[augment-overlay] LiveClient 英雄未能映射: {} / {}",
                player.champion_name,
                player.raw_champion_name
            );
            None
        }
        Err(error) => {
            log::warn!("[augment-overlay] 定位对局英雄失败: {error}");
            None
        }
    }
}

async fn hide_all(app: AppHandle, reason: &str) {
    let mut ctrl = controller().lock().await;
    let was_visible = ctrl.visible;
    ctrl.visible = false;
    ctrl.in_game = false;
    ctrl.revealing = false;
    ctrl.scanning = false;
    ctrl.scan_generation = ctrl.scan_generation.wrapping_add(1);
    ctrl.current_offers.clear();
    ctrl.cards_dismissed = false;
    ctrl.last_payload = None;
    if matches!(reason, "gameflow-phase" | "disabled" | "non-hextech") {
        ctrl.guide = None;
        ctrl.queue_id = None;
        ctrl.expected_champion_id = None;
        ctrl.prefetching = None;
    }
    drop(ctrl);

    emit_visibility(&app, false);
    emit_cleared(&app, reason);
    window::hide_side_panel(&app);
    if was_visible {
        log::info!("[augment-overlay] 已隐藏推荐侧栏 ({reason})");
    }
}

async fn prefetch_guide(champion_id: Option<i32>) -> Option<ChampionGuide> {
    let champion_id = champion_id.filter(|id| *id > 0)?;
    {
        let mut ctrl = controller().lock().await;
        if let Some(guide) = &ctrl.guide {
            if guide.champion_id == champion_id {
                return Some(guide.clone());
            }
        }
        if ctrl.prefetching == Some(champion_id) {
            drop(ctrl);
            return wait_for_guide(champion_id).await;
        }
        ctrl.expected_champion_id = Some(champion_id);
        ctrl.prefetching = Some(champion_id);
    }

    let loaded = load_guide(champion_id).await;
    let mut ctrl = controller().lock().await;
    ctrl.prefetching = None;
    match loaded {
        Some(guide) if ctrl.expected_champion_id == Some(champion_id) => {
            log::info!(
                "[augment-overlay] 已预载 {} 推荐 · 三连 {} · 增强 {}",
                guide.name,
                guide.recommended_trios.len(),
                guide.recommended_augments.len()
            );
            ctrl.guide = Some(guide.clone());
            Some(guide)
        }
        _ => None,
    }
}

async fn wait_for_guide(champion_id: i32) -> Option<ChampionGuide> {
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let ctrl = controller().lock().await;
        if let Some(guide) = &ctrl.guide {
            if guide.champion_id == champion_id {
                return Some(guide.clone());
            }
        }
        if ctrl.prefetching != Some(champion_id) && ctrl.guide.as_ref().map(|g| g.champion_id) != Some(champion_id)
        {
            return None;
        }
    }
    None
}

async fn load_guide(champion_id: i32) -> Option<ChampionGuide> {
    match service::get_champion_detail(champion_id).await {
        Ok(detail) => Some(build_guide(detail)),
        Err(error) => {
            log::warn!("[augment-overlay] 海克斯详情失败: {error}");
            None
        }
    }
}

fn build_guide(detail: HextechChampionDetail) -> ChampionGuide {
    let recommended_augments: Vec<OverlayGuideAugment> =
        detail.augments.iter().map(to_guide_augment).collect();
    let by_id: std::collections::HashMap<i32, &HextechAugmentStat> =
        detail.augments.iter().map(|item| (item.id, item)).collect();
    let recommended_trios: Vec<OverlayGuideTrio> = detail
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
                        .unwrap_or_else(|| OverlayGuideAugment {
                            id: *id,
                            name: format!("#{id}"),
                            icon_url: String::new(),
                            rarity_name: String::new(),
                            rarity_display_name: String::new(),
                            win_rate: 0.0,
                            pick_rate: 0.0,
                            games: None,
                            tier: None,
                        })
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

async fn emit_pending(app: &AppHandle, champion_id: Option<i32>) {
    let (champion_name, recommended_augments, recommended_trios) = {
        let ctrl = controller().lock().await;
        match &ctrl.guide {
            Some(guide) if Some(guide.champion_id) == champion_id => (
                Some(guide.name.clone()),
                guide.recommended_augments.clone(),
                guide.recommended_trios.clone(),
            ),
            _ => (None, Vec::new(), Vec::new()),
        }
    };
    emit_payload(
        app,
        champion_id,
        champion_name,
        recommended_augments,
        recommended_trios,
        true,
    )
    .await;
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
    let pending = winrate_pending && recommended_augments.is_empty() && recommended_trios.is_empty();
    let offers = controller().lock().await.current_offers.clone();
    let payload = AugmentDetectedPayload {
        success: true,
        game_phase: "augment-select".to_string(),
        champion_id,
        champion_name,
        augments: offers,
        recommended_augments,
        recommended_trios,
        analysis_confidence: 1.0,
        partial_update: false,
        timestamp: chrono::Utc::now().timestamp_millis(),
        winrate_pending: pending,
    };
    controller().lock().await.last_payload = Some(payload.clone());
    emit_detected(app, &payload);
}

fn emit_detected(app: &AppHandle, payload: &AugmentDetectedPayload) {
    let _ = app.emit("augment-detected", payload);
    let _ = app.emit_to(window::SIDE_PANEL_LABEL, "augment-detected", payload);
    let _ = app.emit_to(window::CARD_LABEL, "augment-detected", payload);
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
    let _ = app.emit_to(window::CARD_LABEL, "augment-cleared", &payload);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::data_services::external::hextech::types::{
        HextechAugmentTrio, HextechChampionSummary,
    };

    fn sample_stat(id: i32, name: &str, win_rate: f64) -> HextechAugmentStat {
        HextechAugmentStat {
            id,
            name: name.into(),
            rarity: 1,
            rarity_name: "gold".into(),
            rarity_display_name: "黄金".into(),
            icon_url: format!("icon-{id}"),
            win_rate,
            pick_rate: 0.1,
            games: Some(50),
            wins: Some(25),
            tier: Some(1),
            rank: Some(1),
        }
    }

    #[test]
    fn build_guide_keeps_top_augments_and_trios() {
        let detail = HextechChampionDetail {
            summary: HextechChampionSummary {
                champion_id: 56,
                name: "魔腾".into(),
                alias: "Nocturne".into(),
                title: String::new(),
                icon_url: String::new(),
                roles: vec!["assassin".into()],
                win_rate: 0.52,
                pick_rate: 0.04,
                tier: Some(2),
                data_version: "1".into(),
                game_patch: None,
            },
            augments: vec![
                sample_stat(11, "灵魂虹吸", 0.61),
                sample_stat(12, "炽烈黎明", 0.54),
                sample_stat(13, "珠宝护手", 0.49),
            ],
            augment_trios: vec![HextechAugmentTrio {
                augment_ids: vec![11, 12, 13],
                win_rate: 0.66,
                pick_rate: 0.08,
                games: Some(30),
                wins: Some(20),
            }],
            summoner_spells: vec![],
            skill_orders: vec![],
            starting_items: vec![],
            core_items: vec![],
        };
        let guide = build_guide(detail);
        assert_eq!(guide.name, "魔腾");
        assert_eq!(guide.recommended_augments.len(), 3);
        assert_eq!(guide.recommended_augments[0].name, "灵魂虹吸");
        assert_eq!(guide.recommended_augments[0].rarity_name, "gold");
        assert_eq!(guide.recommended_trios.len(), 1);
        assert_eq!(guide.recommended_trios[0].augments.len(), 3);
        assert_eq!(guide.recommended_trios[0].augments[1].name, "炽烈黎明");
        assert_eq!(guide.recommended_trios[0].win_rate, 0.66);
    }

    #[test]
    fn live_raw_champion_name_uses_alias_suffix() {
        assert_eq!(
            live_champion_alias("game_character_displayname_Nocturne"),
            "Nocturne"
        );
        assert_eq!(live_champion_alias("Nocturne"), "Nocturne");
    }

    #[test]
    fn auto_show_only_on_first_enter() {
        let mut ctrl = OverlayController::new();
        assert!(ctrl.mark_in_game_and_should_reveal());
        assert!(!ctrl.mark_in_game_and_should_reveal());
        ctrl.in_game = false;
        ctrl.enabled = false;
        assert!(!ctrl.mark_in_game_and_should_reveal());
    }

    #[test]
    fn cached_champion_skips_when_guide_matches() {
        let mut ctrl = OverlayController::new();
        assert!(!ctrl.has_guide_for(Some(56)));
        ctrl.guide = Some(ChampionGuide {
            champion_id: 56,
            name: "魔腾".into(),
            recommended_augments: vec![],
            recommended_trios: vec![],
        });
        assert!(ctrl.has_guide_for(Some(56)));
        assert!(ctrl.has_guide_for(None));
        assert!(!ctrl.has_guide_for(Some(1)));
        ctrl.expected_champion_id = Some(56);
        assert_eq!(ctrl.cached_champion_id(), Some(56));
    }

    #[test]
    fn insert_toggles_hide_and_show() {
        assert_eq!(toggle_action(true, true, true), ToggleAction::Hide);
        assert_eq!(toggle_action(true, false, true), ToggleAction::Show);
        assert_eq!(toggle_action(true, false, false), ToggleAction::Ignore);
        assert_eq!(toggle_action(false, false, true), ToggleAction::Ignore);
    }

    #[test]
    fn insert_can_show_again_with_cached_guide() {
        let mut ctrl = OverlayController::new();
        ctrl.in_game = false;
        ctrl.visible = false;
        ctrl.queue_id = Some(2400);
        assert!(!ctrl.can_manual_show());
        ctrl.guide = Some(ChampionGuide {
            champion_id: 56,
            name: "魔腾".into(),
            recommended_augments: vec![],
            recommended_trios: vec![],
        });
        assert!(ctrl.can_manual_show());
        assert_eq!(toggle_action(true, false, ctrl.can_manual_show()), ToggleAction::Show);
    }

    #[test]
    fn scan_requires_ocr_flag() {
        let mut ctrl = OverlayController::new();
        ctrl.enabled = true;
        ctrl.in_game = true;
        ctrl.queue_id = Some(2400);
        assert!(!ctrl.can_scan());
        ctrl.ocr_enabled = true;
        assert!(ctrl.can_scan());
        ctrl.ocr_enabled = false;
        assert!(!ctrl.can_scan());
    }

    #[test]
    fn overlay_only_for_hextech_queue() {
        assert!(is_hextech_queue(Some(2400)));
        assert!(!is_hextech_queue(Some(450)));
        assert!(!is_hextech_queue(Some(420)));
        assert!(!is_hextech_queue(None));
        let mut ctrl = OverlayController::new();
        ctrl.in_game = true;
        ctrl.guide = Some(ChampionGuide {
            champion_id: 56,
            name: "魔腾".into(),
            recommended_augments: vec![],
            recommended_trios: vec![],
        });
        assert!(!ctrl.can_manual_show());
        ctrl.queue_id = Some(2400);
        assert!(ctrl.can_manual_show());
    }

    #[test]
    fn transient_gameflow_clear_does_not_end_in_game_overlay() {
        assert!(!should_end_overlay(None, true));
        assert!(!should_end_overlay(Some("None"), true));
        assert!(should_end_overlay(None, false));
        assert!(should_end_overlay(Some("EndOfGame"), true));
        assert!(should_end_overlay(Some("Lobby"), true));
        assert!(!should_end_overlay(Some("InProgress"), true));
    }
}
