use super::manager::SaveManager;
use super::{Result, SaveError};
use serde_json::json;

pub const RANK_NAMES: [&str; 10] = [
    "Street Rat",
    "Hoodlum",
    "Peddler",
    "Hustler",
    "Bagman",
    "Enforcer",
    "Shot Caller",
    "Block Boss",
    "Underlord",
    "Baron",
];

/// XP required to advance through each of the 50 tiers (10 ranks x 5 tiers).
pub const XP_PER_TIER: [i64; 50] = [
    0, 200, 200, 200, 200, 400, 400, 400, 400, 400, 625, 625, 625, 625, 625, 825, 825, 825, 825,
    825, 1025, 1025, 1025, 1025, 1025, 1050, 1050, 1050, 1050, 1050, 1450, 1450, 1450, 1450, 1450,
    1675, 1675, 1675, 1675, 1675, 1875, 1875, 1875, 1875, 1875, 2075, 2075, 2075, 2075, 2075,
];

/// Builds the dropdown labels: "Street Rat 1" .. "Baron 5".
pub fn preset_labels() -> Vec<String> {
    let mut out = Vec::new();
    for name in RANK_NAMES.iter() {
        for tier in 1..=5 {
            out.push(format!("{name} {tier}"));
        }
    }
    out
}

impl SaveManager {
    fn write_rank(&self, rank: i64, tier: i64, xp: i64, total_xp: i64) -> Result<()> {
        self.backups().backup_files("Rank", &["Rank.json"])?;
        let mut data = self
            .read_opt("Rank.json")?
            .ok_or_else(|| SaveError::Msg("Rank.json not found".into()))?;
        data["Rank"] = json!(rank);
        data["Tier"] = json!(tier);
        data["XP"] = json!(xp);
        data["TotalXP"] = json!(total_xp);
        self.write("Rank.json", &data)?;
        Ok(())
    }

    /// Applies a preset by combined index (rank_index * 5 + (tier-1)).
    pub fn apply_rank_preset(&self, index: usize) -> Result<()> {
        let index = index.min(49);
        let rank_index = (index / 5) as i64;
        let tier = (index % 5) as i64 + 1;
        let total_xp: i64 = XP_PER_TIER[..=index].iter().sum();
        self.write_rank(rank_index, tier, 0, total_xp)
    }

    pub fn apply_rank_manual(&self, rank: i64, tier: i64, xp: i64) -> Result<()> {
        // Best-effort TotalXP: cumulative up to this rank/tier plus current xp.
        let idx = ((rank * 5 + (tier - 1)).max(0) as usize).min(49);
        let total: i64 = XP_PER_TIER[..=idx].iter().sum::<i64>() + xp;
        self.write_rank(rank, tier, xp, total)
    }

    /// Max rank/tier to unlock all items and weeds.
    #[allow(dead_code)]
    pub fn max_rank(&self) -> Result<()> {
        self.write_rank(999, 999, 0, 0)
    }
}
