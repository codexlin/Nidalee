use crate::infrastructure::match_management::matches::service::get_recent_matches_by_puuid;
use crate::shared::utils::{lcu_get, lcu_patch_no_content};
use crate::infrastructure::data_services::summoner::service::get_summoner_by_id;
use crate::shared::types::{ChampSelectPlayer, ChampSelectSession, PlayerMatchStats, SummonerInfo};
use reqwest::Client;
use serde_json::{Number, Value};
use std::collections::HashMap;

// 获取选人会话信息 (简化版本，返回 Value)
pub async fn get_champ_select_session_raw(client: &Client) -> Result<Value, String> {
    lcu_get(client, "/lol-champ-select/v1/session").await
}

// 选择/禁用英雄的通用函数
pub async fn champion_action(client: &Client, action_id: u64, champion_id: u64, completed: bool) -> Result<(), String> {
    let url = format!("/lol-champ-select/v1/session/actions/{}", action_id);
    let body = serde_json::json!({
        "championId": champion_id,
        "completed": completed
    });

    lcu_patch_no_content(client, &url, body).await
}

// 选择英雄 (hover 或 lock)
pub async fn pick_champion(client: &Client, action_id: u64, champion_id: u64, completed: bool) -> Result<(), String> {
    champion_action(client, action_id, champion_id, completed).await
}

// 禁用英雄
pub async fn ban_champion(client: &Client, action_id: u64, champion_id: u64) -> Result<(), String> {
    champion_action(client, action_id, champion_id, true).await
}

// ---------- 数据清洗函数 ----------

fn fix_team_array(team: &mut Vec<Value>) {
    for player in team {
        if let Some(player_obj) = player.as_object_mut() {
            // summonerId 转字符串
            if let Some(summoner_id) = player_obj.get("summonerId") {
                if let Some(id) = summoner_id.as_u64() {
                    player_obj.insert("summonerId".to_string(), Value::String(id.to_string()));
                }
            }
            // 处理其他大数值字段
            for field in [
                "championId",
                "championPickIntent",
                "selectedSkinId",
                "spell1Id",
                "spell2Id",
            ] {
                if let Some(value) = player_obj.get(field) {
                    if let Some(num) = value.as_f64() {
                        if num == 1.8446744073709552e19 || num == 0.0 {
                            player_obj.insert(field.to_string(), Value::Null);
                        } else if let Some(number) = Number::from_f64(num) {
                            player_obj.insert(field.to_string(), Value::Number(number));
                        }
                    }
                }
            }
        }
    }
}

fn fix_bans(ban_list: &mut Vec<Value>) {
    for ban in ban_list {
        if let Some(num) = ban.as_f64() {
            if num == 0.0 {
                *ban = Value::Null;
            } else if let Some(number) = Number::from_f64(num) {
                *ban = Value::Number(number);
            }
        }
    }
}

/// 批量 enrich 召唤师信息
async fn enrich_champ_select_session(client: &Client, session: &mut ChampSelectSession) {
    // 收集所有 summoner_id
    let mut all_ids = vec![];
    for p in session.my_team.iter().chain(session.their_team.iter()) {
        if let Some(sid) = &p.summoner_id {
            if sid != "0" && !all_ids.contains(sid) {
                all_ids.push(sid.clone());
            }
        }
    }
    // 查询所有召唤师信息和 puuid
    let mut info_map = std::collections::HashMap::new();
    let mut puuid_map = std::collections::HashMap::new();
    for sid in &all_ids {
        if let Ok(id) = sid.parse::<u64>() {
            if let Ok(info) = get_summoner_by_id(client, id).await {
                puuid_map.insert(sid.clone(), info.puuid.clone());
                info_map.insert(sid.clone(), info);
            }
        }
    }
    // 补全 my_team
    for p in session.my_team.iter_mut() {
        enrich_player(p, &info_map);
    }
    // 补全 their_team
    for p in session.their_team.iter_mut() {
        enrich_player(p, &info_map);
    }
}

