use super::manager::{to_pretty, SaveManager};
use super::nested::edit_string_items;
use super::Result;
use serde_json::{json, Value};

const NPCS: &str = "NPCs.json";

impl SaveManager {
    /// Sets the cash balance on any dealer inventory stored in `NPCs.json`.
    ///
    /// Walks each NPC's `AdditionalDatas`; for entries whose `Contents` decode
    /// to something holding an `Items` list (an inventory), every `CashData`
    /// slot has its balance set. Safe no-op when no such data exists.
    pub fn set_all_dealer_cash(&self, value: f64) -> Result<usize> {
        let Some(mut data) = self.read_opt(NPCS)? else {
            return Ok(0);
        };
        self.backups().backup_files("DealerCash", &[NPCS])?;

        let mut count = 0;
        if let Some(npcs) = data.get_mut("NPCs").and_then(|v| v.as_array_mut()) {
            for npc in npcs.iter_mut() {
                let Some(addl) = npc.get_mut("AdditionalDatas").and_then(|v| v.as_array_mut())
                else {
                    continue;
                };
                for entry in addl.iter_mut() {
                    let Some(contents) = entry.get("Contents").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Ok(mut decoded) = serde_json::from_str::<Value>(contents) else {
                        continue;
                    };
                    let mut changed = false;
                    if let Some(items) = decoded.get_mut("Items") {
                        let n = edit_string_items(items, |item| set_cash(item, value))?;
                        if n > 0 {
                            changed = true;
                            count += n;
                        }
                    }
                    if changed {
                        entry["Contents"] = Value::String(to_pretty(&decoded)?);
                    }
                }
            }
        }
        self.write(NPCS, &data)?;
        Ok(count)
    }
}

fn set_cash(item: &mut Value, value: f64) -> bool {
    if item.get("DataType").and_then(|v| v.as_str()) == Some("CashData") {
        item["CashBalance"] = json!(value);
        true
    } else {
        false
    }
}
