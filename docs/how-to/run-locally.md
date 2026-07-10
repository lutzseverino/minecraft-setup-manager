# Run The Manager Locally

Run either the native Tauri app or the browser-only interface during development.

## Steps

Install Node.js 22, npm 11.6.2, stable Rust, and the
[Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/) for your
operating system. Then run:

```bash
git submodule update --init
npm ci
npm run tauri:dev
```

Use `npm run dev` for the browser-only interface. Browser mode supplies demo
command responses and does not modify Minecraft files.

Run the complete local quality gate with:

```bash
npm run check:version
npm run check:docs
npm run typecheck
npm run check:architecture
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

## Verification

The native command opens Minecraft Setup Manager. The browser command serves the
interface at the URL printed by Vite.
