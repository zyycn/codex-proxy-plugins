# 插件工作台

一个可安装的完整插件示例：体验请求处理与管理扩展，也可以用它完成摘要、翻译、改写等文本任务。

| 页面 | 功能 |
| --- | --- |
| 基础示例 | 文字转大写、追踪一次请求、调用插件接口 |
| 接入指南 | 终端命令、自定义客户端认证 |
| 文本工作台 | 粘贴文本或读取网页，流式生成、取消、继续调整，保存最近 8 条任务 |

安装、开发环境、检查和打包命令统一见[仓库 README](../../README.md)。独立预览使用模拟数据；安装到兼容宿主后才能验证实际能力。

## 从哪里读起

先读 [`plugin.json`](plugin.json) 和 [`backend/src/app.rs`](backend/src/app.rs)：前者声明插件提供什么、访问什么，后者用 `PluginBuilder` 连接实际处理器。随后只选择需要的能力：

| 想实现什么 | 后端入口 | 对应页面或调用方 |
| --- | --- | --- |
| 简单管理接口 | [`management/workbench.rs`](backend/src/management/workbench.rs) 的 `echo` | [`api/modules/echo.ts`](frontend/src/api/modules/echo.ts) |
| 改写请求正文 | [`request/middleware.rs`](backend/src/request/middleware.rs) | 基础示例「文字转大写」 |
| 模型路由、账号选择 | [`request/routing.rs`](backend/src/request/routing.rs)、[`scheduler.rs`](backend/src/request/scheduler.rs) | 基础示例「追踪一次请求」 |
| 请求、用量、WebSocket 观察 | [`request/observer.rs`](backend/src/request/observer.rs) | [`useExampleRunner.ts`](frontend/src/composables/useExampleRunner.ts) |
| 保存状态、读取网页 | [`management/tasks.rs`](backend/src/management/tasks.rs)、[`text.rs`](backend/src/management/text.rs) | [`useTextWorkbench.ts`](frontend/src/composables/useTextWorkbench.ts) |
| 终端命令、客户端认证 | [`command.rs`](backend/src/command.rs)、[`authentication.rs`](backend/src/authentication.rs) | 接入指南中的命令 |

其余代码负责支撑这些能力，按需阅读：

- [`main.rs`](backend/src/main.rs)：校验握手并运行 SDK 会话；stdout 用于协议通信。
- [`management/`](backend/src/management/)：`registration.rs` 声明页面与路由，`router.rs` 分发请求，`validation.rs` 和 `response.rs` 统一校验与错误响应。
- [`host_calls.rs`](backend/src/host_calls.rs)：把宿主回调编码为 SDK 合同；[`request/scope.rs`](backend/src/request/scope.rs) 跟踪演示请求范围。
- [`evidence.rs`](backend/src/evidence.rs)：保存最近 64 条执行记录，供页面展示能力状态。只实现业务功能的插件通常不需要这套演示记录。
- [`tests/`](backend/tests/)：通过公开 SDK 会话验证处理器，模拟宿主资源回调。

复制示例开发新插件时，同步修改清单身份、包名、二进制名、注册描述和构建脚本；删除不使用的能力、页面与权限声明。

## 页面如何调用插件

```text
页面 → api/modules → api/request.ts → 宿主桥 → management/router.rs → 业务处理器
```

[`frontend/src/api/`](frontend/src/api/) 按业务列出路由、参数与响应校验。`request({ url, method, data })` 接收插件的相对路由，由 `window.codexProxyPlugin.request` 交给宿主；页面不直接访问宿主 HTTP 接口、管理 Cookie 或 Key 明文。

文本生成使用宿主桥的 `models.responses`，保留 JSON/SSE 响应与取消信号，进入宿主正常的模型请求链。Key 和模型选择分别使用非秘密 Key 列表及模型目录。

