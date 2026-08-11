// 应用配置模块 - 负责应用的初始化和配置
use crate::{infrastructure, initialization, tray};
use std::sync::Arc;
use tauri::{App, Manager};
use tokio::sync::RwLock;

/// 缩短模块路径，避免控制台被 `nidalee_lib::infrastructure::...` 淹没
#[cfg(debug_assertions)]
fn short_log_target(target: &str) -> String {
    let trimmed = target
        .strip_prefix("nidalee_lib::")
        .or_else(|| target.strip_prefix("nidalee::"))
        .unwrap_or(target);
    let parts: Vec<&str> = trimmed.split("::").collect();
    match parts.as_slice() {
        [] => trimmed.to_string(),
        [one] => (*one).to_string(),
        [.., prev, last] => format!("{prev}::{last}"),
    }
}

/// 应用启动时的设置函数
pub fn setup_app(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // 开发模式下启用日志
    #[cfg(debug_assertions)]
    {
        app.handle().plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                // 压掉常见嘈杂依赖
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("reqwest", log::LevelFilter::Warn)
                .level_for("tungstenite", log::LevelFilter::Warn)
                .level_for("tokio_tungstenite", log::LevelFilter::Warn)
                // 自定义更易扫读的格式（勿再调 timezone_strategy，它会覆盖 format）
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{} {:>5} {:<22} {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        record.level(),
                        short_log_target(record.target()),
                        message
                    ))
                })
                .build(),
        )?;
    }

    // 设置系统托盘
    tray::setup_system_tray(app).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // 初始化连接管理器
    let connection_manager = Arc::new(RwLock::new(
        infrastructure::game_session::connection::service::ConnectionManager::new(app.handle().clone()),
    ));
    app.handle().manage(connection_manager.clone());

    // 启动 WebSocket 连接状态投影服务
    start_services(app, connection_manager);

    // 🌐 初始化游戏数据（异步加载，不阻塞应用启动）
    initialization::start_game_data_initialization();

    Ok(())
}

/// 启动各种后台服务
fn start_services(
    _app: &mut App,
    connection_manager: Arc<RwLock<infrastructure::game_session::connection::service::ConnectionManager>>,
) {
    // 连接管理器只投影 WebSocket 生命周期，不再执行独立认证轮询。
    let connection_manager_clone = connection_manager.clone();
    tokio::spawn(async move {
        let manager = connection_manager_clone.read().await;
        manager.start_monitoring().await;
    });

    log::info!("[应用] WebSocket 连接状态投影已启动");
}
