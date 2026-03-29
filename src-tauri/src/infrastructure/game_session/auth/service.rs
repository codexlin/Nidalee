use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

use crate::shared::types::LcuAuthInfo;
use crate::shared::{NidaleeError, Result};

pub static AUTH_INFO: Lazy<RwLock<Option<LcuAuthInfo>>> = Lazy::new(|| RwLock::new(None));
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
static AUTH_TIMESTAMP: Lazy<RwLock<Option<Instant>>> = Lazy::new(|| RwLock::new(None));
// 配置：token 最多允许缓存多久，超时自动刷新
// 注意：LCU 的 remoting token 通常在客户端存活期内保持稳定，因此无需频繁刷新。
// 这里将默认刷新间隔从 60s 提升到 30 分钟，显著减少无意义的刷新与日志噪音。
const AUTH_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// 获取（并自动刷新）最新有效的 LCU AuthInfo
pub fn ensure_valid_auth_info() -> Option<LcuAuthInfo> {
    // 1. 先检测缓存是否存在且未超时
    {
        let auth = AUTH_INFO.read().unwrap();
        let ts = AUTH_TIMESTAMP.read().unwrap();
        if let (Some(a), Some(t)) = (auth.as_ref(), ts.as_ref()) {
            if t.elapsed() < AUTH_REFRESH_INTERVAL {
                log::debug!("[LCU] 使用缓存的 AuthInfo，距离上次刷新: {:?}秒", t.elapsed().as_secs());
                return Some(a.clone());
            } else {
                // 仅作为调试信息打印，避免在正常空闲期间产生高频噪音
                log::debug!("[LCU] AuthInfo 缓存已过期，准备刷新");
            }
        } else {
            log::debug!("[LCU] 当前无有效缓存，准备刷新");
        }
    }

    // 2. 带重试的自动刷新（LOL 启动初期可能需要多次尝试）
    for attempt in 1..=3 {
        match refresh_auth_info() {
            Ok(auth) => {
                log::info!("[LCU] 自动刷新 AuthInfo 成功 (尝试 {}/3)", attempt);
                return Some(auth);
            }
            Err(e) => {
                log::warn!("[LCU] 自动刷新 AuthInfo 失败 (尝试 {}/3): {}", attempt, e);
                if attempt < 3 {
                    // 短暂等待后重试
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }

    log::error!("[LCU] 多次尝试后仍无法获取有效的 AuthInfo");
    None
}

/// 手动强制刷新 AuthInfo（一般不直接用，内部自动调用）
pub fn refresh_auth_info() -> Result<LcuAuthInfo> {
    log::info!("[LCU] 开始强制刷新 AuthInfo");
    let cmdline = match get_lcu_cmdline() {
        Some(cmd) => cmd,
        None => {
            log::error!("[LCU] LeagueClientUx.exe 进程未找到，无法刷新 AuthInfo");
            invalidate_auth_info();
            return Err(NidaleeError::LcuNotFound);
        }
    };
    let riotclient_token_re = Regex::new(r"--riotclient-auth-token=([^\s]+)").unwrap();
    let riotclient_port_re = Regex::new(r"--riotclient-app-port=([^\s]+)").unwrap();
    let remoting_token_re = Regex::new(r"--remoting-auth-token=([^\s]+)").unwrap();
    let app_port_re = Regex::new(r"--app-port=([^\s]+)").unwrap();

    let riotclient_auth_token = riotclient_token_re
        .captures(&cmdline)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());
    let riotclient_app_port = riotclient_port_re
        .captures(&cmdline)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u16>().ok());
    let remoting_auth_token = remoting_token_re
        .captures(&cmdline)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());
    let app_port = app_port_re
        .captures(&cmdline)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u16>().ok());
    if let (Some(r_token), Some(r_port), Some(m_token), Some(a_port)) = (
        riotclient_auth_token,
        riotclient_app_port,
        remoting_auth_token,
        app_port,
    ) {
        let auth = LcuAuthInfo {
            riotclient_auth_token: r_token,
            riotclient_app_port: r_port,
            remoting_auth_token: m_token,
            app_port: a_port,
        };
        {
            let mut info = AUTH_INFO.write().unwrap();
            *info = Some(auth.clone());
        }
        {
            let mut ts = AUTH_TIMESTAMP.write().unwrap();
            *ts = Some(Instant::now());
        }
        log::info!(
            "[LCU] AuthInfo 刷新成功，端口: {}, token: {}... (已隐藏)",
            auth.app_port,
            &auth.remoting_auth_token[..8.min(auth.remoting_auth_token.len())]
        );
        Ok(auth)
    } else {
        log::error!("[LCU] 解析 LeagueClientUx.exe 启动参数失败，清空缓存");
        invalidate_auth_info();
        Err(NidaleeError::LcuAuth("解析 LeagueClientUx 启动参数失败".to_string()))
    }
}

