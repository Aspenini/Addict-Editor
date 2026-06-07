# Addict Editor

A fast save-file editor for **Schedule I** (`0.4.5f2` format), built in **Rust** + **Slint**.

## Features

Staged editing: changes are queued and only written when you hit **Apply** (or thrown away with **Discard**). Loading a save makes a full backup first.

| Tab | What you can do |
|-----|-----------------|
| **Save** | Auto-detect saves (`…\TVGS\Schedule I\saves\<steamid>\SaveGame_N`) or browse to a folder |
| **Money** | Online balance, net worth, lifetime earnings, weekly deposit, player cash |
| **Rank** | Presets (`Street Rat 1`–`Baron 5`), manual rank/tier/XP, per-region toggles |
| **Products** | Discover base products, generate custom ones, delete generated |
| **Properties / Businesses** | Per-item ownership toggles, bulk-fill storage (qty/quality/packaging) |
| **Inventory** | Set dealer cash and player cash |
| **NPCs** | Set relationship value, recruit dealers, add missing NPCs |
| **Quests** | Complete all quests and objectives |
| **Misc** | Organisation name, console setting, appearance presets |
| **Backups** | Revert a single feature or restore the whole save |
| **Themes** | Dark, Light, Dracula, Solarized |

Numeric inputs are capped to the maximum the game accepts. The sidebar collapses to icons.

## Safety

- Every write is backed up; the first load snapshots the whole save to `<SaveName>_AddictBackups\initial`.
- Output keeps the game's 4-space indentation and preserves unknown fields (round-trip safe).
- Never touches or downloads anything to your game install.

## Build, run & test

```bash
cargo run --release   # launch
cargo test            # run integration tests against a copy of reference material/SaveGame_1
```

## Layout

- `ui/` — Slint UI (`app.slint`, `theme.slint`, `widgets.slint`)
- `src/save/` — save IO + per-feature logic (`manager`, `paths`, `backup`, `money`, `rank`, `products`, `properties`, `npcs`, `quests`, `inventory`, `appearance`, `misc`, `templates`)
- `src/bridge.rs` — Rust ↔ Slint model conversions
- `src/main.rs` — window setup, staging model, callback wiring

Inspired by [Schedule-1-Save-File-Editor](https://github.com/N0edL/Schedule-1-Save-File-Editor). Use at your own risk — keep your own backups.
