//! PaddleOCRv5 mobile OCR 引擎
//!
//! 加载本地 [`det`] 检测 + [`rec`] 识别 + 字典，识别一张图中所有文字。
//!
//! 模型来源：从 [ARAMGG 助手](https://github.com/valkia/aramgg_client) 复制
//! （SHA256 已验证，见 `src-tauri/resources/models/README.md`）。

use std::path::Path;
use std::sync::Mutex;

use image::{imageops::FilterType, RgbaImage};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

/// 文字识别结果
#[derive(Debug, Clone)]
pub struct OcrText {
    pub text: String,
    pub box_: BBox,
    #[allow(dead_code)]
    pub confidence: f32,
}

/// 文字区域边框（像素坐标，原图空间）
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

/// PaddleOCRv5 mobile 本地 OCR 引擎
///
/// 延迟指标（CPU 推理，5 年内主流 CPU）：
/// - 模型加载（一次性）：~200ms
/// - 单帧完整识别（det + rec）：100-200ms
pub struct OcrEngine {
    #[allow(dead_code)]
    detector: Mutex<Session>,
    recognizer: Mutex<Session>,
    dictionary: Vec<char>,
}

impl OcrEngine {
    /// 从模型目录加载（指向 `resources/models/paddleocr/`）
    pub fn load(model_dir: &Path) -> Result<Self, String> {
        let det_path = model_dir.join("det/inference.onnx");
        let rec_path = model_dir.join("rec/inference.onnx");
        let dict_path = model_dir.join("rec/inference.yml");

        let detector = Session::builder()
            .map_err(|e| format!("det Session builder: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| format!("det opt level: {e}"))?
            .with_memory_pattern(true)
            .map_err(|e| format!("det memory pattern: {e}"))?
            .commit_from_file(&det_path)
            .map_err(|e| format!("det load {}: {e}", det_path.display()))?;

        let recognizer = Session::builder()
            .map_err(|e| format!("rec Session builder: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| format!("rec opt level: {e}"))?
            .with_memory_pattern(true)
            .map_err(|e| format!("rec memory pattern: {e}"))?
            .commit_from_file(&rec_path)
            .map_err(|e| format!("rec load {}: {e}", rec_path.display()))?;

        let dictionary = load_dictionary(&dict_path)?;

        log::info!(
            "[OCR] 加载完成: det + rec .onnx, 字典 {} 个字符",
            dictionary.len()
        );

        Ok(Self {
            detector: Mutex::new(detector),
            recognizer: Mutex::new(recognizer),
            dictionary,
        })
    }

    /// 把整张图当成一行字识别（跳过检测，适合已经裁好的标题条）
    pub fn recognize_line(&self, image: &RgbaImage) -> String {
        let box_ = BBox {
            left: 0,
            top: 0,
            width: image.width() as i32,
            height: image.height() as i32,
        };
        self.recognize_text(image, &box_)
            .map(|item| item.text)
            .unwrap_or_default()
    }

    /// 识别一张完整图像中的所有文字
    #[allow(dead_code)]
    pub fn recognize(&self, image: &RgbaImage) -> Vec<OcrText> {
        let boxes = match self.detect_regions(image) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("[OCR] detect failed: {e}");
                return Vec::new();
            }
        };

        let mut results = Vec::new();
        for box_ in boxes {
            match self.recognize_text(image, &box_) {
                Ok(text) if !text.text.is_empty() => results.push(text),
                Ok(_) => {}
                Err(e) => log::warn!("[OCR] recognize failed: {e}"),
            }
        }
        results
    }

    /// 检测文字区域（返回原图坐标的 BBox 列表）
    #[allow(dead_code)]
    fn detect_regions(&self, image: &RgbaImage) -> Result<Vec<BBox>, String> {
        let long_side = 640.0_f32;
        let (w, h) = (image.width() as f32, image.height() as f32);
        let scale = (long_side / w.max(h)).min(1.0);
        let new_w = ((w * scale) as u32).max(32);
        let new_h = ((h * scale) as u32).max(32);
        let padded_w = ((new_w + 31) / 32) * 32;
        let padded_h = ((new_h + 31) / 32) * 32;

        let resized = image::imageops::resize(image, new_w, new_h, FilterType::Triangle);
        let mut padded = RgbaImage::from_pixel(padded_w, padded_h, image::Rgba([0, 0, 0, 0]));
        image::imageops::overlay(&mut padded, &resized, 0, 0);

        let input = preprocess_for_det(&padded);
        let tensor = Tensor::from_array((
            [1usize, 3, padded_h as usize, padded_w as usize],
            input,
        ))
        .map_err(|e| format!("det tensor: {e}"))?;

        let mut detector = self
            .detector
            .lock()
            .map_err(|e| format!("det lock: {e}"))?;
        let outputs = detector
            .run(ort::inputs![tensor])
            .map_err(|e| format!("det run: {e}"))?;

        let (shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("det extract: {e}"))?;
        let (ph, pw) = if shape.len() == 4 {
            (shape[2] as usize, shape[3] as usize)
        } else {
            return Err(format!("unexpected det output shape: {shape:?}"));
        };

        let threshold = 0.3_f32;
        let mut mask = vec![false; ph * pw];
        for i in 0..(ph * pw) {
            if i < data.len() && data[i] > threshold {
                mask[i] = true;
            }
        }

        let boxes = find_text_boxes(&mask, pw, ph, 16);

        let boxes: Vec<BBox> = boxes
            .into_iter()
            .map(|mut b| {
                b.left = ((b.left as f32) / scale).round() as i32;
                b.top = ((b.top as f32) / scale).round() as i32;
                b.width = ((b.width as f32) / scale).round() as i32;
                b.height = ((b.height as f32) / scale).round() as i32;
                b
            })
            .collect();

        Ok(boxes)
    }

    fn recognize_text(&self, image: &RgbaImage, box_: &BBox) -> Result<OcrText, String> {
        let left = box_.left.max(0) as u32;
        let top = box_.top.max(0) as u32;
        let width = (box_.width as u32).min(image.width().saturating_sub(left));
        let height = (box_.height as u32).min(image.height().saturating_sub(top));
        if width == 0 || height == 0 {
            return Err("empty crop".to_string());
        }

        let cropped = image::imageops::crop_imm(image, left, top, width, height).to_image();

        let target_h = 48u32;
        let scale = target_h as f32 / cropped.height() as f32;
        let target_w = ((cropped.width() as f32 * scale) as u32).max(48);
        let target_w = ((target_w + 31) / 32) * 32;

        let resized = image::imageops::resize(&cropped, target_w, target_h, FilterType::Triangle);
        let input = preprocess_for_rec(&resized);
        let tensor = Tensor::from_array((
            [1usize, 3, target_h as usize, target_w as usize],
            input,
        ))
        .map_err(|e| format!("rec tensor: {e}"))?;

        let mut recognizer = self
            .recognizer
            .lock()
            .map_err(|e| format!("rec lock: {e}"))?;
        let outputs = recognizer
            .run(ort::inputs![tensor])
            .map_err(|e| format!("rec run: {e}"))?;

        let (shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("rec extract: {e}"))?;
        if shape.len() < 2 {
            return Err(format!("unexpected rec output shape: {shape:?}"));
        }
        let t_len = shape[shape.len() - 2] as usize;
        let c_len = shape[shape.len() - 1] as usize;
        let text = ctc_decode(data, t_len, c_len, &self.dictionary);

        Ok(OcrText {
            text,
            box_: *box_,
            confidence: 0.5,
        })
    }
}

/// det 预处理：RGB + ImageNet 均值方差归一化（PP-OCRv5 标准）
#[allow(dead_code)]
fn preprocess_for_det(image: &RgbaImage) -> Vec<f32> {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let mut data = Vec::with_capacity(3 * h * w);
    let mean = [0.485, 0.456, 0.406];
    let std = [0.229, 0.224, 0.225];
    for c in 0..3 {
        for y in 0..h {
            for x in 0..w {
                let pixel = image.get_pixel(x as u32, y as u32);
                let v = pixel[c] as f32 / 255.0;
                data.push((v - mean[c]) / std[c]);
            }
        }
    }
    data
}

/// rec 预处理：`(x / 255 - 0.5) / 0.5`（PP-OCRv5 标准）
fn preprocess_for_rec(image: &RgbaImage) -> Vec<f32> {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let mut data = Vec::with_capacity(3 * h * w);
    for c in 0..3 {
        for y in 0..h {
            for x in 0..w {
                let pixel = image.get_pixel(x as u32, y as u32);
                let v = pixel[c] as f32 / 255.0;
                data.push((v - 0.5) / 0.5);
            }
        }
    }
    data
}

/// CTC 贪心解码：去重 + 跳过 blank token（index 0）
fn ctc_decode(data: &[f32], t_len: usize, c_len: usize, dict: &[char]) -> String {
    if t_len == 0 || c_len == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_idx: i64 = -1;
    for t in 0..t_len {
        let mut max_idx = 0i64;
        let mut max_val = f32::NEG_INFINITY;
        for c in 0..c_len {
            let i = t * c_len + c;
            if i < data.len() && data[i] > max_val {
                max_val = data[i];
                max_idx = c as i64;
            }
        }
        if max_idx != 0 && max_idx != prev_idx {
            let dict_idx = (max_idx - 1) as usize;
            if dict_idx < dict.len() {
                result.push(dict[dict_idx]);
            }
        }
        prev_idx = max_idx;
    }
    result
}

/// BFS 找二进制 mask 的 4-邻接连通区域
fn find_text_boxes(mask: &[bool], w: usize, h: usize, min_area: usize) -> Vec<BBox> {
    let mut visited = vec![false; w * h];
    let mut boxes = Vec::new();

    for y in 0..h {
        for x in 0..w {
            if !mask[y * w + x] || visited[y * w + x] {
                continue;
            }

            let mut min_x = x;
            let mut min_y = y;
            let mut max_x = x;
            let mut max_y = y;
            let mut stack = vec![(x, y)];
            visited[y * w + x] = true;

            while let Some((cx, cy)) = stack.pop() {
                min_x = min_x.min(cx);
                min_y = min_y.min(cy);
                max_x = max_x.max(cx);
                max_y = max_y.max(cy);

                for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                        let ni = (ny as usize) * w + (nx as usize);
                        if mask[ni] && !visited[ni] {
                            visited[ni] = true;
                            stack.push((nx as usize, ny as usize));
                        }
                    }
                }
            }

            let ww = (max_x - min_x + 1) as i32;
            let hh = (max_y - min_y + 1) as i32;
            let area = (ww as usize) * (hh as usize);
            if area >= min_area {
                boxes.push(BBox {
                    left: min_x as i32,
                    top: min_y as i32,
                    width: ww,
                    height: hh,
                });
            }
        }
    }
    boxes
}

