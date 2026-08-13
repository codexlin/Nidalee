//! OCR 标题文本 → 海克斯增强目录模糊匹配
//!
//! 规则对齐 ARAMGG `augment-title-matcher.ts`。

use crate::infrastructure::data_services::external::hextech::parser::AugmentCatalogEntry;

const OCR_TITLE_MAX_EXTRA_NORMALIZED_CHARS: usize = 8;

const MATCH_BLACKLIST: &[&str] = &[
    "攻击", "防御", "生命", "法术", "魔法", "伤害", "护甲", "技能", "冷却", "移速", "暴击", "吸血",
    "穿透", "功能", "能力", "效果", "被动", "主动", "额外", "持续", "提供", "增加", "获得", "造成",
    "复原力", "回复能力", "速度", "功能性",
];

const TRAILING_UI_TAGS: &[&str] = &[
    "回复能力", "复原力", "功能性", "伤害", "速度", "防御", "生命", "法术", "魔法", "冷却", "护甲",
    "暴击", "吸血", "穿透", "移速", "攻击", "技能", "功能", "能力", "效果",
];

const PUNCTUATION: &[char] = &[
    ' ', '\t', '\n', '"', '\'', '“', '”', '‘', '’', '`', '.', ',', '，', '。', ':', '：', ';', '；',
    '!', '！', '?', '？', '、', '|', '｜', '/', '\\', '(', ')', '[', ']', '{', '}', '<', '>', '《',
    '》', '【', '】', '「', '」', '『', '』', '-', '_', '=', '+', '~', '·', '•',
];

#[derive(Debug, Clone)]
pub struct MatchedAugment {
    pub id: i32,
    pub name: String,
    pub rarity: String,
    pub rarity_display_name: String,
    pub icon_url: String,
    pub confidence: f32,
}

pub fn normalize_ocr_text(value: &str) -> String {
    value
        .chars()
        .filter_map(|c| {
            let mapped = if ('\u{FF01}'..='\u{FF5E}').contains(&c) {
                char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
            } else {
                c
            };
            if mapped.is_whitespace() || PUNCTUATION.contains(&mapped) {
                None
            } else {
                Some(mapped)
            }
        })
        .collect()
}

pub fn has_meaningful_slot_text(text: &str) -> bool {
    let normalized = normalize_ocr_text(text);
    if normalized.is_empty() {
        return false;
    }
    let cjk = normalized.chars().filter(|c| ('\u{4E00}'..='\u{9FFF}').contains(c)).count();
    cjk >= 2 || normalized.chars().count() >= 4
}

fn is_blacklisted(name: &str) -> bool {
    MATCH_BLACKLIST.iter().any(|item| *item == name)
}

pub fn strip_trailing_ui_tags(text: &str) -> String {
    let mut out = text.to_string();
    let mut changed = true;
    while changed {
        changed = false;
        for token in TRAILING_UI_TAGS {
            if out.ends_with(token) && out.chars().count() > token.chars().count() + 1 {
                let keep = out.chars().count() - token.chars().count();
                out = out.chars().take(keep).collect();
                changed = true;
                break;
            }
        }
    }
    out
}

fn alias_for(name: &str) -> Option<&'static str> {
    match name {
        "一板一眼" => Some("板一眼"),
        _ => None,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, a_ch) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, b_ch) in b.iter().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            current[j + 1] = (prev[j + 1] + 1).min(current[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut current);
    }
    prev[b.len()]
}

fn fuzzy_find(text: &str, name: &str) -> Option<(usize, usize, usize)> {
    let name_len = name.chars().count();
    let text_len = text.chars().count();
    if name_len == 0 || text_len == 0 {
        return None;
    }
    if text_len >= name_len {
        if let Some(index) = text.find(name) {
            return Some((index, 0, name_len));
        }
        if let Some(alias) = alias_for(name) {
            let alias = normalize_ocr_text(alias);
            if let Some(index) = text.find(&alias) {
                return Some((index, 0, alias.chars().count()));
            }
        }
        if name_len <= 2 {
            return None;
        }
        let max_distance = if name_len == 3 { 1 } else { name_len / 3 };
        let text_chars: Vec<char> = text.chars().collect();
        let name_chars: Vec<char> = name.chars().collect();
        let mut best: Option<(usize, usize, usize)> = None;
        for i in 0..=text_len - name_len {
            let window: String = text_chars[i..i + name_len].iter().collect();
            let name_s: String = name_chars.iter().collect();
            let dist = edit_distance(&window, &name_s);
            if dist <= max_distance && best.is_none_or(|(_, d, _)| dist < d) {
                best = Some((i, dist, name_len));
            }
        }
        return best;
    }

    if text_len < 3 {
        return None;
    }
    if name.contains(text) {
        return Some((0, 0, text_len));
    }
    let max_distance = if text_len == 3 { 1 } else { text_len / 3 };
    let text_chars: Vec<char> = text.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();
    let mut best: Option<(usize, usize, usize)> = None;
    for i in 0..=name_len - text_len {
        let window: String = name_chars[i..i + text_len].iter().collect();
        let text_s: String = text_chars.iter().collect();
        let dist = edit_distance(&window, &text_s);
        if dist <= max_distance && best.is_none_or(|(_, d, _)| dist < d) {
            best = Some((i, dist, text_len));
        }
    }
    best
}

