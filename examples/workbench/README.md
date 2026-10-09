# 插件工作台

一个可安装的完整插件示例：体验请求处理与管理扩展，也可以用它完成摘要、翻译、改写，以及生成和编辑图片。

| 页面 | 功能 |
| --- | --- |
| 基础示例 | 文字转大写、追踪一次请求、调用插件接口 |
| 接入指南 | 终端命令、自定义客户端认证 |
| 文本工作台 | 粘贴文本或读取网页，流式生成、取消、继续调整，保存最近 8 条任务 |
| 图片工作台 | 自然语言生图、多轮续改、图片版本、参考图与取消 |

安装、开发环境、检查和打包命令统一见[仓库 README](../../README.md)。独立预览使用模拟数据；安装到兼容宿主后才能验证实际能力。

## 从哪里读起

先读 [`plugin.json`](plugin.json) 和 [`backend/src/app.rs`](backend/src/app.rs)：前者声明插件提供什么，后者用 `PluginBuilder` 连接实际处理器。随后只选择需要的能力：

| 想实现什么 | 后端入口 | 对应页面或调用方 |
| --- | --- | --- |
| 简单管理接口 | [`management/workbench.rs`](backend/src/management/workbench.rs) 的 `echo` | [`api/modules/echo.ts`](frontend/src/api/modules/echo.ts) |
| 改写请求正文 | [`request/middleware.rs`](backend/src/request/middleware.rs) | 基础示例「文字转大写」 |
| 图片交互 | 宿主已有 Responses 页面桥 | [`images.ts`](frontend/src/api/modules/images.ts)、[`imageConversation.ts`](frontend/src/utils/imageConversation.ts)、[`useImageWorkbench.ts`](frontend/src/composables/useImageWorkbench.ts) |
| 图片上传协议适配 | [`request/image_edit.rs`](backend/src/request/image_edit.rs) | HTTP 客户端调用 `/v1/images/edits` |
| 模型路由、账号选择 | [`request/routing.rs`](backend/src/request/routing.rs)、[`scheduler.rs`](backend/src/request/scheduler.rs) | 基础示例「追踪一次请求」 |
| `observer` 完成与 WebSocket 事件 | [`request/observer.rs`](backend/src/request/observer.rs) | [`useExampleRunner.ts`](frontend/src/composables/useExampleRunner.ts) |
| 保存状态、读取网页 | [`management/tasks.rs`](backend/src/management/tasks.rs)、[`text.rs`](backend/src/management/text.rs) | [`useTextWorkbench.ts`](frontend/src/composables/useTextWorkbench.ts) |
| 终端命令、客户端认证 | [`command.rs`](backend/src/command.rs)、[`authentication.rs`](backend/src/authentication.rs) | 接入指南中的命令 |

其余代码负责支撑这些能力，按需阅读：

- [`main.rs`](backend/src/main.rs)：校验握手并运行 SDK 会话；stdout 用于协议通信。
- [`management/`](backend/src/management/)：`registration.rs` 声明页面与路由，`router.rs` 分发请求，`validation.rs` 和 `response.rs` 统一校验与错误响应。
- [`host_calls.rs`](backend/src/host_calls.rs)：把宿主回调编码为 SDK 合同；[`request/scope.rs`](backend/src/request/scope.rs) 跟踪演示请求范围。
- [`evidence.rs`](backend/src/evidence.rs)：保存最近 64 条执行记录，供页面展示能力状态。只实现业务功能的插件通常不需要这套演示记录。
- [`tests/`](backend/tests/)：通过公开 SDK 会话验证处理器，模拟宿主资源回调。

复制示例开发新插件时，同步修改清单身份、包名、二进制名、注册描述和构建脚本；删除不使用的能力与页面。

## 页面如何调用插件

```text
页面 → api/modules → api/request.ts → 宿主桥 → management/router.rs → 业务处理器
```