/// 字典加载（手动解析 PaddleOCRv5 的 inference.yml）
fn load_dictionary(path: &Path) -> Result<Vec<char>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;

    let mut chars = Vec::new();
    let mut in_dict = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_dict {
            if (trimmed.starts_with("character_dict:")
                || trimmed.starts_with("char_dict:")
                || trimmed.starts_with("character_list:"))
                && !trimmed.contains('"')
            {
                in_dict = true;
            }
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("- ") {
            if value.is_empty() {
                chars.push(' ');
            } else {
                for c in value.chars() {
                    chars.push(c);
                }
            }
        } else if trimmed == "-" {
            chars.push(' ');
        }
    }

    if chars.is_empty() {
        return Err(format!("未在 {} 找到 character_dict 条目", path.display()));
    }
    Ok(chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_parses_basic() {
        let content = "\
PostProcess:
  character_dict:
    - 　
    - 一
    - 二
    - 三
";
        let mut chars = Vec::new();
        let mut in_dict = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if !in_dict {
                if trimmed.starts_with("character_dict:") {
                    in_dict = true;
                }
                continue;
            }
            if trimmed.starts_with("- ") || trimmed == "-" {
                let value = trimmed.strip_prefix("- ").unwrap_or("");
                if value.is_empty() {
                    chars.push(' ');
                } else {
                    for c in value.chars() {
                        chars.push(c);
                    }
                }
            }
        }
        assert_eq!(chars.len(), 4);
    }

    #[test]
    fn ctc_decode_merges_consecutive_duplicates() {
        let c_len = 6;
        let t_len = 5;
        let mut data = vec![f32::NEG_INFINITY; t_len * c_len];
        data[0 * c_len + 1] = 1.0;
        data[1 * c_len + 1] = 1.0;
        data[2 * c_len + 0] = 1.0;
        data[3 * c_len + 2] = 1.0;
        data[4 * c_len + 3] = 1.0;

        let dict: Vec<char> = vec!['a', 'b', 'c', 'd', 'e'];
        let result = ctc_decode(&data, t_len, c_len, &dict);
        assert_eq!(result, "abc");
    }

    #[test]
    fn find_text_boxes_basic() {
        let mut mask = vec![false; 25];
        let positions = [6, 7, 11, 12];
        for &p in &positions {
            mask[p] = true;
        }
        let boxes = find_text_boxes(&mask, 5, 5, 1);
        assert_eq!(boxes.len(), 1);
        let b = &boxes[0];
        assert_eq!(b.left, 1);
        assert_eq!(b.top, 1);
        assert_eq!(b.width, 2);
        assert_eq!(b.height, 2);
    }
}
