//! Integration tests that run the editor operations against a throwaway copy
//! of the bundled reference save (`reference material/SaveGame_1`).

use crate::save::backup::copy_dir;
use crate::save::manager::SaveManager;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn reference_save() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("reference material")
        .join("SaveGame_1")
}

/// Copies the reference save into a unique temp folder and returns a manager.
fn fresh_save() -> (SaveManager, PathBuf) {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let root = std::env::temp_dir().join(format!("addict_test_{pid}_{id}"));
    let _ = std::fs::remove_dir_all(&root);
    let dest = root.join("SaveGame_1");
    copy_dir(&reference_save(), &dest).expect("copy reference save");
    let mgr = SaveManager::open(dest.clone()).expect("open save");
    (mgr, root)
}

fn read(mgr: &SaveManager, rel: &str) -> Value {
    mgr.read(rel).expect("read json")
}

#[test]
fn reference_save_exists() {
    assert!(
        reference_save().join("Game.json").exists(),
        "reference save missing"
    );
}

#[test]
fn money_round_trip() {
    let (mgr, root) = fresh_save();
    mgr.set_money(Some(1_000_000.0), Some(2_000_000.0), None, None)
        .unwrap();
    let money = read(&mgr, "Money.json");
    assert_eq!(money["OnlineBalance"].as_f64(), Some(1_000_000.0));
    assert_eq!(money["Networth"].as_f64(), Some(2_000_000.0));
    // Untouched fields preserved.
    assert!(money["LifetimeEarnings"].as_f64().unwrap() > 0.0);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn player_cash_is_set() {
    let (mgr, root) = fresh_save();
    let n = mgr.set_player_cash(424242.0).unwrap();
    assert!(n >= 1, "expected at least one cash slot updated");
    // Every CashData balance should now be the new value.
    for dir in mgr.player_dirs() {
        let name = dir.file_name().unwrap().to_str().unwrap();
        let rel = format!("Players/{name}/Inventory.json");
        if !mgr.exists(&rel) {
            continue;
        }
        let inv = read(&mgr, &rel);
        for item in inv["Items"].as_array().unwrap() {
            let parsed: Value = serde_json::from_str(item.as_str().unwrap()).unwrap();
            if parsed["DataType"] == "CashData" {
                assert_eq!(parsed["CashBalance"].as_f64(), Some(424242.0));
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn rank_preset_and_max() {
    let (mgr, root) = fresh_save();
    mgr.apply_rank_preset(7).unwrap(); // Hoodlum 3 -> rank 1, tier 3
    let rank = read(&mgr, "Rank.json");
    assert_eq!(rank["Rank"].as_i64(), Some(1));
    assert_eq!(rank["Tier"].as_i64(), Some(3));

    mgr.max_rank().unwrap();
    let rank = read(&mgr, "Rank.json");
    assert_eq!(rank["Rank"].as_i64(), Some(999));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn quests_completed() {
    let (mgr, root) = fresh_save();
    mgr.complete_quests().unwrap();
    let quests = read(&mgr, "Quests.json");
    for q in quests["Quests"].as_array().unwrap() {
        assert_eq!(q["State"].as_i64(), Some(2));
        if let Some(entries) = q["Entries"].as_array() {
            for e in entries {
                assert_eq!(e["State"].as_i64(), Some(2));
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn variables_flipped() {
    let (mgr, root) = fresh_save();
    let n = mgr.flip_variables().unwrap();
    assert!(n > 0);
    // No variable should remain "False".
    for dir in mgr.player_dirs() {
        let name = dir.file_name().unwrap().to_str().unwrap();
        let rel = format!("Players/{name}/Variables.json");
        if !mgr.exists(&rel) {
            continue;
        }
        let data = read(&mgr, &rel);
        if let Some(vars) = data["Variables"].as_array() {
            for v in vars {
                assert_ne!(v["Value"].as_str(), Some("False"));
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn properties_owned_and_filled() {
    let (mgr, root) = fresh_save();
    let owned = mgr.own_all_properties().unwrap();
    assert!(owned > 0);
    // Existing property files now owned.
    let barn = read(&mgr, "Properties/Barn.json");
    assert_eq!(barn["IsOwned"].as_bool(), Some(true));

    let changed = mgr.fill_storage(99, 0, "Heavenly", "jar").unwrap();
    assert!(changed > 0, "expected storage slots to be filled");

    // Verify a known weed/item slot got the new quantity by scanning Barn.
    let barn = read(&mgr, "Properties/Barn.json");
    let mut found = false;
    for obj in barn["Objects"].as_array().unwrap() {
        let base: Value = serde_json::from_str(obj["BaseData"].as_str().unwrap()).unwrap();
        if let Some(items) = base.pointer("/Contents/Items").and_then(|v| v.as_array()) {
            for it in items {
                let parsed: Value = serde_json::from_str(it.as_str().unwrap()).unwrap();
                let dt = parsed["DataType"].as_str().unwrap_or("");
                let id = parsed["ID"].as_str().unwrap_or("");
                if !id.is_empty() && (dt == "ItemData" || dt == "IntegerItemData" || dt == "WeedData")
                {
                    assert_eq!(parsed["Quantity"].as_i64(), Some(99));
                    found = true;
                }
            }
        }
    }
    assert!(found, "expected at least one filled slot in Barn");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn businesses_owned() {
    let (mgr, root) = fresh_save();
    let n = mgr.own_all_businesses().unwrap();
    assert!(n > 0);
    let laundromat = read(&mgr, "Businesses/Laundromat.json");
    assert_eq!(laundromat["IsOwned"].as_bool(), Some(true));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn products_discover_and_generate() {
    let (mgr, root) = fresh_save();
    mgr.discover_all_products().unwrap();
    let products = read(&mgr, "Products.json");
    let discovered = products["DiscoveredProducts"].as_array().unwrap();
    let has = |id: &str| discovered.iter().any(|v| v.as_str() == Some(id));
    assert!(has("cocaine") && has("meth"));

    let before = discovered.len();
    let made = mgr.generate_products(5, 8, 1234, true).unwrap();
    assert_eq!(made, 5);
    let products = read(&mgr, "Products.json");
    assert_eq!(products["DiscoveredProducts"].as_array().unwrap().len(), before + 5);

    let removed = mgr.delete_generated().unwrap();
    assert_eq!(removed, 5);
    let products = read(&mgr, "Products.json");
    assert_eq!(products["DiscoveredProducts"].as_array().unwrap().len(), before);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn npc_relationships_maxed() {
    let (mgr, root) = fresh_save();
    let n = mgr.max_relationships().unwrap();
    assert!(n > 0);
    let npcs = read(&mgr, "NPCs.json");
    for npc in npcs["NPCs"].as_array().unwrap() {
        if let Some(addl) = npc["AdditionalDatas"].as_array() {
            for entry in addl {
                if entry["Name"].as_str() == Some("Relationship") {
                    let rel: Value =
                        serde_json::from_str(entry["Contents"].as_str().unwrap()).unwrap();
                    assert_eq!(rel["Unlocked"].as_bool(), Some(true));
                    assert_eq!(rel["RelationDelta"].as_f64(), Some(999.0));
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn org_and_settings() {
    let (mgr, root) = fresh_save();
    mgr.set_org_name("Test Cartel").unwrap();
    mgr.set_console_enabled(true).unwrap();
    let game = read(&mgr, "Game.json");
    assert_eq!(game["OrganisationName"].as_str(), Some("Test Cartel"));
    assert_eq!(game["Settings"]["ConsoleEnabled"].as_bool(), Some(true));
    assert!(mgr.console_enabled());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unlock_regions_works() {
    let (mgr, root) = fresh_save();
    mgr.unlock_regions().unwrap();
    let rank = read(&mgr, "Rank.json");
    assert_eq!(rank["UnlockedRegions"].as_array().unwrap().len(), 9);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn backup_and_revert() {
    let (mgr, root) = fresh_save();
    let original = read(&mgr, "Money.json");
    let original_online = original["OnlineBalance"].as_f64().unwrap();

    mgr.set_money(Some(777.0), None, None, None).unwrap();
    assert_eq!(read(&mgr, "Money.json")["OnlineBalance"].as_f64(), Some(777.0));

    // A feature backup should now exist.
    let list = mgr.backups().list();
    assert!(list.iter().any(|b| b.feature == "Money"));

    mgr.backups().revert_all().unwrap();
    let reverted = read(&mgr, "Money.json");
    assert_eq!(reverted["OnlineBalance"].as_f64(), Some(original_online));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn output_is_four_space_indented() {
    let (mgr, root) = fresh_save();
    mgr.set_money(Some(1.0), None, None, None).unwrap();
    let text = std::fs::read_to_string(mgr.file("Money.json")).unwrap();
    assert!(text.contains("\n    \"DataType\""), "expected 4-space indent");
    let _ = std::fs::remove_dir_all(root);
}
