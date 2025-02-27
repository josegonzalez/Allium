# Allium

Allium is a custom launcher for the Miyoo Mini and Miyoo Mini Plus handheld devices, similar to [OnionOS](https://github.com/OnionUI/Onion) and [MiniUI](https://github.com/shauninman/MiniUI).

## Project Goals

The goal of Allium is to replace MainUI (stock UI) with a faster and more user-friendly UI.
- Fast
- Clean, user-friendly UI
- RetroArch (with Netplay, achievements)
- Box art
- Support running on both Miyoo Mini and Miyoo Mini Plus without changes

# Screenshots

<div>
    <img alt="Main menu" src="assets/screenshots/main-menu.png" width="49%">
    <img alt="Ingame menu" src="assets/screenshots/ingame-menu.png" width="49%">
    <img alt="Guide" src="assets/screenshots/guide.png" width="49%">
    <img alt="Settings" src="assets/screenshots/settings.png" width="49%">
    <img alt="Themes" src="assets/screenshots/themes.png" width="49%">
    <img alt="Localization" src="assets/screenshots/localization.png" width="49%">
</div>

## Installation

Allium supports both the Miyoo Mini and Miyoo Mini Plus on the same SD card.

1. Format the SD card to [FAT32](https://github.com/anzz1/DotUI-X/wiki/fat32format).
2. Download the latest release and extract into your SD card. e.g. `E:/`.

The SD card layout should look like this:
- .allium
- .tmp_update
- BIOS
- RetroArch
- Roms
- Apps
- Saves (optional, if you have existing saves from OnionOS)

## Features
- Supports stock/Onion/DotUI SD card layout
- Works without configuration
- Box art (250px wide, PNG, JPG, GIF)
- Supports gameslist.xml with nested folders
- Recents list (sort by last played or playtime)
- Activity tracker
- [RetroArch for all supported cores](https://github.com/goweiwen/Allium/wiki/Console-Mapper)
- Volume & Brightness (select/start + l/r) control
- In-game menu (save, load, reset, access RetroArch menu, [guide](https://github.com/goweiwen/Allium/wiki/In-game-Guide-Walkthrough-Reader), disk changer, quit)
- Automatic resume when powering off/on
- Settings page
    - WiFi (IP Address, NTP, Telnet, FTP)
    - Change LCD settings
    - Customize theme colours, font
    - Change system language

## Planned Features
(roughly in order of priority)
- Search
- Suspend
- Favorites
- Clock adjustment
- WiFi stuff:
    - OTA update
    - Metadata/box art scraper
    - Cloud save sync
    - Seamless netplay from ingame menu
- UI improvements:
    - Folder icon
    - Volume indicator
    - Brightness indicator
    - Error toast (e.g. no core found for game)
    - Anti-aliased circles
- Theme manager
    - Built-in themes
    - Save current theme to file

## Development

Allium comes with a simulator that can be used for development. The simulator requires SDL2 to be installed.

### Requirements
1. `make`, `cargo`
2. [SDL2](https://github.com/Rust-SDL2/rust-sdl2#sdl20-development-libraries) (optional, if simulator is not used)
3. [cross](https://github.com/cross-rs/cross): `cargo install cross --git https://github.com/cross-rs/cross` (optional, for cross-compilation)

### Architecture
Allium is split into 3 binaries:
- `alliumd` (daemon that handles launcher/game/menu launching, vol/brightness hotkeys, poweroff)
- `allium-launcher` (main menu, including games, recents, settings)
- `allium-menu` (ingame menu, including guide reader)

Shared code is located in the `common` crate.

### Simulator
There is no simulator for `alliumd` (no UI, only logic).
```
# Run main menu (allium-launcher)
make simulator-launcher

# Run ingame menu (allium-menu)
make simulator-menu
```

### Building

Running `make` will build Allium and RetroArch, then copy the built and static files into `dist/`.
```
make all
cp -r dist/. <sdcard>
```

## Acknowledgements

Allium is only possible thanks to the Miyoo Mini community, including but not limited to:
- eggs: RetroArch port, [many code samples](https://www.dropbox.com/sh/hqcsr1h1d7f8nr3/AABtSOygIX_e4mio3rkLetWTa), answering questions on Discord
- [Onion team](https://github.com/OnionUI/Onion) (Aemiii91, Schmurtz, Totofaki, and more): Maintaining a sane-defaults RetroArch configuration, and the huge village
- kebabstorm: [Miyoo Mini resources](https://github.com/anzz1/miyoomini-resources)
- shauninman: Allium is heavily inspired by [MiniUI](https://github.com/shauninman/MiniUI)'s simplicity and clean design
- Early adopters and testers of Allium
