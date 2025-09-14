use std::error::Error;

use async_openai::{Client, config::OpenAIConfig};
use rmcp::{
    RmcpError, ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::json;
use tokio::process::Command;

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
    // ToDo: LLM conversation with tool calls

    Ok(())
}
