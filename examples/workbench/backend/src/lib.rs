//! 工作台插件的公开入口；业务处理器只通过公开 SDK 与宿主通信。

mod app;
mod authentication;
mod command;
mod evidence;
mod host_calls;
mod management;
mod manifest;
mod provider;
mod request;

pub use app::plugin;
pub use manifest::{PLUGIN_ID, manifest};
