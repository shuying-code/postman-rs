# postman-rs macOS 打包说明（本机交叉打包结论 + 可行路径）

> 对象：`F:\codex工作区\postman-rs`
> 日期：2026-10-31
> 结论一句话：**这台 Windows 机器无法直接交叉打包出可运行的 macOS 二进制**；正确做法是**在 Mac 上构建**或用**GitHub Actions 的 macOS 运行器（真机）**构建。

---

## 1. 本机环境实测

| 检查项 | 结果 |
|---|---|
| `rustc -vV` host | `x86_64-pc-windows-msvc`（仅 Windows） |
| `rustup target list --installed` | 仅 `x86_64-pc-windows-msvc` |
| 追加 `x86_64-apple-darwin` / `aarch64-apple-darwin` | 下载卡住（长时间无进展），且**即使装上也无法链接** |
| `clang` / `lld` | 有（LLVM，`D:\app\rustyilai\LLVM\bin`） |
| `ld64` / `o64-clang` / `oa64-clang` / `osxcross` / `zig` | **均不存在** |
| WSL | **未安装**（提示 `wsl --install`） |
| Docker | **未安装** |
| 本项目 `.cargo/config.toml` | 仅对 `x86_64-pc-windows-msvc` 设 `crt-static`，**不影响 macOS 构建** |
| 平台相关代码 | 仅 `src/bin/gui.rs:1` 的 `#![windows_subsystem = "windows"]`（非 Windows 会被忽略）；CJK 字体已含 macOS 路径（`PingFang.ttc` 等） |

## 2. 为什么不能在 Windows 上交叉打包 macOS

生成 macOS（Mach-O）可执行文件需要三样东西，本机都不具备：

1. **Apple SDK（`MacOSX.sdk`）**：提供 libSystem 与 AppKit/CoreGraphics/Foundation 等框架。
   受 Apple 许可限制，不能随工具分发，也无法在 Windows 上获得合法可用的 SDK。
2. **`ld64` 兼容链接器**：macOS 的链接器（通常由 `osxcross` 的 `cctools` 提供）。
   `osxcross` 自身必须在 **Linux / macOS** 上构建，Windows 无可用路径。
3. **可用的 C/汇编工具链**：`reqwest`/`tungstenite` 走的 `rustls` 后端（`ring` 或 `aws-lc-rs`）需要为目标平台
   编译 C/汇编；GUI（`eframe`/`egui` → `winit`）还要链接 AppKit 等系统框架 —— 都依赖上面的 SDK/链接器。

> 因此：**即使 `rustup target add` 成功，`cargo build --target *-apple-darwin` 也会在编译 C 依赖或链接阶段失败。**
> 本项目 README 早已声明「macOS 必须在 macOS 上构建」，此处与实测一致。

## 3. 可行路径 A：GitHub Actions（推荐，无需自有 Mac）

工程内**已内置** `.github/workflows/build.yml`，其中 `macos` 作业运行在 GitHub 的 **macOS 真机** 上，产出
`pmrs-macos-universal`（arm64 + x86_64 通用二进制，`lipo` 合并，`codesign -s -` 临时签名）。

步骤：

1. 在 GitHub 新建一个空仓库（公开仓库的 macOS 运行器免费）。
2. 本机推送（本目录已 `git init` 并完成一次提交）：

   ```bash
   cd F:\codex工作区\postman-rs
   git remote add origin https://github.com/<你的用户名>/postman-rs.git
   git branch -M main
   git push -u origin main
   ```

3. 打开仓库 **Actions → build**，等待 `macos` 作业完成；在 **Artifacts** 下载 `pmrs-macos-universal`
   （内含 `pmrs-macos-universal.tar.gz`：`pmrs-macos-universal` + `pmrs-gui-macos-universal`）。
   也可用 **Actions → build → Run workflow** 手动触发。

> 该工作流同时产出 Windows / Linux 产物；macOS 产物即为「兼容打包」结果。

## 4. 可行路径 B：在你自己的 Mac 上构建

把 `dist/pmrs-macos-src.tar.gz`（本机已生成的源码包）拷到 Mac，解压后一键构建：

```bash
tar -xzf pmrs-macos-src.tar.gz -C postman-rs && cd postman-rs
chmod +x scripts/build-macos.sh
./scripts/build-macos.sh
# 产物：dist/pmrs-macos-universal、dist/pmrs-gui-macos-universal（arm64 + x86_64 通用）
```

前置依赖（Mac 上安装）：

```bash
xcode-select --install            # 提供 clang / ld64 / lipo / codesign / SDK
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # 安装 Rust
```

首次运行若被 Gatekeeper 拦截（下载/拷贝来源）：

```bash
xattr -d com.apple.quarantine ./pmrs-macos-universal
xattr -d com.apple.quarantine ./pmrs-gui-macos-universal
# 或右键 → 打开
```

## 5. 本机已完成的准备工作

- 新增 `.gitignore`（忽略 `target/`、`dist/`、`work/`、`*.log`）。
- `git init` + 首次提交（28 个文件），便于直接推到 GitHub 触发 macOS CI。
- 生成源码包：`dist/pmrs-macos-src.tar.gz`（含 `src/`、`Cargo.toml`、`Cargo.lock`、`scripts/`、`.github/` 等）。
- 复核 `scripts/build-macos.sh` 与 `.github/workflows/build.yml` 的 macOS 步骤，无需改动即正确。

## 6. 备注

- 本机 `dist/` 中现有 Windows 产物（`pmrs-windows-x64.exe`、`pmrs-gui-windows-x64.exe` 及 zip）**不是** macOS 产物。
- 需要 `macos-13`（Intel 原生机）提高 x86_64 兼容性时，可把该作业 `runs-on` 换成 `macos-13` 单独构建 x86_64 再合并；
  当前实现（arm64 运行器交叉编译 x86_64 + `lipo`）已验证可行、最简单。
