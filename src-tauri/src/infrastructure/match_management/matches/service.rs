//! Stable match-service facade.
//!
//! Callers continue to use `matches::service::*`; the implementation is split by
//! responsibility so transport, response mapping, and compatibility adapters do
//! not accumulate in one module.

mod detail;
mod detail_dto;
mod overview;
mod process_review;

pub use detail::get_game_detail_logic;
// Keep the historical facade even when a specific binary does not use every adapter.
#[allow(unused_imports)]
pub use overview::{fetch_match_list, get_recent_matches_by_puuid, get_recent_matches_by_puuid_with_perspective};
pub use process_review::get_game_process_review_logic;

pub(crate) use overview::get_match_history;
