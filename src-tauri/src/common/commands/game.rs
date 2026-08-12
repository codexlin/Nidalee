use std::path::{Path, PathBuf};

fn validate_game_executable(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let canonical = std::fs::canonicalize(path.as_ref()).map_err(|error| format!("游戏路径无效或不可访问: {error}"))?;
    if !canonical.is_file() {
        return Err("游戏路径必须指向客户端可执行文件".to_string());
    }

    let file_name = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "游戏可执行文件名无效".to_string())?;
    let parent_name = canonical
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    let known_launcher = file_name.eq_ignore_ascii_case("LeagueClient.exe")
        || (file_name.eq_ignore_ascii_case("Client.exe") && parent_name.eq_ignore_ascii_case("Launcher"))
        || (file_name.eq_ignore_ascii_case("launcher.exe") && parent_name.eq_ignore_ascii_case("WeGameLauncher"))
        || (file_name.eq_ignore_ascii_case("client.exe") && parent_name.eq_ignore_ascii_case("TCLS"));

    if !known_launcher {
        return Err("只能选择英雄联盟或 WeGame 的官方启动程序".to_string());
    }

    Ok(canonical)
}

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
    let path = validate_game_executable(&game_path)?;

    // WeGame Client.exe 清单含 requireAdministrator。
    // 普通 CreateProcess/Command::spawn 会直接报 os error 740；
    // 需走 ShellExecute，才能弹出 UAC 提权启动。
    #[cfg(target_os = "windows")]
    {
        shell_execute_open(&path)?;
        Ok(true)
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(&path)
            .spawn()
            .map_err(|e| format!("启动失败: {}", e))?;
        Ok(true)
    }
}

/// 通过 ShellExecuteW 启动程序（支持需 UAC 提权的清单）
#[cfg(target_os = "windows")]
fn shell_execute_open(path: &Path) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: *mut core::ffi::c_void,
            lp_operation: *const u16,
            lp_file: *const u16,
            lp_parameters: *const u16,
            lp_directory: *const u16,
            n_show_cmd: i32,
        ) -> isize;
    }

    fn to_wide(s: impl AsRef<OsStr>) -> Vec<u16> {
        s.as_ref().encode_wide().chain(Some(0)).collect()
    }

    let file = to_wide(path.as_os_str());
    let operation = to_wide("open");
    let directory = path.parent().map(|p| to_wide(p.as_os_str())).unwrap_or_default();

    // SW_SHOWNORMAL = 1；返回值 > 32 表示成功
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            if directory.is_empty() {
                std::ptr::null()
            } else {
                directory.as_ptr()
            },
            1,
        )
    };

    if result > 32 {
        Ok(())
    } else {
        Err(format!(
            "启动失败: ShellExecute 错误码 {}（若需管理员权限，请在 UAC 弹窗中确认）",
            result
        ))
    }
}

#[tauri::command]
pub async fn detect_game_path() -> Result<String, String> {
    tokio::task::spawn_blocking(detect_game_path_sync)
        .await
        .map_err(|error| format!("游戏路径检测任务失败: {error}"))?
}

fn detect_game_path_sync() -> Result<String, String> {
    // 1) WeGame：扫描各盘符 WeGameApps 下名称包含「英雄联盟」的目录
    #[cfg(target_os = "windows")]
    if let Some(path) = detect_wegame_lol_path() {
        log::info!("[GamePath] detected (WeGame): {}", path);
        return Ok(path);
    }

    // 2) 国际服 Riot 固定路径
    let riot_paths = [
        "C:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "D:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "E:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "F:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "G:\\Riot Games\\League of Legends\\LeagueClient.exe",
        "C:\\Program Files\\Riot Games\\League of Legends\\LeagueClient.exe",
        "C:\\Program Files (x86)\\Riot Games\\League of Legends\\LeagueClient.exe",
    ];
    for path in &riot_paths {
        if Path::new(path).exists() {
            log::info!("[GamePath] detected (Riot): {}", path);
            return Ok(path.to_string());
        }
    }

    // 3) 注册表（国际服）
    #[cfg(target_os = "windows")]
    {
        match get_game_path_from_registry() {
            Ok(path) if !path.is_empty() => {
                log::info!("[GamePath] detected (registry): {}", path);
                return Ok(path);
            }
            _ => {}
        }
    }

    Err("未检测到游戏安装路径".to_string())
}

/// 枚举本机存在的盘符根路径（如 `C:\`、`D:\`）
/// 注意：不能用 `C:` —— `Path::new("C:").join("x")` 会得到 `C:x`（盘符相对路径），不是 `C:\x`
#[cfg(target_os = "windows")]
fn existing_drive_roots() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .filter_map(|letter| {
            let root = PathBuf::from(format!("{}:\\", letter as char));
            if root.exists() {
                Some(root)
            } else {
                None
            }
        })
        .collect()
}

/// WeGame 国服启动器相对路径（按优先级）
#[cfg(target_os = "windows")]
const WEGAME_EXE_RELATIVES: &[&str] = &[
    "Launcher\\Client.exe",
    "LeagueClient\\LeagueClient.exe",
    "WeGameLauncher\\launcher.exe",
    "TCLS\\client.exe",
];

