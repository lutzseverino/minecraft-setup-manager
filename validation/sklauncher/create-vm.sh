#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$script_dir/versions.env"

state_dir=${SKLAUNCHER_VM_STATE_DIR:-/var/lib/minecraft-setup-manager/sklauncher-validation}
control_dir=${SKLAUNCHER_VM_CONTROL_DIR:-$HOME/.local/state/minecraft-setup-manager/sklauncher-validation}
ssh_port=${SKLAUNCHER_VM_SSH_PORT:-22222}
vm_user=msm-sklauncher-vm
firewall_table=msm_sklauncher_validation

case "$state_dir" in
  /|/var|/var/lib|/srv|/srv/homelab) echo "Refusing unsafe VM state directory: $state_dir" >&2; exit 2 ;;
esac
case "$control_dir" in
  /|"$HOME") echo "Refusing unsafe VM control directory: $control_dir" >&2; exit 2 ;;
esac

for command in cloud-localds curl nft qemu-img qemu-system-x86_64 ssh ssh-keygen; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "Required command is unavailable: $command" >&2
    exit 2
  }
done

if [ -f "$state_dir/vm.pid" ] && sudo -n kill -0 "$(cat "$state_dir/vm.pid")" 2>/dev/null; then
  echo "SKlauncher validation VM is already running." >&2
  exit 2
fi

mkdir -p "$control_dir"
chmod 0700 "$control_dir"
if [ ! -f "$control_dir/id_ed25519" ]; then
  ssh-keygen -q -t ed25519 -N '' -C sklauncher-validation -f "$control_dir/id_ed25519"
fi
public_key=$(cat "$control_dir/id_ed25519.pub")

if ! id "$vm_user" >/dev/null 2>&1; then
  sudo -n useradd --system --no-create-home --home-dir /nonexistent \
    --shell /usr/sbin/nologin --groups kvm "$vm_user"
fi
vm_uid=$(id -u "$vm_user")

sudo -n install -d -m 0700 -o "$vm_user" -g "$vm_user" "$state_dir"
base_image="$state_dir/ubuntu-26.04-server-cloudimg-amd64.img"
disk_image="$state_dir/validation.qcow2"
seed_image="$state_dir/seed.img"

if [ ! -f "$base_image" ]; then
  sudo -n -u "$vm_user" curl --fail --location --proto '=https' --tlsv1.2 \
    --output "$base_image.part" "$UBUNTU_IMAGE_URL"
  sudo -n -u "$vm_user" mv "$base_image.part" "$base_image"
fi
echo "$UBUNTU_IMAGE_SHA256  $base_image" | sudo -n -u "$vm_user" sha256sum --check --strict

rm -f "$control_dir/user-data" "$control_dir/meta-data"
cat >"$control_dir/user-data" <<EOF
#cloud-config
users:
  - name: validator
    uid: 1000
    groups: [adm, docker, sudo]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - $public_key
ssh_pwauth: false
disable_root: true
package_update: true
packages:
  - build-essential
  - ca-certificates
  - cargo
  - curl
  - docker.io
  - git
  - libappindicator3-dev
  - librsvg2-dev
  - libwebkit2gtk-4.1-dev
  - patchelf
  - pkg-config
  - rustc
runcmd:
  - [systemctl, enable, --now, docker]
  - [touch, /var/lib/cloud/instance/sklauncher-validation-ready]
EOF
cat >"$control_dir/meta-data" <<EOF
instance-id: minecraft-setup-manager-sklauncher-validation
local-hostname: msm-sklauncher-validation
EOF

sudo -n -u "$vm_user" rm -f "$disk_image" "$seed_image" "$state_dir/serial.log" "$state_dir/vm.pid"
sudo -n -u "$vm_user" qemu-img create -q -f qcow2 -F qcow2 -b "$base_image" "$disk_image" 30G
sudo -n cloud-localds "$seed_image" "$control_dir/user-data" "$control_dir/meta-data"
sudo -n chown "$vm_user:$vm_user" "$seed_image"

sudo -n nft list table inet "$firewall_table" >/dev/null 2>&1 \
  && sudo -n nft delete table inet "$firewall_table"
sudo -n nft -f - <<EOF
table inet $firewall_table {
  chain output {
    type filter hook output priority -10; policy accept;
    meta skuid $vm_uid ct state established,related accept
    meta skuid $vm_uid ip daddr 127.0.0.53 meta l4proto { tcp, udp } th dport 53 accept
    meta skuid $vm_uid ip daddr { 127.0.0.0/8, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16 } reject
    meta skuid $vm_uid ip6 daddr { ::1/128, fc00::/7, fe80::/10 } reject
  }
}
EOF

sudo -n -u "$vm_user" qemu-system-x86_64 \
  -name msm-sklauncher-validation \
  -machine q35,accel=kvm \
  -cpu host \
  -smp 4 \
  -m 6144 \
  -drive "file=$disk_image,if=virtio,format=qcow2,cache=none" \
  -drive "file=$seed_image,if=virtio,format=raw,readonly=on" \
  -device virtio-rng-pci \
  -netdev "user,id=net0,ipv6=off,hostfwd=tcp:127.0.0.1:$ssh_port-:22" \
  -device virtio-net-pci,netdev=net0 \
  -display none \
  -serial "file:$state_dir/serial.log" \
  -daemonize \
  -pidfile "$state_dir/vm.pid"

ssh_args="-i $control_dir/id_ed25519 -p $ssh_port -o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$control_dir/known_hosts"
attempt=0
until ssh $ssh_args validator@127.0.0.1 'cloud-init status --wait >/dev/null'; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 120 ]; then
    echo "VM did not finish cloud-init within ten minutes. See $state_dir/serial.log" >&2
    exit 1
  fi
  sleep 5
done

ssh $ssh_args validator@127.0.0.1 'curl -fsS --max-time 15 https://example.com >/dev/null'
if ssh $ssh_args validator@127.0.0.1 'curl -fsS --connect-timeout 3 http://192.168.0.2 >/dev/null 2>&1'; then
  echo "VM unexpectedly reached the private homelab network." >&2
  exit 1
fi

echo "SKlauncher validation VM is ready on host loopback port $ssh_port."
echo "SSH: ssh $ssh_args validator@127.0.0.1"
