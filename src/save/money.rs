use super::manager::SaveManager;
use super::nested::edit_string_items;
use super::{Result, SaveError};
use serde_json::{json, Value};
use std::path::PathBuf;

impl SaveManager {
    /// Lists `Players/Player_*` directories.
    pub fn player_dirs(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let players = self.file("Players");
        if let Ok(rd) = std::fs::read_dir(&players) {
            for e in rd.filter_map(|e| e.ok()) {
                let p = e.path();
                if p.is_dir()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("Player_"))
                        .unwrap_or(false)
                {
                    out.push(p);
                }
            }
        }
        out.sort();
        out
    }

    fn player_inventory_rels(&self) -> Vec<String> {
        self.player_dirs()
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n.to_string()))
            .map(|name| format!("Players/{name}/Inventory.json"))
            .filter(|rel| self.exists(rel))
            .collect()
    }

    /// Largest on-hand cash balance found across players (for display).
    pub fn player_cash(&self) -> Option<f64> {
        let mut best: Option<f64> = None;
        for rel in self.player_inventory_rels() {
            if let Ok(inv) = self.read(&rel) {
                if let Some(items) = inv.get("Items").and_then(|v| v.as_array()) {
                    for s in items {
                        if let Some(text) = s.as_str() {
                            if let Ok(item) = serde_json::from_str::<Value>(text) {
                                if item.get("DataType").and_then(|v| v.as_str()) == Some("CashData")
                                {
                                    let bal =
                                        item.get("CashBalance").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    best = Some(best.map_or(bal, |b| b.max(bal)));
                                }
                            }
                        }
                    }
                }
            }
        }
        best
    }

    /// Sets the `CashData` balance in every player's inventory.
    pub fn set_player_cash(&self, value: f64) -> Result<usize> {
        let rels = self.player_inventory_rels();
        let rel_refs: Vec<&str> = rels.iter().map(|s| s.as_str()).collect();
        self.backups().backup_files("PlayerCash", &rel_refs)?;
        let mut count = 0;
        for rel in &rels {
            let mut inv = self.read(rel)?;
            if let Some(items) = inv.get_mut("Items") {
                let changed = edit_string_items(items, |item| {
                    if item.get("DataType").and_then(|v| v.as_str()) == Some("CashData") {
                        item["CashBalance"] = json!(value);
                        true
                    } else {
                        false
                    }
                })?;
                if changed > 0 {
                    self.write(rel, &inv)?;
                    count += changed;
                }
            }
        }
        Ok(count)
    }

    /// Updates the bank `Money.json` fields. Empty strings are skipped.
    pub fn set_money(
        &self,
        online: Option<f64>,
        networth: Option<f64>,
        lifetime: Option<f64>,
        weekly: Option<f64>,
    ) -> Result<()> {
        self.backups().backup_files("Money", &["Money.json"])?;
        let mut money = self
            .read_opt("Money.json")?
            .ok_or_else(|| SaveError::Msg("Money.json not found".into()))?;
        if let Some(v) = online {
            money["OnlineBalance"] = json!(v);
        }
        if let Some(v) = networth {
            money["Networth"] = json!(v);
        }
        if let Some(v) = lifetime {
            money["LifetimeEarnings"] = json!(v);
        }
        if let Some(v) = weekly {
            money["WeeklyDepositSum"] = json!(v);
        }
        self.write("Money.json", &money)?;
        Ok(())
    }
}
