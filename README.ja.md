<p align="center">🌐 <b>言語</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

Rust で書かれた**ローカル志向・アカウント不要・課金なし・会員制限なし**の Postman 風 API クライアントです。Postman の最もよく使われる**ローカル**機能を再現し、**Postman Collection v2.1** 互換フォーマットを採用した、クロスプラットフォームの単一バイナリ（Windows / macOS / Linux）です。

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> 本プロジェクトは Postman クラウドに接続しません。ログイン・サブスク・試用・Stripe・テレメトリはありません。**すべての機能を誰でも無条件に利用できます。**

---

## なぜ「無制限」なのか

Postman の有料 / ログイン制限は主に**クラウド**機能（同期、チーム、クラウド Mock/Monitor、Vault、Governance、AI など）に対するものです。本ツールは**クラウド非依存のローカル**機能のみを実装しているため、アカウントも課金もクォータも不要です。設計上、ログイン / サブスク / 会員の制限を取り除き、すべてのローカル機能を無条件に利用できます。

---

## 機能

- 任意の HTTP メソッド。Header / Query / Body（raw / json / urlencoded / form-data / file / graphql）
- 変数 `{{name}}`、スコープ `cli vars > iteration data > environment > collection > globals`、動的変数（`$randomInt`、`$guid`、`$timestamp`、`$isoTimestamp` など）
- 認証：Bearer / Basic / API Key（header または query）
- コレクション / フォルダ / リクエストのツリー、コレクション変数
- 環境 / グローバル変数、履歴
- Collection Runner：複数イテレーション、CSV/JSON データ駆動、遅延、アサーション集計、CI 終了コード
- コード生成：curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- インポート：Postman v2.1/v2.0/v1、OpenAPI 3 / Swagger 2（JSON）、cURL、HAR
- エクスポート：Postman Collection v2.1 / Environment
- アサーション：宣言型（status / json_path / header / max_time_ms / regex など）+ 一般的な `pm.test(...)` 解析
- プロトコル：WebSocket（`pmrs ws`）、SSE、MQTT、GraphQL、ローカル Mock（`pmrs mock`）、性能試験（`pmrs perf`）
- ネットワーク：プロキシ（`PMRS_PROXY`）、CA バンドル、クライアント証明書、検証スキップ（`PMRS_INSECURE=1`）、タイムアウト、リダイレクト、永続 Cookie jar
- OAuth2：client_credentials / password / refresh_token / accessToken

## GUI（pmrs-gui）

**egui/eframe** で作られたデスクトップアプリ（純 Rust、単一 exe、WebView 不要）。中国語 UI と CJK フォント自動読込。左にコレクション / 環境ドロップダウン / 履歴、右にマルチタブのリクエストビルダー（メソッド / URL / ヘッダー / ボディ / レスポンス）。ボディエディタ（form-data / urlencoded / raw / binary / GraphQL）、コレクションへ保存、v2.1 エクスポート、環境エディタ、ツール（インポート / Runner / コード生成）、レスポンスパネル、Postman コンソール、リクエスト説明、**8 テーマ**、統一デザインシステム。

## ビルド

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

クロスプラットフォーム：Rust + reqwest(rustls)、OpenSSL 不要。Windows / macOS / Linux でビルド可能。

## ビルド済み / クロスプラットフォーム成果物

| プラットフォーム | 成果物 |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`、`pmrs-gui-windows-x64.exe` |
| macOS（Universal：arm64 + x86_64） | `pmrs-macos-universal`、`pmrs-gui-macos-universal`（macOS 上でビルド） |
| Linux x64 | `pmrs-linux-x64`、`pmrs-gui-linux-x64` |

`.github/workflows/build.yml` により `windows-latest` / `macos-latest` / `ubuntu-latest` で自動ビルドされます。

### コンテナイメージ（GHCR）

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## 使い方

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## ワークスペース（ローカル）

既定 `~/.postman-rs`（`PMRS_HOME` で上書き可）：`collections/*.json`、`environments/*.json`、`history.jsonl`。プレーンファイルで git 管理可能、データベースもクラウドも不要。

## 検証

```bash
cargo test          # ユニット + 統合（ローカル HTTP サーバーを起動）
cargo build --release
```

業務コードに login/token/paywall/subscription/stripe/license の強制分岐はありません。

## よくある質問（FAQ）

**Q：ビルド後のファイルはどこ？** GUI：`dist/pmrs-gui-windows-x64.exe`、CLI：`dist/pmrs-windows-x64.exe`、zip は `dist/`。

**Q：Windows で「インストールできない」のはなぜ？** これはインストーラではなくポータブルの単一ファイルです。`pmrs-gui-windows-x64.exe` をダブルクリックするだけです。SmartScreen に止められたら「詳細情報 → 実行」を選んでください。

**Q：テーマを変えるには？** 右上の「Theme」ドロップダウン（8 種、記憶される）か `PMRS_THEME=0..7`。

**Q：macOS 版はどう入手？** Mach-O は Windows でクロスコンパイルできません。Mac で `scripts/build-macos.sh` を実行するか CI でビルドしてください。

## フィードバック（Issues）

GitHub Issues をご利用ください：<https://github.com/shuying-code/postman-rs/issues/new/choose>。投稿前に **[フィードバックガイド](docs/FEEDBACK.md)** をお読みください。メンテナは通常週 1 回まとめて対応します。

## CLI コマンド一覧

16 個のサブコマンド：`send`、`request`、`run`、`import`、`export`、`codegen`、`collection`、`env`、`globals`、`history`、`ws`、`sse`、`mqtt`、`graphql`、`mock`、`perf`。詳細は `pmrs --help`。

## 環境変数

`PMRS_HOME`、`PMRS_PROXY`、`PMRS_CA_BUNDLE`、`PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`、`PMRS_INSECURE`、`PMRS_CONNECT_TIMEOUT`、`PMRS_MAX_REDIRECTS`、`PMRS_THEME`、`PMRS_GUI_SIDEBAR`。

## ライセンス

MIT OR Apache-2.0。本プロジェクトは独立実装であり、Postman の商標資産を含まず、フォーマット相互運用のみを目的とします。
