# 插件工作台

一个可安装、可交互的插件开发示例，组合基础体验与文本处理工作流

- **基础示例**：文字转大写、追踪一次请求、调用插件接口
- **接入指南**：终端命令、自定义客户端认证
- **文本工作台**：摘要、翻译、改写、取消生成、继续调整与保存记录

使用公开 SDK 0.1、清单 v1、通信协议 v1，覆盖 9 类扩展能力。它不是宿主内置功能，不会自动安装或启用。

## 工程结构

```text
workbench/
├── plugin.json                插件身份、能力、权限与资源清单
├── backend/
│   ├── Cargo.toml             Rust 依赖与构建配置
│   ├── Cargo.lock
│   ├── src/
│   │   ├── lib.rs             公开入口，不承载业务实现
│   │   ├── main.rs            会话启动与握手校验
│   │   ├── app.rs             组合 SDK 处理器
│   │   ├── manifest.rs        作者清单
│   │   ├── management/        路由、请求校验与管理业务
│   │   ├── request/           中间件、路由、调度与观察
│   │   ├── authentication.rs  客户端身份识别
│   │   ├── command.rs         终端命令
│   │   ├── evidence.rs        有界执行记录与示例状态
│   │   └── host_calls.rs      类型化宿主回调
│   └── tests/                 镜像业务目录，通过公开 SDK 会话验证
└── frontend/
    ├── package.json           独立依赖与开发命令
    ├── pnpm-lock.yaml
    ├── pnpm-workspace.yaml    包管理配置，不声明仓库级 workspace
    ├── eslint.config.ts
    ├── tsconfig*.json         应用与构建配置分别检查
    ├── vite.config.ts
    └── src/
        ├── App.vue            页面组合与视图切换
        ├── main.ts            挂载入口
        ├── api/
        │   ├── index.ts       统一出口
        │   ├── modules/       按业务列出路由、方法和参数
        │   ├── schemas/       快照、历史记录等复杂响应校验
        │   ├── types/         接口与宿主桥数据合同
        │   ├── request.ts     宿主桥请求封装
        │   ├── host.ts        获取公开宿主桥
        │   └── validation.ts  接口数据校验
        ├── views/             以 index.vue 作为页面入口
        ├── components/        展示与输入组件
        ├── composables/       请求、历史记录和生成状态
        ├── constants/         示例定义与固定选项
        ├── types/             页面类型与环境声明
        ├── utils/             流式解析、格式化等纯函数
        ├── styles/            样式入口
        └── preview/           独立开发预览及模拟数据
```

前端通过公开宿主桥调用插件，不能读取管理 Cookie、Key 明文或宿主内部模块。Vue 保留 SFC，逻辑与配置使用 TypeScript，通用控件和主题来自 `@codex-proxy/ui`。

管理接口的通信路径：

```text
api/modules → request → 宿主页面消息桥 → 网关 HTTP 接口 → 插件处理器
```

`request({ url, method, data })` 只负责 JSON 编解码和错误处理，`url` 是插件注册的相对路由，不是可直接访问的 HTTP 地址。接口模块校验响应后返回业务类型，页面不处理底层消息，也不需要 Axios 适配器。模型请求单独使用 `models.responses`，保留原始 JSON/SSE 与取消信号。

宿主自动提供最小页面高度并同步内容尺寸，插件无需手动上报。页面保持自然文档流，由宿主承接整页滚动；Vue 根容器通过 `min-height: inherit` 延续最小高度。

## 本地开发

依赖准备见[仓库说明](../../README.md)。以下命令在本示例目录执行：

```bash
pnpm --dir frontend install --frozen-lockfile
pnpm --dir frontend dev
```

独立预览使用模拟宿主，不调用真实模型，也不能作为插件能力已通过验证的证据。

```bash
pnpm --dir frontend lint
pnpm --dir frontend build
RUST_MIN_STACK=16777216 cargo test --locked --manifest-path backend/Cargo.toml
cargo clippy --locked --all-targets --manifest-path backend/Cargo.toml -- -D warnings
```

前端以 ESLint、类型检查、构建和实际页面验收为验证入口，不维护前端测试文件。后端测试保留。

## 打包与安装

```bash
cargo build --release --locked --target x86_64-unknown-linux-gnu --manifest-path backend/Cargo.toml
pnpm --dir frontend build

cpr-plugin package \
  --manifest plugin.json \
  --binary backend/target/x86_64-unknown-linux-gnu/release/codex-proxy-plugin-workbench \
  --target x86_64-unknown-linux-gnu \
  --resource-map web=frontend/dist \
  --output-dir dist
```

`web/` 是安装包中的资源路径，映射 `frontend/dist/` 构建产物，不是源码目录。公开静态资源示例复用 `web/app.css`，无需额外演示文件。

上传生成的 `.tar.gz` 安装包，在宿主查看权限并安装。安装包包含本机进程；只在信任源码和所需权限时启用。宿主版本须满足清单的 `engines` 范围，开发版宿主也需提供符合范围的版本号。

## 能力与边界

请求示例与文本任务使用宿主内置 OpenAI/xAI 的可用模型，模型候选由所选 Key 的范围决定。
平台候选唯一时插件确认该平台，多个候选时继续由宿主按模型能力选路。账号调度按在途数、失败率与权重选择候选；最终资格和租约由宿主复核。

模型请求会产生真实用量与费用。网页取文通过宿主网络能力执行，仅接收无凭据的 HTTP(S) 地址，最多读取 256 KiB，不跟随重定向。

请求扩展只处理带 `metadata.capability_workbench: "true"` 标记的请求。大写转换还需 `capability_workbench_uppercase: "true"`，普通文本任务不会被转换为大写。

文本工作台不发送演示标记，首次生成和继续调整均不依赖演示中间件是否启用或匹配，避免将示例参数带给真实模型服务。

## 管理接口

| 接口 | 前端模块（`src/api/modules/`） | 用途 |
| --- | --- | --- |
| `GET api/snapshot` | `workbench.ts` | 可用 Key、能力摘要与本进程执行记录 |
| `POST api/echo` | `echo.ts` | 原样返回一段文本 |
| `POST api/models` | `models.ts` | 查询指定 Key 可用的模型 |
| `GET / POST api/tasks` | `tasks.ts` | 读取与保存最多 8 条文本任务 |
| `POST api/fetch-text` | `text.ts` | 通过宿主读取网页正文 |
| `POST api/log` | `log.ts` | 写入固定的非敏感诊断事件 |

调用路径不带前导斜杠。历史记录使用版本校验，过期写入返回 HTTP 409，避免覆盖其他页面的更新。Key 列表仅包含 ID、名称和启用状态。

## 实际体验

页面记录只表示真实收到的调用，不以声明成功或注册成功代替验证结果

- 在基础示例中运行一次请求，查看输入、输出、模型、Token 与对应请求记录
- WebSocket 观察需要真实 WebSocket 客户端，HTTP/SSE 成功不能代替它
- 终端执行 `codex-proxy-rs plugin <实例 ID> ping`
- 自定义认证仅在独立测试环境启用，将示例身份映射到已有测试 Key，再按页面命令调用

执行记录属于当前插件进程，重启后会重置；已保存文本任务属于插件私有状态。