[`frontend/src/api/`](frontend/src/api/) 按业务列出路由、参数与响应校验。`request({ url, method, data })` 接收插件的相对路由，由 `window.codexProxyPlugin.request` 交给宿主；页面不直接访问宿主 HTTP 接口、管理 Cookie 或 Key 明文。

文本和图片生成使用宿主桥的 `models.responses`，保留 JSON/SSE 响应与取消信号，进入宿主正常的模型请求链。Key 和模型选择分别使用非秘密 Key 列表及模型目录。

Vue 页面保留 SFC，使用 TypeScript 和 `@codex-proxy/ui`。宿主负责页面标题、主题同步和整页滚动，插件根容器通过 `min-height: inherit` 延续最小高度。独立预览位于 [`preview/`](frontend/src/preview/)，包内资源由 [`vite.config.ts`](frontend/vite.config.ts) 构建。

## 能力与边界

清单使用 SDK 0.1、清单 v2、通信协议 v2、中间件 v4，声明 7 类扩展能力。工作台要求的宿主版本与安装步骤见[体验插件](../../README.md#体验插件)。

| 行为 | 触发条件与边界 |
| --- | --- |
| 演示请求处理 | 仅处理带 `metadata.capability_workbench: "true"` 的请求；大写转换还需 `capability_workbench_uppercase: "true"`。两者都是字符串，转发上游前移除演示字段 |
| 图片编辑适配 | request 中间件自动处理 OpenAI `/v1/images/edits` 的 multipart 上传，不需要演示标记；JSON 请求保持原样 |
| 文本工作台 | 不发送演示标记，普通文本生成与继续调整保持原始输入 |
| 路由与调度 | 使用所选 Key 可用的内置 OpenAI/xAI 模型；平台候选唯一时确认平台，多个候选交由宿主选路。按在途数、失败率和权重选择账号，宿主复核资格与租约 |
| 网页取文 | 使用宿主受管网络，接收无凭据的 HTTP(S) 地址，不跟随重定向，保留最多 256 KiB 的 UTF-8 文本 |
| 执行记录 | 属于当前进程，重启后清空；已保存的文本任务使用宿主私有状态持久化 |
| 自定义认证 | 仅供独立测试环境演示；启用认证绑定并将示例 principal 映射到已有测试 Key 后使用 |

插件以 `trustedProcess` 运行，与宿主具有相同系统身份。安装意味着完整信任，清单不声明权限，宿主不按字段或访问域限制插件。

## 图片工作台

选择有图片工具权限的 Key 与 OpenAI 主模型，直接描述想要的图片；生成后继续发送「换成红色」「背景改回最开始的颜色」等消息，无需切换编辑模式或重新上传结果。左侧画布展示选中的图片，底部缩略图可回看每版，右侧保留每轮指令和模型回复。参考图是可选附件，支持 PNG/JPEG/WebP，单张最多 4 MiB。图片设置可调整画质与尺寸，每次最多生成一张，也支持只回复文字。

页面经 `models.responses` 调用 `image_generation` 工具，工具模型为 `gpt-image-2`，输出 PNG，`action: auto` 由模型根据会话决定生成或编辑。会话控制器在 [`imageConversation.ts`](frontend/src/utils/imageConversation.ts)：

1. 首轮通过 `generate: false`、`store: false` 预热完整输入，让宿主建立可续接的上游 WebSocket；拿到响应 ID 后发送空增量开始生成。
2. 正常续聊携带上一轮成功完成的 `previous_response_id`，只发送本轮文字和可选参考图，不重复发送历史图片。页面到宿主仍走 HTTP/SSE，宿主负责固定账号并复用上游连接。
3. 页面保留成功轮次的原始输入与输出项，包括图片、文字和加密推理项。上游明确返回 `previous_response_not_found` 且尚无语义输出时，最多自动恢复一次：用完整历史和本轮消息重新预热，再开始生成。普通错误、超时与取消不会自动重发。
4. 失败或停止的轮次不进入历史，续接句柄作废；手动重试从已提交历史建立新链。会话固定 Key 与主模型，调整画质或尺寸会重新预热，避免沿用不同请求参数的上下文。

这一做法参考 [Codex 的 WebSocket 增量续接与预热](https://github.com/openai/codex/blob/7f6c0f9387a0a60f396f61cc58f6b38bc98f2473/codex-rs/core/src/client.rs)。这里没有实现 Codex app-server 的 `threadId`；插件负责页面历史与发送顺序，宿主负责连接、账号授权与计费。正常轮次的请求大小主要取决于新增消息，但模型仍需处理累计上下文，增量传输不代表恒定推理时延或免费历史。

图片版本选择只改变预览，不改变对话分支。页面桥每次请求最多 8 MiB；正常增量可在历史超过此值后继续发送，但失效恢复或更改设置可能超过上限，此时明确提示新建对话，不删除或摘要历史。历史接近恢复上限时页面显示提示，累计保留内容达到 32 MiB 后阻止后续轮次，需新建对话释放历史。单次返回图片最多 32 MiB 的 Base64 数据。

当前示例不提供持久化会话列表，刷新或离开图片页会清空会话并取消请求。目录只表示 Key 可选模型，不保证每个模型或账号都支持图片工具与 WebSocket 预热；实际用量按宿主 Key 规则计费。

图片使用宿主允许的 `data:` 地址预览。隔离页不支持下载导航，因此保留浏览器图片保存交互。开发预览只绘制标明「模拟预览」的色块，不调用真实模型。

这条页面调用用于展示生图／编辑交互；下面的 multipart 中间件用于外部 Images 客户端，两者是独立示例，页面成功不能代替中间件适配验证。

## 图片编辑适配示例

启用工作台的 `middleware` / `request` 绑定，并确认请求在绑定范围内。客户端上传的 `image` 或 `image[]` 文件会转换成 `images: [{ image_url: "data:image/png;base64,…" }]`，然后通过一次 `next` 交给宿主原有 Images 路径。账号选择、OAuth 和上游响应仍由宿主处理；适配通过宿主的模型请求中间件执行。

```bash
curl "$CPR_BASE_URL/v1/images/edits" \
  -H "Authorization: Bearer $CPR_API_KEY" \
  -F 'model=gpt-image-2' \
  -F 'prompt=把蓝色方块改成红色，保留白色背景' \
  -F 'image=@input.png;type=image/png' \
  -F 'n=1' -F 'quality=low' -F 'size=1024x1024'
```

这是最小适配示例：支持 PNG/JPEG/WebP、最多 16 张图片、multipart 正文最多 16 MiB、单个文本字段最多 32 KiB。文本参数支持 `model`、`prompt`、`n`、`size`、`quality`、`background`；可指定 `response_format=b64_json`。`model`、`prompt` 和图片必填。mask、URL 输出及其他未支持参数会返回 HTTP 400，避免静默丢弃编辑条件。

成功响应沿用上游的 `data[].b64_json`；下游错误原样交回，不额外重试。工作台记录 `image_edit_multipart_adapted` 和下游状态，不记录图片、提示词或凭据。原生 JSON、其他路径和 attempt 阶段不转换。

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
- 在图片工作台验证生图、三轮增量续改、连接失效恢复、文字回复、版本回看、参考图、取消、错误后重试与明暗主题；切换页面后确认请求结束。
- 读取文本网页，检查重定向、非文本响应和网络失败的提示。
- 执行 `codex-proxy-rs plugin <实例 ID> ping`；自定义认证按接入指南在独立测试环境验证。
- WebSocket 观察需要实际产生上游 WebSocket 事件；仅确认 HTTP/SSE 请求成功不能证明观察已生效。

页面执行记录只表示实际收到的调用。安装成功、独立预览和测试通过都不能替代目标环境的业务验证。
