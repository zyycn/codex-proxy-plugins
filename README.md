# Codex Proxy 插件示例

面向插件作者的官方示例仓库。插件独立于宿主发布，通过公开 SDK 和宿主桥接入，不复制宿主业务模块。

[插件工作台](examples/workbench/README.md) 提供基础能力体验、接入指南，以及摘要、翻译、改写组成的文本处理示例。

## 目录

```text
examples/workbench/
├── plugin.json        插件清单
├── backend/           Rust 工程与后端测试
├── frontend/          Vue 工程、独立依赖及前端工具配置
└── README.md          功能与开发说明

scripts/package        构建与打包入口
dist/                  安装包与校验文件，不入库
```

仓库根目录不维护 Node 工程。每个示例的前后端分别管理依赖，具体目录和职责见示例说明。

## 本地开发

工具链：Node.js 24、pnpm 11.7、Rust 1.97

仅需检出本仓库。SDK 固定到 `f770ba127d1293bb482921019e61cae7cb3b7de9`，UI 使用 GitHub Release `v0.1.0` 的安装包；两者均由锁文件固定，不依赖本机同级目录。在仓库根目录执行：

```bash
pnpm --dir examples/workbench/frontend install --frozen-lockfile
pnpm --dir examples/workbench/frontend dev
```

验证入口：

```bash
pnpm --dir examples/workbench/frontend lint
pnpm --dir examples/workbench/frontend build
RUST_MIN_STACK=16777216 cargo test --locked --manifest-path examples/workbench/backend/Cargo.toml
```

Vite 独立预览与宿主安装是两种环境。已安装插件使用包内 JS/CSS，修改源码后需重新构建、打包并切换版本。

## 打包

安装宿主提供的打包工具，再运行脚本：

```bash
cargo install --locked --git https://github.com/zyycn/codex-proxy-rs.git --rev f770ba127d1293bb482921019e61cae7cb3b7de9 codex-proxy-plugin-cli --root .tools
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" bash scripts/package
```

默认构建本机平台，也可传入目标平台：`bash scripts/package aarch64-unknown-linux-gnu`。支持 Linux x86_64、Linux aarch64、macOS aarch64，交叉构建需准备对应 Rust target 和链接工具。

产物进入根目录 `dist/`，包括 `.tar.gz` 安装包和 `.sha256`。打包不代表安装、启用或实际能力验证成功。

## 依赖与发行

SDK、打包器和 UI 组件库各自维护版本。更新依赖时使用发布包或固定 Git 提交，重新生成并提交锁文件，不使用浮动分支或开发机路径。

更新插件清单版本和 `release/notes.md`，从 `main` 推送对应的 `v<插件版本>` 标签。发布工作流在质量检查通过后构建 Linux x86_64、Linux aarch64 和 macOS aarch64 安装包，并附 `.sha256` 校验文件。

当前 SDK 与插件接口处于实验阶段，插件发布标记为 Pre-release。插件工作台要求支持清单 v1、协议 v1 的宿主，版本范围为 `>=3.14.0, <4.0.0`，不能安装到不支持插件能力的 `3.13.1` 正式宿主。

宿主文档按所用 SDK 版本查阅：

| 宿主仓库路径 | 内容 |
| --- | --- |
| `backend/crates/gateway-plugin/sdk/README.md` | 清单、会话与能力合同 |
| `backend/apps/plugin-cli/README.md` | 打包参数与平台要求 |
| `docs/plugins.md` | 安装、权限、使用与版本管理 |
