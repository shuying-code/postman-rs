# Changelog

All notable changes to this project are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/), versioning follows [SemVer](https://semver.org/).

## [Unreleased]

### Added
- **pm.* assertion matchers**: `.not` negation, `lengthOf`/`length`, `property`, `a`/`an`, `match(/regex/)`,
  `oneOf`, `closeTo`, `include` (array membership), `empty`, `null`, `exist`, and aliases.
- **GUI Postman Console**: new sidebar section logging each request/response and test results (clear / copy all).
- **Request Description editor**: new "描述" sub-tab, saved to `item.description`, kept on v2.1 export.
- **Inheritance on send**: GUI now inherits collection/folder auth, folder variables and collection/folder
  pre-request/test scripts (Runner already did).
- **macOS packaging**: `.github/workflows/build.yml` (macos job → universal arm64+x86_64), `scripts/build-macos.sh`,
  source bundle `dist/pmrs-macos-src.tar.gz`, and `docs/13_macOS打包说明.md`.
- **Dynamic variables**: ~30 more (`$randomLorem*`, `$randomImageUrl`, `$randomAvatar`, `$randomUserName`,
  `$randomBankAccount*`, `$randomCreditCardNumber`, `$randomAmount`, …).
- **Extended HTTP method dropdown** (COPY/LINK/UNLINK/PURGE/LOCK/UNLOCK/PROPFIND/VIEW/TRACE).
- **Container image (GHCR)**: `Dockerfile` + `.github/workflows/docker.yml` build and push the CLI
  image to GitHub Packages (`ghcr.io/zhouyulong123/postman-rs`).
- `LICENSE-MIT`, `LICENSE-APACHE`, `CHANGELOG.md`, `.gitignore`.

### Fixed
- **GUI ignored global variables** when sending (`ctx_from` did not merge `globals`).
- `pm.response.json()` no longer collapses JSON `null` into empty string.

## [0.1.0]

### Added
- Local, account-free, payment-free Postman-style API client in Rust (`pmrs` CLI + `pmrs-gui` egui desktop).
- HTTP requests (all methods), variables/scopes + dynamic variables, collections/folders/requests (v2.1),
  environments, globals, history.
- Collection Runner (iterations, CSV/JSON data, delay, assertions, CI exit code).
- Import: Postman v1/v2.1, OpenAPI 3 / Swagger 2, cURL, HAR. Export: Collection v2.1 / Environment.
- Code generation for 13 languages.
- Assertions: declarative + common `pm.test(...)` parsing.
- Protocols: WebSocket, SSE, MQTT (plain TCP), GraphQL, local Mock, performance test.
- Networking: proxy, CA bundle, client cert, insecure, persistent cookie jar.
- Auth: Bearer / Basic / API Key / OAuth2 / OAuth1 / JWT / AWS SigV4 / Digest (NTLM honest "unsupported").
