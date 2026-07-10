# Supported Capabilities

This page records the setup actions the current release can complete and
validate.

| Capability | Status | Notes |
| --- | --- | --- |
| Official Minecraft Launcher | Supported | Creates and validates an isolated launcher profile |
| Vanilla Minecraft | Supported | Uses a pinned Minecraft version |
| Fabric | Supported | Installs a pinned loader profile from Fabric Meta |
| Modrinth resources | Supported | Pins project and version metadata and verifies the file |
| Direct resources | Supported | Requires public HTTPS and a SHA-256 or SHA-512 hash |
| Multiplayer server entry | Supported | Preserves unrelated entries in `servers.dat` |
| Setup updates | Supported | Previews additions, changes, and safe removals before applying them |
| Server setup codes | Supported | Checks in with the server only after local validation passes |
| SKlauncher 3.2 | Supported | Version 3.2.18 passed an isolated clean-launcher round trip for the standard Minecraft workspace |
| Relocated SKlauncher and SKlauncher 4.0 | Planned | Disabled until their workspace or instance contracts can be discovered safely |
| Manual profiles | Planned | Disabled until an interoperable handoff format is selected |
| Signed manifests | Planned | Trust currently comes from the server address and the player's review |

The backend rejects unsupported plan actions. A partial install cannot be
reported as complete.
