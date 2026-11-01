<p align="center">🌐 <b>Langue</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

Un client API de style Postman **local, sans compte, sans paiement et sans restriction d'adhésion**, écrit en Rust. Il reproduit les capacités **locales** les plus utilisées de Postman, utilise le format compatible **Postman Collection v2.1** et se présente comme un binaire unique multiplateforme (Windows / macOS / Linux).

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> Ce projet ne se connecte jamais au cloud Postman ; pas de connexion, d'abonnement, d'essai, de Stripe ni de télémétrie. **Toutes les fonctionnalités sont disponibles sans condition pour tous.**

---

## Pourquoi « sans limites »

Les contraintes payantes/connexion de Postman concernent surtout les fonctionnalités **cloud** (synchronisation, équipes, Mock/Monitor cloud, Vault, Governance, AI, etc.). Cet outil n'implémente que des capacités **locales indépendantes du cloud** : aucun compte, aucun paiement, aucun quota. Par conception, les limites de connexion/abonnement/adhésion sont supprimées ; toutes les fonctions locales sont disponibles sans condition.

---

## Fonctionnalités

- Toute méthode HTTP ; Header / Query / Body (raw / json / urlencoded / form-data / file / graphql)
- Variables `{{name}}` ; portées `cli vars > iteration data > environment > collection > globals` ; variables dynamiques (`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp`, ...)
- Authentification : Bearer / Basic / API Key (header ou query)
- Arbre collections / dossiers / requêtes ; variables de collection
- Environnements / variables globales ; historique
- Collection Runner : itérations multiples, données CSV/JSON, délai, statistiques d'assertions, code de sortie CI
- Génération de code : curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- Import : Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2 (JSON), cURL, HAR
- Export : Postman Collection v2.1 / Environment
- Assertions : déclaratives (status / json_path / header / max_time_ms / regex...) + analyse des `pm.test(...)` courants
- Protocoles : WebSocket (`pmrs ws`), SSE, MQTT, GraphQL, Mock local (`pmrs mock`), performance (`pmrs perf`)
- Réseau : proxy (`PMRS_PROXY`), bundle CA, certificat client, ignorer la validation (`PMRS_INSECURE=1`), délais, redirections, cookie jar persistant
- OAuth2 : client_credentials / password / refresh_token / accessToken

## GUI (pmrs-gui)

Application de bureau basée sur **egui/eframe** (Rust pur, exe unique, sans WebView). Interface en chinois avec chargement auto des polices CJK ; à gauche collections / sélecteur d'environnement / historique, à droite constructeur de requêtes à onglets (méthode / URL / en-têtes / corps / réponse) ; éditeur de corps complet (form-data / urlencoded / raw / binary / GraphQL) ; enregistrer dans une collection ; export v2.1 ; éditeur d'environnements ; outils (import / runner / génération de code) ; panneau de réponse ; console Postman ; description de requête ; **8 thèmes** ; système de design unifié.

## Compilation

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

Multiplateforme : Rust + reqwest(rustls), sans dépendance OpenSSL ; compile sur Windows / macOS / Linux.

## Artefacts précompilés / multiplateformes

| Plateforme | Artefact |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS (universal : arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal` (compilation sur macOS) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

Compilé automatiquement par `.github/workflows/build.yml` sur `windows-latest` / `macos-latest` / `ubuntu-latest`.

### Image conteneur (GHCR)

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## Utilisation

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## Espace de travail (local)

Par défaut `~/.postman-rs` (remplaçable via `PMRS_HOME`) : `collections/*.json`, `environments/*.json`, `history.jsonl`. Fichiers simples, compatibles git, sans base de données ni cloud.

## Vérification

```bash
cargo test          # unitaire + intégration (lance un serveur HTTP local)
cargo build --release
```

Le code métier ne contient aucune branche forcée login/token/paywall/subscription/stripe/license.

## FAQ

**Q : Où sont les fichiers compilés ?** GUI : `dist/pmrs-gui-windows-x64.exe` ; CLI : `dist/pmrs-windows-x64.exe` ; les zip dans `dist/`.

**Q : Pourquoi « impossible d'installer » sous Windows ?** C'est un programme portable en un seul fichier, pas un installateur MSI/EXE — double-cliquez simplement sur `pmrs-gui-windows-x64.exe`. Si SmartScreen le bloque, choisissez « Informations complémentaires → Exécuter quand même ».

**Q : Comment changer de thème ?** Via le menu déroulant « Theme » en haut à droite (8 préréglages, mémorisés) ou `PMRS_THEME=0..7`.

**Q : Comment obtenir la version macOS ?** Les binaires Mach-O ne se compilent pas en croisé sous Windows ; compilez sur un Mac avec `scripts/build-macos.sh` ou via la CI.

## Retour d'information (Issues)

Utilisez GitHub Issues : <https://github.com/shuying-code/postman-rs/issues/new/choose>. Lisez le **[guide de retour](docs/FEEDBACK.md)** avant de soumettre. Les mainteneurs traitent généralement les issues une fois par semaine.

## Référence CLI

16 sous-commandes : `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. Voir `pmrs --help`.

## Variables d'environnement

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## Licence

MIT OR Apache-2.0. Implémentation indépendante ; n'inclut aucun actif de marque Postman et n'assure que l'interopérabilité de format.
