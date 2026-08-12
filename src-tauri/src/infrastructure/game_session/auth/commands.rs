use crate::infrastructure::game_session::auth::service::{
    verify_lockfile_vs_cmdline as do_verify_lockfile_vs_cmdline, AUTH_INFO,
};
use crate::shared::types::LcuAuthInfo;

#[cfg(debug_assertions)]
#[tauri::command]
pub fn get_auth_info() -> Option<LcuAuthInfo> {
    AUTH_INFO.read().ok().and_then(|auth| auth.as_ref().cloned())
}

/// 🔍 验证命令：对比 lockfile 和进程命令行获取的信息
/// 前端调用：invoke('verify_lockfile_vs_cmdline')
#[cfg(debug_assertions)]
#[tauri::command]
pub fn verify_lockfile_vs_cmdline() {
    do_verify_lockfile_vs_cmdline();
}
