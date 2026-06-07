use super::manager::SaveManager;
use super::Result;
use serde_json::{json, Value};

const BIG_VALUE: &str = "999999999";

impl SaveManager {
    /// Collects every `VariableCollectionData` file (root + per-player).
    fn variable_files(&self) -> Vec<String> {
        let mut rels = Vec::new();
        if self.exists("Variables.json") {
            rels.push("Variables.json".to_string());
        }
        // Root Variables/ folder (older layout), if present.
        if let Ok(rd) = std::fs::read_dir(self.file("Variables")) {
            for e in rd.filter_map(|e| e.ok()) {
                if e.path().extension().and_then(|x| x.to_str()) == Some("json") {
                    if let Some(n) = e.file_name().to_str() {
                        rels.push(format!("Variables/{n}"));
                    }
                }
            }
        }
        for dir in self.player_dirs() {
            if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                let rel = format!("Players/{name}/Variables.json");
                if self.exists(&rel) {
                    rels.push(rel);
                }
            }
        }
        rels
    }

    /// Flips `False` -> `True` and sets numeric variables to a large value.
    #[allow(dead_code)]
    pub fn flip_variables(&self) -> Result<usize> {
        let rels = self.variable_files();
        let refs: Vec<&str> = rels.iter().map(|s| s.as_str()).collect();
        if !refs.is_empty() {
            self.backups().backup_files("Variables", &refs)?;
        }

        let mut count = 0;
        for rel in &rels {
            let mut data = self.read(rel)?;
            let mut changed = false;

            // Modern: { "Variables": [ { "Name", "Value" }, ... ] }
            if let Some(arr) = data.get_mut("Variables").and_then(|v| v.as_array_mut()) {
                for var in arr.iter_mut() {
                    if apply_variable(var) {
                        changed = true;
                        count += 1;
                    }
                }
            } else if data.get("Value").is_some() {
                // Older: a single { "Value": ... } document.
                if apply_variable(&mut data) {
                    changed = true;
                    count += 1;
                }
            }

            if changed {
                self.write(rel, &data)?;
            }
        }
        Ok(count)
    }
}

fn apply_variable(var: &mut Value) -> bool {
    let Some(value) = var.get("Value").and_then(|v| v.as_str()).map(|s| s.to_string()) else {
        return false;
    };
    match value.as_str() {
        "False" => {
            var["Value"] = json!("True");
            true
        }
        "True" => false,
        _ => {
            if value == BIG_VALUE {
                false
            } else {
                var["Value"] = json!(BIG_VALUE);
                true
            }
        }
    }
}
