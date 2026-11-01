<p align="center">🌐 <b>언어</b>：<a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.ru.md">Русский</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.pt-BR.md">Português</a></p>

# postman-rs (`pmrs`)

Rust 로 작성된 **로컬 우선, 계정 불필요, 유료 없음, 멤버십 제한 없음** 의 Postman 스타일 API 클라이언트입니다. Postman 의 가장 많이 쓰이는 **로컬** 기능을 재현하고, **Postman Collection v2.1** 호환 형식을 사용하는 크로스 플랫폼 단일 바이너리(Windows / macOS / Linux)입니다.

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> 이 프로젝트는 Postman 클라우드에 연결하지 않습니다. 로그인, 구독, 체험, Stripe, 텔레메트리가 없습니다. **모든 기능을 누구나 무조건 사용할 수 있습니다.**

---

## 왜 "무제한"인가

Postman 의 유료/로그인 제약은 주로 **클라우드** 기능(동기화, 팀, 클라우드 Mock/Monitor, Vault, Governance, AI 등)에 대한 것입니다. 이 도구는 **클라우드에 의존하지 않는 로컬** 기능만 구현하므로 계정도, 결제도, 할당량도 필요 없습니다. 설계상 로그인/구독/멤버십 제한을 제거하여 모든 로컬 기능을 무조건 사용할 수 있습니다.

---

## 기능

- 모든 HTTP 메서드; Header / Query / Body(raw / json / urlencoded / form-data / file / graphql)
- 변수 `{{name}}`; 스코프 `cli vars > iteration data > environment > collection > globals`; 동적 변수(`$randomInt`, `$guid`, `$timestamp`, `$isoTimestamp` 등)
- 인증: Bearer / Basic / API Key(header 또는 query)
- 컬렉션 / 폴더 / 요청 트리; 컬렉션 변수
- 환경 / 전역 변수; 기록
- Collection Runner: 다중 반복, CSV/JSON 데이터 기반, 지연, 검증 통계, CI 종료 코드
- 코드 생성: curl / http / rust(reqwest) / python(requests) / js(fetch) / go / nodejs(axios) / java(okhttp) / csharp / php / ruby / powershell / swift
- 가져오기: Postman v2.1/v2.0/v1, OpenAPI 3 / Swagger 2(JSON), cURL, HAR
- 내보내기: Postman Collection v2.1 / Environment
- 검증: 선언형(status / json_path / header / max_time_ms / regex 등) + 일반적인 `pm.test(...)` 파싱
- 프로토콜: WebSocket(`pmrs ws`), SSE, MQTT, GraphQL, 로컬 Mock(`pmrs mock`), 성능 테스트(`pmrs perf`)
- 네트워크: 프록시(`PMRS_PROXY`), CA 번들, 클라이언트 인증서, 검증 건너뛰기(`PMRS_INSECURE=1`), 타임아웃, 리디렉션, 지속 쿠키 jar
- OAuth2: client_credentials / password / refresh_token / accessToken

## GUI(pmrs-gui)

**egui/eframe** 로 만든 데스크톱 앱(순수 Rust, 단일 exe, WebView 불필요). 중국어 UI와 자동 CJK 폰트. 왼쪽에 컬렉션 / 환경 드롭다운 / 기록, 오른쪽에 멀티 탭 요청 빌더(메서드 / URL / 헤더 / 본문 / 응답). 본문 편집기(form-data / urlencoded / raw / binary / GraphQL), 컬렉션에 저장, v2.1 내보내기, 환경 편집기, 도구(가져오기 / Runner / 코드 생성), 응답 패널, Postman 콘솔, 요청 설명, **8개 테마**, 통합 디자인 시스템.

## 빌드

```bash
cargo build --release
# target/release/pmrs(.exe)      — CLI
# target/release/pmrs-gui(.exe)  — GUI
```

크로스 플랫폼: Rust + reqwest(rustls), OpenSSL 불필요. Windows / macOS / Linux 에서 빌드 가능.

## 사전 빌드 / 크로스 플랫폼 산출물

| 플랫폼 | 산출물 |
|---|---|
| Windows x64 | `pmrs-windows-x64.exe`, `pmrs-gui-windows-x64.exe` |
| macOS(Universal: arm64 + x86_64) | `pmrs-macos-universal`, `pmrs-gui-macos-universal`(macOS 에서 빌드 필요) |
| Linux x64 | `pmrs-linux-x64`, `pmrs-gui-linux-x64` |

`.github/workflows/build.yml` 이 `windows-latest` / `macos-latest` / `ubuntu-latest` 에서 자동 빌드합니다.

### 컨테이너 이미지(GHCR)

```bash
docker pull ghcr.io/shuying-code/postman-rs:latest
docker run --rm ghcr.io/shuying-code/postman-rs:latest send "https://httpbin.org/get" -m GET
```

## 사용법

```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs collection new "My API"; pmrs collection list
pmrs env new "dev"; pmrs env set <env.json> baseUrl https://api.test
pmrs import examples/openapi.json --format openapi
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs history list
```

## 워크스페이스(로컬)

기본 `~/.postman-rs`(`PMRS_HOME` 로 재정의): `collections/*.json`, `environments/*.json`, `history.jsonl`. 순수 파일, git 관리 가능, 데이터베이스와 클라우드 불필요.

## 검증

```bash
cargo test          # 단위 + 통합(로컬 HTTP 서버 기동)
cargo build --release
```

비즈니스 코드에는 login/token/paywall/subscription/stripe/license 강제 분기가 없습니다.

## 자주 묻는 질문(FAQ)

**Q: 빌드된 파일은 어디에?** GUI: `dist/pmrs-gui-windows-x64.exe`; CLI: `dist/pmrs-windows-x64.exe`; zip 은 `dist/`.

**Q: Windows 에서 "설치할 수 없다"는?** 설치는 필요 없는 포터블 단일 파일입니다. `pmrs-gui-windows-x64.exe` 를 더블클릭하세요. SmartScreen 이 막으면 "추가 정보 → 실행"을 선택하세요.

**Q: 테마 변경은?** 우측 상단 "Theme" 드롭다운(8종, 기억됨) 또는 `PMRS_THEME=0..7`.

**Q: macOS 빌드는?** Mach-O 는 Windows 에서 크로스 컴파일할 수 없습니다. Mac 에서 `scripts/build-macos.sh` 를 실행하거나 CI 를 사용하세요.

## 피드백(Issues)

GitHub Issues 를 사용하세요: <https://github.com/shuying-code/postman-rs/issues/new/choose>. 제출 전 **[피드백 가이드](docs/FEEDBACK.md)** 를 읽어 주세요. 관리자는 보통 주 1회 일괄 처리합니다.

## CLI 명령 참고

16개 하위 명령: `send`, `request`, `run`, `import`, `export`, `codegen`, `collection`, `env`, `globals`, `history`, `ws`, `sse`, `mqtt`, `graphql`, `mock`, `perf`. 자세한 내용은 `pmrs --help`.

## 환경 변수

`PMRS_HOME`, `PMRS_PROXY`, `PMRS_CA_BUNDLE`, `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY`, `PMRS_INSECURE`, `PMRS_CONNECT_TIMEOUT`, `PMRS_MAX_REDIRECTS`, `PMRS_THEME`, `PMRS_GUI_SIDEBAR`.

## 라이선스

MIT OR Apache-2.0. 독립 구현이며 Postman 상표 자산을 포함하지 않고 형식 상호 운용만 제공합니다.
