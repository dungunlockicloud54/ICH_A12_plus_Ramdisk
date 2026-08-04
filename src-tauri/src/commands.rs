use serde::Serialize;
use std::process::{Command, Stdio};
use anyhow::Result;
use tauri::api::process::CommandChild;
use tauri::Window;
use std::io::{BufRead, BufReader};
use std::thread;

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
pub fn boot_ramdisk(window: Window, udid: String, chip: String, extra_args: Option<String>) -> Result<String, String> {
    // Calls the existing ich_a12_plus_ramdisk binary in repo root. The exact CLI flags depend on your local binary.
    let mut cmd = Command::new("./ich_a12_plus_ramdisk");
    cmd.arg("--udid").arg(&udid).arg("--chip").arg(&chip);
    if let Some(args) = extra_args {
        for token in args.split_whitespace() {
            cmd.arg(token);
        }
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            let mut stdout = child.stdout.take();
            let mut stderr = child.stderr.take();
            let w1 = window.clone();
            // stdout thread
            if let Some(out) = stdout {
                thread::spawn(move || {
                    let reader = BufReader::new(out);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            let _ = w1.emit("boot-log", l);
                        }
                    }
                });
            }
            // stderr thread
            let w2 = window.clone();
            if let Some(err) = stderr {
                thread::spawn(move || {
                    let reader = BufReader::new(err);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            let _ = w2.emit("boot-log", format!("[stderr] {}", l));
                        }
                    }
                });
            }

            // spawn a thread to wait for exit and emit finished event
            let w3 = window.clone();
            thread::spawn(move || {
                match child.wait() {
                    Ok(status) => {
                        let _ = w3.emit("boot-finished", format!("exit:{}", status.code().unwrap_or(-1)));
                    }
                    Err(e) => {
                        let _ = w3.emit("boot-finished", format!("error:{}", e));
                    }
                }
            });

            Ok("started".to_string())
        }
        Err(e) => Err(format!("Failed to execute ich_a12_plus_ramdisk: {}", e))
    }
}

#[tauri::command]
pub fn backup_files(udid: String, dest_folder: String, host: String, port: u16, user: String, password: String, overwrite: bool) -> Result<String, String> {
    // Calls the Python backup helper script gui/scripts/backup_active_files.py
    let mut cmd = Command::new("python3");
    cmd.arg("./gui/scripts/backup_active_files.py");
    cmd.arg(&udid).arg(&dest_folder).arg(&host).arg(port.to_string()).arg(&user).arg(&password).arg(if overwrite {"yes"} else {"no"});
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
    match Command::new("sh").arg("-c").arg(&cmdline).output() {
        Ok(out) => Ok(format!("{}\n---stderr---\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))),
        Err(e) => Err(format!("Failed to run command: {}", e))
    }
}
