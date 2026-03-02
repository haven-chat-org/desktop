# Haven Desktop — Project Guide

## What Is This?

Tauri v2 desktop shell for Haven, the E2EE chat platform. This repo wraps the Haven web frontend (`../web`) in a native macOS (and eventually Windows/Linux) application.

## Architecture

```
desktop/
├── src-tauri/              # Rust side (Tauri shell)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   └── lib.rs          # Tauri builder, plugins, tray icon setup
│   ├── Cargo.toml
│   ├── tauri.conf.json     # App config, build commands, window settings, updater
│   ├── capabilities/       # Permission definitions
│   ├── gen/schemas/        # Auto-generated permission schemas (committed)
│   └── icons/              # App icons (.icns, .ico, .png)
├── .github/workflows/
│   └── release.yml         # Build + sign + notarize macOS .dmg on tag push
├── package.json            # @tauri-apps/cli, dev/build scripts
└── CLAUDE.md
```

## How It Works

- The desktop app has NO frontend code of its own — it loads the Haven web frontend
- Dev mode: `tauri.conf.json` runs `npm run dev` in `../web` and connects to `http://localhost:5173`
- Prod build: builds `../web` with `npm run build`, bundles the output into the native app
- The web frontend detects Tauri via `isTauri()` in `web/src/lib/tauriEnv.ts` and adapts behavior (native notifications, server URL config, titlebar drag region)

## Plugins

| Plugin | Purpose |
|--------|---------|
| `tauri-plugin-notification` | Native macOS notifications |
| `tauri-plugin-shell` | Open external URLs in default browser |
| `tauri-plugin-updater` | Auto-update via GitHub Releases |
| `tauri-plugin-process` | App restart after update |

## System Tray

- Left-click the tray icon: shows and focuses the main window
- Right-click: context menu with "Show Haven" and "Quit Haven"
- Tray icon uses the app's default window icon

## Auto-Updater

- Checks `https://github.com/haven-chat-org/desktop/releases/latest/download/latest.json`
- Artifacts signed with Tauri updater keypair
- Public key goes in `tauri.conf.json` → `plugins.updater.pubkey`
- Private key stored as `TAURI_SIGNING_PRIVATE_KEY` GitHub secret

## Development

```bash
# Prerequisites: Rust toolchain, Node.js, web project deps installed
cd ../web && npm ci  # ensure web deps are installed
cd ../desktop
npm install          # install @tauri-apps/cli
npm run dev          # starts web dev server + Tauri window
```

## CI / Release

- Push a `v*` tag to trigger `.github/workflows/release.yml`
- Builds macOS aarch64 (Apple Silicon) and x86_64 (Intel) bundles
- Uses `tauri-apps/tauri-action` for build + GitHub Release creation
- Code signing + notarization requires these GitHub secrets:
  - `APPLE_CERTIFICATE` — Base64-encoded .p12 certificate
  - `APPLE_CERTIFICATE_PASSWORD`
  - `APPLE_SIGNING_IDENTITY` — e.g. "Developer ID Application: Name (TEAMID)"
  - `APPLE_ID` — Apple ID email for notarization
  - `APPLE_PASSWORD` — App-specific password
  - `APPLE_TEAM_ID`
  - `TAURI_SIGNING_PRIVATE_KEY` — Updater signing key
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

## Key Conventions

- Window has `decorations: false` — the web frontend renders a custom titlebar with `data-tauri-drag-region`
- CSP allows `connect-src https: wss:` for connecting to the Haven server
- The `../web` directory must be present and have dependencies installed

## Common Gotchas

- If `npm run dev` fails, make sure `../web/node_modules` exists (`cd ../web && npm ci`)
- If the Tauri window is blank, check that the Vite dev server started on port 5173
- Icons must exist for production builds — generate them with `npx tauri icon <source.png>`
- The updater `pubkey` in `tauri.conf.json` is empty until you generate keys with `npx tauri signer generate`
- Never commit `src-tauri/.tauri-updater-key` — it's in `.gitignore`
