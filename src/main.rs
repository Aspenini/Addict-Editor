#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod bridge;
mod save;

#[cfg(test)]
mod tests;

use save::manager::SaveManager;
use save::{appearance, paths, properties, rank};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

type State = Rc<RefCell<Option<SaveManager>>>;
type Pend = Rc<RefCell<Pending>>;

#[derive(Default, Clone)]
struct MoneyEdit {
    online: Option<f64>,
    networth: Option<f64>,
    lifetime: Option<f64>,
    weekly: Option<f64>,
    cash: Option<f64>,
}

/// Holds edits that have been staged but not yet written to disk.
#[derive(Default)]
struct Pending {
    money: Option<MoneyEdit>,
    rank: Option<(i64, i64, i64)>,
    rank_preset: Option<usize>,
    regions: BTreeMap<i64, bool>,
    properties: BTreeMap<String, bool>,
    businesses: BTreeMap<String, bool>,
    relationships: Option<f64>,
    recruit_dealers: bool,
    add_missing: bool,
    complete_quests: bool,
    discover: bool,
    generate: Option<(usize, usize, i64, bool)>,
    delete_generated: bool,
    dealer_cash: Option<f64>,
    org: Option<String>,
    console: Option<bool>,
    appearance: Option<usize>,
    fill: Option<(i64, usize, String, String)>,
}

impl Pending {
    fn count(&self) -> usize {
        let mut n = 0;
        n += self.money.is_some() as usize;
        n += self.rank.is_some() as usize;
        n += self.rank_preset.is_some() as usize;
        n += self.regions.len();
        n += self.properties.len();
        n += self.businesses.len();
        n += self.relationships.is_some() as usize;
        n += self.recruit_dealers as usize;
        n += self.add_missing as usize;
        n += self.complete_quests as usize;
        n += self.discover as usize;
        n += self.generate.is_some() as usize;
        n += self.delete_generated as usize;
        n += self.dealer_cash.is_some() as usize;
        n += self.org.is_some() as usize;
        n += self.console.is_some() as usize;
        n += self.appearance.is_some() as usize;
        n += self.fill.is_some() as usize;
        n
    }

