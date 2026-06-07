# Addict Editor

A save-file editor for the game **Schedule I**, built in **Rust** with the **Slint** UI toolkit.
It targets the modern `0.4.5f2` save format and is inspired by the older
[Schedule-1-Save-File-Editor](https://github.com/N0edL/Schedule-1-Save-File-Editor) (Python/PySide6),
re-implemented for the current save layout.

## Features

- **Save selection** - auto-detects saves under `%USERPROFILE%\AppData\LocalLow\TVGS\Schedule I\saves\<steamid>\SaveGame_N`, or browse to any folder.
- **Money** - online balance, net worth, lifetime earnings, weekly deposit sum, and on-hand player cash.
- **Rank** - presets (`Street Rat 1` .. `Baron 5`) with correct cumulative XP, manual rank/tier/XP, or max to 999.
- **Products** - discover base products, generate custom products (with prices and mix recipes), delete generated.
- **Properties / Businesses** - own everything, bulk-fill storage slots with quantity/quality/packaging.
- **Unlocks** - rank 999, all map regions, and one-click "unlock everything".
- **Inventory** - set dealer cash and player cash.
- **NPCs** - max all relationships, recruit dealers, add missing NPCs from bundled templates.
- **Quests** - complete all quests and objectives.
- **Variables** - flip `False` flags to `True` and max numeric counters.
- **Misc** - organisation name, console setting, appearance presets.
- **Saves** - generate a new save, import an external save folder, delete saves.
- **Backups** - an initial full backup plus per-feature timestamped snapshots; revert a feature or revert everything.
- **Themes** - Dark, Light, Dracula, Solarized.
- **Mods** - detects the game install and MelonLoader status; links to the official installer (no auto-downloads).

## Safety

- Every edit first writes a backup. The first edit also snapshots the whole save folder (`<SaveName>_AddictBackups\initial`).
- Files are written back with the game's 4-space indentation, and unknown fields are preserved (round-trip safe via `serde_json` with `preserve_order`).
- The editor never auto-downloads or modifies your game installation.

## Build & run

```bash
cargo run --release
```

## Test

Integration tests run every operation against a throwaway copy of `reference material/SaveGame_1`:

```bash
cargo test
```

## Project layout

- `ui/` - Slint UI (`app.slint`, `theme.slint`, `widgets.slint`).
- `src/save/` - save IO and per-feature logic (`manager`, `paths`, `backup`, `money`, `rank`, `products`, `properties`, `npcs`, `quests`, `variables`, `inventory`, `appearance`, `misc`, `generate`, `mods`, `templates`).
- `src/bridge.rs` - conversions between the save layer and Slint models.
- `src/main.rs` - window setup and callback wiring.

## Disclaimer

Use at your own risk. Always keep your own backups of important saves.
