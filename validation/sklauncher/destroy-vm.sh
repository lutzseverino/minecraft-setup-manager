#!/bin/sh
set -eu

state_dir=${SKLAUNCHER_VM_STATE_DIR:-/var/lib/minecraft-setup-manager/sklauncher-validation}
control_dir=${SKLAUNCHER_VM_CONTROL_DIR:-$HOME/.local/state/minecraft-setup-manager/sklauncher-validation}
firewall_table=msm_sklauncher_validation
vm_user=msm-sklauncher-vm

case "$state_dir" in
  /|/var|/var/lib|/srv|/srv/homelab) echo "Refusing unsafe VM state directory: $state_dir" >&2; exit 2 ;;
esac
case "$control_dir" in
  /|"$HOME") echo "Refusing unsafe VM control directory: $control_dir" >&2; exit 2 ;;
esac

if id "$vm_user" >/dev/null 2>&1; then
  sudo -n pkill -TERM -u "$vm_user" 2>/dev/null || true
  attempt=0
  while sudo -n pgrep -u "$vm_user" >/dev/null 2>&1 && [ "$attempt" -lt 30 ]; do
    attempt=$((attempt + 1))
    sleep 1
  done
  sudo -n pkill -KILL -u "$vm_user" 2>/dev/null || true
  sleep 1
  if sudo -n pgrep -u "$vm_user" >/dev/null 2>&1; then
    echo "Validation processes are still running; containment was left in place." >&2
    exit 1
  fi
fi

sudo -n nft list table inet "$firewall_table" >/dev/null 2>&1 \
  && sudo -n nft delete table inet "$firewall_table"

sudo -n find "$state_dir" -mindepth 1 -delete 2>/dev/null || true
sudo -n rmdir "$state_dir" 2>/dev/null || true
find "$control_dir" -mindepth 1 -delete 2>/dev/null || true
rmdir "$control_dir" 2>/dev/null || true

if id "$vm_user" >/dev/null 2>&1; then
  sudo -n userdel "$vm_user"
fi

echo "SKlauncher validation VM, firewall rules, disks, system account, and ephemeral SSH key were removed."
