# HC Forge

HC Forge is a desktop/mobile toolbox for Huawei Cloud operations built with Tauri, Vue 3, TypeScript, and Rust.

## What It Covers

- ECS create and lifecycle operations
- EIP and EVS listing and management
- Integrated SSH terminal and startup task automation
- Platform Ops helpers for Docker, Minikube, and Nix
- CCE cluster, node pool, NAT, and access workflows
- OBS bucket and object management

## Stack

- Frontend: Vue 3 + TypeScript + Vite
- Desktop runtime: Tauri v2
- Backend: Rust with signed Huawei Cloud API requests
- Local storage: Tauri Store plugin

## Prerequisites

- Node.js 20 or newer
- pnpm 9 or newer
- Rust stable toolchain
- Platform build dependencies required by Tauri

## Development

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm build
pnpm tauri build
```

## Supported Platforms

- Windows (x64)
- Linux (x64)
- Android (APK, manually provided on releases)

## Downloads

Prebuilt files are available on the Releases page:

- https://github.com/araujoviana/hc-forge/releases

## GitHub Release Build

A workflow is available at `.github/workflows/release-builds.yml`.

- Trigger: `release.published` (also supports manual `workflow_dispatch`)
- Artifacts:
  - Windows: `hc-forge-windows-x64.exe`
  - Linux: `hc-forge-linux-x64` and `hc-forge-linux-x64.tar.gz`
- Release uploads: on release events, all built artifacts are attached to the GitHub Release.

## Test

```bash
pnpm test:frontend
pnpm test:rust
```

## Project Layout

- `src/`: Vue application, components, shared types, utilities
- `src-tauri/src/`: Rust commands, API client, validators, and tests
- `src/utils/platformOps.js`: command builders and parsers for Platform Ops

## Security Notes

- Credentials are stored locally through Tauri Store.
- VM passwords are encrypted before local persistence.
- Use least-privilege IAM credentials for daily operations.

## TODO

- [ ] Create a HC Forge custom icon
- [ ] Expand service coverage
- [ ] Add pod management and remote access
- [ ] Add pagination to OBS buckets list
- [ ] Add cycling animation for long OBS object names

## License

MIT. See `LICENSE`.
