# Publish an Application Release

Use this guide to select the application updater trust root, configure protected
release secrets, publish a tagged prerelease, and verify the update channel.

## Steps

1. Generate the updater key once on a trusted offline or operator-controlled
   computer. Choose and retain a strong password when prompted.

   ```bash
   mkdir -p ~/.tauri
   npm run tauri signer generate -- -w ~/.tauri/minecraft-setup-manager.key
   ```

2. Back up `minecraft-setup-manager.key`, its password, and the generated public
   key in separate durable secure storage. Do not continue until recovery has
   been exercised. Losing the private key prevents shipping updates to clients
   that trust it.

3. Record only the public key in the repository, then review the diff.

   ```bash
   npm run set:updater-public-key -- ~/.tauri/minecraft-setup-manager.key.pub
   npm run check:updater-config:release
   git diff -- src-tauri/tauri.release.conf.json
   ```

   Commit the public key before the first updater-enabled release. Never add the
   private key, its password, `.env` files, certificates, or credentials.

4. In GitHub repository settings, create a `release` environment. Store the
   private key content as `TAURI_SIGNING_PRIVATE_KEY` and the password as
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` environment secrets. Prefer a required
   reviewer and restrict deployment to version tags. GitHub documents
   [environment protection and secrets](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments).

   With GitHub CLI authenticated, the key secret can be read from standard input
   without placing it in shell history:

   ```bash
   gh secret set --env release TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/minecraft-setup-manager.key
   gh secret set --env release TAURI_SIGNING_PRIVATE_KEY_PASSWORD
   ```

5. Enable GitHub immutable releases, protect `v*` tag creation with a repository
   ruleset, and require review for changes to the release workflow, updater
   public key, and update-channel tooling. Updater signing does not protect a
   workflow that an attacker can change before a trusted tag is built.

6. Set the next application version consistently in `package.json`,
   `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Run the complete local
   checks before tagging:

   ```bash
   npm ci
   npm run check
   npm run check:updater-config:release
   cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
   cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
   cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
   ```

7. Create and push the matching tag. For version `0.1.4`:

   ```bash
   git tag v0.1.4
   git push origin v0.1.4
   ```

   The workflow validates first, builds signed updater artifacts for every
   supported package, assembles and checks the draft `latest.json`, publishes the
   GitHub prerelease, and finally promotes the update channel with an optimistic
   blob-SHA update. An older concurrent release cannot roll the channel back.

## Verification

1. Confirm the GitHub release is public, still marked **Pre-release**, and has
   each native installer plus its matching `.sig` asset.
2. Open the raw update-channel URL from the
   [application update reference](../reference/application-updates.md). Confirm
   its version matches the release and it contains all five installer-aware
   platform keys.
3. Install the previous updater-enabled release on each supported target, open
   it, and confirm it discovers the candidate without blocking the setup wizard.
4. Choose **Later** and confirm nothing downloads. Reopen the update surface,
   choose **Update and restart**, observe progress, and confirm the reopened app
   reports the new version as current.
5. Repeat for macOS Apple Silicon, macOS Intel, Windows NSIS, Linux AppImage, and
   Linux DEB. Confirm expected OS authorization or trust prompts rather than
   describing them as updater-signature failures.
6. In a disposable test channel/key setup, alter a signed artifact byte and
   confirm Tauri refuses installation. Never replace an asset in the real
   versioned release to perform this test.

## Notes

- `v0.1.3` cannot update itself. Install the first updater-enabled release
  manually, then use that build as the baseline for its successor.
- Key rotation requires a transition release signed by the old private key and
  embedding the new public key. Do not replace the configured key without that
  migration.
- Apple Developer ID/notarization and Windows Authenticode are separate operator
  projects. Until configured, keep the operating-system warning in the README
  and release notes.
- If publication succeeds but channel promotion fails, rerun only the failed
  `finalize` job. Its public-release validation and channel update are
  idempotent; rerunning all bundle jobs against an already-public release is not.
