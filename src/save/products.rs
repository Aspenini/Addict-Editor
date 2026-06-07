use super::manager::{game_version, SaveManager};
use super::{templates, Result, SaveError};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_json::{json, Value};
use std::collections::HashSet;

const REL: &str = "Products.json";

const PROPERTY_POOL: &[&str] = &[
    "athletic", "balding", "gingeritis", "spicy", "jennerising", "thoughtprovoking",
    "tropicthunder", "giraffying", "longfaced", "sedating", "smelly", "paranoia", "laxative",
    "caloriedense", "energizing",
];

const INGREDIENTS: &[&str] = &[
    "flumedicine", "gasoline", "mouthwash", "horsesemen", "iodine", "chili", "paracetamol",
    "energydrink", "donut", "banana", "viagra", "cuke", "motoroil",
];

impl SaveManager {
    fn load_products(&self) -> Result<Value> {
        if let Some(v) = self.read_opt(REL)? {
            Ok(v)
        } else {
            Ok(json!({
                "DataType": "ProductManagerData",
                "DataVersion": 0,
                "GameVersion": game_version(&self.path),
                "DiscoveredProducts": [],
                "ListedProducts": [],
                "ActiveMixOperation": { "ProductID": "", "IngredientID": "" },
                "IsMixComplete": false,
                "MixRecipes": [],
                "ProductPrices": []
            }))
        }
    }

    fn push_unique(arr: &mut Vec<Value>, id: &str) -> bool {
        if arr.iter().any(|v| v.as_str() == Some(id)) {
            false
        } else {
            arr.push(json!(id));
            true
        }
    }

    /// Adds the base discoverable products.
    pub fn discover_all_products(&self) -> Result<usize> {
        self.backups().backup_files("Products", &[REL])?;
        let mut data = self.load_products()?;
        let mut count = 0;
        {
            let discovered = data
                .get_mut("DiscoveredProducts")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| SaveError::Msg("malformed Products.json".into()))?;
            for id in templates::BASE_PRODUCTS {
                if Self::push_unique(discovered, id) {
                    count += 1;
                }
            }
        }
        self.write(REL, &data)?;
        Ok(count)
    }

    /// Generates `count` random custom products with prices and mix recipes.
    pub fn generate_products(
        &self,
        count: usize,
        id_length: usize,
        price: i64,
        add_to_listed: bool,
    ) -> Result<usize> {
        self.backups().backup_files("Products", &[REL])?;
        let gv = game_version(&self.path);
        let mut data = self.load_products()?;
        let mut rng = rand::thread_rng();

        let mut existing: HashSet<String> = data
            .get("DiscoveredProducts")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let mut new_ids = Vec::new();
        for _ in 0..count {
            let mut id = random_id(&mut rng, id_length.max(1));
            while existing.contains(&id) {
                id = random_id(&mut rng, id_length.max(1));
            }
            existing.insert(id.clone());
            new_ids.push(id);
        }

        // Discovered
        if let Some(arr) = data.get_mut("DiscoveredProducts").and_then(|v| v.as_array_mut()) {
            for id in &new_ids {
                arr.push(json!(id));
            }
        }
        // Prices
        if let Some(arr) = data.get_mut("ProductPrices").and_then(|v| v.as_array_mut()) {
            for id in &new_ids {
                arr.push(json!({ "String": id, "Int": price }));
            }
        }
        // Mix recipes
        if let Some(arr) = data.get_mut("MixRecipes").and_then(|v| v.as_array_mut()) {
            for id in &new_ids {
                let mixer = INGREDIENTS.choose(&mut rng).copied().unwrap_or("donut");
                arr.push(json!({ "Product": "ogkush", "Mixer": mixer, "Output": id }));
            }
        }
        // Listed
        if add_to_listed {
            if let Some(arr) = data.get_mut("ListedProducts").and_then(|v| v.as_array_mut()) {
                for id in &new_ids {
                    arr.push(json!(id));
                }
            }
        }

        self.write(REL, &data)?;

        // Product definition files.
        for id in &new_ids {
            let props: Vec<&&str> = PROPERTY_POOL.choose_multiple(&mut rng, 8).collect();
            let def = json!({
                "DataType": "WeedProductData",
                "DataVersion": 0,
                "GameVersion": gv,
                "Name": id,
                "ID": id,
                "DrugType": 0,
                "Properties": props,
                "AppearanceSettings": {
                    "MainColor": rand_color(&mut rng),
                    "SecondaryColor": rand_color(&mut rng),
                    "LeafColor": rand_color(&mut rng),
                    "StemColor": rand_color(&mut rng)
                }
            });
            self.write(&format!("CreatedProducts/{id}.json"), &def)?;
        }

        Ok(new_ids.len())
    }

    /// Removes everything created by `generate_products`.
    pub fn delete_generated(&self) -> Result<usize> {
        let dir = self.file("CreatedProducts");
        if !dir.exists() {
            return Ok(0);
        }
        self.backups().backup_files("Products", &[REL, "CreatedProducts"])?;

        let ids: HashSet<String> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("json") {
                    p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

        if let Some(mut data) = self.read_opt(REL)? {
            retain_not_in(&mut data, "DiscoveredProducts", &ids, |v| v.as_str());
            retain_not_in(&mut data, "ListedProducts", &ids, |v| v.as_str());
            retain_not_in(&mut data, "ProductPrices", &ids, |v| {
                v.get("String").and_then(|s| s.as_str())
            });
            retain_not_in(&mut data, "MixRecipes", &ids, |v| {
                v.get("Output").and_then(|s| s.as_str())
            });
            self.write(REL, &data)?;
        }

        std::fs::remove_dir_all(&dir)?;
        Ok(ids.len())
    }
}

fn random_id(rng: &mut impl Rng, length: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    (0..length)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

fn rand_color(rng: &mut impl Rng) -> Value {
    json!({
        "r": rng.gen_range(0..=255),
        "g": rng.gen_range(0..=255),
        "b": rng.gen_range(0..=255),
        "a": 255
    })
}

fn retain_not_in<F>(data: &mut Value, key: &str, ids: &HashSet<String>, extract: F)
where
    F: Fn(&Value) -> Option<&str>,
{
    if let Some(arr) = data.get_mut(key).and_then(|v| v.as_array_mut()) {
        arr.retain(|v| match extract(v) {
            Some(s) => !ids.contains(s),
            None => true,
        });
    }
}
