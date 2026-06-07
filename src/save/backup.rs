use super::manager::SaveManager;
use super::{Result, SaveError};
use std::path::{Path, PathBuf};

impl SaveManager {
    pub fn backups(&self) -> Backups {
        Backups::new(&self.path)
    }
}

/// Manages automatic backups for a single save folder.
///
/// Layout (sibling of the save folder, so it is never mistaken for a save):
///   <SaveName>_AddictBackups/
///     initial/                      full copy of the save on first edit
///     features/<feature>/<ts>/...   per-feature snapshots of changed files
pub struct Backups {
    save: PathBuf,
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BackupRecord {
    pub feature: String,
    pub timestamp: String,
    pub label: String,
}

impl Backups {
    pub fn new(save: &Path) -> Self {
        let name = save
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Save")
            .to_string();
        let root = save
            .parent()
            .unwrap_or(save)
            .join(format!("{name}_AddictBackups"));
        Self {
            save: save.to_path_buf(),
            root,
        }
    }

    fn initial_dir(&self) -> PathBuf {
        self.root.join("initial")
    }

    fn features_dir(&self) -> PathBuf {
        self.root.join("features")
    }

    /// Copies the whole save folder once, if no initial backup exists yet.
    /// Returns `true` if a fresh backup was created by this call.
    pub fn ensure_initial(&self) -> Result<bool> {
        let initial = self.initial_dir();
        if !initial.exists() {
            copy_dir(&self.save, &initial)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Snapshots the given relative files under a timestamped feature folder.
    pub fn backup_files(&self, feature: &str, rels: &[&str]) -> Result<()> {
        self.ensure_initial()?;
        let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
        let dest = self.features_dir().join(feature).join(&ts);
        for rel in rels {
            let src = self.save.join(rel);
            if src.is_file() {
                let d = dest.join(rel);
                if let Some(parent) = d.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src, &d)?;
            } else if src.is_dir() {
                copy_dir(&src, &dest.join(rel))?;
            }
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<BackupRecord> {
        let mut out = Vec::new();
        let features = self.features_dir();
        if let Ok(rd) = std::fs::read_dir(&features) {
            for feat in rd.filter_map(|e| e.ok()) {
                if !feat.path().is_dir() {
                    continue;
                }
                let feature = feat.file_name().to_string_lossy().to_string();
                if let Ok(stamps) = std::fs::read_dir(feat.path()) {
                    for st in stamps.filter_map(|e| e.ok()) {
                        if !st.path().is_dir() {
                            continue;
                        }
                        let timestamp = st.file_name().to_string_lossy().to_string();
                        out.push(BackupRecord {
                            label: format!("{}  -  {}", feature, pretty_ts(&timestamp)),
                            feature: feature.clone(),
                            timestamp,
                        });
                    }
                }
            }
        }
        out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        out
    }

    pub fn revert_feature(&self, feature: &str, timestamp: &str) -> Result<()> {
        let dir = self.features_dir().join(feature).join(timestamp);
        if !dir.exists() {
            return Err(SaveError::Msg("backup not found".into()));
        }
        restore_tree(&dir, &dir, &self.save)?;
        Ok(())
    }

    pub fn revert_all(&self) -> Result<()> {
        let initial = self.initial_dir();
        if !initial.exists() {
            return Err(SaveError::Msg("no initial backup found".into()));
        }
        remove_dir_contents(&self.save)?;
        copy_dir(&initial, &self.save)?;
        Ok(())
    }

    pub fn delete_all(&self) -> Result<()> {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }
}

fn pretty_ts(ts: &str) -> String {
    // 20260607013344 -> 2026-06-07 01:33:44
    if ts.len() == 14 && ts.chars().all(|c| c.is_ascii_digit()) {
        format!(
            "{}-{}-{} {}:{}:{}",
            &ts[0..4],
            &ts[4..6],
            &ts[6..8],
            &ts[8..10],
            &ts[10..12],
            &ts[12..14]
        )
    } else {
        ts.to_string()
    }
}

/// Recursively copies every file under `base` back to its place in `save`.
fn restore_tree(base: &Path, current: &Path, save: &Path) -> Result<()> {
    for entry in std::fs::read_dir(current)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            restore_tree(base, &path, save)?;
        } else if let Ok(rel) = path.strip_prefix(base) {
            let dest = save.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

pub fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn remove_dir_contents(dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(dir)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}
