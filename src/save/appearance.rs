use super::manager::SaveManager;
use super::Result;
use serde_json::{json, Value};

pub const APPEARANCE_PRESETS: [&str; 4] = ["None", "Walter White", "Clean Shaven", "Bald"];

impl SaveManager {
    fn appearance_files(&self) -> Vec<String> {
        self.player_dirs()
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n.to_string()))
            .map(|name| format!("Players/{name}/Appearance.json"))
            .filter(|rel| self.exists(rel))
            .collect()
    }

    /// Applies a cosmetic preset to every player's `Appearance.json`.
    pub fn apply_appearance(&self, index: usize) -> Result<usize> {
        if index == 0 {
            return Ok(0); // "None"
        }
        let rels = self.appearance_files();
        let refs: Vec<&str> = rels.iter().map(|s| s.as_str()).collect();
        if !refs.is_empty() {
            self.backups().backup_files("Appearance", &refs)?;
        }

        let mut count = 0;
        for rel in &rels {
            let mut data = self.read(rel)?;
            apply_preset(&mut data, index);
            self.write(rel, &data)?;
            count += 1;
        }
        Ok(count)
    }
}

fn apply_preset(data: &mut Value, index: usize) {
    match index {
        1 => {
            // "Walter White": bald + goatee, pale skin.
            data["HairStyle"] = json!("");
            data["FacialHair"] = json!("Avatar/Layers/Face/FacialHair_Goatee");
            data["SkinColor"] = json!({ "r": 0.9, "g": 0.78, "b": 0.7, "a": 1.0 });
        }
        2 => {
            // "Clean Shaven"
            data["FacialHair"] = json!("");
        }
        3 => {
            // "Bald"
            data["HairStyle"] = json!("");
        }
        _ => {}
    }
}
