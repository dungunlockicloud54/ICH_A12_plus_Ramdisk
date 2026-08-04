use tauri::{Manager, Window};
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::get_device_info,
            commands::boot_ramdisk,
            commands::backup_files,
            commands::run_custom_cmd
        ])
        .setup(|app| {
            let window = app.get_window("main");
            if let Some(w) = window {
                w.set_title("DienThoai88 Ramdisk Tool").ok();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