    /// Writes every staged change to disk, in a sensible order.
    fn apply(&self, mgr: &SaveManager) -> save::Result<usize> {
        let mut applied = 0;
        if let Some(m) = &self.money {
            mgr.set_money(m.online, m.networth, m.lifetime, m.weekly)?;
            if let Some(c) = m.cash {
                mgr.set_player_cash(c)?;
            }
            applied += 1;
        }
        if let Some(idx) = self.rank_preset {
            mgr.apply_rank_preset(idx)?;
            applied += 1;
        }
        if let Some((r, t, x)) = self.rank {
            mgr.apply_rank_manual(r, t, x)?;
            applied += 1;
        }
        for (idx, on) in &self.regions {
            mgr.set_region(*idx, *on)?;
            applied += 1;
        }
        for (name, owned) in &self.properties {
            mgr.set_ownership(
                "Properties",
                name,
                *owned,
                save::templates::PROPERTIES,
                "PropertyData",
            )?;
            applied += 1;
        }
        for (name, owned) in &self.businesses {
            mgr.set_ownership(
                "Businesses",
                name,
                *owned,
                save::templates::BUSINESSES,
                "BusinessData",
            )?;
            applied += 1;
        }
        if let Some((qty, ft, q, p)) = &self.fill {
            mgr.fill_storage(*qty, *ft, q, p)?;
            applied += 1;
        }
        if let Some(v) = self.relationships {
            mgr.set_all_relationships(v)?;
            applied += 1;
        }
        if self.recruit_dealers {
            mgr.recruit_dealers()?;
            applied += 1;
        }
        if self.add_missing {
            mgr.add_missing_npcs()?;
            applied += 1;
        }
        if self.complete_quests {
            mgr.complete_quests()?;
            applied += 1;
        }
        if self.discover {
            mgr.discover_all_products()?;
            applied += 1;
        }
        if let Some((count, len, price, listed)) = self.generate {
            mgr.generate_products(count, len, price, listed)?;
            applied += 1;
        }
        if self.delete_generated {
            mgr.delete_generated()?;
            applied += 1;
        }
        if let Some(v) = self.dealer_cash {
            mgr.set_all_dealer_cash(v)?;
            applied += 1;
        }
        if let Some(name) = &self.org {
            mgr.set_org_name(name)?;
            applied += 1;
        }
        if let Some(c) = self.console {
            mgr.set_console_enabled(c)?;
            applied += 1;
        }
        if let Some(idx) = self.appearance {
            mgr.apply_appearance(idx)?;
            applied += 1;
        }
        Ok(applied)
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let state: State = Rc::new(RefCell::new(None));
    let pending: Pend = Rc::new(RefCell::new(Pending::default()));

    // Static option models.
    let rank_presets: Vec<String> = rank::preset_labels();
    let rank_refs: Vec<&str> = rank_presets.iter().map(|s| s.as_str()).collect();
    ui.set_rank_presets(bridge::strings_model(&rank_refs));
    ui.set_quality_options(bridge::strings_model(&properties::QUALITIES));
    ui.set_packaging_options(bridge::strings_model(&properties::PACKAGINGS));
    ui.set_fill_type_options(bridge::strings_model(&properties::FILL_TYPES));
    ui.set_appearance_presets(bridge::strings_model(&appearance::APPEARANCE_PRESETS));
    ui.set_theme_names(bridge::strings_model(&["Dark", "Light", "Dracula", "Solarized"]));

    // Try to populate the saves list at startup.
    refresh_saves(&ui, &paths::list_saves());

    wire_callbacks(&ui, &state, &pending);

    ui.run()
}

// =========================================================================
// Status / refresh helpers
// =========================================================================

fn set_status(ui: &AppWindow, msg: impl Into<String>, error: bool) {
    ui.set_status_message(msg.into().into());
    ui.set_status_error(error);
}

fn refresh_saves(ui: &AppWindow, entries: &[save::SaveInfoEntry]) {
    ui.set_available_saves(bridge::saves_model(entries));
}

fn refresh_loaded(ui: &AppWindow, mgr: &SaveManager) {
    ui.set_save_loaded(true);
    ui.set_current_save_path(mgr.path.to_string_lossy().to_string().into());

    if let Ok(s) = mgr.summary() {
        ui.set_info_version(s.game_version.into());
        ui.set_info_created(s.created.into());
        ui.set_info_org(s.org.clone().into());
        ui.set_info_online_money(s.online_money.into());
        ui.set_info_networth(s.networth.into());
        ui.set_info_rank(s.rank.into());
        ui.set_org_name(s.org.into());
    }
    if let Ok(m) = mgr.money_values() {
        ui.set_money_online(m.online.into());
        ui.set_money_networth(m.networth.into());
        ui.set_money_lifetime(m.lifetime.into());
        ui.set_money_weekly(m.weekly.into());
        ui.set_money_cash(m.cash.into());
    }
    ui.set_console_enabled(mgr.console_enabled());

    // Rank integer fields.
    if let Ok(Some(rank)) = mgr.read_opt("Rank.json") {
        let g = |k: &str| rank.get(k).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        ui.set_rank_int(g("Rank").clamp(0, 999));
        ui.set_tier_int(g("Tier").clamp(0, 999));
        ui.set_xp_int(g("XP").clamp(0, 100_000));
    }

    // Precise ownership + region models.
    ui.set_properties_list(bridge::toggles_model(
        &mgr.list_ownership("Properties", save::templates::PROPERTIES),
    ));
    ui.set_businesses_list(bridge::toggles_model(
        &mgr.list_ownership("Businesses", save::templates::BUSINESSES),
    ));
    ui.set_regions(bridge::regions_model(
        &mgr.unlocked_regions(),
        &SaveManager::REGION_NAMES,
    ));

    refresh_backups(ui, mgr);
}

const MONEY_MAX: f64 = 999_999_999.0;

fn clamp_money(v: Option<f64>) -> Option<f64> {
    v.map(|x| x.clamp(0.0, MONEY_MAX))
}

fn refresh_backups(ui: &AppWindow, mgr: &SaveManager) {
    let list = mgr.backups().list();
    ui.set_backups(bridge::backups_model(&list));
}

fn refresh_pending(ui: &AppWindow, pending: &Pend) {
    ui.set_pending_count(pending.borrow().count() as i32);
}

/// Rebuilds the region/property/business toggle models to reflect disk state
/// with the currently staged overrides applied on top.
fn rebuild_toggle_models(ui: &AppWindow, mgr: &SaveManager, pending: &Pend) {
    let p = pending.borrow();

    let mut regions: Vec<i64> = mgr.unlocked_regions();
    for (idx, on) in &p.regions {
        regions.retain(|r| r != idx);
        if *on {
            regions.push(*idx);
        }
    }
    ui.set_regions(bridge::regions_model(&regions, &SaveManager::REGION_NAMES));

    let mut props = mgr.list_ownership("Properties", save::templates::PROPERTIES);
    for (name, owned) in &p.properties {
        if let Some(row) = props.iter_mut().find(|(n, _)| n == name) {
            row.1 = *owned;
        }
    }
    ui.set_properties_list(bridge::toggles_model(&props));

    let mut biz = mgr.list_ownership("Businesses", save::templates::BUSINESSES);
    for (name, owned) in &p.businesses {
        if let Some(row) = biz.iter_mut().find(|(n, _)| n == name) {
            row.1 = *owned;
        }
    }
    ui.set_businesses_list(bridge::toggles_model(&biz));
}

// =========================================================================
// Callback wiring
// =========================================================================

fn wire_callbacks(ui: &AppWindow, state: &State, pending: &Pend) {
    // ---- Detect / browse / load ----
    {
        let ui_weak = ui.as_weak();
        ui.on_detect_saves(move || {
            let ui = ui_weak.unwrap();
            let saves = paths::list_saves();
            refresh_saves(&ui, &saves);
            if saves.is_empty() {
                set_status(&ui, "No saves found in the default location.", true);
            } else {
                set_status(&ui, format!("Found {} save(s).", saves.len()), false);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_browse_save(move || {
            let ui = ui_weak.unwrap();
            let Some(picked) = rfd::FileDialog::new().pick_folder() else {
                return;
            };
            if picked.join("Game.json").exists() {
                load_path(&ui, &state, &pending, &picked);
                if let Some(parent) = picked.parent() {
                    refresh_saves(&ui, &paths::list_saves_in(parent));
                }
            } else {
                let saves = paths::list_saves_in(&picked);
                refresh_saves(&ui, &saves);
                if saves.is_empty() {
                    set_status(&ui, "No save folders found there.", true);
                } else {
                    set_status(&ui, format!("Found {} save(s).", saves.len()), false);
                }
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_load_save(move |path| {
            let ui = ui_weak.unwrap();
            load_path(&ui, &state, &pending, Path::new(path.as_str()));
        });
    }

    // ---- Apply / discard staged changes ----
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_apply_changes(move || {
            let ui = ui_weak.unwrap();
            let borrow = state.borrow();
            let Some(mgr) = borrow.as_ref() else {
                set_status(&ui, "No save loaded.", true);
                return;
            };
            let result = pending.borrow().apply(mgr);
            match result {
                Ok(n) => {
                    *pending.borrow_mut() = Pending::default();
                    set_status(&ui, format!("Applied {n} change(s) to the save."), false);
                    refresh_loaded(&ui, mgr);
                    refresh_pending(&ui, &pending);
                }
                Err(e) => set_status(&ui, format!("Error: {e}"), true),
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_discard_changes(move || {
            let ui = ui_weak.unwrap();
            *pending.borrow_mut() = Pending::default();
            if let Some(mgr) = state.borrow().as_ref() {
                refresh_loaded(&ui, mgr);
            }
            refresh_pending(&ui, &pending);
            set_status(&ui, "Discarded staged changes.", false);
        });
    }

    // ---- Money (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_money(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().money = Some(MoneyEdit {
                online: clamp_money(bridge::parse_f64(ui.get_money_online().as_str())),
                networth: clamp_money(bridge::parse_f64(ui.get_money_networth().as_str())),
                lifetime: clamp_money(bridge::parse_f64(ui.get_money_lifetime().as_str())),
                weekly: clamp_money(bridge::parse_f64(ui.get_money_weekly().as_str())),
                cash: clamp_money(bridge::parse_f64(ui.get_money_cash().as_str())),
            });
            refresh_pending(&ui, &pending);
        });
    }

    // ---- Rank (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_rank_preset(move || {
            let ui = ui_weak.unwrap();
            let idx = ui.get_rank_preset_index().max(0) as usize;
            ui.set_rank_int(((idx / 5) as i32).clamp(0, 999));
            ui.set_tier_int(((idx % 5) as i32 + 1).clamp(0, 999));
            ui.set_xp_int(0);
            let mut p = pending.borrow_mut();
            p.rank = None;
            p.rank_preset = Some(idx);
            drop(p);
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_rank(move || {
            let ui = ui_weak.unwrap();
            let mut p = pending.borrow_mut();
            p.rank_preset = None;
            p.rank = Some((
                ui.get_rank_int() as i64,
                ui.get_tier_int() as i64,
                ui.get_xp_int() as i64,
            ));
            drop(p);
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_stage_region(move |region, on| {
            let ui = ui_weak.unwrap();
            if let Some(mgr) = state.borrow().as_ref() {
                let disk_on = mgr.unlocked_regions().contains(&(region as i64));
                {
                    let mut p = pending.borrow_mut();
                    if on == disk_on {
                        p.regions.remove(&(region as i64));
                    } else {
                        p.regions.insert(region as i64, on);
                    }
                }
                rebuild_toggle_models(&ui, mgr, &pending);
                refresh_pending(&ui, &pending);
            }
        });
    }

    // ---- Products (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_discover(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().discover = true;
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_generate(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().generate = Some((
                ui.get_product_count().max(0) as usize,
                ui.get_product_id_length().max(1) as usize,
                ui.get_product_price() as i64,
                ui.get_product_list_them(),
            ));
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_delete_generated(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().delete_generated = true;
            refresh_pending(&ui, &pending);
        });
    }

    // ---- Properties / Businesses (staged) ----
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_stage_property_owned(move |name, owned| {
            let ui = ui_weak.unwrap();
            if let Some(mgr) = state.borrow().as_ref() {
                let name = name.to_string();
                let disk = mgr
                    .list_ownership("Properties", save::templates::PROPERTIES)
                    .into_iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, o)| o)
                    .unwrap_or(false);
                {
                    let mut p = pending.borrow_mut();
                    if owned == disk {
                        p.properties.remove(&name);
                    } else {
                        p.properties.insert(name, owned);
                    }
                }
                rebuild_toggle_models(&ui, mgr, &pending);
                refresh_pending(&ui, &pending);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_stage_business_owned(move |name, owned| {
            let ui = ui_weak.unwrap();
            if let Some(mgr) = state.borrow().as_ref() {
                let name = name.to_string();
                let disk = mgr
                    .list_ownership("Businesses", save::templates::BUSINESSES)
                    .into_iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, o)| o)
                    .unwrap_or(false);
                {
                    let mut p = pending.borrow_mut();
                    if owned == disk {
                        p.businesses.remove(&name);
                    } else {
                        p.businesses.insert(name, owned);
                    }
                }
                rebuild_toggle_models(&ui, mgr, &pending);
                refresh_pending(&ui, &pending);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_fill(move || {
            let ui = ui_weak.unwrap();
            let quality = properties::QUALITIES
                .get(ui.get_quality_index().max(0) as usize)
                .copied()
                .unwrap_or("Standard")
                .to_string();
            let packaging = properties::PACKAGINGS
                .get(ui.get_packaging_index().max(0) as usize)
                .copied()
                .unwrap_or("none")
                .to_string();
            pending.borrow_mut().fill = Some((
                ui.get_fill_quantity() as i64,
                ui.get_fill_type_index().max(0) as usize,
                quality,
                packaging,
            ));
            refresh_pending(&ui, &pending);
        });
    }

    // ---- Inventory (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_dealer_cash(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().dealer_cash =
                clamp_money(bridge::parse_f64(ui.get_dealer_cash_value().as_str()));
            refresh_pending(&ui, &pending);
        });
    }

    // ---- NPCs (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_relationships(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().relationships = Some(ui.get_relationship_value().clamp(0, 999) as f64);
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_recruit_dealers(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().recruit_dealers = true;
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_add_missing(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().add_missing = true;
            refresh_pending(&ui, &pending);
        });
    }

    // ---- Quests (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_complete_quests(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().complete_quests = true;
            refresh_pending(&ui, &pending);
        });
    }

    // ---- Misc (staged) ----
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_org(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().org = Some(ui.get_org_name().to_string());
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_console(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().console = Some(ui.get_console_enabled());
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let pending = pending.clone();
        ui.on_stage_appearance(move || {
            let ui = ui_weak.unwrap();
            pending.borrow_mut().appearance = Some(ui.get_appearance_index().max(0) as usize);
            refresh_pending(&ui, &pending);
        });
    }

    // ---- Backups ----
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_refresh_backups(move || {
            let ui = ui_weak.unwrap();
            if let Some(mgr) = state.borrow().as_ref() {
                refresh_backups(&ui, mgr);
                set_status(&ui, "Backups refreshed.", false);
            } else {
                set_status(&ui, "No save loaded.", true);
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_revert_backup(move |feature, timestamp| {
            let ui = ui_weak.unwrap();
            *pending.borrow_mut() = Pending::default();
            run(&ui, &state, |mgr| {
                mgr.backups().revert_feature(feature.as_str(), timestamp.as_str())?;
                Ok("Reverted backup.".into())
            });
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let pending = pending.clone();
        ui.on_revert_all(move || {
            let ui = ui_weak.unwrap();
            *pending.borrow_mut() = Pending::default();
            run(&ui, &state, |mgr| {
                mgr.backups().revert_all()?;
                Ok("Reverted all changes to the initial backup.".into())
            });
            refresh_pending(&ui, &pending);
        });
    }
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        ui.on_delete_backups(move || {
            let ui = ui_weak.unwrap();
            if let Some(mgr) = state.borrow().as_ref() {
                match mgr.backups().delete_all() {
                    Ok(()) => {
                        refresh_backups(&ui, mgr);
                        set_status(&ui, "All backups deleted.", false);
                    }
                    Err(e) => set_status(&ui, e.to_string(), true),
                }
            } else {
                set_status(&ui, "No save loaded.", true);
            }
        });
    }
}

// =========================================================================
// Operation runner
// =========================================================================

/// Runs `f` against the loaded manager, updating status and refreshing the UI.
fn run<F>(ui: &AppWindow, state: &State, f: F)
where
    F: FnOnce(&SaveManager) -> save::Result<String>,
{
    let borrow = state.borrow();
    let Some(mgr) = borrow.as_ref() else {
        set_status(ui, "No save loaded.", true);
        return;
    };
    match f(mgr) {
        Ok(msg) => {
            set_status(ui, msg, false);
            refresh_loaded(ui, mgr);
        }
        Err(e) => set_status(ui, format!("Error: {e}"), true),
    }
}

fn load_path(ui: &AppWindow, state: &State, pending: &Pend, path: &Path) {
    match SaveManager::open(path.to_path_buf()) {
        Ok(mgr) => {
            // Drop any staged edits from a previous save.
            *pending.borrow_mut() = Pending::default();
            // Make a fresh full backup of the save the first time it is opened.
            let made = matches!(mgr.backups().ensure_initial(), Ok(true));
            refresh_loaded(ui, &mgr);
            refresh_pending(ui, pending);
            set_status(
                ui,
                if made {
                    "Save loaded. Initial backup created."
                } else {
                    "Save loaded."
                },
                false,
            );
            *state.borrow_mut() = Some(mgr);
        }
        Err(e) => set_status(ui, e.to_string(), true),
    }
}

// Keep PathBuf import used even if optimized.
#[allow(dead_code)]
fn _unused(_: PathBuf) {}
