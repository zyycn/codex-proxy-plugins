# OMP Codex 适配

通过 HTTP 和下游 WebSocket 中间件接入 OMP 的 `openai-codex` provider，无管理页面、业务配置或持久状态

## 安装与启用

插件要求宿主 **>=3.21.1, <4.0.0**，支持清单 v2、进程协议 v2 和中间件 v4

1. 按下方命令构建宿主平台的安装包，用同名 `.sha256` 核对摘要
2. 在宿主「插件管理」上传 `.tar.gz`，确认完整信任并安装
3. 启用默认实例，保留 `codex-proxy.omp-compat.middleware` 的 `http` 和 `websocket` 两项绑定

默认绑定的空范围允许插件检查所有入口，实际适配由路径和连接范围决定。HTTP 挂载发生在认证之前，保留其空身份范围；需要限制可用模型或账号时使用宿主 Client Key 的权限设置

插件进程与宿主使用相同系统身份，安装意味着完整信任。源码和 GitHub 的自动源码归档不能直接作为插件安装包

## OMP 配置

在 OMP 的 `models.yml` 中合并以下配置，保留其他 providers：

```yaml
providers:
  openai-codex:
    baseUrl: http://127.0.0.1:8080/v1
    apiKey: CODEX_PROXY_API_KEY
    api: openai-codex-responses
    discovery:
      type: openai-models-list
```

启动 OMP 前，将 `CODEX_PROXY_API_KEY` 环境变量设为宿主管理端创建的 Client Key；远程部署使用实际 HTTPS `/v1` 基址。模型使用该 Key 在 `/v1/models` 中可见的 ID

## 行为与边界

- 精确匹配的 `GET` / `POST /v1/codex/responses` 改写为 `/v1/responses`，查询参数保留原始编码和顺序。其他路径和方法继续由宿主处理
- 请求正文、普通多值头、宿主设置、超时和响应正文使用 SDK 原有资源透传，认证、版本检查、调度、continuation 和用量仍由宿主负责
- 通过 OMP 别名建立的 WebSocket 收到 `response.steer` 时，返回 `response.steer.failed`，以原 `steer.previous_response_id` 关联，错误码为 `unsupported_steering`。确认写入成功后丢弃这条控制消息，当前响应继续完成；输入由客户端在后续 `response.create` 中提交
- 原生 `/v1/responses` 连接不启用 steering 适配。出站消息、二进制和控制帧原样透传；其他入站文本保留原始字节并交给宿主解析
- HTTP 的 `stream` 缺省值与显式值均由宿主解释，插件不读取或补写正文。对话、hosted search、工具输出和压缩请求共用宿主 Responses 链；插件不赋予模型额外的搜索或压缩能力
- 停用后新连接按宿主原有路由处理，既有连接沿用宿主冻结的绑定计划直到断开。建议启停后重连 OMP

适配合同对应 [OMP `401778d0` 的 URL 与 steering 处理](https://github.com/can1357/oh-my-pi/blob/401778d0/packages/ai/src/providers/openai-codex-responses.ts)及[输入确认流程](https://github.com/can1357/oh-my-pi/blob/401778d0/packages/ai/src/providers/openai-codex/live-steering.ts)。官方 Codex 的 [`response.create` 请求序列化](https://github.com/openai/codex/blob/44fe510c/codex-rs/codex-api/src/endpoint/responses_websocket.rs)不需要这项 OMP 入口适配

## 开发与打包

使用仓库的 Rust 1.97 工具链，在仓库根目录执行：

```bash
cargo fmt --manifest-path examples/omp-compat/backend/Cargo.toml --check
RUST_MIN_STACK=16777216 cargo clippy --manifest-path examples/omp-compat/backend/Cargo.toml --all-targets --all-features --locked -- -D warnings
RUST_MIN_STACK=16777216 cargo test --manifest-path examples/omp-compat/backend/Cargo.toml --locked
```

测试通过真实插件子进程和模拟宿主 RPC 对端检查处理器行为，不调用真实模型。实际接入还需使用目标宿主和 OMP 核对鉴权、HTTP/SSE、WebSocket 工具续轮及生成中的 steering 拒绝

SDK 与打包 CLI 使用同一固定宿主提交 `f174320e2ac8987578146d4684e69ad196db0286`：

```bash
cargo install --locked --git https://github.com/zyycn/codex-proxy-rs.git --rev f174320e2ac8987578146d4684e69ad196db0286 codex-proxy-plugin-cli --root .tools
PLUGIN_CLI="$PWD/.tools/bin/cpr-plugin" bash scripts/package-omp
```

脚本只构建 Rust 二进制，在 `dist/` 生成 `codex-proxy.omp-compat-0.1.0-<target>.tar.gz` 和 `.sha256`，不需要 Node.js。默认使用本机平台，可传 `x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu` 或 `aarch64-apple-darwin`；交叉构建需要对应 Rust target 和链接工具
