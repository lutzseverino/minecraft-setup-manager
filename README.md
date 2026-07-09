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
- Installs manifest-pinned Fabric launcher profiles from the official Fabric Meta API.
- Creates or updates the official Minecraft Launcher profile.
- Downloads direct resources with size limits and SHA-256/SHA-512 verification.
- Resolves pinned Modrinth versions and verifies project, compatibility, file, and hash metadata.
- Removes stale managed resources when their recorded file is still safe to manage.
- Validates the local setup slice.

Not implemented yet:

- `servers.dat` writes.
- Signed manifests.
- Plugin-assisted server compatibility checks.

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

## Architecture

- `src/screens` owns wizard composition and UI state.
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

## Links

- [Architecture notes](docs/architecture.md)
- [Setup manifest guide](docs/manifest.md)
- [Implementation roadmap](docs/implementation-roadmap.md)
