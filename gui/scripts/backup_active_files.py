#!/usr/bin/env python3
"""
Backup helper that connects to the ramdisk SSH server and downloads selected paths via SFTP.
Usage:
  backup_active_files.py <UDID> <DEST_FOLDER> <HOST> <PORT> <USER> <PASSWORD> <OVERWRITE_yes_no>

Defaults in UI: host=127.0.0.1, port=2222, user=root, password=alpine

Requires: paramiko (pip3 install paramiko)

This script will attempt to download a set of common directories. Adjust PATHS_TO_PULL to your needs.
"""
import sys
import os
import stat

try:
    import paramiko
except ImportError:
    print("Missing dependency 'paramiko'. Install with: pip3 install paramiko")
    sys.exit(2)

if len(sys.argv) < 8:
    print("Usage: backup_active_files.py <UDID> <DEST_FOLDER> <HOST> <PORT> <USER> <PASSWORD> <OVERWRITE_yes_no>")
    sys.exit(1)

UDID = sys.argv[1]
DEST = sys.argv[2]
HOST = sys.argv[3]
PORT = int(sys.argv[4])
USER = sys.argv[5]
PASSWORD = sys.argv[6]
OVERWRITE = sys.argv[7].lower() in ("yes","y","true","1")

PATHS_TO_PULL = [
    '/var/mobile/Media',
    '/private/var/mobile',
]

os.makedirs(DEST, exist_ok=True)

report_lines = []
report_lines.append(f"Backup run for UDID={UDID}")
report_lines.append(f"Host={HOST}:{PORT} user={USER} overwrite={OVERWRITE}")

transport = paramiko.Transport((HOST, PORT))
try:
    transport.connect(username=USER, password=PASSWORD)
    sftp = paramiko.SFTPClient.from_transport(transport)

    def download_dir(sftp, remote_dir, local_dir):
        try:
            os.makedirs(local_dir, exist_ok=True)
            for entry in sftp.listdir_attr(remote_dir):
                rpath = remote_dir.rstrip('/') + '/' + entry.filename
                lpath = os.path.join(local_dir, entry.filename)
                if stat.S_ISDIR(entry.st_mode):
                    download_dir(sftp, rpath, lpath)
                else:
                    if os.path.exists(lpath) and not OVERWRITE:
                        report_lines.append(f"Skipping existing file: {lpath}")
                        continue
                    try:
                        sftp.get(rpath, lpath)
                        report_lines.append(f"Downloaded: {rpath} -> {lpath}")
                    except Exception as e:
                        report_lines.append(f"Failed to download {rpath}: {e}")
        except IOError as e:
            report_lines.append(f"Remote path not found: {remote_dir} ({e})")

    for p in PATHS_TO_PULL:
        local_target = os.path.join(DEST, UDID.replace(':','_'), p.strip('/').replace('/','_'))
        report_lines.append(f"Pulling {p} -> {local_target}")
        download_dir(sftp, p, local_target)

    sftp.close()
    transport.close()
    report_lines.append("Backup completed")
    report_path = os.path.join(DEST, f"backup_report_{UDID}_{os.getpid()}.txt")
    with open(report_path, 'w') as f:
        f.write('\n'.join(report_lines))
    print('\n'.join(report_lines))
    print(f"Report: {report_path}")
    sys.exit(0)
except Exception as e:
    print(f"SSH/SFTP error: {e}")
    try:
        transport.close()
    except:
        pass
    sys.exit(3)
