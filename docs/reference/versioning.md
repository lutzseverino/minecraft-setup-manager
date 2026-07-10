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

The independently consumed Minecraft Setup Protocol follows Semantic Versioning
and is versioned separately from this application.
