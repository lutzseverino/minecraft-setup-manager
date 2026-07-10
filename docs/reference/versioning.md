# Versioning

This repository follows
[Romantic Versioning](https://romversioning.github.io/romver/), written as
`PROJECT.MAJOR.MINOR`.

Project version `0` marks initial development. A major increment represents an
incompatible change within the same project, and a minor increment represents a
compatible feature or fix. Reaching a stable product moves the project number to
`1.0.0`.

Release tags use the form `v0.1.0`. A tag must exactly match the version in
`package.json`, Cargo, and Tauri configuration. Pushing a valid version tag runs
the release workflow and publishes native installers. `v0.*` tags are marked as
development releases automatically.

Application discovery intentionally does not use GitHub's standard latest
release endpoint because that endpoint excludes prereleases. After a tagged
release is complete and public, the workflow promotes its signed Tauri manifest
to `update-channel/latest.json`. Promotion accepts only a higher numeric
application version, or an identical retry of the current version.

Every updater-enabled release must use the public key committed in the
release-only Tauri configuration. Changing that trust root requires an
old-key-signed transition release; it is not an ordinary version bump.

The independently consumed Minecraft Setup Protocol follows Semantic Versioning
and is versioned separately from this application.
