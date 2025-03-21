use crate::notion::NotionClient;
use crate::error::NotionMcpError;
use actix_web::{web, App, HttpServer, Responder};
use actix_cors::Cors;
use log::{debug, error, info};
use serde_json::{json, Value};
use std::sync::Arc;

async fn handle_search(
    notion_client: web::Data<Arc<NotionClient>>,
    params: web::Json<Value>,
) -> impl Responder {
    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return web::Json(json!({
            "error": "Missing 'query' parameter"
        }))
    };
    
    let limit = params.get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    
    match notion_client.search(query, limit).await {
        Ok(results) => web::Json(json!(results)),
        Err(e) => web::Json(json!({
            "error": e.to_string()
        }))
    }
}

async fn handle_get_page(
    notion_client: web::Data<Arc<NotionClient>>,
    params: web::Json<Value>,
) -> impl Responder {
    let page_id = match params.get("page_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return web::Json(json!({
            "error": "Falta parámetro 'page_id'"
        }))
    };
    
    match notion_client.get_page(page_id).await {
        Ok(page) => web::Json(json!(page)),
        Err(e) => web::Json(json!({
            "error": e.to_string()
        }))
    }
}

async fn handle_get_page_content(
    notion_client: web::Data<Arc<NotionClient>>,
    params: web::Json<Value>,
) -> impl Responder {
    let page_id = match params.get("page_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return web::Json(json!({
            "error": "Falta parámetro 'page_id'"
        }))
    };
    
    match notion_client.get_page_content(page_id).await {
        Ok(content) => web::Json(json!({
            "content": content,
            "text": NotionClient::extract_text_from_blocks(&content)
        })),
        Err(e) => web::Json(json!({
            "error": e.to_string()
        }))
    }
}

async fn handle_query_database(
    notion_client: web::Data<Arc<NotionClient>>,
    params: web::Json<Value>,
) -> impl Responder {
    let database_id = match params.get("database_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return web::Json(json!({
            "error": "Missing 'database_id' parameter"
        }))
    };
    
    let mut filter_obj = json!({});
    
    // Manejar filtro de highlighted
    if let Some(highlighted) = params.get("highlighted").and_then(|v| v.as_bool()) {
        filter_obj = json!({
            "property": "00. Highlighted",
            "checkbox": {
                "equals": highlighted
            }
        });
    }
    
    // Manejar filtro de servicios
    if let Some(services) = params.get("services").and_then(|v| v.as_array()) {
        if !services.is_empty() {
            filter_obj = json!({
                "property": "Services",
                "multi_select": {
                    "contains": services[0]
                }
            });
        }
    }
    
    let filter = if filter_obj != json!({}) { Some(filter_obj) } else { None };
    let limit = params.get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    
    debug!("Query params - database_id: {}, filter: {:?}, limit: {:?}", database_id, filter, limit);
    
    match notion_client.query_database(database_id, filter, limit).await {
        Ok(results) => {
            debug!("Query successful, {} results", results.len());
            web::Json(json!({
                "results": results
            }))
        },
        Err(e) => {
            error!("Query error: {}", e);
            web::Json(json!({
                "error": e.to_string()
            }))
        }
    }
}

async fn handle_create_page(
    notion_client: web::Data<Arc<NotionClient>>,
    params: web::Json<Value>,
) -> impl Responder {
    let parent_id = match params.get("parent_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return web::Json(json!({
            "error": "Missing 'parent_id' parameter"
        }))
    };
    
    let properties = match params.get("properties") {
        Some(props) => props.clone(),
        None => return web::Json(json!({
            "error": "Falta parámetro 'properties'"
        }))
    };
    
    let content = params.get("content")
        .and_then(|v| v.as_array())
        .map(|arr| arr.to_vec());
    
    match notion_client.create_page(parent_id, properties, content).await {
        Ok(page) => web::Json(json!(page)),
        Err(e) => web::Json(json!({
            "error": e.to_string()
        }))
    }
}

async fn handle_update_page(
    notion_client: web::Data<Arc<NotionClient>>,
    params: web::Json<Value>,
) -> impl Responder {
    let page_id = match params.get("page_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return web::Json(json!({
            "error": "Falta parámetro 'page_id'"
        }))
    };
    
    let properties = match params.get("properties") {
        Some(props) => props.clone(),
        None => return web::Json(json!({
            "error": "Falta parámetro 'properties'"
        }))
    };
    
    match notion_client.update_page(page_id, properties).await {
        Ok(page) => web::Json(json!(page)),
        Err(e) => web::Json(json!({
            "error": e.to_string()
        }))
    }
}

pub async fn run_notion_mcp_server(notion_client: Arc<NotionClient>, port: u16) -> std::io::Result<()> {
    info!("Starting HTTP server on port {}", port);
    
    let notion_client_data = web::Data::new(notion_client);
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .wrap(cors)
            .app_data(notion_client_data.clone())
            .route("/api/search", web::post().to(handle_search))
            .route("/api/get_page", web::post().to(handle_get_page))
            .route("/api/get_page_content", web::post().to(handle_get_page_content))
            .route("/api/query_database", web::post().to(handle_query_database))
            .route("/api/create_page", web::post().to(handle_create_page))
            .route("/api/update_page", web::post().to(handle_update_page))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