/// 是否像国服英雄联盟安装目录：名称含「英雄联盟」，或具备典型子目录结构
#[cfg(target_os = "windows")]
fn looks_like_wegame_lol_dir(game_root: &Path, dir_name: &str) -> bool {
    if dir_name.contains("英雄联盟") || dir_name.to_ascii_lowercase().contains("league") {
        return true;
    }
    // 结构兜底：不依赖目录名（避免编码/改名问题）
    game_root.join("LeagueClient").join("LeagueClient.exe").exists()
        || game_root.join("Launcher").join("Client.exe").exists()
        || game_root.join("Game").join("League of Legends.exe").exists()
}

#[cfg(target_os = "windows")]
fn resolve_wegame_exe(game_root: &Path) -> Option<String> {
    for relative in WEGAME_EXE_RELATIVES {
        let candidate = game_root.join(relative);
        // 使用 exists：部分环境下 is_file 对特殊属性文件可能偏严
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    find_client_exe_limited(game_root, &["Client.exe", "LeagueClient.exe", "launcher.exe"], 2)
}

/// WeGameApps 模糊匹配：`{盘符}\WeGameApps\*英雄联盟*\...`，并辅以目录结构识别
#[cfg(target_os = "windows")]
fn detect_wegame_lol_path() -> Option<String> {
    let roots = existing_drive_roots();
    log::info!(
        "[GamePath] scanning drive roots: {:?}",
        roots.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
    );

    for root in roots {
        let wegame_root = root.join("WeGameApps");
        if !wegame_root.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&wegame_root) {
            Ok(e) => e,
            Err(err) => {
                log::warn!("[GamePath] read_dir failed for {}: {}", wegame_root.display(), err);
                continue;
            }
        };

        for entry in entries.flatten() {
            let game_root = entry.path();
            if !game_root.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !looks_like_wegame_lol_dir(&game_root, &name) {
                continue;
            }
            log::info!("[GamePath] candidate dir: {}", game_root.display());
            if let Some(exe) = resolve_wegame_exe(&game_root) {
                return Some(exe);
            }
            log::warn!("[GamePath] matched dir but no launcher exe: {}", game_root.display());
        }
    }

    None
}

/// 限深查找启动器，跳过反作弊/浏览器等干扰目录
#[cfg(target_os = "windows")]
fn find_client_exe_limited(base: &Path, exe_names: &[&str], max_depth: usize) -> Option<String> {
    if !base.is_dir() {
        return None;
    }

    const SKIP_DIRS: &[&str] = &[
        "AntiCheatExpert",
        "qbblinktrial",
        "DiagnosticAssistant",
        "FeedBack",
        "NetworkAssist",
        "TQM",
        "tiny_cache",
        "rail_files",
    ];

    let mut stack: Vec<(PathBuf, usize)> = vec![(base.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if depth >= max_depth {
                    continue;
                }
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if SKIP_DIRS.iter().any(|&skip| dir_name.eq_ignore_ascii_case(skip)) {
                    continue;
                }
                stack.push((path, depth + 1));
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if exe_names.iter().any(|&exe| name.eq_ignore_ascii_case(exe)) {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_game_path_from_registry() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_paths = [
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
        .set_title("选择英雄联盟客户端 (Client.exe / LeagueClient.exe)")
        .add_filter("可执行文件", &["exe"])
        .pick_file(move |file| {
            let _ = tx.send(file);
        });
    match rx.await {
        Ok(Some(path)) => match path.as_path() {
            Some(p) => validate_game_executable(p).map(|path| path.to_string_lossy().to_string()),
            None => Err("文件路径无效".to_string()),
        },
        Ok(None) => Err("未选择文件".to_string()),
        Err(_) => Err("文件选择失败".to_string()),
    }
}

#[tauri::command]
pub async fn save_game_path(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || save_game_path_sync(path))
        .await
        .map_err(|error| format!("保存游戏路径任务失败: {error}"))?
}

fn save_game_path_sync(path: String) -> Result<(), String> {
    use serde_json::json;
    use std::fs;
    let path = validate_game_executable(path)?;
    let config_dir = dirs::config_dir().ok_or("无法获取配置目录")?.join("nidalee");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let config_file = config_dir.join("game_config.json");
    let config = json!({
        "game_path": path.to_string_lossy()
    });
    fs::write(&config_file, config.to_string()).map_err(|e| format!("保存配置失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_saved_game_path() -> Result<String, String> {
    tokio::task::spawn_blocking(get_saved_game_path_sync)
        .await
        .map_err(|error| format!("读取游戏路径任务失败: {error}"))?
}

fn get_saved_game_path_sync() -> Result<String, String> {
    use serde_json::Value;
    use std::fs;
    let config_dir = dirs::config_dir().ok_or("无法获取配置目录")?.join("nidalee");
    let config_file = config_dir.join("game_config.json");
    if !config_file.exists() {
        return Ok(String::new());
    }
    let config_content = fs::read_to_string(&config_file).map_err(|e| format!("读取配置失败: {}", e))?;
    let config: Value = serde_json::from_str(&config_content).map_err(|e| format!("解析配置失败: {}", e))?;
    let Some(path) = config["game_path"].as_str() else {
        return Ok(String::new());
    };
    validate_game_executable(path).map(|path| path.to_string_lossy().to_string())
}
