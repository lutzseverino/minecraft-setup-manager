# Implementation Roadmap

1. Add semantic manifest validation and schema-version migration rules.
2. Replace string setup steps with typed setup actions and a pure planner.
3. Record verified local resource file hashes after downloads.
4. Add update and repair previews: add, replace, remove managed files, leave user
   files alone.
5. Implement Fabric loader installation through a loader adapter.
6. Resolve Modrinth pinned files, hashes, and compatibility constraints.
7. Download resources into managed folders with hash verification and temp-file
   renames.
8. Write server entries through a tested `servers.dat` module.
9. Add SKlauncher profile adapter behavior.
10. Add manifest signing or trust pinning before accepting public remote configs.
11. Define the future Minecraft server plugin around this manifest contract.

Each step should keep command handlers thin and add tests around the backend
module that owns the behavior.
