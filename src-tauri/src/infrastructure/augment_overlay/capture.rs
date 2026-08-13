//! 屏幕截图 + 海克斯三卡标题区域裁剪 / 活动门控
//!
//! 区域比例对齐 ARAMGG `image-analyzer.ts`（相对整屏宽高）。

use image::{imageops::FilterType, RgbaImage};

const CARD_WIDTH_RATIO: f32 = 0.168;
const CARD_GAP_RATIO: f32 = 0.0175;
const TITLE_INSET_X: f32 = 0.02;
const TITLE_WIDTH_RATIO: f32 = 0.96;
const PADDLE_TITLE_TOP: f32 = 0.36;
const PADDLE_TITLE_HEIGHT: f32 = 0.07;
const FALLBACK_TITLE_TOP: f32 = 0.33;
const FALLBACK_TITLE_HEIGHT: f32 = 0.10;
const GATE_TITLE_TOP: f32 = 0.37;
const GATE_TITLE_HEIGHT: f32 = 0.07;
#[allow(dead_code)]
const TITLE_SCALE: u32 = 3;

const OCR_TITLE_ACTIVE_BRIGHT_RATIO: f32 = 0.012;
const OCR_TITLE_WEAK_BRIGHT_RATIO: f32 = 0.004;
const AUGMENT_REROLL_BUTTON_MIN_RATIO: f32 = 0.08;
const AUGMENT_REROLL_BUTTON_MIN_VISIBLE_SLOTS: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct CropRect {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct RowBound {
    pub top: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy)]
struct CardLayout {
    card_width: f32,
    card_gap: f32,
    group_left: f32,
}

fn card_layout(width: u32, _height: u32) -> CardLayout {
    let w = width as f32;
    let card_width = w * CARD_WIDTH_RATIO;
    let card_gap = w * CARD_GAP_RATIO;
    let group_width = card_width * 3.0 + card_gap * 2.0;
    let group_left = (w - group_width) / 2.0;
    CardLayout {
        card_width,
        card_gap,
        group_left,
    }
}

fn slot_left(layout: CardLayout, index: usize) -> f32 {
    layout.group_left + index as f32 * (layout.card_width + layout.card_gap)
}

fn clamp_rect(image_w: u32, image_h: u32, left: f32, top: f32, w: f32, h: f32) -> CropRect {
    let left = left.max(0.0).floor() as u32;
    let top = top.max(0.0).floor() as u32;
    let left = left.min(image_w.saturating_sub(1));
    let top = top.min(image_h.saturating_sub(1));
    let width = (w.max(1.0).round() as u32).min(image_w.saturating_sub(left).max(1));
    let height = (h.max(1.0).round() as u32).min(image_h.saturating_sub(top).max(1));
    CropRect {
        left,
        top,
        width,
        height,
    }
}

fn title_regions_at(width: u32, height: u32, top_ratio: f32, height_ratio: f32) -> [CropRect; 3] {
    let layout = card_layout(width, height);
    let h = height as f32;
    core::array::from_fn(|index| {
        let left = slot_left(layout, index) + layout.card_width * TITLE_INSET_X;
        clamp_rect(
            width,
            height,
            left,
            h * top_ratio,
            layout.card_width * TITLE_WIDTH_RATIO,
            h * height_ratio,
        )
    })
}

pub fn paddle_ocr_title_regions(width: u32, height: u32) -> [CropRect; 3] {
    title_regions_at(width, height, PADDLE_TITLE_TOP, PADDLE_TITLE_HEIGHT)
}

/// 裁出三张标题条并做成深底金字 → 黑字白底，供单行 OCR。
pub fn crop_title_slots(image: &RgbaImage) -> [RgbaImage; 3] {
    crop_title_slots_at(image, PADDLE_TITLE_TOP, PADDLE_TITLE_HEIGHT)
}

pub fn crop_fallback_title_slots(image: &RgbaImage) -> [RgbaImage; 3] {
    crop_title_slots_at(image, FALLBACK_TITLE_TOP, FALLBACK_TITLE_HEIGHT)
}

fn crop_title_slots_at(image: &RgbaImage, top_ratio: f32, height_ratio: f32) -> [RgbaImage; 3] {
    let regions = title_regions_at(image.width(), image.height(), top_ratio, height_ratio);
    core::array::from_fn(|index| {
        let rect = regions[index];
        let crop = image::imageops::crop_imm(image, rect.left, rect.top, rect.width, rect.height).to_image();
        enhance_title_crop(&crop)
    })
}

