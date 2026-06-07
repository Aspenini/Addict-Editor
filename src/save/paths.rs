use super::SaveInfoEntry;
use std::path::{Path, PathBuf};

/// `%USERPROFILE%/AppData/LocalLow/TVGS/Schedule I/saves`
pub fn saves_root() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let p = home
        .join("AppData")
        .join("LocalLow")
        .join("TVGS")
        .join("Schedule I")
        .join("saves");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

pub fn is_steamid(name: &str) -> bool {
    name.len() == 17 && name.chars().all(|c| c.is_ascii_digit())
}

/// First 17-digit steam-id folder under the saves root.
pub fn steamid_folder() -> Option<PathBuf> {
    let root = saves_root()?;
    std::fs::read_dir(&root)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(is_steamid)
                    .unwrap_or(false)
        })
}

/// Reads the organisation name out of a save folder's `Game.json`.
pub fn org_name(save_path: &Path) -> String {
    let game = save_path.join("Game.json");
    if let Ok(text) = std::fs::read_to_string(&game) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(name) = v.get("OrganisationName").and_then(|n| n.as_str()) {
                return name.to_string();
            }
        }
    }
    "Unknown Organisation".to_string()
}

fn read_field(save_path: &Path, file: &str, key: &str) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(save_path.join(file)).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get(key).cloned()
}

fn game_version(save_path: &Path) -> String {
    read_field(save_path, "Game.json", "GameVersion")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "?".into())
}

fn last_played(save_path: &Path) -> String {
    match read_field(save_path, "Metadata.json", "LastPlayedDate") {
        Some(c) => {
            let g = |k: &str| c.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
            format!("{:04}-{:02}-{:02}", g("Year"), g("Month"), g("Day"))
        }
        None => "unknown".into(),
    }
}

fn is_save_folder(name: &str) -> bool {
    // SaveGame_1 .. SaveGame_9
    name.starts_with("SaveGame_")
        && name
            .strip_prefix("SaveGame_")
            .map(|s| s.len() == 1 && s.chars().all(|c| c.is_ascii_digit() && c != '0'))
            .unwrap_or(false)
}

/// Lists `SaveGame_N` folders under the detected steam-id folder.
pub fn list_saves() -> Vec<SaveInfoEntry> {
    let Some(folder) = steamid_folder() else {
        return Vec::new();
    };
    list_saves_in(&folder)
}

/// Lists `SaveGame_N` folders under a specific steam-id (or any) folder.
pub fn list_saves_in(folder: &Path) -> Vec<SaveInfoEntry> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(folder) {
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if is_save_folder(name) {
                out.push(SaveInfoEntry {
                    name: name.to_string(),
                    org: org_name(&path),
                    version: game_version(&path),
                    played: last_played(&path),
                    path,
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}
