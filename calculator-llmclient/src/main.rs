use std::error::Error;

use async_openai::{Client, config::OpenAIConfig};
use rmcp::{
    RmcpError, ServiceExt,
    model::ListToolsResult,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::{Value, json};
use tokio::process::Command;

async fn format_tools(tools: &ListToolsResult) -> Result<Vec<Value>, Box<dyn Error>> {
    let tools_json = serde_json::to_value(tools)?;
    let Some(tools_array) = tools_json.get("tools").and_then(|t| t.as_array()) else {
        return Ok(vec![]);
    };

    let formatted_tools = tools_array
        .iter()
        .filter_map(|tool| {
            let name = tool.get("name")?.as_str()?;
            let description = tool.get("description")?.as_str()?;
            let schema = tool.get("inputSchema")?;

            Some(json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": {
                        "type": "object",
                        "properties": schema.get("properties").unwrap_or(&json!({})),
                        "required": schema.get("required").unwrap_or(&json!([]))
                    }
                }
            }))
        })
        .collect();

    Ok(formatted_tools)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initial message
    let mut message = vec![json!({
        "role": "user",
        "content": "What is the sum of 3 and 2?"
    })];

    // Setup OpenAI client
    let openai_api_key = std::env::var("OPENAI_API_KEY")?;
    let openai_api_base = std::env::var("OPENAI_BASE_URL")?;
    let openai_client = Client::with_config(
        OpenAIConfig::new()
            .with_api_base(openai_api_base)
            .with_api_key(openai_api_key),
    );

    // Setup MCP client
    let server_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("calculator-server");

    let mcp_client = ()
        .serve(
            TokioChildProcess::new(Command::new("cargo").configure(|cmd| {
                cmd.arg("run").current_dir(server_dir);
            }))
            .map_err(RmcpError::transport_creation::<TokioChildProcess>)?,
        )
        .await?;

    // ToDo: Get MCP tool listing
    let tools = mcp_client.list_tools(Default::default()).await?;

    // ToDo: LLM conversation with tool calls

    Ok(())
}