fn enhance_title_crop(image: &RgbaImage) -> RgbaImage {
    let mut min_l = 255u8;
    let mut max_l = 0u8;
    let mut sum = 0u64;
    let mut count = 0u64;
    for pixel in image.pixels() {
        let luminance = ((pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16) / 3) as u8;
        min_l = min_l.min(luminance);
        max_l = max_l.max(luminance);
        sum += u64::from(luminance);
        count += 1;
    }
    let mean = if count == 0 { 0 } else { (sum / count) as u8 };
    let invert = mean < 140;
    let range = u16::from((max_l - min_l).max(1));
    let mut out = image.clone();
    for (src, dst) in image.pixels().zip(out.pixels_mut()) {
        let luminance = ((src[0] as u16 + src[1] as u16 + src[2] as u16) / 3) as u8;
        let mut stretched = ((u16::from(luminance.saturating_sub(min_l)) * 255) / range) as u8;
        if invert {
            stretched = 255 - stretched;
        }
        *dst = image::Rgba([stretched, stretched, stretched, 255]);
    }
    out
}

fn gate_title_regions(width: u32, height: u32) -> [CropRect; 3] {
    let layout = card_layout(width, height);
    let h = height as f32;
    core::array::from_fn(|index| {
        let left = slot_left(layout, index) + layout.card_width * TITLE_INSET_X;
        clamp_rect(
            width,
            height,
            left,
            h * GATE_TITLE_TOP,
            layout.card_width * TITLE_WIDTH_RATIO,
            h * GATE_TITLE_HEIGHT,
        )
    })
}

fn reroll_button_regions(width: u32, height: u32) -> [CropRect; 3] {
    let layout = card_layout(width, height);
    let h = height as f32;
    core::array::from_fn(|index| {
        let left = slot_left(layout, index) + layout.card_width * 0.56;
        clamp_rect(
            width,
            height,
            left,
            h * 0.63,
            layout.card_width * 0.34,
            h * 0.045,
        )
    })
}

pub struct CaptureFrame {
    pub image: RgbaImage,
    pub source: &'static str,
}

fn is_lol_game_window(title: &str, app_name: &str) -> bool {
    let title_l = title.to_ascii_lowercase();
    let app_l = app_name.to_ascii_lowercase();
    if title_l.contains("riot client") || app_l.contains("riot client") {
        return false;
    }
    title.contains("League of Legends (TM) Client")
        || (title.contains("League of Legends") && !title_l.contains("client_info"))
}

fn capture_lol_window_region() -> Option<RgbaImage> {
    let windows = xcap::Window::all().ok()?;
    let window = windows.into_iter().find(|item| {
        let title = item.title().unwrap_or_default();
        let app = item.app_name().unwrap_or_default();
        !item.is_minimized().unwrap_or(true) && is_lol_game_window(&title, &app)
    })?;
    let monitor = window.current_monitor().ok()?;
    let mx = monitor.x().ok()?;
    let my = monitor.y().ok()?;
    let mw = monitor.width().ok()?;
    let mh = monitor.height().ok()?;
    let wx = window.x().ok()?;
    let wy = window.y().ok()?;
    let ww = window.width().ok()?;
    let wh = window.height().ok()?;
    if ww < 640 || wh < 360 {
        return None;
    }
    let x = (wx - mx).clamp(0, mw.saturating_sub(1) as i32) as u32;
    let y = (wy - my).clamp(0, mh.saturating_sub(1) as i32) as u32;
    let width = ww.min(mw.saturating_sub(x));
    let height = wh.min(mh.saturating_sub(y));
    monitor.capture_region(x, y, width, height).ok()
}

/// 优先截 LoL 游戏窗口所在区域（桌面复制，避免 D3D 窗口截到黑屏）。
pub fn capture_game_frame() -> Result<CaptureFrame, String> {
    if let Some(image) = capture_lol_window_region() {
        return Ok(CaptureFrame {
            image,
            source: "lol-window",
        });
    }
    let monitors = xcap::Monitor::all().map_err(|e| format!("列举显示器失败: {e}"))?;
    let monitor = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or_else(|| "未找到显示器".to_string())?;
    let image = monitor
        .capture_image()
        .map_err(|e| format!("截屏失败: {e}"))?;
    Ok(CaptureFrame {
        image,
        source: "primary-monitor",
    })
}

#[derive(Debug, Clone, Copy)]
pub struct AugmentUiGate {
    pub title_likely: bool,
    pub reroll_visible: bool,
}

fn region_bright_ratio(image: &RgbaImage, rect: CropRect) -> f32 {
    let sample_w = 160.min(rect.width).max(1);
    let sample_h = 50.min(rect.height).max(1);
    let cropped = image::imageops::crop_imm(image, rect.left, rect.top, rect.width, rect.height).to_image();
    let resized = image::imageops::resize(&cropped, sample_w, sample_h, FilterType::Nearest);
    let total = (sample_w * sample_h) as f32;
    if total == 0.0 {
        return 0.0;
    }
    let mut bright = 0u32;
    for pixel in resized.pixels() {
        if pixel[0] > 180 && pixel[1] > 180 && pixel[2] > 170 {
            bright += 1;
        }
    }
    bright as f32 / total
}