fn version_priority(id: i32) -> i32 {
    if id >= 1000 { id + 100_000 } else { id }
}

fn confidence_from_distance(distance: usize, name_len: usize) -> f32 {
    if distance == 0 {
        0.95
    } else {
        (1.0 - distance as f32 / name_len.max(1) as f32).clamp(0.4, 0.9)
    }
}

pub fn match_slot_text(
    raw_text: &str,
    catalog: &[AugmentCatalogEntry],
    seen_ids: &[i32],
) -> Option<MatchedAugment> {
    let stripped = strip_trailing_ui_tags(&normalize_ocr_text(raw_text));
    let normalized = if stripped.is_empty() {
        normalize_ocr_text(raw_text)
    } else {
        stripped
    };
    if normalized.is_empty() || is_blacklisted(&normalized) {
        return None;
    }

    let mut entries: Vec<&AugmentCatalogEntry> = catalog.iter().collect();
    entries.sort_by(|a, b| {
        let a_len = normalize_ocr_text(&a.name).chars().count();
        let b_len = normalize_ocr_text(&b.name).chars().count();
        b_len.cmp(&a_len).then_with(|| version_priority(b.id).cmp(&version_priority(a.id)))
    });

    let mut best: Option<(MatchedAugment, usize, usize)> = None;
    for entry in entries {
        if seen_ids.contains(&entry.id) {
            continue;
        }
        let name = normalize_ocr_text(&entry.name);
        if name.is_empty() || is_blacklisted(&entry.name) || is_blacklisted(&name) {
            continue;
        }
        let Some((start, distance, match_len)) = fuzzy_find(&normalized, &name) else {
            continue;
        };
        let extra = normalized.chars().count().saturating_sub(name.chars().count());
        if name.chars().count() <= 2 && extra > OCR_TITLE_MAX_EXTRA_NORMALIZED_CHARS {
            continue;
        }
        let candidate = MatchedAugment {
            id: entry.id,
            name: entry.name.clone(),
            rarity: if entry.rarity_name.is_empty() {
                entry.rarity.to_string()
            } else {
                entry.rarity_name.clone()
            },
            rarity_display_name: entry.rarity_display_name.clone(),
            icon_url: entry.icon_url.clone(),
            confidence: confidence_from_distance(distance, match_len),
        };
        let better = match &best {
            None => true,
            Some((_, best_dist, best_len)) => {
                distance < *best_dist || (distance == *best_dist && match_len > *best_len)
            }
        };
        if better {
            let _ = start;
            best = Some((candidate, distance, match_len));
        }
    }
    best.map(|(matched, _, _)| matched)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: i32, name: &str) -> AugmentCatalogEntry {
        AugmentCatalogEntry {
            id,
            name: name.to_string(),
            icon_url: String::new(),
            rarity: 1,
            rarity_name: "gold".to_string(),
            rarity_display_name: "金色".to_string(),
        }
    }

    #[test]
    fn match_slot_text_returns_exact_chinese_title() {
        let catalog = vec![entry(12, "珠宝护手"), entry(8, "熟练狙击手")];
        let matched = match_slot_text("珠宝护手", &catalog, &[]).expect("match");
        assert_eq!(matched.id, 12);
        assert_eq!(matched.name, "珠宝护手");
    }

    #[test]
    fn match_slot_text_skips_already_seen_ids() {
        let catalog = vec![entry(12, "珠宝护手")];
        assert!(match_slot_text("珠宝护手", &catalog, &[12]).is_none());
    }

    #[test]
    fn match_slot_text_allows_one_edit_on_three_char_names() {
        let catalog = vec![entry(3, "板一眼")];
        let matched = match_slot_text("板一眼", &catalog, &[]).expect("match");
        assert_eq!(matched.id, 3);
    }

    #[test]
    fn has_meaningful_slot_text_requires_two_cjk_chars() {
        assert!(has_meaningful_slot_text("珠宝"));
        assert!(!has_meaningful_slot_text("珠"));
        assert!(!has_meaningful_slot_text("  "));
    }

    #[test]
    fn match_slot_text_ignores_trailing_description() {
        let catalog = vec![entry(21, "炼狱导管"), entry(8, "熟练狙击手")];
        let matched = match_slot_text("炼狱导管技能造成的伤害会灼烧", &catalog, &[]).expect("match");
        assert_eq!(matched.id, 21);
        assert_eq!(matched.name, "炼狱导管");
    }

    #[test]
    fn match_slot_text_strips_trailing_ui_tags() {
        let catalog = vec![entry(21, "炼狱导管"), entry(8, "歌利亚巨人"), entry(9, "玻璃大炮")];
        let matched = match_slot_text("炼狱导管伤害", &catalog, &[]).expect("match");
        assert_eq!(matched.name, "炼狱导管");
        let matched = match_slot_text("歌利亚巨人回复能力", &catalog, &[]).expect("match");
        assert_eq!(matched.name, "歌利亚巨人");
        let matched = match_slot_text("玻璃大炮速度", &catalog, &[]).expect("match");
        assert_eq!(matched.name, "玻璃大炮");
    }

    #[test]
    fn match_slot_text_rejects_tag_only_ocr() {
        let catalog = vec![entry(21, "炼狱导管")];
        assert!(match_slot_text("伤害", &catalog, &[]).is_none());
        assert!(match_slot_text("复原力", &catalog, &[]).is_none());
    }
}
