use super::manager::SaveManager;
use super::{Result, SaveError};
use serde_json::json;

impl SaveManager {
    pub fn set_org_name(&self, name: &str) -> Result<()> {
        self.backups().backup_files("Game", &["Game.json"])?;
        let mut game = self
            .read_opt("Game.json")?
            .ok_or_else(|| SaveError::Msg("Game.json not found".into()))?;
        game["OrganisationName"] = json!(name);
        self.write("Game.json", &game)?;
        Ok(())
    }

    pub fn set_console_enabled(&self, enabled: bool) -> Result<()> {
        self.backups().backup_files("Game", &["Game.json"])?;
        let mut game = self
            .read_opt("Game.json")?
            .ok_or_else(|| SaveError::Msg("Game.json not found".into()))?;
        if !game.get("Settings").map(|s| s.is_object()).unwrap_or(false) {
            game["Settings"] = json!({});
        }
        game["Settings"]["ConsoleEnabled"] = json!(enabled);
        self.write("Game.json", &game)?;
        Ok(())
    }

    /// Reads the current console-enabled setting.
    pub fn console_enabled(&self) -> bool {
        self.read_opt("Game.json")
            .ok()
            .flatten()
            .and_then(|g| {
                g.get("Settings")
                    .and_then(|s| s.get("ConsoleEnabled"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(false)
    }

    /// Map-region display names, indexed by region id.
    pub const REGION_NAMES: [&'static str; 9] = [
        "Northtown",
        "Westville",
        "Downtown",
        "Docks",
        "Suburbia",
        "Uptown",
        "Region 7",
        "Region 8",
        "Region 9",
    ];

    /// Currently unlocked region ids.
    pub fn unlocked_regions(&self) -> Vec<i64> {
        self.read_opt("Rank.json")
            .ok()
            .flatten()
            .and_then(|r| {
                r.get("UnlockedRegions").and_then(|v| {
                    v.as_array()
                        .map(|a| a.iter().filter_map(|x| x.as_i64()).collect())
                })
            })
            .unwrap_or_default()
    }

    /// Toggles a single map region on/off.
    pub fn set_region(&self, region: i64, on: bool) -> Result<()> {
        self.backups().backup_files("Rank", &["Rank.json"])?;
        let mut rank = self
            .read_opt("Rank.json")?
            .ok_or_else(|| SaveError::Msg("Rank.json not found".into()))?;
        let mut regions = self.unlocked_regions();
        regions.retain(|r| *r != region);
        if on {
            regions.push(region);
        }
        regions.sort_unstable();
        rank["UnlockedRegions"] = json!(regions);
        self.write("Rank.json", &rank)?;
        Ok(())
    }

    /// Unlocks all map regions in `Rank.json` (used by tests and bulk paths).
    #[allow(dead_code)]
    pub fn unlock_regions(&self) -> Result<()> {
        self.backups().backup_files("Rank", &["Rank.json"])?;
        let mut rank = self
            .read_opt("Rank.json")?
            .ok_or_else(|| SaveError::Msg("Rank.json not found".into()))?;
        rank["UnlockedRegions"] = json!([0, 1, 2, 3, 4, 5, 6, 7, 8]);
        self.write("Rank.json", &rank)?;
        Ok(())
    }
}
