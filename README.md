# Codex Proxy Plugins

Codex Proxy 的官方维护插件示例。插件源码和发版节奏独立于宿主；SDK、打包器由宿主仓库维护，UI 组件由 `codex-proxy-ui` 维护，均不复制到本仓库。

当前示例为 [Request Workbench](examples/request-workbench/README.md)，演示请求中间件、管理接口和继承宿主主题的 Vue 页面。它是教学示例，不是默认内置插件，安装后仍需配置范围和显式授权。

## 本地开发

本地开发使用以下同级目录。UI 通过已配置的 pnpm override 链接源码，SDK 使用 Cargo 路径依赖；
相关依赖尚未公开发布，当前不能只检出本仓库后从公网完成独立安装。

```text
Codes/
├── codex-proxy-rs/       # 公开 SDK 和打包工具
├── codex-proxy-ui/       # UI 源码与组件文档
└── codex-proxy-plugins/  # 本仓库
```

工具链为 Node.js 24、pnpm 11.7、Rust 1.97。在 UI 仓库先运行 `pnpm install --frozen-lockfile && pnpm build`，然后在本仓库运行：

```bash
pnpm install --frozen-lockfile
pnpm lint
pnpm build
cargo test --locked --manifest-path examples/request-workbench/Cargo.toml
```

同时修改 UI 时，可在 UI 仓库运行 `pnpm dev` 使用原生 Vite 热更新预览；集成到插件发布包前运行 `pnpm build` 并重新构建示例页面。安装到宿主的插件包含自己的 JS/CSS，不依赖开发机路径。

## 打包

打包器由宿主仓库维护，可以安装到本仓库已忽略的 `.tools` 目录：

```bash
cargo install --locked --path ../codex-proxy-rs/backend/apps/plugin-cli --root .tools
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" pnpm package
```

默认构建本机目标，也可以 `pnpm package aarch64-unknown-linux-gnu`。交叉构建需自行安装对应 Rust target 和链接工具，或在对应平台构建。支持 Linux x86_64、Linux aarch64、macOS aarch64。

产物进入根目录 `dist/`，包括完整插件 ID、版本、平台命名的 `.tar.gz` 与 `.sha256`。上传这个包即可安装，勿上传源码 zip、UI npm 包或仅有页面资源的目录。插件版本、宿主兼容范围和能力声明以示例的 `plugin.json` 为准。

## 依赖与发行

UI npm 包与插件安装包独立发版，宿主发行物不构建或附带本仓库示例。
对外发布前，须将 UI 的本地 override 换成已发布版本、SDK 的路径依赖换成包含所需接口的固定 Git 提交，
并生成对应锁文件；不要依赖浮动 `main` 或开发机路径。

宿主文档按所用 SDK 版本查阅，以下宿主路径相对于 `codex-proxy-rs` 仓库根目录：

| 文档 | 用途 |
| --- | --- |
| SDK：`backend/crates/gateway-plugin/sdk/README.md` | 清单、图标、会话与能力合同 |
| Plugin CLI：`backend/apps/plugin-cli/README.md` | 打包参数与平台要求 |
| [Request Workbench](examples/request-workbench/README.md) | 可运行示例、配置与验证 |
| 插件使用：`docs/plugins.md` | 安装、授权、使用与版本管理 |
