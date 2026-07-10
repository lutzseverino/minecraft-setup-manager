# Validate SKlauncher Compatibility

Validate the stable SKlauncher profile contract without exposing a personal
Minecraft workspace or account.

The validation uses an ephemeral Linux virtual machine as its security boundary.
Inside the VM, a hardened container runs the checksum-pinned SKlauncher JAR in
demo mode against a disposable working directory. A feature-gated probe calls
the production SKlauncher adapter before and after the launcher reloads that
directory.

## Steps

1. Create an ephemeral VM with its own kernel. Do not share host folders,
   credentials, the Docker socket, display devices, or personal launcher data.
2. Give the VM outbound internet access while denying access to the host, local
   networks, and cloud metadata addresses. Do not expose inbound guest ports
   beyond a loopback-bound SSH forwarding port used for provisioning.
3. On an Ubuntu KVM host with QEMU and `cloud-image-utils`, create the pinned VM:

   ```bash
   SKLAUNCHER_VM_STATE_DIR=/path/on-a-large-volume/sklauncher-validation \
   SKLAUNCHER_VM_CONTROL_DIR="$HOME/.local/state/sklauncher-validation" \
     ./validation/sklauncher/create-vm.sh
   ```

   The script creates a dedicated unprivileged QEMU account, applies temporary
   per-user egress rules that reject loopback, private, link-local, and IPv6
   private destinations, and verifies both public access and private-network
   denial before reporting success.
4. Install Docker, the Rust toolchain, and the Linux Tauri build prerequisites
   inside the VM. The provided cloud-init configuration performs this step.
5. Copy a clean source checkout into the VM. Build the validation probe:

   ```bash
   cargo build \
     --manifest-path src-tauri/Cargo.toml \
     --features validation-tools \
     --bin sklauncher-contract-probe
   ```

6. Run the round trip from the repository root:

   ```bash
   SKLAUNCHER_CONTRACT_PROBE="$PWD/src-tauri/target/debug/sklauncher-contract-probe" \
   SKLAUNCHER_VALIDATION_STATE_DIR=/opt/sklauncher-validation \
     ./validation/sklauncher/run-roundtrip.sh
   ```

The runner downloads only the pinned stable JAR, verifies its SHA-256 digest,
builds the pinned Java 21 container, starts SKlauncher with `--demo` and an
explicit `--workDir`, invokes the real adapter, restarts the launcher, and then
checks the profile again.

The container keeps its root filesystem read-only. Its temporary filesystem is
executable only because JavaFX extracts native renderer libraries there; that
filesystem exists solely inside the capability-dropped disposable container and
disappears after each launcher session.

The probe refuses to run unless `MSM_SKLAUNCHER_VALIDATION=1` is set and its
temporary home contains `.minecraft-setup-manager-sklauncher-sandbox`. Those
guards prevent accidental use against a normal Minecraft directory.

An optional visual follow-up may sign in with a disposable offline username
and inspect the installation list in the same isolated workspace. Do not use a
Microsoft account for this check. Confirm that the `Minecraft Setup Manager`
entry is visible, select it, and verify that the launch control reports the
expected version. Treat this as human-reviewed evidence: launcher screen
coordinates and presentation are not a stable integration contract.

## Verification

A successful run prints `SKlauncher round-trip validation passed` and writes:

- `bootstrap.png` and `reload.png`, showing both launcher sessions
- launcher, Xvfb, and window-manager logs for each session
- `probe-write.json`, identifying the created profile and backup
- `probe-verify.json`, confirming exact directory and version matches,
  idempotence, preservation of unknown root fields, and whether SKlauncher
  normalized unknown fields inside the profile object
- `run.json`, recording the SKlauncher version, JAR checksum, and container base

If the optional offline check is performed, capture the installation list and
selected profile, then rerun the probe's `verify` phase after closing the
launcher. Do not retain a personal username or launcher logs from an account
session in repository evidence.

Destroy the VM after copying out this non-secret evidence. A Linux round trip
validates the shared SKlauncher 3.2 profile-store contract; it does not validate
SKlauncher 4.0 or claim native launcher execution on Windows and macOS.

```bash
SKLAUNCHER_VM_STATE_DIR=/path/on-a-large-volume/sklauncher-validation \
SKLAUNCHER_VM_CONTROL_DIR="$HOME/.local/state/sklauncher-validation" \
  ./validation/sklauncher/destroy-vm.sh
```
