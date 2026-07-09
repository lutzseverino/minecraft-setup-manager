# Implementation Roadmap

1. Load `src-tauri/manifest/maresme-client.json` with schema validation instead
   of the current in-code manifest.
2. Keep the frontend `serverCatalog` and backend manifest shape aligned, then
   allow a second server to prove the catalog boundary.
3. Move Maresme server facts from hardcoded Rust/frontend catalogs into the
   validated manifest source of truth.
4. Add SKlauncher discovery and profile adapter behavior.
5. Define manual setup export instructions for unknown launchers.
6. Extend isolated game directory repair beyond the current folder and receipt
   creation.
7. Resolve Fabric installer artifacts for Minecraft `26.1.2` and loader `0.19.3`.
8. Resolve Modrinth pinned files, hashes, and compatibility constraints.
9. Write the Maresme MC server entry through a tested `servers.dat` module.
10. Add real checks for files, hashes, launcher profile metadata, and server entry.

Each step should keep command handlers thin and add tests around the backend
module that owns the behavior.
