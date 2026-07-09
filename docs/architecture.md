# Architecture

Minecraft Setup Manager is a Tauri 2 desktop app with a Vite, React, and
TypeScript frontend.

## Responsibility Boundaries

- `src/screens/` owns wizard composition and UI state.
- `src/components/ui/` owns foundational shadcn/Radix-style primitives.
- `src/components/app/` owns composed app UI pieces.
- `src/config/` owns generic setup UI options, such as launcher display metadata
  and performance profile choices.
- `src/lib/types.ts` defines frontend command contracts.
- `src/lib/tauri.ts` is the only frontend module that imports Tauri APIs.
- `src-tauri/src/commands/` owns thin Tauri command handlers and shared DTOs.
- `src-tauri/src/app_state/` owns saved servers and durable update state.
- `src-tauri/src/server/` owns server address normalization and manifest discovery.
- `src-tauri/src/manifest/` owns setup manifest schema, fetching, validation, and
  fingerprinting.
- `src-tauri/src/launcher/` owns launcher adapter boundaries.
- `src-tauri/src/minecraft/` owns local install preparation, Fabric, Modrinth,
  server entry, file repair, and validation modules.
- `src-tauri/src/system/` owns platform path helpers.
- `src-tauri/src/performance_profiles/` owns local RAM/performance profile decisions.

## Dependency Direction

React screens call typed functions from `src/lib/tauri.ts`. They do not import
`@tauri-apps/api` directly and do not know filesystem paths or launcher profile
formats.

Server-specific facts come from setup manifests. The frontend may display resolved
manifest data, but it must not carry server-specific catalogs.

Tauri commands parse command input, call backend modules, and return DTOs. They
should stay thin. Real installer behavior belongs below command handlers, behind
service modules or launcher adapters.

Launcher adapters may inspect or update launcher-specific profile data. They
must not resolve Modrinth projects, choose Fabric files, or make manifest policy
decisions.

Minecraft modules may resolve downloads, hashes, Fabric installation,
`servers.dat`, and validation. They should not own launcher-specific profile
mutation.

## Current Implementation State

The app currently resolves and stores server manifests, builds setup plans from
saved manifests, creates the isolated game folder, writes a setup receipt,
creates or updates the official Minecraft Launcher profile, validates those
local files and launcher settings, and exports a small report.

Fabric download/install, Modrinth file resolution, SKlauncher profile writes,
manifest signing, and `servers.dat` writes remain backend-owned modules with
adapter boundaries.
