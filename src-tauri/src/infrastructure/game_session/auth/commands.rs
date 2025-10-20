use crate::infrastructure::game_session::auth::service::AUTH_INFO;
use crate::shared::types::LcuAuthInfo;

#[tauri::command]
pub fn get_auth_info() -> Option<LcuAuthInfo> {
    let auth = AUTH_INFO.read().unwrap();
    auth.as_ref().cloned()
}
