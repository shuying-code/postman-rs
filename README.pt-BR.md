<p align="center">🌐 <b>Idioma</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

Um cliente de API no estilo Postman **local, sem conta, sem pagamento e sem restrições de assinatura**, escrito em Rust. Reproduz os recursos **locais** mais usados do Postman, usa o formato compatível **Postman Collection v2.1** e é um único binário multiplataforma (Windows / macOS / Linux).

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> Este projeto nunca se conecta à nuvem do Postman; sem login, assinatura, teste, Stripe ou telemetria. **Todos os recursos estão disponíveis incondicionalmente para todos.**

---

## Por que «sem limites»

As restrições pagas/de login do Postman afetam principalmente recursos **de nuvem** (sincronização, equipes, Mock/Monitor na nuvem, Vault, Governance, AI etc.). Esta ferramenta implementa apenas capacidades **locais independentes da nuvem**, portanto não exige conta, pagamento nem cota. Por design, os limites de login/assinatura/membresia foram removidos; todos os recursos locais estão disponíveis sem condições.

---

## Recursos

- Qualquer método HTTP; Header / Query / Body (raw / json / urlencoded / form-data / file / graphql)
- Variáveis `{{name}}`; escopos `cli vars > iteration data > environment > collection > globals`; variáveis dinâmicas (`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp`, ...)
- Autenticação: Bearer / Basic / API Key (header ou query)
- Árvore de coleções / pastas / requisições; variáveis de coleção
- Ambientes / variáveis globais; histórico
- Collection Runner: múltiplas iterações, dados CSV/JSON, atraso, estatísticas de asserções, código de saída para CI
- Geração de código: curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- Importação: Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2 (JSON), cURL, HAR
- Exportação: Postman Collection v2.1 / Environment
- Asserções: declarativas (status / json_path / header / max_time_ms / regex...) + parsing de `pm.test(...)` comuns
- Protocolos: WebSocket (`pmrs ws`), SSE, MQTT, GraphQL, Mock local (`pmrs mock`), carga/desempenho (`pmrs perf`)
- Rede: proxy (`PMRS_PROXY`), bundle de CA, certificado de cliente, ignorar validação (`PMRS_INSECURE=1`), timeouts, redirecionamentos, cookie jar persistente
- OAuth2: client_credentials / password / refresh_token / accessToken

## GUI (pmrs-gui)

Aplicativo desktop com **egui/eframe** (Rust puro, exe único, sem WebView). Interface em chinês com carregamento automático de fontes CJK; à esquerda coleções / seletor de ambiente / histórico, à direita construtor de requisições com abas (método / URL / headers / corpo / resposta); editor de corpo completo (form-data / urlencoded / raw / binary / GraphQL); salvar em coleção; exportar v2.1; editor de ambientes; ferramentas (importar / runner / geração de código); painel de resposta; console do Postman; descrição da requisição; **8 temas**; sistema de design unificado.

## Compilação

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

Multiplataforma: Rust + reqwest(rustls), sem dependência de OpenSSL; compila no Windows / macOS / Linux.

## Artefatos pré-compilados / multiplataforma

| Plataforma | Artefato |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS (universal: arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal` (compilado no macOS) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

Compilado automaticamente por `.github/workflows/build.yml` em `windows-latest` / `macos-latest` / `ubuntu-latest`.

### Imagem de contêiner (GHCR)

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

## Área de trabalho (local)

Padrão `~/.postman-rs` (pode ser alterado com `PMRS_HOME`): `collections/*.json`, `environments/*.json`, `history.jsonl`. Arquivos simples, compatíveis com git, sem banco de dados nem nuvem.

## Verificação

```bash
cargo test          # unidade + integração (inicia um servidor HTTP local)
cargo build --release
```

O código de negócio não contém ramificações forçadas de login/token/paywall/subscription/stripe/license.

## Perguntas frequentes (FAQ)

**P: Onde ficam os arquivos compilados?** GUI: `dist/pmrs-gui-windows-x64.exe`; CLI: `dist/pmrs-windows-x64.exe`; os zips em `dist/`.

**P: Por que "não instala" no Windows?** É um programa portátil de arquivo único, não um instalador MSI/EXE — basta dar duplo clique em `pmrs-gui-windows-x64.exe`. Se o SmartScreen bloquear, escolha "Mais informações → Executar mesmo assim".

**P: Como troco o tema?** Pelo menu suspenso "Theme" no canto superior direito (8 predefinições, memorizadas) ou `PMRS_THEME=0..7`.

**P: Como obter a versão para macOS?** Binários Mach-O não podem ser compilados de forma cruzada no Windows; compile em um Mac com `scripts/build-macos.sh` ou via CI.

## Feedback (Issues)

Use o GitHub Issues: <https://github.com/shuying-code/postman-rs/issues/new/choose>. Leia o **[guia de feedback](docs/FEEDBACK.md)** antes de enviar. Os mantenedores costumam tratar as issues uma vez por semana.

## Referência de CLI

16 subcomandos: `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. Veja `pmrs --help`.

## Variáveis de ambiente

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## Licença

MIT OR Apache-2.0. Implementação independente; não inclui ativos de marca do Postman e fornece apenas interoperabilidade de formato.
