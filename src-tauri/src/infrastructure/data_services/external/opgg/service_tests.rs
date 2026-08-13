use super::*;
use serde_json::json;

#[test]
fn canonical_tier_is_accepted() {
    assert!(validate_request("kr", "ranked", Some(67), "diamond_plus").is_ok());
}

#[test]
fn non_canonical_tier_is_rejected() {
    assert!(validate_request("kr", "ranked", Some(67), "DIAMOND+").is_err());
}

#[test]
fn parse_ranked_item() {
    let item = json!({
        "id": 86,
        "average_stats": {
            "play": 100,
            "win_rate": 0.52,
            "pick_rate": 0.1,
            "ban_rate": 0.02,
            "kda": 2.5,
            "tier_data": { "tier": 2, "rank": 6 }
        },
        "positions": [{
            "name": "MID",
            "stats": {
                "play": 80,
                "win_rate": 0.53,
                "pick_rate": 0.08,
                "ban_rate": 0.02,
                "kda": 2.6,
                "tier_data": { "tier": 2, "rank": 4 }
            },
            "counters": [{ "champion_id": 103, "play": 10, "win": 6 }]
        }],
        "roles": []
    });
    let parsed = parse_tier_list_item(&item).expect("ranked");
    assert_eq!(parsed.champion_id, 86);
    assert_eq!(parsed.positions.len(), 1);
    assert!((parsed.average_stats.win_rate - 0.52).abs() < 1e-6);
}

#[test]
fn parse_aram_roles_and_null_ban() {
    let item = json!({
        "id": 86,
        "average_stats": {
            "play": 100,
            "win_rate": 0.49,
            "pick_rate": 0.2,
            "ban_rate": null,
            "kda": 3.0,
            "tier_data": { "tier": 1, "rank": 3 }
        },
        "positions": null,
        "roles": [{
            "name": "TANK",
            "stats": { "win_rate": 0.5, "role_rate": 0.6, "play": 60, "win": 30 }
        }]
    });
    let parsed = parse_tier_list_item(&item).expect("aram");
    assert!(parsed.positions.is_empty());
    assert_eq!(parsed.roles.len(), 1);
    assert_eq!(parsed.roles[0].name, "TANK");
    assert_eq!(parsed.average_stats.ban_rate, 0.0);
}

#[test]
fn parse_arena_chicken_rate_and_skip_rip() {
    let rip = json!({ "id": 60001, "is_rip": true, "average_stats": null });
    assert!(parse_tier_list_item(&rip).is_none());

    let item = json!({
        "id": 50,
        "average_stats": {
            "win": 2000,
            "play": 10000,
            "total_place": 40000,
            "first_place": 1580,
            "pick_rate": 0.1,
            "ban_rate": 0.05,
            "kills": 1.0,
            "assists": 2.0,
            "deaths": 1.0,
            "tier_data": { "tier": 1, "rank": 2 }
        },
        "positions": null,
        "roles": null
    });
    let parsed = parse_tier_list_item(&item).expect("arena");
    assert_eq!(parsed.average_stats.first_place, Some(1580));
    assert!((parsed.average_stats.win_rate - 0.158).abs() < 1e-6);
}
