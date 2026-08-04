use serde::Serialize;
use std::process::{Command, Stdio};
use anyhow::Result;
use tauri::Window;
use std::io::{BufRead, BufReader};
use std::thread;
use std::fs;
use tauri::api::path::app_config_dir;
use keyring::Keyring;

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
pub fn save_ssh_password(host: String, port: u16, user: String, password: String) -> Result<String, String> {
    let key = format!("ssh:{}:{}:{}", host, port, user);
    let kr = Keyring::new("DienThoai88 Ramdisk Tool", &key);
    match kr.set_password(&password) {
        Ok(()) => Ok("saved".to_string()),
        Err(e) => Err(format!("failed to save keychain entry: {}", e))
    }
}

#[tauri::command]
pub fn get_ssh_password(host: String, port: u16, user: String) -> Result<Option<String>, String> {
    let key = format!("ssh:{}:{}:{}", host, port, user);
    let kr = Keyring::new("DienThoai88 Ramdisk Tool", &key);
    match kr.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("failed to read keychain entry: {}", e))
    }
}

#[tauri::command]
pub fn delete_ssh_password(host: String, port: u16, user: String) -> Result<String, String> {
    let key = format!("ssh:{}:{}:{}", host, port, user);
    let kr = Keyring::new("DienThoai88 Ramdisk Tool", &key);
    match kr.delete_password() {
        Ok(()) => Ok("deleted".to_string()),
        Err(e) => Err(format!("failed to delete keychain entry: {}", e))
    }
}

#[tauri::command]
pub fn backup_files(udid: String, dest_folder: String, host: String, port: u16, user: String, password: Option<String>, use_keychain: bool, overwrite: bool) -> Result<String, String> {
    // Calls the Python backup helper script gui/scripts/backup_active_files.py
    let mut final_password = None;
    if use_keychain {
        // try retrieve
        match get_ssh_password(host.clone(), port, user.clone()) {
            Ok(opt) => { final_password = opt; }
            Err(e) => return Err(format!("Failed to read password from keychain: {}", e))
        }
    } else {
        final_password = password;
    }

    let pwd_for_arg = final_password.unwrap_or_else(|| "".to_string());

    let mut cmd = Command::new("python3");
    cmd.arg("./gui/scripts/backup_active_files.py");
    cmd.arg(&udid).arg(&dest_folder).arg(&host).arg(port.to_string()).arg(&user).arg(&pwd_for_arg).arg(if overwrite {"yes"} else {"no"});
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    match cmd.output() {
        Ok(out) => Ok(format!("{}\n---stderr---\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))),
        Err(e) => Err(format!("Failed to run backup script: {}", e))
    }
}

#[tauri::command]
pub fn save_config(config_json: String) -> Result<String, String> {
    match app_config_dir() {
        Some(mut path) => {
            path.push("DienThoai88");
            if let Err(e) = fs::create_dir_all(&path) {
                return Err(format!("Failed to create config dir: {}", e));
            }
            path.push("config.json");
            if let Err(e) = fs::write(&path, config_json) {
                return Err(format!("Failed to write config: {}", e));
            }
            Ok(format!("wrote {}", path.display()))
        }
        None => Err("Could not determine app config directory".to_string())
    }
}

#[tauri::command]
pub fn load_config() -> Result<Option<String>, String> {
    match app_config_dir() {
        Some(mut path) => {
            path.push("DienThoai88");
            path.push("config.json");
            match fs::read_to_string(&path) {
                Ok(s) => Ok(Some(s)),
                Err(_) => Ok(None)
            }
        }
        None => Err("Could not determine app config directory".to_string())
    }
}
