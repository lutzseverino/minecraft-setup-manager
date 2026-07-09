<div align="center">
    <h1 align="center">Maresme MC Setup</h1>
    <p>A desktop setup wizard that prepares a clean local Minecraft client folder for Maresme MC.</p>
    <p>
        <img alt="desktop" src="https://img.shields.io/badge/desktop-tauri-0f172a">
        <img alt="frontend" src="https://img.shields.io/badge/frontend-react-1f2937">
        <img alt="ui" src="https://img.shields.io/badge/ui-shadcn-374151">
        <img alt="language" src="https://img.shields.io/badge/language-typescript%2Frust-4b5563">
    </p>
</div>

## Overview

Maresme MC Setup is a cross-platform desktop app for preparing a player's Minecraft client for the
Maresme MC server.

The current app walks players through launcher choice, performance profile choice, local setup,
validation, and report export. It creates an isolated game folder, creates the first Minecraft
subfolders, saves a setup receipt, updates the official Minecraft Launcher profile, and checks that
those local files and launcher settings exist.

It does not yet download Fabric, download mods, or write `servers.dat`. Those behaviors are kept
behind backend-owned module and adapter boundaries so they can be added without moving filesystem,
download, or launcher logic into the UI.

## App Target

- Tauri `2`
- Vite, React, and TypeScript
- Tailwind CSS `4`
- shadcn/Radix-style primitives
- lucide icons
- Rust backend command handlers
- dependency-cruiser architecture checks

## Getting Started

Install dependencies from the project root:

```bash
npm install
```

Run the Vite frontend in the browser:

```bash
npm run dev
```

Run the desktop app in development:

```bash
npm run tauri:dev
```

Build the production frontend and desktop bundles:

```bash
npm run build
npm run tauri:build
```

The macOS app and DMG are written to:

```text
src-tauri/target/release/bundle/macos/Maresme MC Setup.app
src-tauri/target/release/bundle/dmg/Maresme MC Setup_0.1.0_aarch64.dmg
```

## Local Setup

The desktop app currently creates the Maresme MC game folder under the platform application support
directory. On macOS, that is:

```text
~/Library/Application Support/Maresme MC Setup/Maresme MC
```

The first setup slice creates:

- `mods`
- `resourcepacks`
- `shaderpacks`
- `config`
- `maresme-mc-setup.json`

The receipt records the selected launcher, performance profile, server address, Minecraft version,
Fabric loader version, and required/optional mod lists.

For the official Minecraft Launcher, setup also creates or updates the `maresme-mc` launcher
profile so it points at that isolated folder and the configured Fabric version. Before editing
`launcher_profiles.json`, the app writes a timestamped backup next to the original file.

## Configuration

Maresme server data currently exists in two places:

- Frontend display catalog: [src/config/server-catalog.ts](src/config/server-catalog.ts)
- Backend manifest policy: [src-tauri/src/manifest/mod.rs](src-tauri/src/manifest/mod.rs)

The repository also contains the intended JSON manifest shape at
[src-tauri/manifest/maresme-client.json](src-tauri/manifest/maresme-client.json). The next
scalability step is making that validated manifest the single source of truth for server setup data.

## Architecture

The codebase keeps UI, command contracts, filesystem setup, launcher adapters, and Minecraft-specific
work separated.

- `src/screens` owns wizard composition and UI state.
- `src/components/ui` owns foundational shadcn/Radix-style primitives.
- `src/components/app` owns Maresme-specific composed UI pieces.
- `src/config` owns frontend setup choices and display metadata.
- `src/lib/tauri.ts` is the only frontend module that imports Tauri APIs.
- `src-tauri/src/commands` owns thin Tauri command handlers and shared DTOs.
- `src-tauri/src/launcher` owns launcher adapter boundaries and launcher profile writes.
- `src-tauri/src/minecraft` owns local install preparation, Fabric, Modrinth, server entry, and
  validation modules.
- `src-tauri/src/system` owns platform path helpers.
- `src-tauri/src/manifest` owns install-plan decisions.
- `src-tauri/src/performance_profiles` owns RAM/performance profile decisions.

React screens call typed functions from `src/lib/tauri.ts`. Rust owns filesystem, launcher,
Minecraft, and validation concerns.

## Quality Checks

```bash
npm run typecheck
npm run check:architecture
npm run build
cd src-tauri && cargo check
```

The full local release gate is:

```bash
npm run tauri:build
```

`npm run tauri:build` runs the frontend production build, compiles the Rust backend, and produces the
desktop bundles.

## Links

- [Architecture notes](docs/architecture.md)
- [Implementation roadmap](docs/implementation-roadmap.md)
- [Maresme client manifest draft](src-tauri/manifest/maresme-client.json)
