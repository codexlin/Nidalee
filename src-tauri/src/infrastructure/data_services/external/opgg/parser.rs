use crate::infrastructure::data_services::external::opgg::types::*;
use serde_json::Value;

/// 解析OP.GG英雄详细数据
pub fn parse_champion_build(data: Value, position: &str) -> Result<OpggChampionBuild, String> {
    let content = data.get("data").unwrap_or(&data); // 兼容 data.data 或 data

    // summary
    let summary_obj = content.get("summary").ok_or("缺少summary")?;
    let average_stats = summary_obj.get("average_stats").ok_or("缺少average_stats")?;
    let name = summary_obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let champion_id = summary_obj.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let icon = String::new(); // OP.GG未直接提供icon字段
    let play = average_stats.get("play").and_then(|v| v.as_i64()).unwrap_or(0);
    let first_place = average_stats.get("first_place").and_then(|v| v.as_i64());
    // Arena 用吃鸡率填 win_rate，便于前端统一展示
    let win_rate = average_stats.get("win_rate").and_then(|v| v.as_f64()).or_else(|| {
        first_place.and_then(|fp| if play > 0 { Some(fp as f64 / play as f64) } else { None })
    });
    let pick_rate = average_stats.get("pick_rate").and_then(|v| v.as_f64());
    let ban_rate = average_stats.get("ban_rate").and_then(|v| v.as_f64());
    let kda = average_stats.get("kda").and_then(|v| v.as_f64()).or_else(|| {
        // Arena 无 kda 字段时用 K/D/A 粗算
        let kills = average_stats.get("kills").and_then(|v| v.as_f64())?;
        let deaths = average_stats.get("deaths").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let assists = average_stats.get("assists").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Some(if deaths > 0.0 {
            (kills + assists) / deaths
        } else {
            kills + assists
        })
    });
    let tier = average_stats
        .get("tier_data")
        .and_then(|t| t.get("tier"))
        .or_else(|| average_stats.get("tier"))
        .map(|v| v.to_string());
    let rank = average_stats
        .get("tier_data")
        .and_then(|t| t.get("rank"))
        .and_then(|v| v.as_i64())
        .or_else(|| average_stats.get("rank").and_then(|v| v.as_i64()))
        .map(|v| v as i32);
    let summary = OpggChampionSummary {
        name,
        champion_id,
        icon,
        position: position.to_string(),
        win_rate,
        pick_rate,
        ban_rate,
        kda,
        tier,
        rank,
    };

    // Arena 无召唤师技能/符文；缺失时给空列表，勿整包失败
    let summoner_spells = parse_summoner_spells(content).unwrap_or_default();
    let items = parse_items(content)?;
    let counters = parse_counters(content).unwrap_or_else(|_| OpggCounters {
        strong_against: vec![],
        weak_against: vec![],
    });
    let perks = parse_perks(content).unwrap_or_default();
    let champion_skills = parse_champion_skills(content).unwrap_or_else(|_| OpggSkills {
        masteries: vec![],
        order: vec![],
        play: 0,
        win: 0,
        pick_rate: 0.0,
    });

    Ok(OpggChampionBuild {
        summary,
        summoner_spells,
        champion_skills,
        items,
        counters,
        perks,
    })
}

fn parse_summoner_spells(content: &Value) -> Result<Vec<OpggSummonerSpell>, String> {
    let Some(spells_array) = content.get("summoner_spells").and_then(|v| v.as_array()) else {
        return Ok(vec![]);
    };
    let mut spells = Vec::new();
    for spell_data in spells_array.iter() {
        let ids: Vec<i32> = spell_data
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
            .unwrap_or_default();
        let win = spell_data.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let play = spell_data.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let pick_rate = spell_data.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
        // 取第一个id为主id
        let spell_id = ids.get(0).copied().unwrap_or(0);
        spells.push(OpggSummonerSpell {
            spell_id,
            ids,
            win,
            play,
            pick_rate,
        });
    }
    Ok(spells)
}

