# Agent Prompt

Work in this repository:

```text
/Users/lutzseverino/Documents/Projects/minecraft-setup-manager
```

Goal:
Build a server-agnostic Minecraft client setup manager. The app should let
players choose a saved server or enter a server address, fetch that server's
declarative setup manifest, preview required setup/update work, and apply safe
client changes.

Architecture rules:

- React frontend owns UI state only.
- Frontend calls typed Tauri commands through `src/lib/tauri.ts`.
- Rust backend owns filesystem operations, launcher detection/profile edits,
  manifest fetching, downloads, hashing, validation, and diagnostics.
- Server manifests are declarative desired state. They must never execute scripts.
- Keep command handlers thin.
- Keep launcher-specific behavior behind launcher adapters.
- Keep Fabric, Modrinth, managed files, and `servers.dat` behavior in
  Minecraft-owned backend modules.
- Keep install/update decisions manifest-driven.
- Keep saved server/update state in `src-tauri/src/app_state`.

The canonical flow is:

1. Choose a saved server or add a server address.
2. Resolve the setup manifest from that server.
3. Show what the server asks the app to set up or update.
4. Let the user apply the work.
5. Save enough state for future update checks.

Use conventional commits after the git repository has been initialized.
