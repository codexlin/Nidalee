// 应用库 - 提供应用运行的核心功能
mod app;
mod common;
mod http_client;
mod initialization;
mod tray;

// 新架构模块
mod domains;
mod infrastructure;
mod shared;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(app::setup_app)
        .invoke_handler(tauri::generate_handler![
            // 认证 / 连接
            infrastructure::game_session::auth::commands::get_auth_info,
            infrastructure::game_session::connection::commands::get_connection_state,
            infrastructure::game_session::connection::commands::force_refresh_connection,
            infrastructure::game_session::connection::commands::check_connection_state_command,
            // 游戏流程 / 英雄选择 / 匹配
            infrastructure::game_session::gameflow::commands::get_game_version,
            infrastructure::game_session::gameflow::commands::get_live_player_list,
            infrastructure::game_session::gameflow::commands::get_live_events,
            infrastructure::game_session::gameflow::commands::get_game_stats,
            infrastructure::game_session::gameflow::commands::is_liveclient_available,
            infrastructure::champion_selection::champ_select::commands::get_champselect_team_players_info,
            infrastructure::champion_selection::champ_select::commands::get_champ_select_session,
            infrastructure::champion_selection::champ_select::commands::get_champ_select_session_typed,
            infrastructure::champion_selection::champ_select::commands::pick_champion,
            infrastructure::champion_selection::champ_select::commands::ban_champion,
            infrastructure::match_management::matchmaking::commands::start_matchmaking,
            infrastructure::match_management::matchmaking::commands::stop_matchmaking,
            infrastructure::match_management::matchmaking::commands::accept_match,
            infrastructure::match_management::matchmaking::commands::decline_match,
            // 比赛记录
            infrastructure::match_management::matches::commands::get_match_history,
            infrastructure::match_management::matches::commands::get_game_detail,
            infrastructure::match_management::matches::commands::get_player_tactical_advice,
            // 召唤师
            infrastructure::data_services::summoner::commands::get_current_summoner,
            infrastructure::data_services::summoner::commands::get_summoner_by_id,
            infrastructure::data_services::summoner::commands::get_recent_matches_by_puuid,
            infrastructure::data_services::summoner::commands::get_summoners_and_histories,
            infrastructure::data_services::summoner::commands::set_summoner_chat_profile,
            infrastructure::data_services::summoner::commands::set_summoner_background_skin,
            // 召唤师符文
            infrastructure::champion_selection::perks::commands::get_lcu_rune_styles,
            infrastructure::champion_selection::perks::commands::get_lcu_perks,
            infrastructure::champion_selection::perks::commands::get_lcu_perk_icon,
            infrastructure::champion_selection::perks::commands::get_current_rune_page,
            infrastructure::champion_selection::perks::commands::apply_custom_runes,
            // OPGG 相关
            infrastructure::data_services::external::opgg::commands::get_opgg_champion_build,
            infrastructure::data_services::external::opgg::commands::get_opgg_champion_build_raw,
            infrastructure::data_services::external::opgg::commands::get_opgg_tier_list,
            infrastructure::data_services::external::opgg::commands::get_opgg_champion_positions,
            infrastructure::data_services::external::opgg::commands::apply_opgg_runes,
            // LCU WS 测试命令
            infrastructure::real_time::websocket::commands::start_lcu_ws,
            infrastructure::real_time::websocket::commands::stop_lcu_ws,
            // 分析数据命令
            infrastructure::match_management::analysis_data::commands::get_cached_analysis_data,
            // 英雄数据命令
            infrastructure::data_services::champion_data::commands::init_champion_data,
            infrastructure::data_services::champion_data::commands::get_all_champion_data,
            infrastructure::data_services::champion_data::commands::get_champion_by_id,
            infrastructure::data_services::champion_data::commands::get_champion_by_alias,
            infrastructure::data_services::champion_data::commands::get_champion_by_name,
            infrastructure::data_services::champion_data::commands::is_champion_data_loaded,
            infrastructure::data_services::champion_data::commands::get_champion_count_cmd,
            // 召唤师技能数据命令
            infrastructure::champion_selection::summoner_spells::commands::init_summoner_spell_data,
            infrastructure::champion_selection::summoner_spells::commands::get_all_summoner_spell_data,
            infrastructure::champion_selection::summoner_spells::commands::get_summoner_spell_by_id,
            infrastructure::champion_selection::summoner_spells::commands::get_summoner_spell_by_name,
            infrastructure::champion_selection::summoner_spells::commands::is_summoner_spell_data_loaded,
            infrastructure::champion_selection::summoner_spells::commands::get_summoner_spell_count,
            // 房间和聊天命令
            infrastructure::game_session::lobby::commands::get_current_lobby,
            infrastructure::game_session::lobby::commands::send_lobby_chat_message,
            infrastructure::game_session::lobby::commands::send_lobby_formatted_message,
            common::commands::machine::get_machine_hash,
            common::commands::builds::get_champions_list,
            common::commands::builds::get_champion_build_new,
            common::commands::game::launch_game,
            common::commands::game::detect_game_path,
            common::commands::game::select_game_path,
            common::commands::game::save_game_path,
            common::commands::game::get_saved_game_path,
            // 数据收集测试命令
            common::commands::data_collection::generate_test_data_file,
            common::commands::data_collection::analyze_data_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
