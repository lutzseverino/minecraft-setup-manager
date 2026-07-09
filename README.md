<div align="center">
    <h1 align="center">Minecraft Setup Manager</h1>
    <p>A desktop app that prepares and updates a player's Minecraft client from a server-provided setup manifest.</p>
    <p>
        <img alt="desktop" src="https://img.shields.io/badge/desktop-tauri-0f172a">
        <img alt="frontend" src="https://img.shields.io/badge/frontend-react-1f2937">
        <img alt="ui" src="https://img.shields.io/badge/ui-shadcn-374151">
        <img alt="language" src="https://img.shields.io/badge/language-typescript%2Frust-4b5563">
    </p>
</div>

## Overview

Minecraft Setup Manager is a server-agnostic desktop tool. Players enter a Minecraft server address,
the app discovers a declarative setup manifest for that server, previews the requested setup, and
then applies or checks the local client setup.

The app is designed around desired state rather than arbitrary scripts. A server can ask for a
Minecraft version, loader, managed resources, launcher profile, and server entry. The app owns how
those requests are safely applied.

Current implementation:

- Saves server entries in app state.
- Discovers manifests from `https://<host>/.well-known/minecraft-setup-manager/manifest.json`.
- Parses and fingerprints setup manifests.
- Rejects unsupported, ambiguous, or unsafe manifest data before saving it.
- Caches the reviewed manifest and binds plan/apply requests to its fingerprint.
- Uses server-defined setup profiles and explicit resource membership.
- Builds setup plans from saved manifests.
- Creates the managed game folder and setup receipt.
- Installs version-pinned Fabric launcher profiles from the live official Fabric Meta API.
- Creates or updates the official Minecraft Launcher profile.
- Downloads direct resources with size limits and SHA-256/SHA-512 verification.
- Resolves pinned Modrinth versions and verifies project, compatibility, file, and hash metadata.
- Adds or updates optional `servers.dat` entries while preserving unrelated NBT data.
- Verifies loader, profile, receipt, server entry, and every managed file before recording success.
- Removes stale managed resources when their recorded file is still safe to manage.
- Validates the local setup slice.
- Detects the browser language and ships English and Spanish UI copy.
- Builds natively on macOS, Windows, and Linux through the CI matrix.

Not implemented yet:

- SKlauncher and manual launcher profile adapters.
- Signed manifests.
- Plugin-assisted server compatibility checks.
- Code signing, macOS notarization, and release publishing.

## Getting Started

Install dependencies:

```bash
npm install
```

Run the Vite frontend in a browser:

```bash
npm run dev
```

Run the desktop app in development:

```bash
npm run tauri:dev
```

Build production frontend and desktop bundles:

```bash
npm run build
npm run tauri:build
```

## Manifest Discovery

For a server address such as:

```text
play.example.com
```

the app currently checks:

```text
https://play.example.com/.well-known/minecraft-setup-manager/manifest.json
```

Direct manifest URLs are also accepted by the resolver.

Server-specific manifests should live outside this generic app repository.
Planning and installation use the validated snapshot that the player reviewed;
they do not refetch the URL. If a later check finds different manifest bytes,
the player must review the new steps before applying them.
Profiles and their resource sets are also manifest data: each resource can list
the profile IDs that should receive it. Resources without a profile list apply
to every profile.

Schema version 1 is strict. Unknown fields, unsafe path names, duplicate IDs or
overlapping destination files, malformed hashes, invalid profile references,
and inconsistent resource targets are rejected. Direct downloads require
public HTTPS and a pinned SHA-256 or SHA-512 hash. Loopback HTTP is accepted only
when the manifest itself is loaded from the local computer for development.

## Supported Setup

The current installer supports the normal Minecraft Launcher with either
vanilla Minecraft or a pinned Fabric loader. Server manifests can select pinned
Modrinth resources, hash-pinned direct files, an isolated game folder, and an
optional multiplayer server entry.

SKlauncher and manual setup are visible but disabled until their launcher
adapters can produce and validate complete profiles. The backend also refuses
plans containing any unsupported action, so a partial setup cannot be reported
as complete.

## Safety Model

- Manifests describe desired state; they cannot contain commands or scripts.
- The app applies the exact validated manifest fingerprint that the player reviewed.
- Every server gets a game folder namespaced by its stable saved-server identity,
  so two servers cannot collide by choosing the same display folder name.
- Direct downloads require HTTPS and a pinned SHA-256 or SHA-512 hash.
- Modrinth files are checked against the requested project, version, Minecraft
  version, loader, CDN, size, and SHA-512 metadata.
- Managed writes use isolated folders, path and symlink checks, bounded downloads,
  temporary files, and atomic replacement where the target format permits it.
- Existing managed files are removed only while they still match the hash the app
  recorded. Existing files are replaced only when the app can prove it owns the
  previous version. User-modified and unowned files are preserved.
- Setup downloads have a 512 MB per-file limit and a 2 GB total limit per run.
- The preview names the direct download host or exact Modrinth project/version.
- Installed state advances only after a full local conformance check passes.

Individual writes are atomic or backed up, but the full multi-step plan is not a
single filesystem transaction. A failed run keeps the previous installed-state
record and can be run again to repair the incomplete desired state.

A matching hash proves that a download matches what the server requested; it
does not prove that a mod is harmless. Mods run inside Minecraft. Until manifest
signing or trust pinning is implemented, only use setup files from servers you
already trust.

## Architecture

- `src/hooks` owns wizard orchestration, command state, and lifecycle resets.
- `src/screens` owns presentational wizard composition.
- `src/components/ui` owns foundational shadcn/Radix-style primitives.
- `src/components/app` owns composed app UI pieces.
- `src/lib/tauri.ts` is the only frontend module that imports Tauri APIs.
- `src-tauri/src/commands` owns thin Tauri command handlers and DTOs.
- `src-tauri/src/app_state` owns saved servers and global update state.
- `src-tauri/src/server` owns address normalization and manifest discovery.
- `src-tauri/src/manifest` owns manifest schema, fetching, and fingerprinting.
- `src-tauri/src/launcher` owns launcher adapter boundaries and launcher profile writes.
- `src-tauri/src/minecraft` owns local install preparation, validation, and future Minecraft file work.
- `src-tauri/src/system` owns platform path helpers.

React never edits files or knows launcher file formats. Rust owns filesystem, launcher, network,
manifest, and validation behavior.

## Quality Checks

```bash
npm run typecheck
npm run check:architecture
npm run build
cd src-tauri && cargo fmt --check && cargo test && cargo check
```

The full local release gate is:

```bash
npm run tauri:build
```

GitHub Actions runs the frontend checks, Rust tests and lints, and a native
Tauri bundle build on macOS, Windows, and Linux. A successful build on one
operating system does not by itself validate the other two.

Local bundles are suitable for controlled testing. Public downloads should wait
for platform code signing and a release workflow so users can verify the
publisher and avoid operating-system security warnings.

## Links

- [Architecture notes](docs/architecture.md)
- [Setup manifest guide](docs/manifest.md)
- [Implementation roadmap](docs/implementation-roadmap.md)
