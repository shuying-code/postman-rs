<p align="center">🌐 <b>語言</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

一套以 Rust 撰寫的 **本地化、無帳號、無付費、無會員限制** 的 Postman 風格 API 用戶端。它重現了 Postman 最常用的 **本地** 能力，採用 **Postman Collection v2.1** 相容格式，跨平台單一二進位檔（Windows / macOS / Linux）。

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> 本專案不連線 Postman 雲端，不含登入、訂閱、試用、Stripe、遙測。**所有功能對任何人無條件可用。**

---

## 為何「無限制」

Postman 的付費與登入主要約束的是**雲端能力**（同步、團隊、雲端 Mock/Monitor、Vault、Governance、AI 等）。本工具只實作**不依賴雲端**的本地能力，因此天生不需要帳號、不需要付費、沒有配額。設計上移除了登入 / 訂閱 / 會員限制，所有本地功能無條件可用。

---

## 功能

- 任意 HTTP 方法；Header / Query / Body（raw / json / urlencoded / form-data / file / graphql）
- 變數 `{{name}}`；作用域 `cli vars > iteration data > environment > collection > globals`；動態變數（`$randomInt`、`$guid`、`$timestamp`、`$isoTimestamp` 等）
- 鑑權：Bearer / Basic / API Key（header 或 query）
- 集合 / 資料夾 / 請求樹；集合變數
- 環境 / 全域變數；歷史記錄
- Collection Runner：多迭代、CSV/JSON 資料驅動、延遲、斷言統計、CI 退出碼
- 程式碼產生：curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- 匯入：Postman v2.1/v2.0/v1、OpenAPI 3 / Swagger 2（JSON）、cURL、HAR
- 匯出：Postman Collection v2.1 / Environment
- 斷言：宣告式（status / json_path / header / max_time_ms / regex 等）+ 常見 `pm.test(...)` 解析
- 協定：WebSocket（`pmrs ws`）、SSE、MQTT、GraphQL、本地 Mock（`pmrs mock`）、效能壓測（`pmrs perf`）
- 網路：代理（`PMRS_PROXY`）、CA 憑證包、用戶端憑證、跳過校驗（`PMRS_INSECURE=1`）、逾時、重新導向、持久 Cookie jar
- OAuth2：client_credentials / password / refresh_token / accessToken

## 圖形介面（pmrs-gui）

以 **egui/eframe** 打造的桌面程式（純 Rust、單 exe、無需 WebView）。中文介面並自動載入 CJK 字型；左側為集合清單 / 環境下拉 / 歷史，右側為多分頁請求建立器（方法 / URL / 標頭 / 內容 / 回應）；完整內容編輯器（form-data / urlencoded / raw / binary / GraphQL）；儲存至集合；匯出 v2.1；環境編輯器；工具（匯入 / Runner / 產生程式碼）；回應面板（內容 / 標頭 / Cookies / 測試 / 計時）；Postman 主控台；請求描述；**8 套主題**；統一設計系統。

## 建置

```bash
cargo build --release
# target/release/pmrs(.exe)      — 命令列
# target/release/pmrs-gui(.exe)  — 圖形介面
```

跨平台：Rust + reqwest(rustls)，無 OpenSSL 依賴；可在 Windows / macOS / Linux 直接編譯。

## 預編譯 / 跨平台產物

| 平台 | 產物 |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`、`pmrs-gui-windows-x64.exe` |
| macOS（Universal：arm64 + x86_64） | `pmrs-macos-universal`、`pmrs-gui-macos-universal`（需在 macOS 上建置） |
| Linux x64 | `pmrs-linux-x64`、`pmrs-gui-linux-x64` |

由 `.github/workflows/build.yml` 在 `windows-latest` / `macos-latest` / `ubuntu-latest` 自動建置。

### 容器映像（GHCR）

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## 使用

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## 工作區（本地）

預設 `~/.postman-rs`（可用 `PMRS_HOME` 覆寫）：`collections/*.json`、`environments/*.json`、`history.jsonl`。純檔案、可納入 git、無資料庫、無雲端。

## 驗收

```bash
cargo test          # 單元 + 整合（本地起 HTTP 伺服器）
cargo build --release
```

業務程式碼中不存在 login/token/paywall/subscription/stripe/license 的強制分支。

## 常見問題（FAQ）

**Q：建置後的檔案在哪？** 圖形介面：`dist/pmrs-gui-windows-x64.exe`；命令列：`dist/pmrs-windows-x64.exe`；壓縮檔在 `dist/`。

**Q：Windows 產物為何「安裝不了」？** 本專案是免安裝的綠色單檔程式，不是 MSI/EXE 安裝包——直接雙擊 `pmrs-gui-windows-x64.exe` 即可。若被 SmartScreen 攔截，選「更多資訊 → 仍要執行」。

**Q：如何切換主題？** 使用右上角「主題」下拉（8 套，會記憶）或 `PMRS_THEME=0..7`。

**Q：macOS 版本如何取得？** Mach-O 無法在 Windows 交叉編譯；在 Mac 上用 `scripts/build-macos.sh` 或透過 CI 建置。

## 問題回報（Issues）

請使用 GitHub Issues：<https://github.com/shuying-code/postman-rs/issues/new/choose>。提交前請先閱讀 **[使用回報指南](docs/FEEDBACK.md)**。維護者通常每週集中處理一次 issue。

## CLI 命令參考

16 組子命令：`send`、`request`、`run`、`import`、`export`、`codegen`、`collection`、`env`、`globals`、`history`、`ws`、`sse`、`mqtt`、`graphql`、`mock`、`perf`。詳見 `pmrs --help`。

## 環境變數

`PMRS_HOME`、`PMRS_PROXY`、`PMRS_CA_BUNDLE`、`PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`、`PMRS_INSECURE`、`PMRS_CONNECT_TIMEOUT`、`PMRS_MAX_REDIRECTS`、`PMRS_THEME`、`PMRS_GUI_SIDEBAR`。

## 授權

MIT OR Apache-2.0。本專案為獨立實作，不包含 Postman 商標資產，僅做格式互操作。