fn enrich_player(player: &mut ChampSelectPlayer, info_map: &std::collections::HashMap<String, SummonerInfo>) {
    if let Some(sid) = &player.summoner_id {
        if sid == "0" {
            player.display_name = Some("机器人".to_string());
            player.tag_line = None;
            player.profile_icon_id = None;
            player.tier = None;
        } else if let Some(info) = info_map.get(sid) {
            // 优先用 game_name + tag_line
            let display_name = if let (Some(game_name), Some(tag_line)) = (&info.game_name, &info.tag_line) {
                format!("{}#{}", game_name, tag_line)
            } else {
                info.display_name.clone()
            };
            player.display_name = Some(display_name);
            player.tag_line = info.tag_line.clone();
            player.profile_icon_id = Some(info.profile_icon_id);
            player.tier = info.solo_rank_tier.clone();
        }
    }
}

// ---------- 主函数 ----------

/// 获取当前选人阶段的完整 session 信息（最优实践版）
pub async fn get_champ_select_session(client: &Client) -> Result<ChampSelectSession, String> {
    // 直接用通用 LCU 请求工具
    let mut json: Value = lcu_get(client, "/lol-champ-select/v1/session").await?;

    // 数据清洗 -- myTeam & theirTeam
    if let Some(my_team) = json.get_mut("myTeam").and_then(|t| t.as_array_mut()) {
        fix_team_array(my_team);
    }
    if let Some(their_team) = json.get_mut("theirTeam").and_then(|t| t.as_array_mut()) {
        fix_team_array(their_team);
    }

    // 数据清洗 -- bans
    if let Some(bans) = json.get_mut("bans").and_then(|b| b.as_object_mut()) {
        for team in ["myTeamBans", "theirTeamBans"] {
            if let Some(ban_list) = bans.get_mut(team).and_then(|l| l.as_array_mut()) {
                fix_bans(ban_list);
            }
        }
    }
    log::info!("[get_champ_select_session] 原始 session JSON");
    // 反序列化为结构体
    let mut session =
        serde_json::from_value::<ChampSelectSession>(json).map_err(|e| format!("解析 session 响应失败: {}", e))?;
    // enrich
    enrich_champ_select_session(client, &mut session).await;
    Ok(session)
}

// 主函数：批量获取队友和对手信息（无缓存，简洁版）
pub async fn get_champselect_team_players_info(client: &Client) -> Result<HashMap<String, PlayerMatchStats>, String> {
    // 1. 获取当前选人会话
    let session: serde_json::Value = lcu_get(client, "/lol-champ-select/v1/session").await?;
    let my_team = session
        .get("myTeam")
        .and_then(|v| v.as_array())
        .ok_or("myTeam解析失败")?;
    let their_team = session
        .get("theirTeam")
        .and_then(|v| v.as_array())
        .ok_or("theirTeam解析失败")?;

    // 2. 收集所有 summoner_id
    let extract_id = |player: &serde_json::Value| {
        player
            .get("summonerId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .to_string()
    };
    let my_ids: Vec<String> = my_team.iter().map(extract_id).filter(|id| id != "0").collect();
    let their_ids: Vec<String> = their_team.iter().map(extract_id).filter(|id| id != "0").collect();

    // 3. 批量查SummonerInfo
    let mut all_ids = my_ids.clone();
    all_ids.extend(their_ids.iter().cloned());
    all_ids.sort();
    all_ids.dedup();

    let mut info_map = HashMap::new();
    for sid in &all_ids {
        if let Ok(id) = sid.parse::<u64>() {
            if let Ok(info) = get_summoner_by_id(client, id).await {
                info_map.insert(sid.clone(), info);
            }
        }
    }

    // 4. 批量查最近战绩
    let mut match_map = HashMap::new();
    log::info!("准备批量查最近10场战绩, 总人数: {}", info_map.len());
    for (sid, info) in &info_map {
        log::info!("查找召唤师 {} recent matches", sid);
        // 选人阶段，不过滤队列类型
        if let Ok(matches) = get_recent_matches_by_puuid(client, &info.puuid, 20, None).await {
            log::info!("查到 {:?} 场", matches);
            match_map.insert(sid.clone(), matches);
        } else {
            log::warn!("查找 {} 失败", sid);
        }
    }
    Ok(match_map)
}
