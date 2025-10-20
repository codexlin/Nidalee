use std::path::Path;
use std::process::Command;

#[tauri::command]
pub async fn launch_game(custom_path: Option<String>) -> Result<bool, String> {
    let game_path = if let Some(path) = custom_path {
        path
    } else {
        match get_saved_game_path().await {
            Ok(saved_path) if !saved_path.is_empty() => saved_path,
            _ => match detect_game_path().await {
                Ok(detected_path) if !detected_path.is_empty() => detected_path,
                _ => return Err("未找到游戏路径，请手动配置".to_string()),
            },
        }
    };
    if !Path::new(&game_path).exists() {
        return Err("游戏路径不存在".to_string());
    }
    match Command::new(&game_path).spawn() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

#[tauri::command]
pub async fn detect_game_path() -> Result<String, String> {
    
    use std::path::Path;
    let wegame_paths = vec![
        "C:\\WeGameApps\\英雄联盟\\Launcher\\client.exe",
        "D:\\WeGameApps\\英雄联盟\\Launcher\\client.exe",
        "E:\\WeGameApps\\英雄联盟\\Launcher\\client.exe",
        "F:\\WeGameApps\\英雄联盟\\Launcher\\client.exe",
    ];
    for path in &wegame_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    let riot_paths = vec![
        "C:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "D:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "E:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "F:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "C:\\Program Files\\Riot Games\\League of Legends\\LeagueClient.exe",
        "C:\\Program Files (x86)\\Riot Games\\League of Legends\\LeagueClient.exe",
    ];
    for path in &riot_paths {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    #[cfg(target_os = "windows")]
    {
        let drives = ["C:", "D:", "E:", "F:"];
        let folder_names = ["英雄联盟", "League of Legends"];
        let exe_names = ["client.exe", "LeagueClient.exe"];
        for drive in &drives {
            for folder in &folder_names {
                let base = Path::new(drive).join(folder);
                if base.exists() {
                    if let Some(found) = find_client_exe(&base, &exe_names) {
                        return Ok(found);
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        match get_game_path_from_registry() {
            Ok(path) if !path.is_empty() => return Ok(path),
            _ => {}
        }
    }
    Err("未检测到游戏安装路径".to_string())
}

#[cfg(target_os = "windows")]
fn find_client_exe(base: &std::path::Path, exe_names: &[&str]) -> Option<String> {
    use std::fs;
    use std::path::Path;
    if !base.is_dir() {
        return None;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if exe_names.iter().any(|&exe| exe.eq_ignore_ascii_case(name)) {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_game_path_from_registry() -> Result<String, String> {
    use std::path::Path;
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_paths = vec![
        "SOFTWARE\\Riot Games\\League of Legends",
        "SOFTWARE\\WOW6432Node\\Riot Games\\League of Legends",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Riot Game league_of_legends.live",
    ];
    for registry_path in registry_paths {
        if let Ok(riot_key) = hklm.open_subkey(registry_path) {
            if let Ok(install_path) = riot_key.get_value::<String, _>("InstallLocation") {
                let client_path = format!("{}\\LeagueClient.exe", install_path);
                if Path::new(&client_path).exists() {
                    return Ok(client_path);
                }
            }
        }
    }
    Err("注册表中未找到游戏路径".to_string())
}

#[tauri::command]
pub async fn select_game_path(window: tauri::Window) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
    let (tx, rx) = tokio::sync::oneshot::channel();
    let dialog = window.dialog().clone();
    FileDialogBuilder::new(dialog)
        .set_title("选择英雄联盟客户端 (LeagueClient.exe 或 client.exe)")
        .add_filter("可执行文件", &["exe"])
        .pick_file(move |file| {
            let _ = tx.send(file);
        });
    match rx.await {
        Ok(Some(path)) => match path.as_path() {
            Some(p) => Ok(p.to_string_lossy().to_string()),
            None => Err("文件路径无效".to_string()),
        },
        Ok(None) => Err("未选择文件".to_string()),
        Err(_) => Err("文件选择失败".to_string()),
    }
}

#[tauri::command]
pub async fn save_game_path(path: String) -> Result<(), String> {
    use serde_json::json;
    use std::fs;
    let config_dir = dirs::config_dir().ok_or("无法获取配置目录")?.join("nidalee");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let config_file = config_dir.join("game_config.json");
    let config = json!({
        "game_path": path
    });
    fs::write(&config_file, config.to_string()).map_err(|e| format!("保存配置失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_saved_game_path() -> Result<String, String> {
    use serde_json::Value;
    use std::fs;
    let config_dir = dirs::config_dir().ok_or("无法获取配置目录")?.join("nidalee");
    let config_file = config_dir.join("game_config.json");
    if !config_file.exists() {
        return Ok(String::new());
    }
    let config_content = fs::read_to_string(&config_file).map_err(|e| format!("读取配置失败: {}", e))?;
    let config: Value = serde_json::from_str(&config_content).map_err(|e| format!("解析配置失败: {}", e))?;
    Ok(config["game_path"].as_str().unwrap_or("").to_string())
}
