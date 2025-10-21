use crate::shared::types::SummonerTrait;

/// 智能去重优化
pub fn optimize_traits(mut traits: Vec<SummonerTrait>) -> Vec<SummonerTrait> {
    // 1. 去重相同名称的特征
    traits.sort_by(|a, b| {
        let name_cmp = a.name.cmp(&b.name);
        if name_cmp == std::cmp::Ordering::Equal {
            b.score.cmp(&a.score)
        } else {
            name_cmp
        }
    });
    traits.dedup_by(|a, b| a.name == b.name);

    // 2. 处理相似特征合并
    let traits = merge_similar_traits(traits);

    // 3. 按优先级排序
    let mut traits = traits;
    traits.sort_by(|a, b| {
        // good 特征在前
        match (a.trait_type.as_str(), b.trait_type.as_str()) {
            ("good", "bad") => std::cmp::Ordering::Less,
            ("bad", "good") => std::cmp::Ordering::Greater,
            _ => b.score.cmp(&a.score),
        }
    });

    // 4. 限制数量
    traits.truncate(12);

    traits
}

fn merge_similar_traits(traits: Vec<SummonerTrait>) -> Vec<SummonerTrait> {
    let mut result = Vec::new();
    let mut seen_groups: std::collections::HashSet<String> = std::collections::HashSet::new();

    for trait_item in traits {
        let group_key = get_trait_group_key(&trait_item.name);

        if seen_groups.contains(&group_key) {
            continue;
        }

        seen_groups.insert(group_key);
        result.push(trait_item);
    }

    result
}

fn get_trait_group_key(name: &str) -> String {
    // 将相似特征归为同一组
    if name.contains("输出") {
        "输出".to_string()
    } else if name.contains("视野") {
        "视野".to_string()
    } else if name.contains("稳定") || name == "稳如老狗" {
        "稳定".to_string()
    } else if name.contains("连胜") || name.contains("连败") || name.contains("状态") || name.contains("近期") {
        "状态".to_string()
    } else {
        name.to_string()
    }
}
