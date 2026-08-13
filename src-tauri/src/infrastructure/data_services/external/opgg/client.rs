use reqwest::Client;
use serde_json::Value;

/// OP.GG API 客户端
pub struct OpggClient {
    client: Client,
}

impl OpggClient {
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    /// 获取英雄详细数据（按模式拼不同路径）
    ///
    /// - ranked: `.../ranked/{id}/{POSITION}?tier=`
    /// - aram/urf: `.../{mode}/{id}/none`
    /// - arena: `.../arena/{id}`（不可带空 tier query）
    pub async fn get_champion_build(
        &self,
        region: &str,
        mode: &str,
        champion_id: i32,
        position: &str,
        tier: &str,
    ) -> Result<Value, String> {
        let url = champion_build_url(region, mode, champion_id, position, tier);
        log::info!("请求OP.GG API: {}", url);
        self.get_json(&url).await
    }

    /// 获取英雄层级列表
    ///
    /// - ranked: 带 `tier` 筛选
    /// - 其它模式: 不带 query（现网 `?tier=` 虽多半可用，但 arena 构建对空 tier 敏感，列表统一干净路径）
    pub async fn get_tier_list(&self, region: &str, mode: &str, tier: &str) -> Result<Value, String> {
        let url = tier_list_url(region, mode, tier);
        log::info!("请求OP.GG层级列表: {}", url);
        self.get_json(&url).await
    }

    /// 获取英雄可用位置列表（排位用）
    async fn get_json(&self, url: &str) -> Result<Value, String> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("网络请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API 请求失败: HTTP {}", response.status()));
        }

        response.json().await.map_err(|e| format!("解析 JSON 失败: {}", e))
    }
}

pub fn resolve_build_position(mode: &str, position: Option<&str>) -> Result<String, String> {
    match mode {
        "aram" | "urf" => Ok("none".to_string()),
        "arena" => Ok(String::new()),
        _ => {
            let position = position
                .filter(|position| !position.trim().is_empty() && !position.eq_ignore_ascii_case("none"))
                .ok_or_else(|| "排位推荐必须指定有效位置".to_string())?
                .trim()
                .to_ascii_uppercase();
            let canonical = match position.as_str() {
                "MIDDLE" => "MID",
                "BOTTOM" => "ADC",
                "UTILITY" => "SUPPORT",
                value => value,
            };
            matches!(canonical, "TOP" | "JUNGLE" | "MID" | "ADC" | "SUPPORT")
                .then(|| canonical.to_string())
                .ok_or_else(|| format!("不支持的排位位置: {canonical}"))
        }
    }
}

fn champion_build_url(region: &str, mode: &str, champion_id: i32, position: &str, tier: &str) -> String {
    match mode {
        "arena" => format!(
            "https://lol-api-champion.op.gg/api/{}/champions/arena/{}",
            region, champion_id
        ),
        "aram" | "urf" => format!(
            "https://lol-api-champion.op.gg/api/{}/champions/{}/{}/none",
            region, mode, champion_id
        ),
        _ => format!(
            "https://lol-api-champion.op.gg/api/{}/champions/{}/{}/{}?tier={}",
            region, mode, champion_id, position, tier
        ),
    }
}

fn tier_list_url(region: &str, mode: &str, tier: &str) -> String {
    if mode == "ranked" && !tier.is_empty() {
        format!(
            "https://lol-api-champion.op.gg/api/{}/champions/{}?tier={}",
            region, mode, tier
        )
    } else {
        format!("https://lol-api-champion.op.gg/api/{}/champions/{}", region, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranked_build_url_uses_position_and_tier() {
        let u = champion_build_url("kr", "ranked", 86, "MID", "emerald_plus");
        assert_eq!(
            u,
            "https://lol-api-champion.op.gg/api/kr/champions/ranked/86/MID?tier=emerald_plus"
        );
    }

    #[test]
    fn aram_urf_build_url_uses_none_slot() {
        assert_eq!(
            champion_build_url("kr", "aram", 86, "MID", "emerald_plus"),
            "https://lol-api-champion.op.gg/api/kr/champions/aram/86/none"
        );
        assert_eq!(
            champion_build_url("kr", "urf", 1, "", ""),
            "https://lol-api-champion.op.gg/api/kr/champions/urf/1/none"
        );
    }

    #[test]
    fn arena_build_url_has_no_query() {
        assert_eq!(
            champion_build_url("kr", "arena", 86, "", "emerald_plus"),
            "https://lol-api-champion.op.gg/api/kr/champions/arena/86"
        );
    }

    #[test]
    fn tier_list_url_ranked_keeps_tier() {
        assert!(tier_list_url("kr", "ranked", "emerald_plus").contains("tier=emerald_plus"));
        assert_eq!(
            tier_list_url("kr", "aram", "emerald_plus"),
            "https://lol-api-champion.op.gg/api/kr/champions/aram"
        );
        assert_eq!(
            tier_list_url("kr", "arena", ""),
            "https://lol-api-champion.op.gg/api/kr/champions/arena"
        );
    }

    #[test]
    fn resolve_position_by_mode() {
        assert_eq!(resolve_build_position("aram", Some("MID")).as_deref(), Ok("none"));
        assert_eq!(resolve_build_position("urf", None).as_deref(), Ok("none"));
        assert_eq!(resolve_build_position("arena", Some("MID")).as_deref(), Ok(""));
        assert_eq!(resolve_build_position("ranked", Some("TOP")).as_deref(), Ok("TOP"));
        assert_eq!(
            resolve_build_position("ranked", Some("jungle")).as_deref(),
            Ok("JUNGLE")
        );
        assert_eq!(resolve_build_position("ranked", Some("middle")).as_deref(), Ok("MID"));
        assert_eq!(resolve_build_position("ranked", Some("bottom")).as_deref(), Ok("ADC"));
        assert_eq!(
            resolve_build_position("ranked", Some("utility")).as_deref(),
            Ok("SUPPORT")
        );
    }

    #[test]
    fn ranked_position_is_required() {
        assert_eq!(
            resolve_build_position("ranked", None),
            Err("排位推荐必须指定有效位置".to_string())
        );
    }

    #[test]
    fn ranked_position_rejects_unknown_value() {
        assert_eq!(
            resolve_build_position("ranked", Some("FLEX")),
            Err("不支持的排位位置: FLEX".to_string())
        );
    }
}
