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
    // Inicializar variables de entorno desde .env
    dotenv().ok();
    
    // Configurar logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Obtener API key de Notion
    let notion_api_key = env::var("NOTION_API_KEY")
        .expect("No se encontró NOTION_API_KEY en las variables de entorno");
    
    info!("Iniciando MCP para Notion...");
    
    // Crear cliente de Notion
    let notion_client = notion::NotionClient::new(notion_api_key);
    
    // Validar la conexión a Notion
    match notion_client.validate_connection().await {
        Ok(_) => info!("Conexión a Notion validada correctamente"),
        Err(e) => {
            error!("Error al validar la conexión con Notion: {}", e);
            eprintln!("\nError: No se pudo conectar a Notion. Verifica tu API key y la conexión a Internet.");
            eprintln!("Recuerda que el formato actual de las API keys es: ntn_xxxxxxxxxx\n");
            std::process::exit(1);
        }
    }
    
    let notion_client = Arc::new(notion_client);
    
    // Iniciar servidor MCP
    let port = env::var("MCP_PORT")
        .unwrap_or_else(|_| "3004".to_string())
        .parse::<u16>()
        .expect("PORT debe ser un número válido");
    
    match server::run_notion_mcp_server(notion_client, port).await {
        Ok(_) => info!("Servidor MCP finalizado correctamente"),
        Err(e) => error!("Error en el servidor MCP: {}", e),
    }
    
    Ok(())
}
