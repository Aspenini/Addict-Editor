use super::manager::{game_version, SaveManager};
use super::nested::{dump_inner, edit_string_items, parse_inner};
use super::{templates, Result};
use serde_json::{json, Value};

pub const QUALITIES: [&str; 5] = ["Trash", "Poor", "Standard", "Premium", "Heavenly"];
pub const PACKAGINGS: [&str; 3] = ["none", "baggie", "jar"];
pub const FILL_TYPES: [&str; 3] = ["both", "weed", "item"];

impl SaveManager {
    fn json_files_in(&self, folder: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(self.file(folder)) {
            for e in rd.filter_map(|e| e.ok()) {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("json") {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        out.push(format!("{folder}/{name}"));
                    }
                }
            }
        }
        out
    }

    /// Sets `IsOwned` and enables all switch/toggle states for every file in a
    /// folder, then creates any missing entries from the bundled templates.
    #[allow(dead_code)]
    fn own_all_in(
        &self,
        folder: &str,
        defs: &[(&str, &str)],
        data_type: &str,
    ) -> Result<usize> {
        let existing = self.json_files_in(folder);
        let refs: Vec<&str> = existing.iter().map(|s| s.as_str()).collect();
        self.backups().backup_files(folder, &refs)?;

        let mut count = 0;
        for rel in &existing {
            let mut data = self.read(rel)?;
            data["IsOwned"] = json!(true);
            enable_all_bools(&mut data, "SwitchStates");
            enable_all_bools(&mut data, "ToggleableStates");
            self.write(rel, &data)?;
            count += 1;
        }

        // Add any missing known entries.
        let gv = game_version(&self.path);
        let present: Vec<String> = existing
            .iter()
            .filter_map(|r| r.rsplit('/').next().map(|s| s.to_string()))
            .collect();
        for (name, code) in defs {
            let file = format!("{name}.json");
            if present.iter().any(|p| p == &file) {
                continue;
            }
            let rel = format!("{folder}/{file}");
            let template = json!({
                "DataType": data_type,
                "DataVersion": 0,
                "GameVersion": gv,
                "PropertyCode": code,
                "IsOwned": true,
                "SwitchStates": [true, true, true, true],
                "ToggleableStates": [],
                "Employees": [],
                "Objects": []
            });
            self.write(&rel, &template)?;
            count += 1;
        }
        Ok(count)
    }

    #[allow(dead_code)]
    pub fn own_all_properties(&self) -> Result<usize> {
        self.own_all_in("Properties", templates::PROPERTIES, "PropertyData")
    }

    #[allow(dead_code)]
    pub fn own_all_businesses(&self) -> Result<usize> {
        self.own_all_in("Businesses", templates::BUSINESSES, "BusinessData")
    }

    /// Lists ownership state for a folder, merging existing files with the
    /// bundled known set. Returns (display name, owned).
    pub fn list_ownership(&self, folder: &str, defs: &[(&str, &str)]) -> Vec<(String, bool)> {
        let mut rows: Vec<(String, bool)> = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        for rel in self.json_files_in(folder) {
            let stem = rel
                .rsplit('/')
                .next()
                .and_then(|f| f.strip_suffix(".json"))
                .unwrap_or("")
                .to_string();
            if stem.is_empty() {
                continue;
            }
            let owned = self
                .read(&rel)
                .ok()
                .and_then(|d| d.get("IsOwned").and_then(|v| v.as_bool()))
                .unwrap_or(false);
            seen.push(stem.clone());
            rows.push((stem, owned));
        }

        for (name, _code) in defs {
            if !seen.iter().any(|s| s == name) {
                rows.push((name.to_string(), false));
            }
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }

    /// Sets ownership for a single entry, creating it from a template if the
    /// file does not yet exist.
    pub fn set_ownership(
        &self,
        folder: &str,
        name: &str,
        owned: bool,
        defs: &[(&str, &str)],
        data_type: &str,
    ) -> Result<()> {
        let rel = format!("{folder}/{name}.json");
        if self.exists(&rel) {
            self.backups().backup_files(folder, &[rel.as_str()])?;
            let mut data = self.read(&rel)?;
            data["IsOwned"] = json!(owned);
            if owned {
                enable_all_bools(&mut data, "SwitchStates");
                enable_all_bools(&mut data, "ToggleableStates");
            }
            self.write(&rel, &data)?;
        } else if owned {
            let code = defs
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, c)| c.to_string())
                .unwrap_or_else(|| name.to_lowercase().replace(' ', ""));
            let template = json!({
                "DataType": data_type,
                "DataVersion": 0,
                "GameVersion": game_version(&self.path),
                "PropertyCode": code,
                "IsOwned": true,
                "SwitchStates": [true, true, true, true],
                "ToggleableStates": [],
                "Employees": [],
                "Objects": []
            });
            self.write(&rel, &template)?;
        }
        Ok(())
    }

    /// Bulk-fills storage slots inside owned properties.
    /// `fill_type`: 0 both, 1 weed only, 2 item only.
    pub fn fill_storage(
        &self,
        quantity: i64,
        fill_type: usize,
        quality: &str,
        packaging: &str,
    ) -> Result<usize> {
        let files = self.json_files_in("Properties");
        let refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
        self.backups().backup_files("Properties", &refs)?;

        let mut total = 0;
        for rel in &files {
            let mut data = self.read(rel)?;
            let mut file_changed = false;

            if let Some(objects) = data.get_mut("Objects").and_then(|v| v.as_array_mut()) {
                for obj in objects.iter_mut() {
                    let Some(base_str) = obj.get("BaseData").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Ok(mut base) = parse_inner(base_str) else {
                        continue;
                    };
                    let mut obj_changed = false;

                    if let Some(items) = base.pointer_mut("/Contents/Items") {
                        let changed = edit_string_items(items, |item| {
                            apply_fill(item, quantity, fill_type, quality, packaging)
                        })?;
                        if changed > 0 {
                            obj_changed = true;
                            total += changed;
                        }
                    }

                    if obj_changed {
                        obj["BaseData"] = Value::String(dump_inner(&base)?);
                        file_changed = true;
                    }
                }
            }

            if file_changed {
                self.write(rel, &data)?;
            }
        }
        Ok(total)
    }
}

fn apply_fill(
    item: &mut Value,
    quantity: i64,
    fill_type: usize,
    quality: &str,
    packaging: &str,
) -> bool {
    let dt = item.get("DataType").and_then(|v| v.as_str()).unwrap_or("");
    let id_empty = item
        .get("ID")
        .and_then(|v| v.as_str())
        .map(|s| s.is_empty())
        .unwrap_or(true);
    if id_empty {
        return false; // never fill empty slots
    }
    let is_weed = dt == "WeedData";
    let is_item = dt == "ItemData" || dt == "IntegerItemData";
    let modify = match fill_type {
        1 => is_weed,
        2 => is_item,
        _ => is_weed || is_item,
    };
    if !modify {
        return false;
    }
    item["Quantity"] = json!(quantity);
    if is_weed {
        item["Quality"] = json!(quality);
        if packaging != "none" {
            item["PackagingID"] = json!(packaging);
        }
    }
    true
}

fn enable_all_bools(data: &mut Value, key: &str) {
    if let Some(arr) = data.get_mut(key).and_then(|v| v.as_array_mut()) {
        for el in arr.iter_mut() {
            *el = json!(true);
        }
    }
}
