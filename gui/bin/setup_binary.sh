#!/usr/bin/env bash
# Simple convenience script to make the ich_a12_plus_ramdisk binary executable
if [ -f ./ich_a12_plus_ramdisk ]; then
  chmod +x ./ich_a12_plus_ramdisk
  echo "Set executable bit on ./ich_a12_plus_ramdisk"
else
  echo "Binary ./ich_a12_plus_ramdisk not found in repo root. Place it there and rerun."
fi
