# Architecture

## Purpose

Define the manager's ownership boundaries, dependency direction, and current
implementation model.

## Overview

Minecraft Setup Manager is a Tauri 2 desktop app with a Vite, React, and
TypeScript frontend.

The language-neutral manifest contract is pinned through the `protocol`
submodule. Rust manifest models and validators are consumer implementations of
that contract, not its owner. Protocol fixtures run directly in the Rust test
suite and golden fingerprints must match before a protocol update is accepted.

## Key Concepts

### Responsibility Boundaries

- `src/hooks/` owns wizard orchestration, async command state, and lifecycle resets.
- `src/screens/` owns presentational wizard composition.
- `src/components/ui/` owns foundational shadcn/Radix-style primitives.
- `src/components/app/` owns composed app UI pieces.
- `src/config/` owns generic setup UI metadata, such as launcher display details
  and local icon choices. Setup profiles and resource membership come from the
  server manifest.
- `src/lib/types.ts` defines frontend command contracts.
- `src/lib/tauri.ts` is the only frontend module that imports Tauri APIs.
- `src-tauri/src/commands/` owns thin Tauri command handlers and shared DTOs.
- `src-tauri/src/app_state/` owns saved servers and durable update state.
- `src-tauri/src/server/` owns server address normalization, manifest discovery,
  and the optional setup-attestation exchange.
- `src-tauri/src/manifest/` owns setup manifest schema, fetching, validation, and
  fingerprinting.
- `src-tauri/src/launcher/` owns launcher adapter boundaries.
- `src-tauri/src/minecraft/` owns local install preparation, Fabric, Modrinth,
  server entry, file repair, and validation modules.
- `src-tauri/src/system/` owns platform path helpers.

Manifest data crosses a strict validation boundary before it can be saved,
planned, or applied. Validation owns schema support, IDs, limits, relationships,
hash and URL policy, and portable path names. Filesystem owners repeat path and
symlink checks before mutation as defense in depth.

Every protocol resource declares one explicit destination filename. Destination
ownership is globally unique across profiles using a case-folded comparison, so
changing profiles cannot transfer a path between resource IDs. Modrinth remains
a resolution adapter, but its selected primary file must match that declared
destination.

The validated manifest is cached as the current checked snapshot. Plan, apply,
and validation requests carry the fingerprint shown by the UI and fail if it no
longer matches that snapshot. App state and cached manifests use serialized,
atomic writes so a crash cannot truncate the only durable copy.

### Dependency Direction

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

Each saved server owns a namespaced instance root derived from its stable server
ID. Manifest folder names are display leaves inside that root, never global
instance identifiers. A resource destination may be replaced only when the
planner supplies the previous managed file and its current hash still matches.

### Current Implementation State

The app currently resolves and stores server manifests, consumes manifest-defined
setup profiles, builds setup plans from saved manifests, creates the isolated
game folder, installs pinned Fabric launcher metadata, syncs hash-pinned direct
resources and pinned Modrinth files, writes a setup receipt, creates or updates
the optional `servers.dat` entry and official Minecraft Launcher profile,
validates those local files and launcher settings, and exports a small report.
When the player enters a setup code, the backend repeats local validation and
redeems that code against the approved manifest origin. React never submits an
attestation directly.

Apply is idempotent and only advances durable installed state after a conformance
pass verifies the loader version, launcher profile, setup receipt identity,
optional server entry, and every selected resource hash. A failed post-check is
reported as a failed setup and leaves the previous installed-state record intact.

SKlauncher and manual profile writes remain launcher-adapter work. Manifest
trust pinning remains a manifest/app-state concern. Multi-step rollback remains
a setup orchestration concern; today each owned write is atomic or backed up,
and a failed run leaves installed state unchanged so the desired state can be
repaired by rerunning it.

## Implications

- New side effects belong in the Rust module that owns the affected domain.
- Frontend code depends on typed command contracts rather than backend details.
- Protocol changes are adopted through an immutable protocol release and its
  conformance fixtures.
- Unsupported launcher behavior remains disabled until its adapter can apply
  and validate the complete operation.
