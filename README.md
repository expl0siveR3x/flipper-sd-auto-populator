# Flipper SD Auto Populator

A desktop tool for mass-provisioning [Flipper Zero](https://flipperzero.one/) SD cards. Select an SD card (auto-detected or chosen manually), pick a copy **profile**, and it moves files onto the card in the structure the Flipper firmware expects. Contents are then picked up by the firmware's asset installers (e.g. Momentum).

Built with Rust and [egui/eframe](https://github.com/emilk/egui).

## Features

- **Cross-platform drive detection** (Linux + Windows) via `sysinfo`, with automatic recognition of Flipper SD cards by their root folder structure.
- **Profiles** — named collections of mappings, saved to `profiles.json` (created automatically next to the executable on first run).
- **Full in-app profile editor** — add / duplicate / delete profiles, set the default, and manage mappings with folder pickers and quick-add destinations.
- **Two copy modes per mapping:**
  - `Exact Mirror (wipe & copy)` — deletes everything in the destination folder, then copies the source over it.
  - `Only If Missing` — copies only the files the destination doesn't already contain.
- **Live progress log** with per-file status, running in a background thread so the UI stays responsive, plus a Cancel button.
- **Optional zip backup** of the whole SD card before copying (`backups/backup_<timestamp>.zip` next to the executable).

## Build & run

Requires a Rust toolchain (edition 2024).

```sh
cd flipper-sd-auto-populator
cargo run --release
```

Run the test suite and linter:

```sh
cargo test
cargo clippy --all-targets
cargo fmt --check
```

## Usage

1. **Edit Profiles** — create or adjust a profile. Each mapping has:
   - a **source** folder on your computer,
   - a **destination** folder on the SD card (relative, e.g. `subghz`, `infrared`, `nfc`),
   - a **copy mode**.
2. **Pick a target** — either from the detected drives list (Flipper cards are flagged) or via *Choose Drive Manually...*.
3. **Move Files** — the selected profile's mappings are applied to the SD card.

When a mapping uses *Exact Mirror*, a confirmation dialog warns that the destination will be wiped first.

### `profiles.json`

Stored next to the executable. Shape:

```json
[
  {
    "name": "Default",
    "is_default": true,
    "mappings": [
      {
        "source": "/path/to/my/subghz/files",
        "dest": "subghz",
        "mode": "OnlyIfMissing"
      }
    ]
  }
]
```

If the file is missing or invalid, a starter profile is created automatically.