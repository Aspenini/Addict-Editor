use super::manager::SaveManager;
use super::Result;

/// Summary information shown on the Save Info panel.
#[derive(Debug, Clone, Default)]
pub struct SaveSummary {
    pub game_version: String,
    pub created: String,
    pub org: String,
    pub online_money: String,
    pub networth: String,
    pub rank: String,
}

/// Editable money values (as strings for the UI).
#[derive(Debug, Clone, Default)]
pub struct MoneyValues {
    pub online: String,
    pub networth: String,
    pub lifetime: String,
    pub weekly: String,
    pub cash: String,
}

fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl SaveManager {
    pub fn summary(&self) -> Result<SaveSummary> {
        let game = self.read_opt("Game.json")?.unwrap_or_default();
        let money = self.read_opt("Money.json")?.unwrap_or_default();
        let rank = self.read_opt("Rank.json")?.unwrap_or_default();
        let meta = self.read_opt("Metadata.json")?.unwrap_or_default();

        let created = meta
            .get("CreationDate")
            .map(|c| {
                let g = |k: &str| c.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    g("Year"),
                    g("Month"),
                    g("Day"),
                    g("Hour"),
                    g("Minute"),
                    g("Second")
                )
            })
            .unwrap_or_else(|| "Unknown".into());

        Ok(SaveSummary {
            game_version: game
                .get("GameVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            created,
            org: game
                .get("OrganisationName")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            online_money: fmt_num(money.get("OnlineBalance").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            networth: fmt_num(money.get("Networth").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            rank: format!(
                "Rank {} / Tier {}",
                rank.get("Rank").and_then(|v| v.as_i64()).unwrap_or(0),
                rank.get("Tier").and_then(|v| v.as_i64()).unwrap_or(0)
            ),
        })
    }

    pub fn money_values(&self) -> Result<MoneyValues> {
        let money = self.read_opt("Money.json")?.unwrap_or_default();
        Ok(MoneyValues {
            online: fmt_num(money.get("OnlineBalance").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            networth: fmt_num(money.get("Networth").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            lifetime: fmt_num(money.get("LifetimeEarnings").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            weekly: fmt_num(money.get("WeeklyDepositSum").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            cash: fmt_num(self.player_cash().unwrap_or(0.0)),
        })
    }
}
