pub mod paths;
pub mod manager;
pub mod models;
pub mod nested;
pub mod backup;
pub mod money;
pub mod rank;
pub mod products;
pub mod properties;
pub mod businesses;
pub mod npcs;
pub mod quests;
pub mod variables;
pub mod inventory;
pub mod appearance;
pub mod misc;
pub mod templates;

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Msg(String),
}

pub type Result<T> = std::result::Result<T, SaveError>;

/// A discovered save folder.
#[derive(Debug, Clone)]
pub struct SaveInfoEntry {
    pub name: String,
    pub path: PathBuf,
    pub org: String,
    pub version: String,
    pub played: String,
}