fn get_lcu_cmdline() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        get_lcu_cmdline_windows()
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        get_lcu_cmdline_unix()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        log::error!("[LCU] 当前操作系统暂不支持自动获取 LoL 参数");
        None
    }
}

pub fn invalidate_auth_info() {
    let mut info = AUTH_INFO.write().unwrap();
    *info = None;
    let mut ts = AUTH_TIMESTAMP.write().unwrap();
    *ts = None;
    log::info!("[LCU] AuthInfo 缓存已清除");
}

/// 验证 AuthInfo 是否真正可用（通过简单的 API 测试）
pub async fn validate_auth_connection(auth: &LcuAuthInfo) -> bool {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };

    let url = format!("https://127.0.0.1:{}/lol-summoner/v1/current-summoner", auth.app_port);
    let response = client
        .get(&url)
        .basic_auth("riot", Some(&auth.remoting_auth_token))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let success = resp.status().is_success();
            log::debug!("[LCU] 连接验证结果: {}, 状态码: {}", success, resp.status());
            success
        }
        Err(e) => {
            log::debug!("[LCU] 连接验证失败: {}", e);
            false
        }
    }
}
#[cfg(target_os = "windows")]
fn get_lcu_cmdline_windows() -> Option<String> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    let mut ux_cmdline = None;
    let mut client_cmdline = None;

    // 寻找所有可能的 LoL 客户端进程
    let possible_names = ["LeagueClientUx.exe", "LeagueClient.exe", "LeagueOfLegends.exe"];

    for (_pid, process) in system.processes() {
        let process_name_lower = process.name().to_string_lossy().to_lowercase();

        if possible_names
            .iter()
            .any(|name| process_name_lower == name.to_lowercase())
        {
            let cmdline = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            // 优先检查 LeagueClientUx.exe
            if process_name_lower == "leagueclientux.exe" {
                if cmdline.contains("--remoting-auth-token") && cmdline.contains("--app-port") {
                    log::debug!("[LCU] 找到包含 LCU 参数的 LeagueClientUx.exe 进程, PID: {}", _pid);
                    return Some(cmdline); // 找到最理想的目标，直接返回
                } else {
                    ux_cmdline = Some(cmdline); // 暂存，可能没有参数
                }
            }
            // 如果没找到 Ux，再检查 LeagueClient.exe
            else if process_name_lower == "leagueclient.exe" {
                client_cmdline = Some(cmdline);
            }
        }
    }

    // 如果 Ux 进程的命令行里没有参数，尝试使用 Client 进程的命令行
    if let Some(cmd) = client_cmdline {
        if cmd.contains("--remoting-auth-token") && cmd.contains("--app-port") {
            log::warn!("[LCU] LeagueClientUx.exe 命令行中未找到认证参数，回退使用 LeagueClient.exe 的参数。");
            return Some(cmd);
        }
    }

    // 如果 Client 的参数也没有，但 Ux 进程确实存在，则返回它的（不带参数的）命令行
    if let Some(cmd) = ux_cmdline {
        log::warn!(
            "[LCU] 无法在任何进程的命令行中找到认证参数，将使用无参数的 LeagueClientUx.exe 命令行进行后续尝试。"
        );
        return Some(cmd);
    }

    log::debug!("[LCU] 未找到包含有效 LCU 参数的客户端进程");
    None
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_lcu_cmdline_unix() -> Option<String> {
    let mut system = SYSTEM.lock().unwrap();
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    for (_pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name.contains("leagueclientux") {
            let cmdline = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            if cmdline.contains("--remoting-auth-token") && cmdline.contains("--app-port") {
                return Some(cmdline);
            }
        }
    }
    None
}

// #[cfg(target_os = "macos")]
// fn get_lcu_cmdline_macos() -> Option<String> {
//     use std::process::Command;

//     let output = Command::new("sh")
//         .arg("-c")
//         .arg("ps -ax -o command | grep 'LeagueClientUx' | grep -- '--remoting-auth-token' | grep -- '--app-port' | grep -v grep")
//         .output()
//         .ok()?;

//     if !output.status.success() {
//         return None;
//     }

//     let result = String::from_utf8_lossy(&output.stdout);
//     for line in result.lines() {
//         if line.contains("--remoting-auth-token") && line.contains("--app-port") {
//             return Some(line.to_string());
//         }
//     }

//     None
// }
// #[cfg(target_os = "linux")]
// fn get_lcu_cmdline_linux() -> Option<String> {
//     use std::process::Command;

//     let output = Command::new("sh")
//         .arg("-c")
//         .arg("ps -e -o args | grep 'LeagueClientUx' | grep -- '--remoting-auth-token' | grep -- '--app-port' | grep -v grep")
//         .output()
//         .ok()?;

