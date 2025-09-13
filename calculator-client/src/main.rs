use rmcp::{
    RmcpError,
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), RmcpError> {
    // Assume the server is sibling project named "calculator-server" in the same directory
    let server_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Failed to locate worksapce root")
        .join("calculator-server");

    let client = ()
        .serve(
            TokioChildProcess::new(Command::new("cargo").configure(|cmd| {
                cmd.arg("run").current_dir(server_dir);
            }))
            .map_err(RmcpError::transport_creation::<TokioChildProcess>)?,
        )
        .await?;
    // Initialize
    let server_info = client.peer_info();
    println!("Server info: {:?}", server_info);

    // List Tools
    let tools = client.list_tools(Default::default()).await?;
    println!("Available tools: {:?}", tools);

    // Call add tool with arguments = {"a": 3, "b": 2}
    let a = 3;
    let b = 2;
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: "add".into(),
            arguments: serde_json::json!({
                "a": a, "b": b
            })
            .as_object()
            .cloned(),
        })
        .await?;
    println!("Result of {:?} + {:?} = {:?}", a, b, tool_result);

    client.cancel().await?;

    Ok(())
}
