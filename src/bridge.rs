//! Helpers to convert between the save layer and the Slint UI model types.

use crate::save::backup::BackupRecord;
use crate::save::SaveInfoEntry;
use crate::{BackupItem, RegionRow, SaveEntry, ToggleRow};
use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

pub fn strings_model(items: &[&str]) -> ModelRc<SharedString> {
    let v: Vec<SharedString> = items.iter().map(|s| SharedString::from(*s)).collect();
    ModelRc::from(Rc::new(VecModel::from(v)))
}

pub fn saves_model(entries: &[SaveInfoEntry]) -> ModelRc<SaveEntry> {
    let v: Vec<SaveEntry> = entries
        .iter()
        .map(|e| SaveEntry {
            name: e.name.clone().into(),
            path: e.path.to_string_lossy().to_string().into(),
            org: e.org.clone().into(),
            version: e.version.clone().into(),
            played: e.played.clone().into(),
        })
        .collect();
    ModelRc::from(Rc::new(VecModel::from(v)))
}

pub fn toggles_model(rows: &[(String, bool)]) -> ModelRc<ToggleRow> {
    let v: Vec<ToggleRow> = rows
        .iter()
        .map(|(name, owned)| ToggleRow {
            name: name.clone().into(),
            owned: *owned,
        })
        .collect();
    ModelRc::from(Rc::new(VecModel::from(v)))
}

pub fn regions_model(unlocked: &[i64], names: &[&str]) -> ModelRc<RegionRow> {
    let v: Vec<RegionRow> = names
        .iter()
        .enumerate()
        .map(|(i, name)| RegionRow {
            index: i as i32,
            name: (*name).into(),
            unlocked: unlocked.contains(&(i as i64)),
        })
        .collect();
    ModelRc::from(Rc::new(VecModel::from(v)))
}

pub fn backups_model(entries: &[BackupRecord]) -> ModelRc<BackupItem> {
    let v: Vec<BackupItem> = entries
        .iter()
        .map(|e| BackupItem {
            feature: e.feature.clone().into(),
            timestamp: e.timestamp.clone().into(),
            label: e.label.clone().into(),
        })
        .collect();
    ModelRc::from(Rc::new(VecModel::from(v)))
}

/// Parses a numeric text field; empty/invalid -> None.
pub fn parse_f64(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        t.parse::<f64>().ok()
    }
}