fn parse_items(content: &Value) -> Result<OpggItems, String> {
    let start_items = parse_item_category(content, "starter_items")?;
    let core_items = parse_item_category(content, "core_items")?;
    let boots = parse_item_category(content, "boots")?;
    let last_items = parse_item_category(content, "last_items")?;
    Ok(OpggItems {
        start_items,
        core_items,
        boots,
        last_items,
    })
}

fn parse_item_category(content: &Value, category: &str) -> Result<Vec<OpggItem>, String> {
    let binding = vec![];
    let category_data = content.get(category).and_then(|v| v.as_array()).unwrap_or(&binding);
    let mut items = Vec::new();
    for item_data in category_data.iter() {
        let ids: Vec<i32> = item_data
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
            .unwrap_or_default();
        let id = ids.get(0).copied().unwrap_or(0);
        let icons = Vec::new(); // OP.GG未直接提供icon
        let win = item_data.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let play = item_data.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let pick_rate = item_data.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
        items.push(OpggItem {
            id,
            ids: ids.clone(),
            icons,
            win,
            play,
            pick_rate,
        });
    }
    Ok(items)
}

fn parse_counters(content: &Value) -> Result<OpggCounters, String> {
    let binding = vec![];
    let counters_data = content.get("counters").and_then(|v| v.as_array()).unwrap_or(&binding);
    let mut strong_against = Vec::new();
    let mut weak_against = Vec::new();
    for counter in counters_data.iter() {
        let champion_id = counter.get("champion_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let play = counter.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let win = counter.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let win_rate = if play > 0 { win as f64 / play as f64 } else { 0.0 };
        // 这里简单分配，实际可根据业务调整
        if win_rate > 0.5 {
            strong_against.push(OpggCounter { champion_id, win_rate });
        } else {
            weak_against.push(OpggCounter { champion_id, win_rate });
        }
    }
    Ok(OpggCounters {
        strong_against,
        weak_against,
    })
}

fn parse_perks(content: &Value) -> Result<Vec<OpggPerk>, String> {
    let binding = vec![];
    let runes_array = content.get("runes").and_then(|v| v.as_array()).unwrap_or(&binding);
    let mut perks = Vec::new();
    for rune in runes_array.iter() {
        let primary_id = rune.get("primary_page_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let secondary_id = rune.get("secondary_page_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let primary_rune_ids: Vec<i32> = rune
            .get("primary_rune_ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
            .unwrap_or_default();
        let secondary_rune_ids: Vec<i32> = rune
            .get("secondary_rune_ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
            .unwrap_or_default();
        let stat_mod_ids: Vec<i32> = rune
            .get("stat_mod_ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
            .unwrap_or_default();
        // 按主系4+副系2+属性3顺序拼接
        let mut perks_list = Vec::new();
        perks_list.extend(primary_rune_ids.iter().take(4));
        perks_list.extend(secondary_rune_ids.iter().take(2));
        perks_list.extend(stat_mod_ids.iter().take(3));
        let win = rune.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let play = rune.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let pick_rate = rune.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
        perks.push(OpggPerk {
            primary_id,
            secondary_id,
            perks: perks_list,
            win,
            play,
            pick_rate,
        });
    }
    Ok(perks)
}

fn parse_champion_skills(content: &Value) -> Result<OpggSkills, String> {
    // 优先解析 skill_masteries
    if let Some(skill_masteries) = content.get("skill_masteries").and_then(|v| v.as_array()) {
        if let Some(main) = skill_masteries.get(0) {
            let masteries = main
                .get("ids")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let play = main.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let win = main.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let pick_rate = main.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let order = main
                .get("builds")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|build| build.get("order"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            return Ok(OpggSkills {
                masteries,
                order,
                play,
                win,
                pick_rate,
            });
        }
    }
    // fallback: 兼容旧数据
    let binding = vec![];
    let skills_array = content.get("skills").and_then(|v| v.as_array()).unwrap_or(&binding);
    let mut order = Vec::new();
    let masteries = Vec::new();
    let mut play = 0;
    let mut win = 0;
    let mut pick_rate = 0.0;
    if let Some(skill) = skills_array.get(0) {
        order = skill
            .get("order")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        play = skill.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        win = skill.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        pick_rate = skill.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
    }
    Ok(OpggSkills {
        masteries,
        order,
        play,
        win,
        pick_rate,
    })
}
