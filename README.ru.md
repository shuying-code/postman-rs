<p align="center">🌐 <b>Язык</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

**Локальный API-клиент в стиле Postman без аккаунта, без оплаты и без ограничений членства**, написанный на Rust. Он воспроизводит самые востребованные **локальные** возможности Postman, использует совместимый формат **Postman Collection v2.1** и поставляется как кроссплатформенный единый бинарник (Windows / macOS / Linux).

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> Проект не подключается к облаку Postman; нет логина, подписки, пробного периода, Stripe и телеметрии. **Все функции безусловно доступны каждому.**

---

## Почему «без ограничений»

Платные/логин-ограничения Postman относятся в основном к **облачным** функциям (синхронизация, команды, облачные Mock/Monitor, Vault, Governance, AI и т. д.). Этот инструмент реализует только **независимые от облака локальные** возможности, поэтому не требует аккаунта, оплаты и квот. По замыслу ограничения на логин/подписку/членство удалены — все локальные функции доступны безусловно.

---

## Возможности

- Любой HTTP-метод; Header / Query / Body (raw / json / urlencoded / form-data / file / graphql)
- Переменные `{{name}}`; области `cli vars > iteration data > environment > collection > globals`; динамические переменные (`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp`, ...)
- Аутентификация: Bearer / Basic / API Key (header или query)
- Дерево коллекций / папок / запросов; переменные коллекции
- Окружения / глобальные переменные; история
- Collection Runner: несколько итераций, данные CSV/JSON, задержка, статистика проверок, код выхода для CI
- Генерация кода: curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- Импорт: Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2 (JSON), cURL, HAR
- Экспорт: Postman Collection v2.1 / Environment
- Проверки: декларативные (status / json_path / header / max_time_ms / regex и др.) + разбор распространённых `pm.test(...)`
- Протоколы: WebSocket (`pmrs ws`), SSE, MQTT, GraphQL, локальный Mock (`pmrs mock`), нагрузочное тестирование (`pmrs perf`)
- Сеть: прокси (`PMRS_PROXY`), CA-бандл, клиентский сертификат, пропуск проверки (`PMRS_INSECURE=1`), таймауты, редиректы, постоянный cookie jar
- OAuth2: client_credentials / password / refresh_token / accessToken

## GUI (pmrs-gui)

Настольное приложение на **egui/eframe** (чистый Rust, единый exe, без WebView). Китайский интерфейс с автозагрузкой CJK-шрифтов; слева коллекции / выпадающий список окружений / история, справа многозакладочный конструктор запросов (метод / URL / заголовки / тело / ответ); полноценный редактор тела (form-data / urlencoded / raw / binary / GraphQL); сохранение в коллекцию; экспорт v2.1; редактор окружений; инструменты (импорт / runner / генерация кода); панель ответа; консоль Postman; описание запроса; **8 тем**; единая дизайн-система.

## Сборка

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

Кроссплатформенность: Rust + reqwest(rustls), без OpenSSL; собирается на Windows / macOS / Linux.

## Готовые / кроссплатформенные артефакты

| Платформа | Артефакт |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS (universal: arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal` (собирается на macOS) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

Автоматически собирается `.github/workflows/build.yml` на `windows-latest` / `macos-latest` / `ubuntu-latest`.

### Образ контейнера (GHCR)

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## Использование

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## Рабочая область (локально)

По умолчанию `~/.postman-rs` (переопределяется через `PMRS_HOME`): `collections/*.json`, `environments/*.json`, `history.jsonl`. Обычные файлы, удобно для git, без базы данных и облака.

## Проверка

```bash
cargo test          # юнит + интеграция (запускает локальный HTTP-сервер)
cargo build --release
```

В бизнес-коде нет принудительных ветвей login/token/paywall/subscription/stripe/license.

## Частые вопросы (FAQ)

**Вопрос: Где находятся собранные файлы?** GUI: `dist/pmrs-gui-windows-x64.exe`; CLI: `dist/pmrs-windows-x64.exe`; архивы в `dist/`.

**Вопрос: Почему на Windows «не устанавливается»?** Это портативная программа одним файлом, а не установщик MSI/EXE — просто дважды щёлкните `pmrs-gui-windows-x64.exe`. Если SmartScreen блокирует, выберите «Подробнее → Выполнить».

**Вопрос: Как сменить тему?** Через выпадающий список «Theme» справа сверху (8 пресетов, запоминается) или `PMRS_THEME=0..7`.

**Вопрос: Как получить сборку для macOS?** Mach-O нельзя кросс-компилировать на Windows; соберите на Mac через `scripts/build-macos.sh` или в CI.

## Обратная связь (Issues)

Используйте GitHub Issues: <https://github.com/shuying-code/postman-rs/issues/new/choose>. Перед отправкой прочитайте **[руководство по обратной связи](docs/FEEDBACK.md)**. Сопровождающие обычно обрабатывают issues раз в неделю.

## Справочник CLI

16 подкоманд: `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. Подробнее — `pmrs --help`.

## Переменные окружения

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## Лицензия

MIT OR Apache-2.0. Это независимая реализация; она не содержит товарных знаков Postman и обеспечивает только совместимость форматов.
