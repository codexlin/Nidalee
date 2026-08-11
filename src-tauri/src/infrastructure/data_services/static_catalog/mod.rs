//! 版本化静态身份目录（英雄 / 召唤师技能）
//!
//! - 权威拉取与落盘在 Rust
//! - 前端仅通过 IPC 消费
//! - 版本不变读盘；版本变化才打网

pub mod commands;
pub mod service;

pub use service::{
    ensure_static_catalogs, fetch_ddragon_version, get_static_meta, refresh_static_catalogs_if_stale, StaticCatalogMeta,
};
