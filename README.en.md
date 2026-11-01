<p align="center">🌐 <b>Language</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

> 📘 **[User Manual](docs/用户操作手册.en.md)** · [简体中文手册](docs/用户操作手册.md)

A **local, account-free, pay-free, membership-free** Postman-style API client written in Rust. It reproduces Postman's most-used **local** capabilities, uses the **Postman Collection v2.1** compatible format, and is a cross-platform single binary (Windows / macOS / Linux).

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> This project never connects to the Postman cloud; no login, subscription, trial, Stripe, or telemetry. **All features are unconditionally available to everyone.**

---

## Why "unlimited"

Postman's paid/login constraints are mainly **cloud** features (sync, teams, cloud Mock/Monitor, Vault, Governance, AI, etc.). This tool implements only **cloud-independent local** capabilities, so it needs no account, no payment, no quota. By design, login / subscription / membership limits are removed; all local features are unconditionally available.

---

## Features

- Any HTTP method; Header / Query / Body (raw / json / urlencoded / form-data / file / graphql)
- Variables `{{name}}`; scopes `cli vars > iteration data > environment > collection > globals`; dynamic variables (`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp`, ...)
- Auth: Bearer / Basic / API Key (header or query)
- Collection / folder / request tree; collection variables
- Environments / globals; history
- Collection Runner: multiple iterations, CSV/JSON data-driven, delay, assertion stats, CI exit code
- Code generation: curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- Import: Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2 (JSON), cURL, HAR
- Export: Postman Collection v2.1 / Environment
- Assertions: declarative (status / json_path / header / max_time_ms / regex ...) + common `pm.test(...)` parsing
- Protocols: WebSocket (`pmrs ws`), SSE, MQTT, GraphQL, local Mock (`pmrs mock`), performance (`pmrs perf`)
- Network: proxy (`PMRS_PROXY`), CA bundle, client cert, insecure skip (`PMRS_INSECURE=1`), timeouts, redirects, persistent cookie jar
- OAuth2: client_credentials / password / refresh_token / accessToken

## GUI (pmrs-gui)

A desktop app built with **egui/eframe** (pure Rust, single exe, no WebView). Chinese UI with auto-loaded CJK fonts; left panel with collections / environment dropdown / history; right side multi-tab request builder (method / URL / headers / body / response); full body editor (form-data / urlencoded / raw / binary / GraphQL); save to collection; export v2.1; environment editor; tools (import / runner / codegen); response panel (body / headers / cookies / tests / timing); Postman console; request description; **8 themes**; unified design system.

## Build

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

Cross-platform: Rust + reqwest(rustls), no OpenSSL dependency; compiles on Windows / macOS / Linux.

## Prebuilt / cross-platform artifacts

| Platform | Artifact |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS (universal: arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal` (must be built on macOS) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

Built automatically by `.github/workflows/build.yml` on `windows-latest` / `macos-latest` / `ubuntu-latest`.

### Container image (GHCR)

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## Usage

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## Workspace (local)

Default `~/.postman-rs` (override with `PMRS_HOME`): `collections/*.json`, `environments/*.json`, `history.jsonl`. Plain files, git-friendly, no database, no cloud.

## Verification

```bash
cargo test          # unit + integration (starts a local HTTP server)
cargo build --release
```

Business code contains no forced login/token/paywall/subscription/stripe/license branch.

## FAQ

**Q: Where are the built files?** GUI: `dist/pmrs-gui-windows-x64.exe`; CLI: `dist/pmrs-windows-x64.exe`; zips under `dist/`.

**Q: Why "can't install" on Windows?** It's a portable single-file program, not an MSI/EXE installer — just double-click `pmrs-gui-windows-x64.exe`. If SmartScreen blocks it, choose "More info -> Run anyway".

**Q: How do I change the theme?** Use the top-right "Theme" dropdown (8 presets, remembered) or `PMRS_THEME=0..7`.

**Q: How to get the macOS build?** Mach-O binaries can't be cross-compiled on Windows; build on a Mac with `scripts/build-macos.sh` or via CI.

## Feedback (Issues)

Use GitHub Issues: <https://github.com/shuying-code/postman-rs/issues/new/choose>. Please read the **[feedback guide](docs/FEEDBACK.md)** first. Maintainers usually triage issues once per week.

## CLI reference

16 subcommands: `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. Run `pmrs --help` for details.

## Environment variables

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## License

MIT OR Apache-2.0. This is an independent implementation; it contains no Postman trademark assets and only provides format interoperability.
