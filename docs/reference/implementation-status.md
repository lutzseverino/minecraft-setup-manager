# Implementation Status

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
12. Independently versioned protocol v1 with shared schema, semantics,
    conformance fixtures, and RFC 8785 fingerprints.
13. Optional server setup-code redemption after backend-owned local validation.
14. Stable SKlauncher 3.2 adapter implementation, live workspace-presence
    enforcement, and isolated 3.2.18 clean-launcher round-trip validation for
    the standard Minecraft workspace.
15. Prepared consent-driven application update discovery, mandatory artifact
    signature verification, progress feedback, installation, relaunch, and a
    prerelease-compatible update channel with rollback-safe promotion. Operator
    key selection and the first publication remain before operational enablement.

## Known Gaps

1. Add explicit workspace selection for relocated SKlauncher 3.2 installations
   and evaluate SKlauncher 4.0 after its instance contract stabilizes.
2. Decide whether manual mode should export instructions or support another
   interoperable profile format before enabling it.
3. Prove the generic Paper publisher and setup-code exchange in an end-to-end
   Paper integration test.
4. Add trust-on-first-use pinning or signed manifests for server configuration
   origin changes.
5. Add plan-level rollback or a durable setup journal for stronger recovery from
   failures between individually atomic steps.
6. Add recognized publisher signing and macOS notarization before treating the
   published development binaries as production-ready.
7. Exercise the first real updater-enabled release-to-release transition on
   macOS Apple Silicon and Intel, Windows NSIS, Linux AppImage, and Linux DEB.
   `v0.1.3` cannot be the baseline because it has no embedded updater key or
   published signatures.
