use super::{Result, SaveError};
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Owns the currently-loaded save folder and provides JSON IO helpers.
pub struct SaveManager {
    pub path: PathBuf,
}

impl SaveManager {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if !path.exists() {
            return Err(SaveError::Msg(format!(
                "save folder does not exist: {}",
                path.display()
            )));
        }
        Ok(Self { path })
    }

    pub fn file(&self, rel: &str) -> PathBuf {
        self.path.join(rel)
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.file(rel).exists()
    }

    /// Reads a JSON file as a `Value`, preserving key order.
    pub fn read(&self, rel: &str) -> Result<Value> {
        let p = self.file(rel);
        let text = std::fs::read_to_string(&p)?;
        Ok(serde_json::from_str(&text)?)
    }

    /// Reads a JSON file, returning `None` if it is missing.
    pub fn read_opt(&self, rel: &str) -> Result<Option<Value>> {
        if self.exists(rel) {
            Ok(Some(self.read(rel)?))
        } else {
            Ok(None)
        }
    }

    /// Writes a `Value` using the game's 4-space pretty formatting.
    pub fn write(&self, rel: &str, value: &Value) -> Result<()> {
        let p = self.file(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, to_pretty(value)?)?;
        Ok(())
    }
}

/// Serializes a value with 4-space indentation, matching the game's saves.
pub fn to_pretty(value: &Value) -> Result<String> {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(buf).map_err(|e| SaveError::Msg(e.to_string()))?)
}

/// Reads the `GameVersion` of a save, falling back to a recent default.
pub fn game_version(path: &Path) -> String {
    let game = path.join("Game.json");
    if let Ok(text) = std::fs::read_to_string(&game) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(s) = v.get("GameVersion").and_then(|x| x.as_str()) {
                return s.to_string();
            }
        }
    }
    "0.4.5f2".to_string()
}
