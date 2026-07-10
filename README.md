<div align="center">
  <h1>Minecraft Setup Manager</h1>
  <p>Prepare and update a Minecraft client from a server-provided setup file.</p>

  [![CI](https://github.com/lutzseverino/minecraft-setup-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/lutzseverino/minecraft-setup-manager/actions/workflows/ci.yml)
  [![Releases](https://img.shields.io/github/v/release/lutzseverino/minecraft-setup-manager?include_prereleases)](https://github.com/lutzseverino/minecraft-setup-manager/releases)
  [![License: MIT](https://img.shields.io/badge/license-MIT-2f3437)](LICENSE)
</div>

Minecraft Setup Manager is a server-agnostic desktop app for players. Enter a
Minecraft server address, review what that server needs, and let the app prepare
an isolated local setup. Open it again later to check for server-requested
updates.

> [!IMPORTANT]
> The current `0.x` line is a development release. Installers are not yet
> notarized or signed by a recognized publisher. Only load setup files from
> servers you trust.

## How It Works

The app consumes a strict declarative manifest describing the Minecraft version,
loader, resources, launcher profile, and multiplayer entry. It never accepts
scripts or shell commands from a server.

1. Add a server by its Minecraft address.
2. Review the requested setup and choose a supported launcher and profile.
3. Apply the plan and validate the resulting files.
4. Optionally send a setup code back to the server after validation passes.
5. Reopen the app later to review and apply updates.

Each saved server receives its own managed game directory. The app remembers the
exact manifest the player approved and will not silently apply a newer one.

## Download

Development installers for Apple Silicon and Intel macOS, Windows, and Linux are
available on the [GitHub Releases page](https://github.com/lutzseverino/minecraft-setup-manager/releases).
The normal Minecraft Launcher must already be installed.

Because builds are not yet notarized or publisher-signed, the operating system
may show a security warning. Do not bypass a warning for a file obtained outside
this repository.

## Server Contract

For `play.example.com`, the app discovers:

```text
https://play.example.com/.well-known/minecraft-setup-manager/manifest.json
```

The language-neutral schema, semantics, fingerprints, publication rules, and
optional setup attestation are owned by the
[Minecraft Setup Protocol](https://github.com/lutzseverino/minecraft-setup-protocol).
Server-specific manifests belong in server-owned infrastructure, not this app.

## Development

Install Node.js 22, npm 11.6.2, stable Rust, and the Tauri prerequisites, then:

```bash
git submodule update --init
npm ci
npm run tauri:dev
```

See [run the manager locally](docs/how-to/run-locally.md) for browser mode and
verification. CI checks TypeScript, dependency direction, documentation links,
synchronized versions, the production frontend build, Rust formatting, tests,
and strict Clippy.

## Documentation

Start with the [documentation index](docs/README.md). Documentation is organized
by reader intent so durable guidance has one predictable home.

## License

Minecraft Setup Manager is available under the [MIT License](LICENSE).