//     if !output.status.success() {
//         return None;
//     }

//     let result = String::from_utf8_lossy(&output.stdout);
//     for line in result.lines() {
//         if line.contains("--remoting-auth-token") && line.contains("--app-port") {
//             return Some(line.to_string());
//         }
//     }

//     None
// }

/// 🔍 验证：对比 lockfile 和进程命令行获取的信息是否一致
/// 用于验证 lockfile 方案是否可以替代进程命令行方案
pub fn verify_lockfile_vs_cmdline() {
    log::info!("========== 开始验证 lockfile vs 进程命令行 ==========");

    // 1. 从进程命令行获取
    let cmdline_info = get_lcu_cmdline().and_then(|cmdline| {
        let remoting_token_re = Regex::new(r"--remoting-auth-token=([^\s]+)").unwrap();
        let app_port_re = Regex::new(r"--app-port=([^\s]+)").unwrap();

        let token = remoting_token_re.captures(&cmdline)?.get(1)?.as_str().to_string();
        let port = app_port_re.captures(&cmdline)?.get(1)?.as_str().parse::<u16>().ok()?;

        Some((port, token))
    });

    // 2. 从 lockfile 获取
    let lockfile_info = get_lcu_from_lockfile();

    match (cmdline_info, lockfile_info) {
        (Some(cmdline), Some(lockfile)) => {
            let port_match = cmdline.0 == lockfile.0;
            let token_match = cmdline.1 == lockfile.1;

            log::info!("┌─ 进程命令行 ──────────────────────┐");
            log::info!("│ 端口: {:>5}                     │", cmdline.0);
            log::info!("│ Token: {}... ({} 字符) │", &cmdline.1[..8.min(cmdline.1.len())], cmdline.1.len());
            log::info!("└────────────────────────────────┘");

            log::info!("┌─ Lockfile ────────────────────────┐");
            log::info!("│ 端口: {:>5}                     │", lockfile.0);
            log::info!("│ Token: {}... ({} 字符) │", &lockfile.1[..8.min(lockfile.1.len())], lockfile.1.len());
            log::info!("└────────────────────────────────┘");

            log::info!("┌─ 对比结果 ────────────────────────┐");
            log::info!("│ 端口: {}                         │", if port_match { "✓ 一致" } else { "✗ 不一致" });
            log::info!("│ Token: {}                        │", if token_match { "✓ 一致" } else { "✗ 不一致" });
            log::info!("└────────────────────────────────┘");

            if port_match && token_match {
                log::info!("✅ 验证成功：lockfile 和进程命令行获取的信息完全一致！");
            } else {
                log::warn!("⚠️  验证失败：两种方式获取的信息不一致！");
            }
        }
        (None, None) => {
            log::error!("❌ 验证失败：两种方式都无法获取 LCU 信息（LoL 客户端可能未启动）");
        }
        (None, Some(_)) => {
            log::info!("✅ Lockfile 方式成功，进程命令行失败（可能需要管理员权限）");
        }
        (Some(_), None) => {
            log::warn!("⚠️  进程命令行方式成功，lockfile 失败");
        }
    }

    log::info!("==============================================");
}

/// 从 lockfile 读取 LCU 连接信息
/// lockfile 路径：C:\Riot Games\League of Legends\lockfile
#[cfg(target_os = "windows")]
fn get_lcu_from_lockfile() -> Option<(u16, String)> {
    use std::fs;
    use std::path::PathBuf;

    // 尝试多个可能的 lockfile 路径
    let possible_paths = vec![
        r"C:\Riot Games\League of Legends\lockfile",
        r"D:\Riot Games\League of Legends\lockfile",
        r"E:\Riot Games\League of Legends\lockfile",
    ];

    for path_str in possible_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    // lockfile 格式: LeagueClient:PID:Port:Password
                    let parts: Vec<&str> = content.trim().split(':').collect();
                    if parts.len() >= 4 {
                        let _process_name = parts[0];
                        let _pid = parts[1];
                        let port: u16 = parts[2].parse().ok()?;
                        let token = parts[3].to_string();

                        log::debug!("[LCU] 从 lockfile 读取: port={}, token={}...", port, &token[..8.min(token.len())]);
                        return Some((port, token));
                    }
                }
                Err(e) => {
                    log::debug!("[LCU] 无法读取 lockfile {:?}: {}", path, e);
                }
            }
        }
    }

    log::debug!("[LCU] 未找到 lockfile");
    None
}

/// 非 Windows 平台不支持 lockfile
#[cfg(not(target_os = "windows"))]
fn get_lcu_from_lockfile() -> Option<(u16, String)> {
    log::warn!("[LCU] lockfile 方案仅支持 Windows 平台");
    None
}
