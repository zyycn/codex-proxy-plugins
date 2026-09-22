# Request Workbench

Request Workbench 是仓库维护的单一综合插件示例。它只使用公开 Rust SDK，演示两条可直接验证的路径：

- request 阶段的 middleware 在下游成功返回后追加 `x-cpr-example` 响应头，不读取或改写请求正文；
- 受管理身份保护的页面读取本进程统计，并调用一个不访问外网的本地 echo API。

插件 ID 为 `codex-proxy.request-workbench`，示例版本为 `0.1.0`，需要 codex-proxy-rs
`>=3.13.0, <4.0.0`、SDK `0.3.0`、manifest v2 和 protocol v3。它是教学示例，不是内置插件，安装后不会自动启用或获得授权。

## 构建

页面使用 Vue 3、Tailwind CSS v4 和独立的 `@codex-proxy/ui` 包。依赖准备与同级仓库联调方式见[仓库说明](../../README.md#本地开发)。页面不引用宿主前端源码。

宿主统一呈现 `ManagementPage` 声明的标题、副标题和刷新操作，插件只渲染业务内容，不重复添加页面标题或外围容器。内容沿用宿主主题并占满可用宽度，小屏自动堆叠，超出可用高度时在内容区滚动。

在仓库根目录构建页面和本机 Linux 插件：

```bash
corepack pnpm install --frozen-lockfile
corepack pnpm --filter @codex-proxy/request-workbench-web build

cargo +1.97 build \
  --release \
  --locked \
  --target x86_64-unknown-linux-gnu \
  --manifest-path examples/request-workbench/Cargo.toml
```

统一 Rust 打包器从 `web/dist/` 读取已构建页面，生成的归档不会包含页面源码或 `node_modules`：

```bash
cpr-plugin package \
  --manifest examples/request-workbench/plugin.json \
  --binary examples/request-workbench/target/x86_64-unknown-linux-gnu/release/codex-proxy-plugin-request-workbench \
  --target x86_64-unknown-linux-gnu \
  --output-dir dist \
  --resource-map web=web/dist
```

支持的平台与完整参数见宿主仓库的 Plugin CLI 文档，入口见[仓库说明](../../README.md#依赖与发行)。
归档包含 `plugin.json`、`LICENSE`、`bin/plugin`、`assets/icon.png` 和已声明的 `web/` 静态资源，
并在同目录生成 `.sha256`。本示例使用 PNG 图标；其他格式及浅色/深色图标配置见
宿主 SDK 文档的「清单与图标」章节。

## 安装与启用

通过管理端上传生成的 tar.gz，或使用 `POST /api/admin/plugins/artifacts/upload`。安装只保存制品；随后创建实例时还需：

- 显式确认 `trustedProcess: true`；
- 配置可选的 `headerValue`，缺省为 `official-example`，仅接受 1–128 个可见 ASCII 字符且首尾不能是空格；
- 授予清单已声明的 `response_headers_write`；
- 将 `codex-proxy.request-workbench.middleware` 绑定到 `request` 阶段，并按环境收窄 Client Key、账号组、Provider 或模型范围。

### API 配置示例

下面是覆盖全部请求的实例字段示意。共享环境应填写范围数组，而不是直接沿用空数组：

```json
{
  "name": "Request Workbench",
  "artifactSha256": "<安装返回的制品摘要>",
  "enabled": true,
  "trustedProcess": true,
  "configuration": {
    "headerValue": "official-example"
  },
  "secrets": {},
  "grants": [
    {
      "permission": "response_headers_write",
      "operations": [],
      "providerIds": [],
      "accountIds": [],
      "networkOrigins": [],
      "networkRanges": [],
      "namespaces": []
    }
  ],
  "bindings": [
    {
      "contribution": "codex-proxy.request-workbench.middleware",
      "stage": "request",
      "order": 100,
      "failurePolicy": "delegate",
      "clientKeyIds": [],
      "accountGroupIds": [],
      "providerIds": [],
      "models": [],
      "identityBindings": []
    }
  ]
}
```

本示例的管理 API 不调用宿主模型，因此 `management` 声明会直接发布页面与路由，无需额外绑定执行身份；只有 middleware 需要上述 request 阶段绑定。

## 验证

实例发布成功后按以下路径检查，构建成功不等于已安装或已生效：

| 操作 | 预期结果 |
| --- | --- |
| 打开“请求工作台” | 显示响应标记和当前进程处理计数，页面主题与宿主一致 |
| 在页面发送 `hello` | echo 返回相同文本，不访问外网 |
| 用绑定范围内的客户端 Key 发起正常模型请求 | 成功响应包含下列标记，计数增加 |

```text
x-cpr-example: official-example
```

### 管理接口

页面入口来自 `GET /api/admin/plugins/extensions` 返回的 `pages`，使用同一冻结 target 调用：

- `GET .../api/status` → `{ "headerValue": "official-example", "handledRequests": 1 }`；
- `POST .../api/echo`，正文 `{ "message": "hello" }` → `{ "message": "hello" }`。

完整 target 和管理 API 路径以宿主的 [插件管理 API](https://github.com/zyycn/codex-proxy-rs/blob/main/docs/api.md#12-插件管理) 为准。

## 边界

- 计数只保存在插件进程内，重启、重新发布或切换制品后归零，不是持久审计数据。
- echo 最多接受 256 个 UTF-8 字节且拒绝控制字符；它不访问 Provider、账号、凭据或网络。
- 页面资源都需要管理身份，没有声明公开资源，也不需要 `public_resource` 权限。
- 示例没有 Provider、命令行、secret 或私有状态能力；新增这些行为应使用相应 SDK 合同和独立授权，不能复用本示例的响应头权限代替。
