use anyhow::Result;
use dotenv::dotenv;
use log::{info, error};
use std::env;
use std::sync::Arc;

mod notion;
mod server;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment variables from .env
    dotenv().ok();
    
    // Configure logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Get Notion API key
    let notion_api_key = env::var("NOTION_API_KEY")
        .expect("NOTION_API_KEY not found in environment variables");
    
    info!("Starting Notion MCP...");
    
    // Create Notion client
    let notion_client = notion::NotionClient::new(notion_api_key);
    
    // Validate Notion connection
    match notion_client.validate_connection().await {
        Ok(_) => info!("Notion connection validated successfully"),
        Err(e) => {
            error!("Error validating Notion connection: {}", e);
            eprintln!("\nError: Could not connect to Notion. Please verify your API key and Internet connection.");
            eprintln!("Remember that the current API key format is: ntn_xxxxxxxxxx\n");
            std::process::exit(1);
        }
    }
    
    let notion_client = Arc::new(notion_client);
    
    // Start MCP server
    let port = env::var("MCP_PORT")
        .unwrap_or_else(|_| "3004".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    match server::run_notion_mcp_server(notion_client, port).await {
        Ok(_) => info!("MCP server finished successfully"),
        Err(e) => error!("Error in MCP server: {}", e),
    }
    
    Ok(())
}