fn region_button_ratio(image: &RgbaImage, rect: CropRect) -> f32 {
    let sample_w = 120.min(rect.width).max(1);
    let sample_h = 48.min(rect.height).max(1);
    let cropped = image::imageops::crop_imm(image, rect.left, rect.top, rect.width, rect.height).to_image();
    let resized = image::imageops::resize(&cropped, sample_w, sample_h, FilterType::Nearest);
    let total = (sample_w * sample_h) as f32;
    if total == 0.0 {
        return 0.0;
    }
    let mut hits = 0u32;
    for pixel in resized.pixels() {
        let maxc = pixel[0].max(pixel[1]).max(pixel[2]);
        let minc = pixel[0].min(pixel[1]).min(pixel[2]);
        if maxc > 140 && (maxc - minc) < 40 {
            hits += 1;
        }
    }
    hits as f32 / total
}

pub fn detect_augment_ui_gate(image: &RgbaImage) -> AugmentUiGate {
    let (width, height) = (image.width(), image.height());
    let titles = gate_title_regions(width, height);
    let mut strong = 0usize;
    let mut weak = 0usize;
    for rect in titles {
        let ratio = region_bright_ratio(image, rect);
        if ratio >= OCR_TITLE_ACTIVE_BRIGHT_RATIO {
            strong += 1;
        }
        if ratio >= OCR_TITLE_WEAK_BRIGHT_RATIO {
            weak += 1;
        }
    }
    let title_likely = strong >= 2 || weak >= 2;

    let buttons = reroll_button_regions(width, height);
    let visible_slots = buttons
        .iter()
        .filter(|rect| region_button_ratio(image, **rect) >= AUGMENT_REROLL_BUTTON_MIN_RATIO)
        .count();
    let reroll_visible = visible_slots >= AUGMENT_REROLL_BUTTON_MIN_VISIBLE_SLOTS;

    AugmentUiGate {
        title_likely,
        reroll_visible,
    }
}

/// 裁三张标题、3× 放大后垂直拼接，供一次 OCR。
#[allow(dead_code)]
pub fn stack_title_crops(image: &RgbaImage) -> Result<(RgbaImage, [RowBound; 3]), String> {
    let regions = paddle_ocr_title_regions(image.width(), image.height());
    let mut rows: Vec<RgbaImage> = Vec::with_capacity(3);
    for rect in regions {
        if rect.width == 0 || rect.height == 0 {
            return Err("empty title crop".to_string());
        }
        let crop = image::imageops::crop_imm(image, rect.left, rect.top, rect.width, rect.height).to_image();
        let scaled = image::imageops::resize(
            &crop,
            rect.width * TITLE_SCALE,
            rect.height * TITLE_SCALE,
            FilterType::Triangle,
        );
        rows.push(scaled);
    }

    let row_width = rows.iter().map(|r| r.width()).max().unwrap_or(1);
    let gap = rows
        .iter()
        .map(|r| (r.height() as f32 * 0.18).round() as u32)
        .max()
        .unwrap_or(24)
        .max(24);

    let total_height: u32 = rows.iter().map(|r| r.height()).sum::<u32>() + gap * 2;
    let mut stacked = RgbaImage::from_pixel(row_width, total_height, image::Rgba([255, 255, 255, 255]));
    let mut bounds = [RowBound { top: 0, bottom: 0 }; 3];
    let mut y = 0u32;
    for (i, row) in rows.iter().enumerate() {
        image::imageops::overlay(&mut stacked, row, 0, y as i64);
        bounds[i] = RowBound {
            top: y as i32,
            bottom: (y + row.height()) as i32,
        };
        y += row.height() + gap;
    }
    Ok((stacked, bounds))
}

#[allow(dead_code)]
pub fn slot_texts_from_ocr(
    items: &[super::ocr::OcrText],
    row_bounds: &[RowBound; 3],
) -> [String; 3] {
    core::array::from_fn(|index| {
        let row = row_bounds[index];
        let mut texts: Vec<(i32, &str)> = items
            .iter()
            .filter(|item| {
                let center_y = item.box_.top + item.box_.height / 2;
                center_y >= row.top && center_y <= row.bottom
            })
            .map(|item| (item.box_.left, item.text.as_str()))
            .collect();
        texts.sort_by_key(|(left, _)| *left);
        texts
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
            .join("")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paddle_ocr_title_regions_centers_three_cards_on_1080p() {
        let regions = paddle_ocr_title_regions(1920, 1080);
        assert_eq!(regions.len(), 3);
        assert!(regions[0].left < regions[1].left);
        assert!(regions[1].left < regions[2].left);
        assert_eq!(regions[0].top, (1080.0 * PADDLE_TITLE_TOP).floor() as u32);
        let expected_width = (1920.0 * CARD_WIDTH_RATIO * TITLE_WIDTH_RATIO).round() as u32;
        assert!((regions[0].width as i32 - expected_width as i32).abs() <= 1);
    }
}
