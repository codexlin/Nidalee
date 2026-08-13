use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Mutex;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

use crate::shared::types::LcuAuthInfo;
use crate::shared::{NidaleeError, Result};

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
static RIOTCLIENT_TOKEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"--riotclient-auth-token=([^\s]+)").expect("valid auth token regex"));
static RIOTCLIENT_PORT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"--riotclient-app-port=([^\s]+)").expect("valid auth port regex"));
static REMOTING_TOKEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"--remoting-auth-token=([^\s]+)").expect("valid remoting token regex"));
static APP_PORT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"--app-port=([^\s]+)").expect("valid app port regex"));

pub(super) fn discover_auth_info() -> Result<LcuAuthInfo> {
    log::info!("开始强制刷新 AuthInfo");
    let cmdline = match get_lcu_cmdline() {
        Some(cmd) => cmd,
        None => {
            log::error!("LeagueClientUx.exe 进程未找到，无法刷新 AuthInfo");
            return Err(NidaleeError::LcuNotFound);
        }
    };

    let riotclient_auth_token = RIOTCLIENT_TOKEN_RE
        .captures(&cmdline)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string());
    let riotclient_app_port = RIOTCLIENT_PORT_RE
        .captures(&cmdline)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().parse::<u16>().ok());
    let remoting_auth_token = REMOTING_TOKEN_RE
        .captures(&cmdline)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string());
    let app_port = APP_PORT_RE
        .captures(&cmdline)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().parse::<u16>().ok());

    if let (Some(riotclient_auth_token), Some(riotclient_app_port), Some(remoting_auth_token), Some(app_port)) = (
        riotclient_auth_token,
        riotclient_app_port,
        remoting_auth_token,
        app_port,
    ) {
        Ok(LcuAuthInfo {
            riotclient_auth_token,
            riotclient_app_port,
            remoting_auth_token,
            app_port,
        })
    } else {
        log::error!("解析 LeagueClientUx.exe 启动参数失败");
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
        log::error!("当前操作系统暂不支持自动获取 LoL 参数");
        None
    }
}

#[cfg(target_os = "windows")]
fn get_lcu_cmdline_windows() -> Option<String> {
    let mut system = SYSTEM.lock().unwrap_or_else(|poisoned| {
        log::warn!("process cache mutex was poisoned; recovering cached state");
        poisoned.into_inner()
    });
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    let mut ux_cmdline = None;
    let mut client_cmdline = None;
    let possible_names = ["leagueclientux.exe", "leagueclient.exe", "leagueoflegends.exe"];

    for (pid, process) in system.processes() {
        let process_name = process.name().to_string_lossy().to_lowercase();
        if !possible_names.contains(&process_name.as_str()) {
            continue;
        }

        let cmdline = process
            .cmd()
            .iter()
            .map(|part| part.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        if process_name == "leagueclientux.exe" {
            if has_lcu_arguments(&cmdline) {
                log::debug!("找到包含 LCU 参数的 LeagueClientUx.exe 进程, PID: {}", pid);
                return Some(cmdline);
            }
            ux_cmdline = Some(cmdline);
        } else if process_name == "leagueclient.exe" {
            client_cmdline = Some(cmdline);
        }
    }

    if let Some(cmdline) = client_cmdline.filter(|cmdline| has_lcu_arguments(cmdline)) {
        log::warn!("LeagueClientUx.exe 命令行中未找到认证参数，回退使用 LeagueClient.exe 的参数。");
        return Some(cmdline);
    }

    if let Some(cmdline) = ux_cmdline {
        log::warn!("无法在任何进程的命令行中找到认证参数，将使用无参数的 LeagueClientUx.exe 命令行进行后续尝试。");
        return Some(cmdline);
    }

    log::debug!("未找到包含有效 LCU 参数的客户端进程");
    None
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_lcu_cmdline_unix() -> Option<String> {
    let mut system = SYSTEM.lock().unwrap_or_else(|poisoned| {
        log::warn!("process cache mutex was poisoned; recovering cached state");
        poisoned.into_inner()
    });
    system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

    system.processes().values().find_map(|process| {
        let name = process.name().to_string_lossy().to_lowercase();
        if !name.contains("leagueclientux") {
            return None;
        }

        let cmdline = process
            .cmd()
            .iter()
            .map(|part| part.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        has_lcu_arguments(&cmdline).then_some(cmdline)
    })
}

fn has_lcu_arguments(cmdline: &str) -> bool {
    cmdline.contains("--remoting-auth-token") && cmdline.contains("--app-port")
}

/// Diagnostic helper used by the existing auth command.
#[cfg(debug_assertions)]
pub(super) fn verify_lockfile_vs_cmdline() {
    log::info!("========== 开始验证 lockfile vs 进程命令行 ==========");

    let cmdline_info = get_lcu_cmdline().and_then(|cmdline| {
        let token_re = Regex::new(r"--remoting-auth-token=([^\s]+)").ok()?;
        let port_re = Regex::new(r"--app-port=([^\s]+)").ok()?;
        let token = token_re.captures(&cmdline)?.get(1)?.as_str().to_string();
        let port = port_re.captures(&cmdline)?.get(1)?.as_str().parse::<u16>().ok()?;
        Some((port, token))
    });
    let lockfile_info = get_lcu_from_lockfile();

    match (cmdline_info, lockfile_info) {
        (Some(cmdline), Some(lockfile)) => {
            let port_match = cmdline.0 == lockfile.0;
            let token_match = cmdline.1 == lockfile.1;
            log::info!("进程命令行端口: {}, Lockfile 端口: {}", cmdline.0, lockfile.0);
            log::info!(
                "认证信息对比: port={}, token={}",
                if port_match { "一致" } else { "不一致" },
                if token_match { "一致" } else { "不一致" }
            );
        }
        (None, None) => log::error!("两种方式都无法获取 LCU 信息（LoL 客户端可能未启动）"),
        (None, Some(_)) => log::info!("Lockfile 方式成功，进程命令行失败（可能需要管理员权限）"),
        (Some(_), None) => log::warn!("进程命令行方式成功，Lockfile 失败"),
    }
}

#[cfg(target_os = "windows")]
#[cfg(debug_assertions)]
fn get_lcu_from_lockfile() -> Option<(u16, String)> {
    use std::fs;
    use std::path::Path;

    const POSSIBLE_PATHS: [&str; 3] = [
        r"C:\Riot Games\League of Legends\lockfile",
        r"D:\Riot Games\League of Legends\lockfile",
        r"E:\Riot Games\League of Legends\lockfile",
    ];

    for path in POSSIBLE_PATHS.map(Path::new) {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let parts = content.trim().split(':').collect::<Vec<_>>();
        if parts.len() < 4 {
            continue;
        }
        let port = parts[2].parse::<u16>().ok()?;
        return Some((port, parts[3].to_string()));
    }
    None
}

#[cfg(not(target_os = "windows"))]
#[cfg(debug_assertions)]
fn get_lcu_from_lockfile() -> Option<(u16, String)> {
    None
}
