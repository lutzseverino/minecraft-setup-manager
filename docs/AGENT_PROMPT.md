# Agent Prompt

Initialize and scaffold the Maresme MC client setup app in this repository:

```text
/Users/lutzseverino/Documents/Projects/maresme-mc-setup
```

Goal:
Build the foundation for a cross-platform desktop setup app that prepares a
player's Minecraft client for the Maresme MC server. Do not implement the full
installer yet; set up the project so implementation can begin cleanly.

The app should eventually:

- Support the official Minecraft Launcher as the primary path.
- Support SKlauncher from skmedix.pl as a secondary path.
- Support manual/unknown launcher setup as a fallback.
- Create or repair a clean isolated `Maresme MC` game directory.
- Install Fabric for the target Minecraft version.
- Install pinned client mods from a manifest.
- Add the server entry.
- Validate the installation.
- Offer performance profiles: low-end, balanced, shaders.
- Avoid dumping mods into the user's global `.minecraft/mods` unless explicitly
  in advanced/manual mode.

Use Tauri + Vite + React + TypeScript.

Important frontend convention:
Copy the frontend tooling, folder style, shadcn setup, and general conventions
from:

```text
/Users/lutzseverino/Documents/Projects/polity/apps/landing
```

Specifically mirror/adapt:

- Vite + React + TypeScript setup.
- Tailwind CSS v4 via `@tailwindcss/vite`.
- shadcn setup using `components.json`.
- `lucide-react` for icons.
- `radix-ui`, `class-variance-authority`, `clsx`, `tailwind-merge`,
  `tw-animate-css`.
- `src/lib/utils.ts` with `cn(...)`.
- `src/components/ui/` for foundational shadcn components.
- `src/components/app/` for app-specific composed components.
- `src/pages/` or `src/screens/` for screen-level wizard composition.
- `@` alias to `/src`.
- TypeScript build scripts and structure.
- dependency-cruiser style architecture checks if practical.

Use a conventional, pure architecture. Keep responsibilities separated.

Suggested structure:

```text
maresme-mc-setup/
  package.json
  vite.config.ts
  components.json
  tsconfig.json
  tsconfig.app.json
  tsconfig.node.json
  dependency-cruiser.cjs
  src/
    main.tsx
    App.tsx
    index.css
    components/ui/
    components/app/
    screens/
      welcome-screen.tsx
      launcher-screen.tsx
      profile-screen.tsx
      install-screen.tsx
      done-screen.tsx
      diagnostics-screen.tsx
    lib/
      utils.ts
      tauri.ts
      types.ts
    i18n/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    src/main.rs
    src/commands/
    src/launcher/
      official_minecraft.rs
      sklauncher.rs
      manual.rs
    src/minecraft/
      fabric_installer.rs
      modrinth.rs
      servers_dat.rs
      validation.rs
    src/manifest/
    src/profiles/
```

Architecture rules:

- React frontend is only UI state, wizard flow, and calls to Tauri commands.
- Rust/Tauri backend owns filesystem operations, launcher detection, profile
  editing, downloads, hashing, validation, and diagnostics.
- Keep launcher-specific behavior behind launcher adapters.
- Keep Modrinth/Fabric/server-entry logic in Minecraft-specific backend modules.
- Keep install decisions manifest-driven.
- Do not let UI code know filesystem details beyond typed command results.
- Do not let launcher adapters own mod resolution logic.
- Keep commands thin: parse input, call domain/service functions, return typed
  results.

Initial manifest concept:

- Minecraft version: `26.1.2`
- Fabric loader: `0.19.3`
- Server name: `Maresme MC`
- Server address: configurable, default `localhost`
- Required mods:
  - Fabric API
  - Simple Voice Chat
  - Sodium
  - Lithium
  - ImmediatelyFast
- Balanced extras:
  - Sodium Extra
  - Dynamic FPS
  - Entity Culling
  - FerriteCore
  - Mod Menu
- Shaders extras:
  - Iris
  - Reese's Sodium Options

Frontend design:
Make it a real setup wizard, not a marketing landing page. First screen should
start the setup flow immediately. Use quiet, practical UI: status rows,
progress logs, clear action buttons, segmented choices for launcher/profile
selection, icons via lucide, and shadcn components where appropriate. Use
Spanish-ready/i18n-ready text structure, but English is acceptable for initial
skeleton if i18n scaffolding is present.

Initial screens:

1. Welcome
2. Detect launchers
3. Choose launcher path: Official Launcher, SKlauncher, Manual
4. Choose performance profile
5. Install/repair progress
6. Validation results
7. Done / open launcher / export diagnostics

Validation target for this setup pass:

- Package install works.
- Typecheck works.
- Build works.
- Tauri dev/build skeleton is valid enough to compile or clearly document any
  missing system dependency.
- No full installer behavior is required yet.

Do not implement downloads/profile edits yet beyond typed stubs. The task is to
initialize the project, set architecture, create a polished frontend skeleton,
define Tauri command contracts, and document next implementation steps.
