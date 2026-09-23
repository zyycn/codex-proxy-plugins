use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::management::{CommandDescriptor, CommandInvocation, CommandRegistration, CommandResult},
    client::{TypedCall, TypedReply},
};

pub(crate) fn command_registration() -> CommandRegistration {
    CommandRegistration {
        commands: vec![CommandDescriptor {
            name: "ping".to_owned(),
            description: "返回工作台的固定连通性响应。".to_owned(),
            parameters: Vec::new(),
        }],
    }
}

pub(crate) async fn command_line(
    evidence: &EvidenceLog,
    call: TypedCall<CommandInvocation>,
) -> Result<TypedReply<CommandResult>, PluginFault> {
    if call.request.name != "ping" || !call.request.arguments.is_empty() {
        return Ok(TypedReply::new(CommandResult {
            stdout: String::new(),
            stderr: "用法：ping\n".to_owned(),
            exit_code: 2,
            accounts: Vec::new(),
        }));
    }
    evidence.record(EvidenceInput::passed("command_line", "ping_executed"));
    Ok(TypedReply::new(CommandResult {
        stdout: "capability-workbench pong\n".to_owned(),
        stderr: String::new(),
        exit_code: 0,
        accounts: Vec::new(),
    }))
}
