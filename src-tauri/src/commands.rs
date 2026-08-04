use serde::Serialize;
use std::process::{Command, Stdio};
use anyhow::Result;
use tauri::api::process::CommandChild;

#[derive(Serialize)]
pub struct Device { pub udid: String }

// List connected devices using `idevice_id -l` (libimobiledevice)
#[tauri::command]
pub fn list_devices() -> Result<Vec<Device>, String> {
    match Command::new("idevice_id").arg("-l").output() {
        Ok(out) => {
            if !out.status.success() {
                return Err(format!("idevice_id failed: {}", String::from_utf8_lossy(&out.stderr)));
            }
            let s = String::from_utf8_lossy(&out.stdout);
            let devices: Vec<Device> = s.lines().filter(|l| !l.trim().is_empty()).map(|l| Device{udid:l.trim().to_string()}).collect();
            Ok(devices)
        }
        Err(e) => Err(format!("Failed to run idevice_id: {}. Install libimobiledevice via Homebrew.", e))
    }
}

#[tauri::command]
pub fn get_device_info(udid: String) -> Result<String, String> {
    // use ideviceinfo -u <udid>
    match Command::new("ideviceinfo").arg("-u").arg(&udid).output() {
        Ok(out) => {
            if !out.status.success() {
                return Err(format!("ideviceinfo failed: {}", String::from_utf8_lossy(&out.stderr)));
            }
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        }
        Err(e) => Err(format!("Failed to run ideviceinfo: {}", e))
    }
}

#[tauri::command]
pub fn boot_ramdisk(udid: String, chip: String, extra_args: Option<String>) -> Result<String, String> {
    // Calls the existing ich_a12_plus_ramdisk binary in repo root. The exact CLI flags depend on your local binary.
    // This function forwards parameters to the binary. Example usage from UI: extra_args="--enable-ssh"
    let mut cmd = Command::new("./ich_a12_plus_ramdisk");
    cmd.arg("--udid").arg(&udid).arg("--chip").arg(&chip);
    if let Some(args) = extra_args {
        // naive split — UI should provide a safe string
        for token in args.split_whitespace() {
            cmd.arg(token);
        }
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.output() {
        Ok(out) => {
            let mut combined = String::new();
            combined.push_str(&String::from_utf8_lossy(&out.stdout));
            combined.push_str("\n---stderr---\n");
            combined.push_str(&String::from_utf8_lossy(&out.stderr));
            Ok(combined)
        }
        Err(e) => Err(format!("Failed to execute ich_a12_plus_ramdisk: {}", e))
    }
}

#[tauri::command]
pub fn backup_files(udid: String, dest_folder: String, extra_args: Option<String>) -> Result<String, String> {
    // This calls a helper shell script in gui/scripts/backup_active_files.sh
    let mut script = String::from("./gui/scripts/backup_active_files.sh");
    // If the user prefers a different script, update UI configuration.
    let mut cmd = Command::new(&script);
    cmd.arg(&udid).arg(&dest_folder);
    if let Some(args) = extra_args {
        for token in args.split_whitespace() {
            cmd.arg(token);
        }
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    match cmd.output() {
        Ok(out) => Ok(format!("{}\n---stderr---\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))),
        Err(e) => Err(format!("Failed to run backup script: {}", e))
    }
}

#[tauri::command]
pub fn run_custom_cmd(cmdline: String) -> Result<String, String> {
    // Very generic: runs a shell command. Use with caution. UI should constrain usage.
    // We'll run via `sh -c` so the user can pass multiple tokens.
    match Command::new("sh").arg("-c").arg(&cmdline).output() {
        Ok(out) => Ok(format!("{}\n---stderr---\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))),
        Err(e) => Err(format!("Failed to run command: {}", e))
    }
}
