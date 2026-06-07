use super::manager::{game_version, to_pretty, SaveManager};
use super::nested::{dump_inner, parse_inner};
use super::{templates, Result};
use serde_json::{json, Value};
use std::collections::HashSet;

const REL: &str = "NPCs.json";

fn base_id(npc: &Value) -> Option<String> {
    let base = npc.get("BaseData").and_then(|v| v.as_str())?;
    let parsed = parse_inner(base).ok()?;
    parsed
        .get("ID")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

impl SaveManager {
    fn load_npcs(&self) -> Result<Value> {
        if let Some(v) = self.read_opt(REL)? {
            Ok(v)
        } else {
            Ok(json!({
                "DataType": "NPCCollectionData",
                "DataVersion": 0,
                "GameVersion": game_version(&self.path),
                "NPCs": []
            }))
        }
    }

    /// Maxes every NPC's Relationship (delta 999, unlocked).
    #[allow(dead_code)]
    pub fn max_relationships(&self) -> Result<usize> {
        self.set_all_relationships(999.0)
    }

    /// Sets every NPC's Relationship delta to `value` and unlocks them.
    pub fn set_all_relationships(&self, value: f64) -> Result<usize> {
        self.backups().backup_files("NPCs", &[REL])?;
        let mut data = self.load_npcs()?;
        let mut count = 0;
        if let Some(npcs) = data.get_mut("NPCs").and_then(|v| v.as_array_mut()) {
            for npc in npcs.iter_mut() {
                if let Some(addl) = npc.get_mut("AdditionalDatas").and_then(|v| v.as_array_mut()) {
                    for entry in addl.iter_mut() {
                        if entry.get("Name").and_then(|v| v.as_str()) == Some("Relationship") {
                            if let Some(contents) = entry.get("Contents").and_then(|v| v.as_str()) {
                                if let Ok(mut rel) = parse_inner(contents) {
                                    rel["RelationDelta"] = json!(value);
                                    rel["Unlocked"] = json!(true);
                                    rel["UnlockType"] = json!(1);
                                    entry["Contents"] = Value::String(to_pretty(&rel)?);
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        self.write(REL, &data)?;
        Ok(count)
    }

    /// Sets `Recruited` true on dealer NPCs.
    pub fn recruit_dealers(&self) -> Result<usize> {
        self.backups().backup_files("NPCs", &[REL])?;
        let mut data = self.load_npcs()?;
        let mut count = 0;
        if let Some(npcs) = data.get_mut("NPCs").and_then(|v| v.as_array_mut()) {
            for npc in npcs.iter_mut() {
                let Some(base_str) = npc.get("BaseData").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Ok(mut base) = parse_inner(base_str) else {
                    continue;
                };
                let id = base.get("ID").and_then(|v| v.as_str()).unwrap_or("");
                let is_dealer = base.get("DataType").and_then(|v| v.as_str()) == Some("DealerData")
                    || base.get("Recruited").is_some()
                    || templates::DEALER_IDS.contains(&id);
                if is_dealer {
                    base["Recruited"] = json!(true);
                    npc["BaseData"] = Value::String(dump_inner(&base)?);
                    count += 1;
                }
            }
        }
        self.write(REL, &data)?;
        Ok(count)
    }

    /// Appends any bundled NPC ids that are missing, unlocked and maxed.
    pub fn add_missing_npcs(&self) -> Result<usize> {
        self.backups().backup_files("NPCs", &[REL])?;
        let gv = game_version(&self.path);
        let mut data = self.load_npcs()?;
        let existing: HashSet<String> = data
            .get("NPCs")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(base_id).collect())
            .unwrap_or_default();

        let mut added = 0;
        if let Some(npcs) = data.get_mut("NPCs").and_then(|v| v.as_array_mut()) {
            for id in templates::NPC_IDS {
                if existing.contains(*id) {
                    continue;
                }
                let base = json!({
                    "DataType": "NPCData",
                    "DataVersion": 0,
                    "GameVersion": gv,
                    "ID": id
                });
                let relationship = json!({
                    "DataType": "RelationshipData",
                    "DataVersion": 0,
                    "GameVersion": gv,
                    "RelationDelta": 999.0,
                    "Unlocked": true,
                    "UnlockType": 1
                });
                npcs.push(json!({
                    "DataType": "NPCData",
                    "DataVersion": 0,
                    "GameVersion": gv,
                    "BaseData": dump_inner(&base)?,
                    "AdditionalDatas": [
                        { "Name": "Relationship", "Contents": to_pretty(&relationship)? }
                    ]
                }));
                added += 1;
            }
        }
        self.write(REL, &data)?;
        Ok(added)
    }
}
