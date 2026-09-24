use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::{Map, Value};

const MAXIMUM_EVIDENCE: usize = 64;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Evidence {
    pub id: String,
    pub capability: String,
    pub event: String,
    pub outcome: EvidenceOutcome,
    pub occurred_at_ms: u64,
    pub request_id: Option<String>,
    pub provider: Option<String>,
    pub account_id: Option<String>,
    pub model: Option<String>,
    pub details: Map<String, Value>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EvidenceOutcome {
    Passed,
    Failed,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EvidenceSnapshot {
    pub latest: BTreeMap<String, Evidence>,
    pub entries: Vec<Evidence>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExampleStatus {
    pub id: &'static str,
    pub title: &'static str,
    pub capabilities: &'static [&'static str],
    pub status: ExampleOutcome,
    pub run_count: u64,
    pub last_evidence_id: Option<String>,
    pub trigger: &'static str,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExampleOutcome {
    NotRun,
    Passed,
    Failed,
    Pending,
}

struct ExampleDefinition {
    id: &'static str,
    title: &'static str,
    capabilities: &'static [&'static str],
    trigger: &'static str,
}

const EXAMPLES: &[ExampleDefinition] = &[
    ExampleDefinition {
        id: "page-and-configuration",
        title: "页面与配置",
        capabilities: &["management"],
        trigger: "打开页面，或调用文本回显接口",
    },
    ExampleDefinition {
        id: "request-processing",
        title: "请求加工",
        capabilities: &["middleware"],
        trigger: "发送匹配绑定范围的请求；metadata.capability_workbench_uppercase 设为字符串 true 时转换为大写",
    },
    ExampleDefinition {
        id: "routing-and-scheduling",
        title: "模型路由与账号调度",
        capabilities: &["model_router", "scheduler"],
        trigger: "选择可用模型发送带工作台标记的请求",
    },
    ExampleDefinition {
        id: "request-observation",
        title: "请求观察",
        capabilities: &["request_lifecycle", "usage", "web_socket_observer"],
        trigger: "完成一次模型请求，并单独发送真实 WebSocket 请求",
    },
    ExampleDefinition {
        id: "command-line",
        title: "命令行",
        capabilities: &["command_line"],
        trigger: "在宿主终端执行 codex-proxy-rs plugin <实例 ID> ping",
    },
    ExampleDefinition {
        id: "client-authentication",
        title: "客户端认证",
        capabilities: &["frontend_authentication"],
        trigger: "在测试环境启用认证绑定，再使用演示请求头调用",
    },
];

#[derive(Default)]
pub(crate) struct EvidenceLog {
    next_id: AtomicU64,
    state: Mutex<EvidenceState>,
}

#[derive(Default)]
struct EvidenceState {
    entries: VecDeque<Evidence>,
    latest: BTreeMap<String, Evidence>,
    counts: BTreeMap<String, u64>,
}

impl EvidenceLog {
    pub fn record(&self, input: EvidenceInput<'_>) -> Evidence {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let evidence = Evidence {
            id: format!("e-{id}"),
            capability: input.capability.to_owned(),
            event: input.event.to_owned(),
            outcome: input.outcome,
            occurred_at_ms: now_ms(),
            request_id: input.request_id.map(str::to_owned),
            provider: input.provider.map(str::to_owned),
            account_id: input.account_id.map(str::to_owned),
            model: input.model.map(str::to_owned),
            details: input.details,
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .latest
            .insert(evidence.capability.clone(), evidence.clone());
        let count = state.counts.entry(evidence.capability.clone()).or_default();
        *count = count.saturating_add(1);
        state.entries.push_back(evidence.clone());
        while state.entries.len() > MAXIMUM_EVIDENCE {
            state.entries.pop_front();
        }
        evidence
    }

    pub fn snapshot(&self) -> EvidenceSnapshot {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        EvidenceSnapshot {
            latest: state.latest.clone(),
            entries: state.entries.iter().cloned().collect(),
        }
    }

    pub fn examples(&self) -> Vec<ExampleStatus> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        EXAMPLES
            .iter()
            .map(|definition| {
                let latest = definition
                    .capabilities
                    .iter()
                    .filter_map(|capability| state.latest.get(*capability))
                    .collect::<Vec<_>>();
                let status = if latest
                    .iter()
                    .any(|entry| entry.outcome == EvidenceOutcome::Failed)
                {
                    ExampleOutcome::Failed
                } else if latest.len() == definition.capabilities.len() {
                    ExampleOutcome::Passed
                } else if latest.is_empty() {
                    ExampleOutcome::NotRun
                } else {
                    ExampleOutcome::Pending
                };
                ExampleStatus {
                    id: definition.id,
                    title: definition.title,
                    capabilities: definition.capabilities,
                    status,
                    run_count: definition
                        .capabilities
                        .iter()
                        .fold(0_u64, |total, capability| {
                            total
                                .saturating_add(state.counts.get(*capability).copied().unwrap_or(0))
                        }),
                    last_evidence_id: latest
                        .into_iter()
                        .max_by_key(|entry| evidence_number(&entry.id))
                        .map(|entry| entry.id.clone()),
                    trigger: definition.trigger,
                }
            })
            .collect()
    }
}

fn evidence_number(id: &str) -> u64 {
    id.strip_prefix("e-")
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}

pub(crate) struct EvidenceInput<'a> {
    pub capability: &'a str,
    pub event: &'a str,
    pub outcome: EvidenceOutcome,
    pub request_id: Option<&'a str>,
    pub provider: Option<&'a str>,
    pub account_id: Option<&'a str>,
    pub model: Option<&'a str>,
    pub details: Map<String, Value>,
}

impl<'a> EvidenceInput<'a> {
    pub fn passed(capability: &'a str, event: &'a str) -> Self {
        Self {
            capability,
            event,
            outcome: EvidenceOutcome::Passed,
            request_id: None,
            provider: None,
            account_id: None,
            model: None,
            details: Map::new(),
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}
