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
pub use process_review::get_game_process_review_logic;

#[cfg(debug_assertions)]
pub use overview::fetch_match_list;
#[cfg(debug_assertions)]
pub(crate) use overview::get_match_history;
