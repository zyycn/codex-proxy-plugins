use gateway_plugin_sdk::{ErrorCode, PluginFault, call::provider::Usage};
use serde_json::Value;

const MAXIMUM_TEXT_CHUNKS: usize = 48;

pub(super) fn extract_input_text(body: &[u8]) -> Result<String, PluginFault> {
    let document: Value = serde_json::from_slice(body)
        .map_err(|_| PluginFault::new(ErrorCode::InvalidInput, "请求正文必须是 JSON"))?;
    let input = document
        .get("input")
        .ok_or_else(|| PluginFault::new(ErrorCode::InvalidInput, "请求缺少 input"))?;
    let mut parts = Vec::new();
    collect_text(input, &mut parts);
    let text = parts.join("\n");
    if text.is_empty() || text.len() > 256 * 1024 {
        return Err(PluginFault::new(
            ErrorCode::InvalidInput,
            "请求文本须为 1–262144 个 UTF-8 字节",
        ));
    }
    Ok(text)
}

fn collect_text(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(text) => output.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                collect_text(item, output);
            }
        }
        Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(Value::as_str) {
                output.push(text.to_owned());
            } else if let Some(content) = object.get("content") {
                collect_text(content, output);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

pub(super) fn split_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let characters_per_chunk = text.chars().count().div_ceil(MAXIMUM_TEXT_CHUNKS).max(1);
    let mut current_characters = 0;
    for character in text.chars() {
        current.push(character);
        current_characters += 1;
        if current_characters >= characters_per_chunk {
            chunks.push(std::mem::take(&mut current));
            current_characters = 0;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

pub(super) fn usage(text: &str) -> Usage {
    let tokens = u64::try_from(text.chars().count().div_ceil(4)).unwrap_or(u64::MAX);
    Usage {
        input_tokens: Some(tokens),
        output_tokens: Some(tokens),
        cached_tokens: Some(0),
        cache_write_tokens: Some(0),
        reasoning_tokens: Some(0),
        image_input_tokens: Some(0),
        image_output_tokens: Some(0),
        total_tokens: tokens.checked_mul(2),
    }
}
