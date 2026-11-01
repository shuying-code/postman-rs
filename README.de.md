<p align="center">🌐 <b>Sprache</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

Ein **lokaler API-Client im Postman-Stil ohne Konto, ohne Bezahlung und ohne Mitgliedschaftsbeschränkung**, geschrieben in Rust. Er bildet die meistgenutzten **lokalen** Funktionen von Postman nach, nutzt das kompatible **Postman Collection v2.1**-Format und ist ein plattformübergreifendes Einzelbinary (Windows / macOS / Linux).

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> Dieses Projekt verbindet sich nie mit der Postman-Cloud; kein Login, Abo, Trial, Stripe oder Telemetrie. **Alle Funktionen sind für jeden bedingungslos verfügbar.**

---

## Warum „unbegrenzt"

Die bezahlten/Login-Beschränkungen von Postman betreffen vor allem **Cloud**-Funktionen (Sync, Teams, Cloud-Mock/Monitor, Vault, Governance, AI usw.). Dieses Tool implementiert nur **cloud-unabhängige lokale** Fähigkeiten und benötigt daher kein Konto, keine Zahlung und kein Kontingent. Konstruktionsbedingt sind Login-/Abo-/Mitgliedschaftsgrenzen entfernt; alle lokalen Funktionen sind bedingungslos verfügbar.

---

## Funktionen

- Beliebige HTTP-Methode; Header / Query / Body (raw / json / urlencoded / form-data / file / graphql)
- Variablen `{{name}}`; Gültigkeitsbereiche `cli vars > iteration data > environment > collection > globals`; dynamische Variablen (`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp`, ...)
- Authentifizierung: Bearer / Basic / API Key (header oder query)
- Sammlungs- / Ordner- / Request-Baum; Sammlungsvariablen
- Umgebungen / globale Variablen; Verlauf
- Collection Runner: mehrere Iterationen, CSV/JSON-Daten, Verzögerung, Assertion-Statistiken, CI-Exit-Code
- Codegenerierung: curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- Import: Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2 (JSON), cURL, HAR
- Export: Postman Collection v2.1 / Environment
- Assertions: deklarativ (status / json_path / header / max_time_ms / regex ...) + Parsing gängiger `pm.test(...)`
- Protokolle: WebSocket (`pmrs ws`), SSE, MQTT, GraphQL, lokaler Mock (`pmrs mock`), Lasttest (`pmrs perf`)
- Netzwerk: Proxy (`PMRS_PROXY`), CA-Bundle, Client-Zertifikat, Prüfung überspringen (`PMRS_INSECURE=1`), Timeouts, Redirects, persistenter Cookie-Jar
- OAuth2: client_credentials / password / refresh_token / accessToken

## GUI (pmrs-gui)

Desktop-App auf Basis von **egui/eframe** (reines Rust, eine exe, kein WebView). Chinesische UI mit automatisch geladenen CJK-Schriften; links Sammlungen / Umgebungs-Dropdown / Verlauf, rechts Multi-Tab-Request-Builder (Methode / URL / Header / Body / Response); vollständiger Body-Editor (form-data / urlencoded / raw / binary / GraphQL); in Sammlung speichern; v2.1-Export; Umgebungseditor; Tools (Import / Runner / Codegen); Response-Panel; Postman-Konsole; Request-Beschreibung; **8 Themes**; einheitliches Designsystem.

## Build

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

Plattformübergreifend: Rust + reqwest(rustls), keine OpenSSL-Abhängigkeit; kompiliert auf Windows / macOS / Linux.

## Vorgefertigte / plattformübergreifende Artefakte

| Plattform | Artefakt |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS (universal: arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal` (auf macOS bauen) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

Automatisch gebaut durch `.github/workflows/build.yml` auf `windows-latest` / `macos-latest` / `ubuntu-latest`.

### Container-Image (GHCR)

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## Verwendung

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## Arbeitsbereich (lokal)

Standard `~/.postman-rs` (überschreibbar via `PMRS_HOME`): `collections/*.json`, `environments/*.json`, `history.jsonl`. Reine Dateien, git-tauglich, keine Datenbank, keine Cloud.

## Verifikation

```bash
cargo test          # Unit + Integration (startet lokalen HTTP-Server)
cargo build --release
```

Der Geschäftscode enthält keine erzwungenen login/token/paywall/subscription/stripe/license-Zweige.

## FAQ

**F: Wo liegen die gebauten Dateien?** GUI: `dist/pmrs-gui-windows-x64.exe`; CLI: `dist/pmrs-windows-x64.exe`; ZIPs unter `dist/`.

**F: Warum lässt sich das unter Windows „nicht installieren"?** Es ist ein portables Einzeldatei-Programm, kein MSI/EXE-Installer — einfach `pmrs-gui-windows-x64.exe` doppelklicken. Bei SmartScreen „Weitere Informationen → Trotzdem ausführen" wählen.

**F: Wie ändere ich das Theme?** Über das Dropdown „Theme" oben rechts (8 Presets, wird gemerkt) oder `PMRS_THEME=0..7`.

**F: Wie bekomme ich den macOS-Build?** Mach-O lässt sich unter Windows nicht cross-kompilieren; auf einem Mac mit `scripts/build-macos.sh` oder per CI bauen.

## Feedback (Issues)

Nutze GitHub Issues: <https://github.com/shuying-code/postman-rs/issues/new/choose>. Lies vor dem Einreichen den **[Feedback-Leitfaden](docs/FEEDBACK.md)**. Maintainer bearbeiten Issues in der Regel einmal pro Woche.

## CLI-Referenz

16 Unterbefehle: `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. Details via `pmrs --help`.

## Umgebungsvariablen

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## Lizenz

MIT OR Apache-2.0. Unabhängige Implementierung; enthält keine Postman-Markenassets und bietet nur Format-Interoperabilität.
