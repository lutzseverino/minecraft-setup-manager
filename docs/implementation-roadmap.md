# Implementation Roadmap

1. Add semantic manifest validation and schema-version migration rules. Strict
   schema-version-1 validation is in place; future schema migrations remain.
2. Replace string setup steps with typed setup actions and a pure planner.
3. Record verified local resource file hashes after downloads. Direct resources
   are supported; resolved Modrinth files remain.
4. Add update and repair previews: add, replace, and remove managed files while
   leaving user files alone. Preview and direct-file execution are in place;
   modified-file removal protection remains.
5. Implement Fabric loader installation through a loader adapter. The app now
   installs validated launcher JSON from the pinned official Fabric Meta endpoint
   without executing an installer JAR.
6. Resolve Modrinth pinned files, hashes, and compatibility constraints. Pinned
   version/project resolution and primary-file validation are in place.
7. Download resources into managed folders with hash verification and temp-file
   renames. Direct and Modrinth files use the shared verified install path.
8. Write server entries through a tested `servers.dat` module. NBT-preserving,
   backed-up, atomic upserts and validation are in place.
9. Add SKlauncher profile adapter behavior.
10. Add manifest signing or trust pinning before accepting public remote configs.
11. Define the future Minecraft server plugin around this manifest contract.

Each step should keep command handlers thin and add tests around the backend
module that owns the behavior.
