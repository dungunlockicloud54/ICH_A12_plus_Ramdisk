#!/bin/bash
# Minimal backup script that copies over files reachable via ssh on the ramdisk.
# This is a helper placeholder. The actual backup method depends on the ramdisk exposing SSH or a mount.
# Usage: backup_active_files.sh <UDID> <DEST_FOLDER> [extra]

set -e
UDID="$1"
DEST="$2"
shift 2
EXTRA="$@"

if [ -z "$UDID" ] || [ -z "$DEST" ]; then
  echo "Usage: $0 <UDID> <DEST_FOLDER>"
  exit 1
fi

mkdir -p "$DEST"

# Example: if the ramdisk exposes ssh on port 22 with a known user, we could scp. This is a placeholder.
# Try scp from root@<device_ip>:/path/to/important /dest
# For now, write a small report file
REPORT="$DEST/backup_report_${UDID}_$(date +%Y%m%d_%H%M%S).txt"
{
  echo "Backup placeholder"
  echo "UDID: $UDID"
  echo "Destination: $DEST"
  echo "Extra args: $EXTRA"
  echo "Make sure to adapt backup_active_files.sh to copy files from your ramdisk after it's booted and SSH is available."
} > "$REPORT"

echo "Wrote placeholder report to $REPORT"
