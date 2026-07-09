# Architecture

Maresme MC Setup is scaffolded as a Tauri 2 desktop app with a Vite, React, and
TypeScript frontend.

## Responsibility Boundaries

- `src/screens/` owns wizard composition and UI state.
- `src/components/ui/` owns foundational shadcn/Radix-style primitives.
- `src/components/app/` owns Maresme-specific composed UI pieces.
- `src/config/` owns setup choices that may vary by server, such as the active
  server catalog, launcher display metadata, wizard steps, and performance
  options.
- `src/lib/types.ts` defines frontend command contracts.
- `src/lib/tauri.ts` is the only frontend module that imports Tauri APIs.
- `src-tauri/src/commands/` owns thin Tauri command handlers and shared DTOs.
- `src-tauri/src/launcher/` owns launcher adapter boundaries.
- `src-tauri/src/minecraft/` owns local install preparation, Fabric, Modrinth,
  server entry, and validation service modules.
- `src-tauri/src/system/` owns platform path helpers.
- `src-tauri/src/manifest/` owns manifest loading and install-plan decisions.
- `src-tauri/src/performance_profiles/` owns RAM/performance profile decisions.

## Dependency Direction

React screens call typed functions from `src/lib/tauri.ts`. They do not import
`@tauri-apps/api` directly and do not know filesystem paths or launcher profile
formats.

Server-specific facts flow from `src/config/server-catalog.ts` into the UI and
typed command request. This keeps the Maresme MC one-off path simple while
leaving one narrow place to add another Minecraft server later.

Tauri commands parse command input, call backend modules, and return DTOs. They
should stay thin. Real installer behavior should be implemented below command
handlers, behind service modules or launcher adapters.

Launcher adapters may inspect or update launcher-specific profile data. They
must not resolve Modrinth projects, choose Fabric files, or make manifest policy
decisions.

Minecraft modules may resolve downloads, hashes, Fabric installation, `servers.dat`,
and validation. They should not own launcher-specific profile mutation.

## Current Implementation State

The desktop app currently performs the first real local setup slice: it creates
the isolated game folder, creates the initial Minecraft subfolders, writes a
setup receipt, creates or updates the official Minecraft Launcher profile,
validates those local files and launcher settings, and exports a small report.

The browser-only Vite runtime uses deterministic fallback responses because it
cannot access the user's filesystem. Fabric download, Modrinth file resolution,
SKlauncher profile writes, and `servers.dat` writes remain backend-owned modules
with adapter boundaries.
