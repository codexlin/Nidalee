/// 应用初始化模块
/// 负责应用启动时的各种数据初始化任务
use crate::infrastructure;

/// 初始化所有游戏相关数据
///
/// 包括：
/// - 英雄数据（champion_data）
/// - 召唤师技能数据（summoner_spells）
///
/// 所有数据并行加载，不阻塞应用启动
pub async fn init_game_data() {
    log::info!("[初始化] 🌐 开始加载游戏数据...");

    // 并行加载英雄和召唤师技能数据
    let (champion_result, spell_result): (
        Result<(), Box<dyn std::error::Error + Send + Sync>>,
        Result<(), Box<dyn std::error::Error + Send + Sync>>,
    ) = tokio::join!(
        infrastructure::data_services::champion_data::load_champion_data(),
        infrastructure::champion_selection::summoner_spells::load_summoner_spell_data()
    );

    // 处理加载结果
    match champion_result {
        Ok(_) => {
            let count = infrastructure::data_services::champion_data::get_champion_count();
            log::info!("[初始化] ✅ 英雄数据加载成功，共 {} 个英雄", count);
        }
        Err(e) => {
            log::error!("[初始化] ❌ 英雄数据加载失败: {}", e);
        }
    }

    match spell_result {
        Ok(_) => {
            let count = infrastructure::champion_selection::summoner_spells::get_spell_count();
            log::info!("[初始化] ✅ 召唤师技能数据加载成功，共 {} 个技能", count);
        }
        Err(e) => {
            log::error!("[初始化] ❌ 召唤师技能数据加载失败: {}", e);
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
