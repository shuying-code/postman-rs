# 使用反馈指南（如何提交高质量问题）

感谢使用 **postman-rs (`pmrs`)**！为了让反馈能被**快速定位、复现和修复**，请在提交 issue 前花 2 分钟阅读本指南。

## 反馈入口

- **新建问题 / 建议**：<https://github.com/zhouyulong123/postman-rs/issues/new/choose>
  - 🐞 **Bug / 问题反馈** —— 可复现的功能异常、崩溃、报错
  - 💡 **功能建议** —— 新功能或改进
  - ❓ **使用咨询 / 反馈** —— 用法、配置、一般性反馈
- 也可以直接在仓库顶部 **Issues** 标签页创建。若不确定属于哪类，选「使用咨询 / 反馈」即可。

## 处理节奏（更新频率）

- 维护者**每周集中处理 / 回复一次** issue（约每周更新一次）。
- 新提交的 issue 会先做**分类与去重**；能复现的会标记优先级并排期；缺少关键信息（无法复现）的会标注 `need-repro` 并请你补充。
- 请**不要重复提交**同一问题；补充信息请直接在原 issue 里追加评论。

## 提交前检查清单

- [ ] 已搜索 [现有 issues](https://github.com/zhouyulong123/postman-rs/issues?q=is%3Aissue)，没有重复。
- [ ] 已查阅 [README](../README.md) 与 [FAQ](../README.md#常见问题faq)。
- [ ] 使用的是**最新版本**（见 [Releases](https://github.com/zhouyulong123/postman-rs/releases)）；旧版本请先升级复测。
- [ ] 已移除 **token / 密码 / 隐私数据**等敏感信息。

## 需要提供的关键信息（越全越好）

| 项目 | 说明 | 示例 |
|---|---|---|
| 版本 / 构建号 | `pmrs --version`，或 Release 文件名 / commit | `v0.1.0 · ecdfe4b` |
| 运行方式 | GUI / CLI / 容器镜像 / 源码构建 | `GUI（桌面 pmrs-gui）` |
| 系统与架构 | OS + CPU 架构 | `Windows 11 x64`、`macOS 14 arm64` |
| 复现步骤 | 一步一步，别人照做能重现 | 见下方示例 |
| 预期结果 | 你认为应该发生什么 | |
| 实际结果 | 实际发生了什么（可附截图/录屏） | |
| 日志 / 报错 | 控制台或终端完整输出 | GUI「控制台」侧栏 / 终端 stderr |
| 最小样例 | 最小 Collection/环境 JSON、cURL 命令或 方法+URL+头+体 | 见下文 |
| 出现频率 | 必现 / 偶现（大约几次一现） | `必现` |
| 网络环境 | 是否代理 / 自签证书 / mTLS / 内网 | `公司代理 + 自签 CA` |

### 复现步骤示例

```
1. 打开 pmrs-gui（v0.1.0）
2. 新建请求，方法选 POST，URL 填 https://httpbin.org/post
3. 请求体类型选 JSON，内容 {"a":1}
4. 点“发送”
5. 期望返回 200 和 JSON 回显；实际界面卡在“发送中…”且控制台无输出
```

## 如何获取这些信息

- **版本号**：终端执行 `pmrs --version`；GUI 版本对应 Release 文件名与 commit。
- **日志 / 控制台**：
  - GUI：左侧「控制台」侧栏会记录请求/响应/脚本测试日志，点「复制全部」粘贴到 issue。
  - CLI：把终端里的完整报错（含堆栈）粘贴出来（用 ``` 包成代码块更清晰）。
- **最小复现样例（强烈建议）**：
  - **cURL**：GUI 请求栏点「cURL」一键复制；或「导出」导出 `.sh`。
  - **集合 / 环境**：只保留能触发问题的最小请求，导出为 Postman v2.1 JSON 后贴出（可用代码块或附件）。
  - **接口类问题**：给出一条可复现的 `方法 + URL + 关键请求头 + 请求体`，尽量不要依赖私有接口。
- **截图 / 录屏**：GUI 交互问题建议附截图（含侧栏与请求区）或短录屏。

## 敏感信息脱敏

提交前务必替换：`Authorization`、`Cookie`、`token`、账号密码、内网域名/IP、真实业务数据等，用 `xxx` 或示例值代替。

## 安全问题（漏洞）

请**不要**在公开 issue 中披露安全漏洞。请使用 GitHub 仓库 **Security → Report a vulnerability**（Private vulnerability reporting）私下上报，我们会优先处理。

## Issue 标签与处理流程

| 标签 | 含义 |
|---|---|
| `bug` | 已确认为缺陷 |
| `enhancement` | 功能建议 / 改进 |
| `question` | 使用咨询 |
| `need-repro` | 信息不足、暂时无法复现，等待补充 |
| `duplicate` | 与已有 issue 重复 |
| `wontfix` | 暂不处理（附原因） |

处理流程：**分类去重 → 复现确认 → 定优先级排期 → 修复并合并 → 在 Release/CHANGELOG 记录 → 关闭 issue**。

---

反馈越具体，修复越快。谢谢你的支持！🙌