Vue 页面保留 SFC，使用 TypeScript 和 `@codex-proxy/ui`。宿主负责页面标题、主题同步和整页滚动，插件根容器通过 `min-height: inherit` 延续最小高度。独立预览位于 [`preview/`](frontend/src/preview/)，包内资源由 [`vite.config.ts`](frontend/vite.config.ts) 构建。

## 能力与边界

清单使用 SDK 0.1、清单 v1、通信协议 v1，声明 9 类扩展能力。工作台要求的宿主版本与安装步骤见[体验插件](../../README.md#体验插件)。

| 行为 | 触发条件与边界 |
| --- | --- |
| 演示请求处理 | 仅处理带 `metadata.capability_workbench: "true"` 的请求；大写转换还需 `capability_workbench_uppercase: "true"`。两者都是字符串，转发上游前移除演示字段 |
| 文本工作台 | 不发送演示标记，普通文本生成与继续调整保持原始输入 |
| 路由与调度 | 使用所选 Key 可用的内置 OpenAI/xAI 模型；平台候选唯一时确认平台，多个候选交由宿主选路。按在途数、失败率和权重选择账号，宿主复核资格与租约 |
| 网页取文 | 使用宿主受管网络，接收无凭据的 HTTP(S) 地址，不跟随重定向，保留最多 256 KiB 的 UTF-8 文本 |
| 执行记录 | 属于当前进程，重启后清空；已保存的文本任务使用宿主私有状态持久化 |
| 自定义认证 | 仅供独立测试环境演示；启用认证绑定并将示例 principal 映射到已有测试 Key 后使用 |

权限包括 `network`（网页取文）、`models`（Key、模型目录与模型调用）、`requests`（请求处理与观察）、`public_endpoints`（公开的 `web/app.css` 示例资源）。日志和自身私有状态无需额外权限。插件以 `trustedProcess` 运行，与宿主具有相同系统身份。

## 管理接口

路径不带前导斜杠，不接受查询参数。GET 不带正文或内容类型；POST 使用 `application/json`，正文最多 512 KiB。

| 接口 | 参数 | 结果 |
| --- | --- | --- |
| `GET api/snapshot` | 无 | 首批最多 100 个 Key、后续游标、能力摘要与执行记录 |
| `POST api/echo` | `{ "message": "你好" }` | 原样返回文本，限 1–4096 个 UTF-8 字节且不含控制字符 |
| `POST api/models` | `{ "clientKeyId": "…" }` | 指定 Key 可用的模型 |
| `GET api/tasks` | 无 | `{ version, value }`，未保存时均为 `null` |
| `POST api/tasks` | `{ expectedVersion, value }` | 保存任务并返回新版本；`value` 结构见[任务合同](frontend/src/api/types/workbench.ts) |
| `POST api/fetch-text` | `{ "url": "https://…" }` | 文本、HTTP 状态、内容类型、字节数与截断标记 |
| `POST api/log` | `{}` | 写入固定诊断事件并返回 `{ recorded }` |

成功响应直接返回业务数据，错误格式为 `{ "error": { "code": "…", "message": "…" } }`。任务最多保存 8 条，总编码大小不超过 256 KiB；写入使用版本校验，过期返回 HTTP 409，需重新加载后再保存。

## 验证实际能力

自动检查见[本地开发](../../README.md#本地开发)，业务验收需要在已安装并启用的插件上进行：

- 在基础示例中发送请求，核对输入、输出、模型、Token 与执行记录；未带标记的请求保持原行为。
- 在文本工作台生成、取消、继续调整，保存后重新打开，并验证多页面保存冲突。
- 读取文本网页，检查重定向、非文本响应和网络失败的提示。
- 执行 `codex-proxy-rs plugin <实例 ID> ping`；自定义认证按接入指南在独立测试环境验证。
- WebSocket 观察需要实际 WebSocket 请求；HTTP/SSE 成功不能证明它已生效。

页面执行记录只表示实际收到的调用。安装成功、独立预览和测试通过都不能替代目标环境的业务验证。
