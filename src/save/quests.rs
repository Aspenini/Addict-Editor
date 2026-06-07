use super::manager::SaveManager;
use super::Result;
use serde_json::json;

const REL: &str = "Quests.json";

impl SaveManager {
    /// Marks every quest and objective as completed (State = 2).
    /// Returns (quests_completed, objectives_completed).
    pub fn complete_quests(&self) -> Result<(usize, usize)> {
        let Some(mut data) = self.read_opt(REL)? else {
            return Ok((0, 0));
        };
        self.backups().backup_files("Quests", &[REL])?;

        let mut quests = 0;
        let mut objectives = 0;
        if let Some(arr) = data.get_mut("Quests").and_then(|v| v.as_array_mut()) {
            for quest in arr.iter_mut() {
                let state = quest.get("State").and_then(|v| v.as_i64()).unwrap_or(2);
                if state == 0 || state == 1 {
                    quest["State"] = json!(2);
                    quests += 1;
                }
                if let Some(entries) = quest.get_mut("Entries").and_then(|v| v.as_array_mut()) {
                    for entry in entries.iter_mut() {
                        let es = entry.get("State").and_then(|v| v.as_i64()).unwrap_or(2);
                        if es == 0 || es == 1 {
                            entry["State"] = json!(2);
                            objectives += 1;
                        }
                    }
                }
            }
        }
        self.write(REL, &data)?;
        Ok((quests, objectives))
    }
}
