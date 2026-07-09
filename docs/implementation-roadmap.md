# Implementation Roadmap

## Complete

1. Strict schema-version-1 validation, portable path rules, and bounded input.
2. Typed setup actions and a manifest-driven pure planner.
3. Reviewed-manifest fingerprint binding across preview, apply, and validation.
4. Durable saved servers, manifest snapshots, update status, and installed hashes.
5. Add, replace, and remove previews with user-modified file protection.
6. Vanilla and pinned Fabric support for the official Minecraft Launcher.
7. Hash-verified direct files and pinned Modrinth file resolution.
8. NBT-preserving `servers.dat` updates and post-apply conformance checks.
9. English and Spanish UI selected from the browser language.
10. Server-namespaced game folders and unowned-file replacement protection.
11. Native macOS, Windows, and Linux CI build coverage.

## Next

1. Add SKlauncher profile creation and validation behind its launcher adapter.
2. Decide whether manual mode should export instructions or support another
   interoperable profile format before enabling it.
3. Add trust-on-first-use pinning or signed manifests for server configuration
   origin changes.
4. Add plan-level rollback or a durable setup journal for stronger recovery from
   failures between individually atomic steps.
5. Add recognized publisher signing, macOS notarization, and app update delivery
   before treating the published development binaries as production-ready.
6. Define the future Minecraft server plugin around the existing manifest
   contract and compatibility status endpoint.
