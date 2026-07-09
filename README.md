<div align="center">
  <h1>Minecraft Setup Manager</h1>
  <p>Prepare and update a Minecraft client from a server-provided setup file.</p>

  [![CI](https://github.com/lutzseverino/minecraft-setup-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/lutzseverino/minecraft-setup-manager/actions/workflows/ci.yml)
  [![Releases](https://img.shields.io/github/v/release/lutzseverino/minecraft-setup-manager?include_prereleases)](https://github.com/lutzseverino/minecraft-setup-manager/releases)
  [![License: MIT](https://img.shields.io/badge/license-MIT-2f3437)](LICENSE)
</div>

Minecraft Setup Manager is a server-agnostic desktop app for players. Enter a
Minecraft server address, review what that server needs, and let the app prepare
an isolated local setup. Open it again later to check for additions, removals,
or other server-requested updates.

> [!IMPORTANT]
> The current `0.x` line is a development release. Its installers are not yet
> notarized or signed by a recognized publisher. Only load setup files from
> servers you trust.

## How It Works

The app uses a declarative manifest: a data file that describes the desired
Minecraft version, loader, resources, launcher profile, and multiplayer entry.
It never accepts scripts or shell commands from a server.

1. Add a server by its Minecraft address.
2. Review the setup the server is requesting.
3. Choose a supported launcher and setup profile.
4. Apply the plan and check the resulting files.
5. Reopen the app later to discover and review updates.

Each saved server receives its own managed game directory. The app remembers
the exact manifest the player approved and will not silently apply a newer one.

## Download

Development installers are available on the
[GitHub Releases page](https://github.com/lutzseverino/minecraft-setup-manager/releases).

| Platform | Package |
| --- | --- |
| macOS on Apple Silicon | `.dmg` for `aarch64` |
| macOS on Intel | `.dmg` for `x86_64` |
| Windows | NSIS `.exe` installer |
| Linux | `.AppImage` and Debian `.deb` |

The normal Minecraft Launcher must already be installed. Run the Minecraft
version requested by your server once before applying a setup, so the launcher
has downloaded its base files.

Because the current builds are not notarized or publisher-signed, the operating
system may show a security warning. On macOS, you may need to approve the app in
**System Settings > Privacy & Security**. Do not bypass a warning for a file you
did not obtain from this repository.

## Supported Setup

| Capability | Status | Notes |
| --- | --- | --- |
| Official Minecraft Launcher | Supported | Creates and validates an isolated launcher profile |
| Vanilla Minecraft | Supported | Uses a pinned Minecraft version |
| Fabric | Supported | Installs a pinned loader profile from Fabric Meta |
| Modrinth resources | Supported | Pins project and version metadata and verifies the file |
| Direct resources | Supported | Requires public HTTPS and a SHA-256 or SHA-512 hash |
| Multiplayer server entry | Supported | Preserves unrelated entries in `servers.dat` |
| Setup updates | Supported | Previews additions, changes, and safe removals before applying them |
| SKlauncher and manual profiles | Planned | Disabled until complete adapters and validation exist |
| Signed manifests | Planned | Trust currently comes from the server address and the player's review |

The backend rejects unsupported plan actions. A partial install cannot be
reported as complete.

## Server Manifests

For a server such as `play.example.com`, the app discovers:

```text
https://play.example.com/.well-known/minecraft-setup-manager/manifest.json
```

A direct manifest URL can also be entered. Server-specific manifests belong in
the server owner's infrastructure or configuration repository, not in this
application repository.

Schema version 1 is strict. It rejects unknown fields, unsafe paths, duplicate
or overlapping resources, malformed hashes, invalid profile references, and
inconsistent targets. Resources may be shared by every setup profile or assigned
to named profiles such as low-power and high-quality configurations.

See the [manifest guide](docs/manifest.md) for the complete format and an example.

## Safety Model

Minecraft mods run inside Minecraft and should be treated as software. A valid
hash proves that a downloaded file matches what the server requested; it does
not prove that the file is harmless.

The installer limits each file to 512 MB and each run to 2 GB. It validates
paths and symlinks, downloads into temporary files, checks content hashes, and
uses atomic replacement where the target format permits it. Existing files are
only replaced or removed when the app can prove ownership of the recorded
version. User-modified and unowned files are preserved.

Individual writes are atomic or backed up, but a complete multi-step plan is not
yet one filesystem transaction. If a run fails, the previous installed-state
record remains in place and the same plan can be run again to repair the setup.

## Development

You need Node.js 22, npm 11.6.2, the stable Rust toolchain, and the
[Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/) for your
operating system.

```bash
npm ci
npm run tauri:dev
```

Use `npm run dev` when you only need the browser UI. Browser mode supplies demo
command responses and does not modify Minecraft files.

Run the local quality gate with:

```bash
npm run typecheck
npm run check:architecture
npm run check:version
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

`npm run tauri:build` produces the native bundles for the current operating
system. GitHub Actions repeats the checks and bundle build on macOS, Windows,
and Linux.

## Architecture

The frontend is intentionally a client of typed Tauri commands. React does not
edit files, parse launcher formats, or perform setup networking. Rust owns those
effects, command handlers stay thin, and launchers implement adapter boundaries.

| Area | Ownership |
| --- | --- |
| `src/hooks` | Wizard orchestration and command lifecycle |
| `src/screens` | Presentational wizard screens |
| `src/components` | Reusable app and UI components |
| `src/lib/tauri.ts` | The frontend's only Tauri API boundary |
| `src-tauri/src/commands` | Thin command handlers and DTOs |
| `src-tauri/src/server` and `manifest` | Address resolution, fetching, validation, and fingerprints |
| `src-tauri/src/launcher` | Launcher detection and profile adapters |
| `src-tauri/src/minecraft` | Install planning, managed files, and conformance checks |
| `src-tauri/src/app_state` | Saved servers, reviewed manifests, and installed state |

The [architecture notes](docs/architecture.md) describe the dependency rules and
the [implementation roadmap](docs/implementation-roadmap.md) records the
remaining hardening work.

## Versioning And Releases

This application follows
[Romantic Versioning](https://romversioning.github.io/romver/), written as
`PROJECT.MAJOR.MINOR`. Project version `0` marks initial development, a major
increment represents an incompatible change within the same project, and a
minor increment represents a compatible feature or fix. Reaching a stable
product will move the project number to `1.0.0`.

Release tags use the form `v0.1.0`. A tag must match the version in `package.json`,
Cargo, and Tauri configuration. Pushing a valid version tag runs the release
workflow and publishes native installers; `v0.*` tags are marked as development
releases automatically.

## License

Minecraft Setup Manager is available under the [MIT License](LICENSE).
