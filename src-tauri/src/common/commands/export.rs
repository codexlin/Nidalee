use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

fn decode_png_base64(png_base64: &str) -> Result<Vec<u8>, String> {
    let payload = png_base64
        .strip_prefix("data:image/png;base64,")
        .or_else(|| png_base64.strip_prefix("data:image/jpeg;base64,"))
        .unwrap_or(png_base64)
        .trim();
    STANDARD
        .decode(payload)
        .map_err(|e| format!("PNG Base64 解码失败: {e}"))
}

/// 打开「另存为」并写入 PNG；取消时返回 Ok(None)。
#[tauri::command]
pub async fn save_png_file(
    window: tauri::Window,
    png_base64: String,
    default_name: String,
) -> Result<Option<String>, String> {
    let bytes = decode_png_base64(&png_base64)?;
    let file_name = if default_name.trim().is_empty() {
        "nidalee-poster.png".to_string()
    } else if default_name.to_ascii_lowercase().ends_with(".png") {
        default_name
    } else {
        format!("{default_name}.png")
    };

    use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
    let (tx, rx) = tokio::sync::oneshot::channel();
    let dialog = window.dialog().clone();
    FileDialogBuilder::new(dialog)
        .set_title("保存战绩海报")
        .set_file_name(&file_name)
        .add_filter("PNG 图片", &["png"])
        .save_file(move |file| {
            let _ = tx.send(file);
        });

    match rx.await {
        Ok(Some(path)) => {
            let path_buf: PathBuf = path
                .as_path()
                .map(|p| p.to_path_buf())
                .ok_or_else(|| "文件路径无效".to_string())?;
            fs::write(&path_buf, &bytes).map_err(|e| format!("写入文件失败: {e}"))?;
            Ok(Some(path_buf.to_string_lossy().to_string()))
        }
        Ok(None) => Ok(None),
        Err(_) => Err("保存对话框失败".to_string()),
    }
}

/// 将 PNG 写入系统剪贴板（图片）。
#[tauri::command]
pub async fn copy_png_to_clipboard(png_base64: String) -> Result<(), String> {
    let bytes = decode_png_base64(&png_base64)?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("解析 PNG 失败: {e}"))?
        .to_rgba8();
    let (width, height) = img.dimensions();

    tauri::async_runtime::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("打开剪贴板失败: {e}"))?;
        clipboard
            .set_image(arboard::ImageData {
                width: width as usize,
                height: height as usize,
                bytes: Cow::Owned(img.into_raw()),
            })
            .map_err(|e| format!("写入剪贴板失败: {e}"))
    })
    .await
    .map_err(|e| format!("剪贴板任务失败: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_data_url_prefix() {
        // 1x1 transparent PNG
        let b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
        let with_prefix = format!("data:image/png;base64,{b64}");
        let a = decode_png_base64(b64).unwrap();
        let b = decode_png_base64(&with_prefix).unwrap();
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }
}
