//! 游戏内海克斯推荐侧栏
//!
//! 选人锁定英雄后预载构建中心的推荐三连 / 推荐增强；
//! 对局里 OCR 当前三张卡，标出 T 级最高且胜率最高的一张。

pub mod capture;
pub mod commands;
pub mod hotkey_hook;
pub mod matcher;
pub mod ocr;
pub mod offer;
pub mod scan;
pub mod shortcut;
pub mod state;
pub mod types;
pub mod window;
