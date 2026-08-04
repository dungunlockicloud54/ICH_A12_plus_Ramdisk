# DienThoai88 Ramdisk Tool — macOS GUI (Tauri + React)

This repository adds a Tauri + React + TailwindGUI wrapper around the existing ich_a12_plus_ramdisk tool to provide a macOS desktop app that: auto-detects iPhones (libimobiledevice), shows device info, boots the ramdisk using the existing tool, provides backup options, bypass actions, realtime logs, and exports reports.

Important notes
- This project DOES NOT reimplement ich_a12_plus_ramdisk. The GUI invokes the existing binary in the repository root. Keep the original executable present and executable.
- You must have libimobiledevice installed (idevice_id, ideviceinfo) for device detection. Install with Homebrew:
  brew install libimobiledevice usbmuxd

Quick dev run (macOS)
1. Install Node (v18+), Rust, Cargo, and Tauri prereqs. On macOS, install Xcode command line tools.
2. From repo root, install frontend deps:
   cd gui
   npm install
3. Start frontend dev server and Tauri dev:
   npm run dev
   # in new terminal
   cargo tauri dev --manifest-path=src-tauri/Cargo.toml

Build (macOS universal)
1. From gui/:
   npm run build
2. Then to create the Tauri bundle:
   cargo tauri build --manifest-path=src-tauri/Cargo.toml

Configuration
- The GUI allows you to pass extra CLI args to the underlying ich_a12_plus_ramdisk binary so you can adapt flags to the specific version you have.

License
- This wrapper is provided as-is. Keep the original tool's license intact.
