<p align="center">🌐 <b>Idioma</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

Un cliente API estilo Postman **local, sin cuenta, sin pago y sin restricciones de membresía**, escrito en Rust. Reproduce las capacidades **locales** más usadas de Postman, usa el formato compatible **Postman Collection v2.1** y es un único binario multiplataforma (Windows / macOS / Linux).

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> Este proyecto nunca se conecta a la nube de Postman; sin inicio de sesión, suscripción, prueba, Stripe ni telemetría. **Todas las funciones están disponibles incondicionalmente para todos.**

---

## Por qué «sin límites»

Las restricciones de pago/inicio de sesión de Postman afectan sobre todo a funciones **en la nube** (sincronización, equipos, Mock/Monitor en la nube, Vault, Governance, AI, etc.). Esta herramienta implementa solo capacidades **locales independientes de la nube**, por lo que no requiere cuenta, pago ni cuota. Por diseño se eliminan los límites de inicio de sesión/suscripción/membresía; todas las funciones locales están disponibles sin condiciones.

---

## Funciones

- Cualquier método HTTP; Header / Query / Body (raw / json / urlencoded / form-data / file / graphql)
- Variables `{{name}}`; ámbitos `cli vars > iteration data > environment > collection > globals`; variables dinámicas (`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp`, ...)
- Autenticación: Bearer / Basic / API Key (header o query)
- Árbol de colecciones / carpetas / solicitudes; variables de colección
- Entornos / variables globales; historial
- Collection Runner: múltiples iteraciones, datos CSV/JSON, retardo, estadísticas de aserciones, código de salida para CI
- Generación de código: curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- Importar: Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2 (JSON), cURL, HAR
- Exportar: Postman Collection v2.1 / Environment
- Aserciones: declarativas (status / json_path / header / max_time_ms / regex...) + análisis de `pm.test(...)` comunes
- Protocolos: WebSocket (`pmrs ws`), SSE, MQTT, GraphQL, Mock local (`pmrs mock`), rendimiento (`pmrs perf`)
- Red: proxy (`PMRS_PROXY`), bundle de CA, certificado de cliente, omitir validación (`PMRS_INSECURE=1`), tiempos de espera, redirecciones, cookie jar persistente
- OAuth2: client_credentials / password / refresh_token / accessToken

## GUI (pmrs-gui)

Aplicación de escritorio con **egui/eframe** (Rust puro, un solo exe, sin WebView). Interfaz en chino con carga automática de fuentes CJK; a la izquierda colecciones / selector de entorno / historial, a la derecha constructor de solicitudes con pestañas (método / URL / cabeceras / cuerpo / respuesta); editor de cuerpo completo (form-data / urlencoded / raw / binary / GraphQL); guardar en colección; exportar v2.1; editor de entornos; herramientas (importar / runner / generación de código); panel de respuesta; consola Postman; descripción de la solicitud; **8 temas**; sistema de diseño unificado.

## Compilación

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

Multiplataforma: Rust + reqwest(rustls), sin dependencia de OpenSSL; compila en Windows / macOS / Linux.

## Artefactos precompilados / multiplataforma

| Plataforma | Artefacto |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS (universal: arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal` (se compila en macOS) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

Compilado automáticamente por `.github/workflows/build.yml` en `windows-latest` / `macos-latest` / `ubuntu-latest`.

### Imagen de contenedor (GHCR)

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## Uso

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## Espacio de trabajo (local)

Por defecto `~/.postman-rs` (se cambia con `PMRS_HOME`): `collections/*.json`, `environments/*.json`, `history.jsonl`. Archivos simples, aptos para git, sin base de datos ni nube.

## Verificación

```bash
cargo test          # unidad + integración (levanta un servidor HTTP local)
cargo build --release
```

El código de negocio no contiene ramas forzadas de login/token/paywall/subscription/stripe/license.

## Preguntas frecuentes (FAQ)

**P: ¿Dónde están los archivos compilados?** GUI: `dist/pmrs-gui-windows-x64.exe`; CLI: `dist/pmrs-windows-x64.exe`; los zip en `dist/`.

**P: ¿Por qué "no se instala" en Windows?** Es un programa portátil de un solo archivo, no un instalador MSI/EXE — solo haz doble clic en `pmrs-gui-windows-x64.exe`. Si SmartScreen lo bloquea, elige "Más información → Ejecutar de todos modos".

**P: ¿Cómo cambio el tema?** Con el desplegable "Theme" arriba a la derecha (8 preajustes, se recuerda) o `PMRS_THEME=0..7`.

**P: ¿Cómo obtengo la versión de macOS?** Los binarios Mach-O no se pueden compilar de forma cruzada en Windows; compila en un Mac con `scripts/build-macos.sh` o vía CI.

## Comentarios (Issues)

Usa GitHub Issues: <https://github.com/shuying-code/postman-rs/issues/new/choose>. Lee la **[guía de comentarios](docs/FEEDBACK.md)** antes de enviar. Los mantenedores suelen atender los issues una vez por semana.

## Referencia de CLI

16 subcomandos: `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. Ver `pmrs --help`.

## Variables de entorno

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## Licencia

MIT OR Apache-2.0. Esta es una implementación independiente; no incluye activos de marca de Postman y solo ofrece interoperabilidad de formato.
