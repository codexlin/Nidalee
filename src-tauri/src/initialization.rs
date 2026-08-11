/// 应用初始化模块
/// 负责应用启动时的各种数据初始化任务
use crate::infrastructure;

/// 初始化所有游戏相关数据
///
/// 身份类静态目录（英雄 / 召唤师技能）由 `static_catalog` 按版本落盘并 hydrate。
pub async fn init_game_data() {
    log::info!("[初始化] 🌐 开始加载游戏静态目录...");

    match infrastructure::data_services::static_catalog::ensure_static_catalogs().await {
        Ok(()) => {
            let meta = infrastructure::data_services::static_catalog::get_static_meta();
            let champ_count = infrastructure::data_services::champion_data::get_champion_count();
            let spell_count = infrastructure::champion_selection::summoner_spells::service::get_spell_count();
            log::info!(
                "[初始化] ✅ 静态目录就绪 version={} source={} champions={} spells={}",
                meta.as_ref().map(|m| m.version.as_str()).unwrap_or("?"),
                meta.as_ref().map(|m| m.source.as_str()).unwrap_or("?"),
                champ_count,
                spell_count
            );
        }
        Err(e) => {
            log::error!("[初始化] ❌ 静态目录加载失败: {e}");
        }
    }

    log::info!("[初始化] 🎉 游戏数据初始化完成");
}

/// 启动游戏数据初始化任务（异步，不阻塞应用启动）
pub fn start_game_data_initialization() {
    tokio::spawn(async move {
        init_game_data().await;
    });
}
