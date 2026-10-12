# Wallo

A lightweight wallpaper manager for Windows 10 and 11 with multiple monitors.

Wallo does what the Windows settings can't: put a different image on each screen, stretch one image across all of them, and fine-tune how each image is placed. Pick your settings, click **Apply**, and Wallo closes. Nothing keeps running in the background.

## Features

- **Profiles** - save several wallpaper setups and switch between them in one click.
- **Three modes**
  - one image spanned across all monitors
  - a different image on each monitor
  - the same image repeated on every monitor
- **Sizing** - Center, Stretch, Fit (with or without cropping), Fit Width, Fit Height, Tile, plus an option to never enlarge small images.
- **Per-monitor adjustments** - scale, rotation, horizontal and vertical offset, blur, horizontal and vertical mirror.
- **Live preview** of every monitor, laid out like your real screens.

## Usage

1. Choose a profile at the top, or create one with **New**.
2. Pick a background mode.
3. In "Use a different image for each monitor" mode, click a monitor in the preview to select it.
4. Click **Load Image**, then adjust sizing and settings. The preview updates as you go.
5. Click **Apply** to set the wallpaper, or **OK** to apply and close.

**Cancel** closes Wallo without saving your changes.

## Where things are stored

- Profiles: `%APPDATA%\wallo\profiles.json`
- Generated wallpapers: `%APPDATA%\wallo\rendered\`

Your original images are never modified.

## Requirements

- Windows 10 or 11
- Microsoft Edge WebView2 Runtime (already included in Windows 11 and up-to-date Windows 10)

## Building from source

Requires [Rust](https://rustup.rs), [Node.js](https://nodejs.org) and the Tauri CLI (`cargo install tauri-cli`).

```
npm install
cargo tauri build
```

The installer is written to `src-tauri\target\release\bundle\`.

## License

MIT
