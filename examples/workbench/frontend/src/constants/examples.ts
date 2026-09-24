import type { ExampleGuide } from '../types'
import { CaseUpper, Fingerprint, PanelsTopLeft, Route, Terminal } from '@lucide/vue'

export const requestMilestones = [
  { event: 'route_decided', label: '选择模型' },
  { event: 'account_ordered', label: '选择账号' },
  { event: 'request_observed', label: '收到请求终态' },
]

export const exampleGuides: ExampleGuide[] = [
  {
    id: 'text-transform',
    title: '转换成大写',
    summary: '插件先把输入文字转成大写，再交给所选模型，调用记录展示中间件是否执行',
    icon: CaseUpper,
    group: 'interactive',
    action: 'uppercase',
    actionLabel: '转换文本',
    capabilities: ['middleware'],
    expected: '模型收到大写后的输入；返回内容由模型生成，可在调用记录中核对 uppercased',
  },
  {
    id: 'request-pipeline',
    title: '追踪一次请求',
    summary: '使用所选模型，由插件从宿主候选中选择平台和账号，观察流式响应与最终用量',
    icon: Route,
    group: 'interactive',
    action: 'request',
    actionLabel: '发送请求',
    capabilities: ['model_router', 'scheduler', 'request_lifecycle', 'usage', 'web_socket_observer'],
    expected: '运行后查看模型、执行账号、Token 用量与插件收到的调用记录',
  },
  {
    id: 'plugin-api',
    title: '调用插件接口',
    summary: '页面把文字发送到插件的 Echo 接口，再显示返回值，不经过模型或账号',
    icon: PanelsTopLeft,
    group: 'interactive',
    action: 'echo',
    actionLabel: '发送文本',
    capabilities: ['management'],
    expected: '例如发送“你好，插件”，接口会返回同样的文字',
  },
  {
    id: 'terminal-command',
    title: '终端命令',
    summary: '不打开管理页面，也可以从网关命令行调用插件，用 ping 确认命令接入',
    icon: Terminal,
    group: 'integration',
    action: 'external',
    actionLabel: '',
    capabilities: ['command_line'],
    expected: '终端输出 capability-workbench pong，退出码为 0',
    steps: [
      '从插件配置中复制实例 ID，替换下方占位符',
      '在网关运行目录执行命令，沿用该环境的配置和数据库连接',
      '直接查看终端输出，CLI 的独立进程不会把结果写进当前网页',
    ],
    command: 'codex-proxy-rs plugin <实例 ID> ping',
  },
  {
    id: 'custom-authentication',
    title: '自定义认证',
    summary: '用插件定义的请求头识别调用者，再映射到宿主已有的测试 Key',
    icon: Fingerprint,
    group: 'integration',
    action: 'external',
    actionLabel: '',
    capabilities: ['frontend_authentication'],
    expected: 'demo 身份得到模型响应，改成 wrong 后应被拒绝，不会替代后台登录',
    steps: [
      '仅在独立测试环境启用本插件的客户端认证',
      '把外部身份 capability-workbench-demo-user 映射到一个可用的测试 Key',
      '选择该 Key 可访问的内置平台模型，替换下方模型和网关地址',
    ],
    command: `curl <网关地址>/v1/responses \\\n  -H 'Authorization: CapabilityWorkbench demo' \\\n  -H 'Content-Type: application/json' \\\n  -d '{"model":"<可用模型>","input":"Hello, plugin!"}'`,
  },
]
