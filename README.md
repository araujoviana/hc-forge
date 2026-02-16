# HC Forge

Desktop Cloud Ops console for Huawei Cloud ECS workflows, built with Tauri + Vue + TypeScript.

## What It Does

- Creates ECS instances with image/flavor/VPC/subnet/root volume selections.
- Optionally allocates EIP during create.
- Supports optional EVS data disks on create (`GPSSD` default, `100GB` default when enabled).
- Lists ECS instances, EIPs, and EVS disks in card/list format (no horizontal table scrolling).
- Includes integrated SSH terminal controls (interactive shell + Ctrl+C/Ctrl+D/Ctrl+U shortcuts).
- Supports startup task automation for newly created VMs:
  - `Update VM on startup`
  - `Setup GUI + RDP on startup` (installs IceWM + XRDP stack using detected package manager)
- Streams startup task progress into UI/logs, including percent markers.
- Sends desktop notifications when:
  - ECS create request succeeds.
  - Startup tasks complete/fail.
- Persists encrypted per-VM SSH passwords locally (Tauri store) so SSH shortcuts continue working after restart.

## Tech Stack

- Frontend: Vue 3 + TypeScript + Vite
- Desktop runtime: Tauri v2
- Backend: Rust (Huawei Cloud signed API client + SSH via `russh`)
- Storage: `@tauri-apps/plugin-store`
- Notifications: `@tauri-apps/plugin-notification`

## Requirements

- Node.js 20+
- pnpm 9+
- Rust stable toolchain
- OS build prerequisites for Tauri:
  - Linux: GTK/WebKit2GTK build deps
  - Windows: MSVC Build Tools + WebView2 runtime

## Development

```bash
pnpm install
pnpm tauri dev
```

## Build

Frontend build:

```bash
pnpm build
```

Rust tests:

```bash
pnpm test:rust
```

Bundle desktop app:

```bash
pnpm tauri build
```

Artifacts are generated under `src-tauri/target/release/bundle`.

## Packaging Targets

This project is set up to eventually produce executables/installers for Linux and Windows through Tauri bundling.

- Linux: AppImage / deb / rpm (depending on host setup)
- Windows: MSI / NSIS / EXE (depending on bundle config and host setup)

Cross-compiling Windows binaries from Linux typically requires additional toolchain setup. For reliable output, build on the target OS.

## Security Notes

- AK/SK credentials are stored locally via Tauri Store.
- VM passwords are persisted per-server in encrypted form using Web Crypto (`AES-GCM` with PBKDF2-derived key from local credentials).
- Keep local workstation access restricted since secrets remain decryptable by this app environment.

## Current Scope

HC Forge currently focuses on ECS lifecycle + SSH operations and supporting network/storage resources in a single desktop workflow.
