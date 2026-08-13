//! 对局里 OCR 当前三张海克斯卡，标出本轮首选。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::OnceCell;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use super::capture::{
    capture_game_frame, crop_fallback_title_slots, crop_title_slots, detect_augment_ui_gate, CaptureFrame,
};
use super::matcher::{has_meaningful_slot_text, match_slot_text};
use super::ocr::OcrEngine;
use super::offer::{mark_recommended, OfferScore};
use super::state;
use super::types::OverlayAugment;
use crate::infrastructure::data_services::external::hextech::service;

static ENGINE: OnceCell<Mutex<Option<Arc<OcrEngine>>>> = OnceCell::new();

fn engine_slot() -> &'static Mutex<Option<Arc<OcrEngine>>> {
    ENGINE.get_or_init(|| Mutex::new(None))
}

fn model_dir(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(dir) = app.path().resource_dir() {
        let path = dir.join("models").join("paddleocr");
        if path.join("rec").join("inference.onnx").exists() {
            return Some(path);
        }
    }
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("models")
        .join("paddleocr");
    if dev.join("rec").join("inference.onnx").exists() {
        Some(dev)
    } else {
        None
    }
}

async fn ensure_engine(app: &AppHandle) -> Option<Arc<OcrEngine>> {
    {
        let slot = engine_slot().lock().await;
        if let Some(engine) = slot.as_ref() {
            return Some(engine.clone());
        }
    }
    let Some(dir) = model_dir(app) else {
        log::warn!("[augment-overlay] 未找到 OCR 模型目录");
        return None;
    };
    let loaded = tokio::task::spawn_blocking(move || OcrEngine::load(&dir))
        .await
        .ok()
        .and_then(|result| result.ok());
    let Some(engine) = loaded else {
        log::warn!("[augment-overlay] OCR 引擎加载失败");
        return None;
    };
    let engine = Arc::new(engine);
    *engine_slot().lock().await = Some(engine.clone());
    Some(engine)
}

pub fn start(app: AppHandle) {
    tokio::spawn(async move {
        if !state::begin_scan().await {
            return;
        }
        let generation = state::scan_generation().await;
        let Some(engine) = ensure_engine(&app).await else {
            state::end_scan(generation).await;
            return;
        };
        log::info!("[augment-overlay] 开始识别本轮三张卡");
        let mut misses = 0u32;
        loop {
            if !state::should_keep_scanning(generation).await {
                break;
            }
            match scan_once(&app, &engine).await {
                Ok(true) => misses = 0,
                Ok(false) => {
                    misses = misses.saturating_add(1);
                    if misses >= 3 {
                        state::clear_current_offers(&app).await;
                    }
                }
                Err(error) => log::debug!("[augment-overlay] OCR 一轮失败: {error}"),
            }
            let wait = if misses == 0 { 450 } else { 900 };
            tokio::time::sleep(Duration::from_millis(wait)).await;
        }
        state::end_scan(generation).await;
        log::info!("[augment-overlay] 已停止识别本轮三张卡");
    });
}

async fn scan_once(app: &AppHandle, engine: &Arc<OcrEngine>) -> Result<bool, String> {
    let frame = tokio::task::spawn_blocking(capture_game_frame)
        .await
        .map_err(|error| format!("截屏 join: {error}"))??;
    let gate = detect_augment_ui_gate(&frame.image);
    if !gate.title_likely && !gate.reroll_visible {
        return Ok(false);
    }

    let engine = Arc::clone(engine);
    let texts = tokio::task::spawn_blocking(move || recognize_slots(&engine, &frame))
        .await
        .map_err(|error| format!("ocr join: {error}"))?;

    let catalog = service::get_augment_catalog().await?;
    let guide = state::cached_guide_augments().await;
    let mut seen = Vec::new();
    let mut augments = Vec::with_capacity(3);
    let mut scores = Vec::with_capacity(3);

    for (slot, text) in texts.iter().enumerate() {
        let matched = if has_meaningful_slot_text(text) {
            match_slot_text(text, &catalog, &seen)
        } else {
            None
        };
        if let Some(matched) = matched {
            seen.push(matched.id);
            let stat = guide.iter().find(|item| item.id == matched.id);
            let sampled = stat.is_some_and(|item| !(item.games.is_none() && item.win_rate == 0.0));
            scores.push(OfferScore {
                slot,
                id: Some(matched.id),
                tier: stat.and_then(|item| item.tier),
                win_rate: stat.map(|item| item.win_rate),
                sampled,
            });
            augments.push(OverlayAugment {
                id: Some(matched.id),
                name: matched.name,
                rarity: matched.rarity,
                rarity_display_name: matched.rarity_display_name,
                icon_url: matched.icon_url,
                confidence: Some(matched.confidence),
                detected_slot: slot as i32,
                missing: false,
                win_rate: stat.map(|item| item.win_rate),
                pick_rate: stat.map(|item| item.pick_rate),
                games: stat.and_then(|item| item.games),
                recommended: false,
            });
        } else {
            scores.push(OfferScore {
                slot,
                id: None,
                tier: None,
                win_rate: None,
                sampled: false,
            });
            augments.push(OverlayAugment::empty(slot as i32));
        }
    }

    if seen.is_empty() {
        return Ok(false);
    }
    mark_recommended(&mut augments, &scores);
    state::publish_current_offers(app, augments).await;
    Ok(true)
}

fn recognize_slots(engine: &OcrEngine, frame: &CaptureFrame) -> [String; 3] {
    let primary = crop_title_slots(&frame.image);
    let fallback = crop_fallback_title_slots(&frame.image);
    core::array::from_fn(|index| {
        let first = engine.recognize_line(&primary[index]);
        if has_meaningful_slot_text(&first) {
            first
        } else {
            engine.recognize_line(&fallback[index])
        }
    })
}
