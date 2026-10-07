<!-- prettier-ignore -->
<div align="center">

<img src="https://raw.githubusercontent.com/zyycn/codex-proxy-rs/main/frontend/public/favicon.svg" alt="Codex Proxy" width="80" height="80" />

# Codex Proxy Plugins

通过可运行的示例，开发你的 Codex Proxy 插件

[![CI](https://github.com/zyycn/codex-proxy-plugins/actions/workflows/ci.yml/badge.svg)](https://github.com/zyycn/codex-proxy-plugins/actions/workflows/ci.yml)
[![插件下载](https://img.shields.io/badge/下载-插件安装包-blue?style=flat-square)](https://github.com/zyycn/codex-proxy-plugins/releases)

[体验插件](#体验插件) · [本地开发](#本地开发) · [构建安装包](#构建安装包) · [文档](#文档)

</div>

[Codex Proxy RS](https://github.com/zyycn/codex-proxy-rs) 的插件示例仓库。后端使用公开 Rust SDK，页面使用宿主桥与 `@codex-proxy/ui`，可以独立构建和发布。

完整示例：[**插件工作台**](examples/workbench/README.md)。你可以先体验功能，再按需要阅读对应处理器：

| 功能 | 可以学到什么 |
| --- | --- |
| 基础示例 | 文本大写转换、模型路由、账号调度、请求与用量观察、管理接口 |
| 接入指南 | 终端命令、自定义客户端认证 |
| 文本工作台 | 摘要、翻译、改写、网页取文、流式生成、取消、继续调整与保存记录 |
| 图片工作台 | 自然语言生图、增量续聊、失效恢复、版本回看与参考图；附 multipart 图片上传适配示例 |

已有发行附件的协议与兼容范围以附件内清单为准

仅需客户端适配时，可使用无页面的 [OMP Codex 适配插件](examples/omp-compat/README.md)，通过 HTTP 和下游 WebSocket 中间件接入 OMP 的 Responses 入口与 steering 拒绝确认；构建使用 `scripts/package-omp`

## 体验插件

1. 从 [Releases](https://github.com/zyycn/codex-proxy-plugins/releases) 下载与**宿主运行平台**匹配的 `.tar.gz`，同时下载 `.sha256` 核对摘要。
2. 在宿主「插件管理」中上传安装包，确认完整信任后安装。
3. 启用自动准备的默认配置，从「扩展页」打开「插件工作台」。模型示例需要选择已有的可用 Key 和模型。

> [!IMPORTANT]
> 当前工作台要求宿主支持清单 v2、协议 v2、中间件 v3，并满足 `>=3.17.0, <4.0.0`。宿主 v3.17.0 正式包只支持旧协议，不能安装本版；可使用宿主提交 `b37d7901441e123cb9126a6f9af4ff3321548d24` 或后续兼容构建。通过 GitHub 来源安装正式版时，无需允许预发行；使用预发行版时需明确填写标签并允许预发行。

安装工作台意味着完整信任，清单不声明访问权限。插件进程与宿主使用相同系统身份；运行模型示例会产生真实用量。各能力的触发条件见[示例说明](examples/workbench/README.md#能力与边界)。

## 本地开发

准备 **Rust 1.97、Node.js 24、pnpm 12.6**，然后执行：

```bash
git clone https://github.com/zyycn/codex-proxy-plugins.git
cd codex-proxy-plugins
pnpm --dir examples/workbench/frontend install --frozen-lockfile
pnpm --dir examples/workbench/frontend dev
```

独立预览使用模拟宿主，适合阅读页面和调试交互，不调用真实模型。后端由宿主通过标准输入输出启动，实际能力需要构建安装包后验证。

```text
examples/workbench/
├── plugin.json    插件身份、能力与资源声明
├── backend/       Rust 处理器与会话测试
├── frontend/      Vue 页面、宿主桥与独立预览
└── README.md      按能力阅读源码与接口说明
scripts/package   构建并打包
dist/             安装包与校验文件（不入库）
```

从 [`app.rs`](examples/workbench/backend/src/app.rs) 查看处理器如何组合，再按[源码导航](examples/workbench/README.md#从哪里读起)选择需要的能力。开发自己的插件时，只保留需要的处理器及对应清单声明。

SDK 与打包 CLI 固定到同一宿主提交 `b37d7901441e123cb9126a6f9af4ff3321548d24`，UI 使用 `v0.3.0`，实际依赖由各自锁文件固定；无需检出宿主或组件库。需要联合修改时，使用宿主的[源码联调入口](https://github.com/zyycn/codex-proxy-rs/blob/main/docs/development.md)，正式构建仍使用锁定依赖。

在本仓库根目录执行检查：

```bash
pnpm --dir examples/workbench/frontend lint
pnpm --dir examples/workbench/frontend build
cargo fmt --manifest-path examples/workbench/backend/Cargo.toml --check
RUST_MIN_STACK=16777216 cargo clippy --manifest-path examples/workbench/backend/Cargo.toml --all-targets --all-features --locked -- -D warnings
RUST_MIN_STACK=16777216 cargo test --manifest-path examples/workbench/backend/Cargo.toml --locked
```

## 构建安装包

安装以下固定提交的打包工具，在仓库根目录运行：

```bash
cargo install --locked --git https://github.com/zyycn/codex-proxy-rs.git --rev b37d7901441e123cb9126a6f9af4ff3321548d24 codex-proxy-plugin-cli --root .tools
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" bash scripts/package
```

脚本构建前后端，在 `dist/` 生成 `.tar.gz` 和 `.sha256`。默认使用本机平台，也可指定目标，例如：

```bash
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" bash scripts/package aarch64-unknown-linux-gnu
```

支持 Linux x86_64、Linux aarch64、macOS aarch64；交叉构建需准备对应 Rust target 和链接工具。已安装插件使用包内资源，修改源码后需重新构建、打包并切换版本。

发行时同步 `plugin.json` 的版本和 `release/notes.md`，从 `main` 推送对应的 `v<插件版本>` 标签。[发布工作流](.github/workflows/release.yml)在检查通过后生成三个平台的安装包与校验文件。不带预发行后缀的版本发布为正式版并更新 Latest，带 `-alpha.N`、`-beta.N` 或 `-rc.N` 后缀的版本发布为 Pre-release。

## 文档

| 任务 | 入口 |
| --- | --- |
| 阅读示例与管理接口 | [插件工作台](examples/workbench/README.md) |
| 安装、配置与管理版本 | [宿主插件使用说明](https://github.com/zyycn/codex-proxy-rs/blob/main/docs/plugins.md) |
| 编写 Rust 插件 | [SDK](https://github.com/zyycn/codex-proxy-rs/blob/b37d7901441e123cb9126a6f9af4ff3321548d24/backend/crates/gateway-plugin/sdk/README.md) · [清单](https://github.com/zyycn/codex-proxy-rs/blob/b37d7901441e123cb9126a6f9af4ff3321548d24/backend/crates/gateway-plugin/sdk/docs/manifest.md) · [能力合同](https://github.com/zyycn/codex-proxy-rs/blob/b37d7901441e123cb9126a6f9af4ff3321548d24/backend/crates/gateway-plugin/sdk/docs/capabilities.md) |
| 自定义打包流程 | [插件 CLI](https://github.com/zyycn/codex-proxy-rs/blob/b37d7901441e123cb9126a6f9af4ff3321548d24/backend/apps/plugin-cli/README.md) |
| 编写管理页面 | [UI 组件库](https://github.com/zyycn/codex-proxy-ui) · [宿主主题约定](https://github.com/zyycn/codex-proxy-rs/blob/main/docs/theme.md) |
