use crate::shared::types::LiveClientPlayer;

#[derive(Debug)]
pub(crate) struct Ambiguous;

/// Zero means no match. Ties at the highest rank cannot identify a player.
pub(crate) fn unique_best<T>(
    items: impl IntoIterator<Item = T>,
    rank: impl Fn(&T) -> u8,
) -> Result<Option<T>, Ambiguous> {
    let mut highest = 0;
    let mut candidate = None;
    for item in items {
        let score = rank(&item);
        if score > highest {
            highest = score;
            candidate = Some(item);
        } else if score == highest {
            candidate = None;
        }
    }
    if highest > 0 && candidate.is_none() {
        Err(Ambiguous)
    } else {
        Ok(candidate)
    }
}

pub(crate) fn riot_id_rank(left: &str, right: &str) -> u8 {
    fn parts(id: &str) -> (&str, Option<&str>) {
        let (name, tag) = id.trim().split_once('#').unwrap_or((id.trim(), ""));
        (name.trim(), Some(tag.trim()).filter(|tag| !tag.is_empty()))
    }
    let (left, a) = parts(left);
    let (right, b) = parts(right);
    if left.is_empty() || !left.eq_ignore_ascii_case(right) {
        return 0;
    }
    match (a, b) {
        (Some(a), Some(b)) if a.eq_ignore_ascii_case(b) => 2,
        (Some(_), Some(_)) => 0,
        _ => 1,
    }
}

pub(crate) fn live_name_match<'a>(
    players: &'a [LiveClientPlayer],
    name: &str,
) -> Result<Option<&'a LiveClientPlayer>, Ambiguous> {
    unique_best(players.iter().filter(|p| !p.is_bot), |p| {
        riot_id_rank(&p.summoner_name, name)
    })
}
