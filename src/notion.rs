use crate::error::{NotionMcpError, NotionResult};
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const NOTION_API_VERSION: &str = "2022-06-28";
const NOTION_BASE_URL: &str = "https://api.notion.com/v1";

#[derive(Debug, Clone)]
pub struct NotionClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotionSearchResponse {
    pub results: Vec<Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotionPageResponse {
    pub id: String,
    pub url: String,
    pub properties: Value,
    pub content: Option<Vec<Value>>,
}

impl NotionClient {
    pub fn new(api_key: String) -> Self {
        // Validate API key format
        if !api_key.starts_with("ntn_") && !api_key.starts_with("secret_") {
            log::warn!("The Notion API key format doesn't seem valid. Current keys start with 'ntn_'");
        }
        
        Self {
            client: Client::new(),
            api_key,
        }
    }
    
    // Validate Notion connection
    pub async fn validate_connection(&self) -> NotionResult<bool> {
        debug!("Validating Notion API connection...");
        
        let response = self.client
            .post(&format!("{}/search", NOTION_BASE_URL))
            .headers(self.headers())
            .json(&json!({
                "query": "",
                "page_size": 1
            }))
            .send()
            .await
            .map_err(|e| {
                error!("Error validating Notion connection: {}", e);
                NotionMcpError::Authentication(format!("Connection error: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            if status.as_u16() == 401 {
                error!("Authentication error: Invalid Notion API token");
                return Err(NotionMcpError::Authentication("Invalid or expired Notion API token".to_string()));
            }
            
            error!("Error in Notion response ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("HTTP Error {}: {}", status, error_text)));
        }
        
        debug!("Notion connection validated successfully");
        Ok(true)
    }

    // Authentication headers
    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.api_key).parse().unwrap(),
        );
        headers.insert(
            "Notion-Version",
            NOTION_API_VERSION.parse().unwrap(),
        );
        headers.insert(
            "Content-Type",
            "application/json".parse().unwrap(),
        );
        headers
    }

    // Search in Notion
    pub async fn search(&self, query: &str, limit: Option<u32>) -> NotionResult<NotionSearchResponse> {
        let limit = limit.unwrap_or(10);
        debug!("Searching in Notion: '{}' (limit: {})", query, limit);
        
        let payload = json!({
            "query": query,
            "page_size": limit,
            "sort": {
                "direction": "descending",
                "timestamp": "last_edited_time"
            }
        });
        
        let response = self.client
            .post(&format!("{}/search", NOTION_BASE_URL))
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Error searching in Notion: {}", e);
                NotionMcpError::NotionApi(format!("Search error: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        let search_response = response.json::<NotionSearchResponse>().await
            .map_err(|e| {
                error!("Error al parsear respuesta JSON: {}", e);
                NotionMcpError::JsonParse(e.to_string())
            })?;
        
        debug!("Search completed, {} results found", search_response.results.len());
        Ok(search_response)
    }

    // Get a page by ID
    pub async fn get_page(&self, page_id: &str) -> NotionResult<NotionPageResponse> {
        debug!("Getting page with ID: {}", page_id);
        
        let response = self.client
            .get(&format!("{}/pages/{}", NOTION_BASE_URL, page_id))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| {
                error!("Error getting page: {}", e);
                NotionMcpError::NotionApi(format!("Error getting page: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        let page = response.json::<NotionPageResponse>().await
            .map_err(|e| {
                error!("Error al parsear respuesta JSON: {}", e);
                NotionMcpError::JsonParse(e.to_string())
            })?;
        
        debug!("Page retrieved successfully: {}", page.id);
        Ok(page)
    }

    // Get page content
    pub async fn get_page_content(&self, page_id: &str) -> NotionResult<Vec<Value>> {
        debug!("Getting page content with ID: {}", page_id);
        
        let response = self.client
            .get(&format!("{}/blocks/{}/children", NOTION_BASE_URL, page_id))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| {
                error!("Error getting page content: {}", e);
                NotionMcpError::NotionApi(format!("Error getting content: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        let content_response: Value = response.json().await
            .map_err(|e| {
                error!("Error al parsear respuesta JSON: {}", e);
                NotionMcpError::JsonParse(e.to_string())
            })?;
        
        let results = content_response["results"].as_array()
            .ok_or_else(|| NotionMcpError::JsonParse("No se encontró campo 'results'".to_string()))?
            .clone();
        
        debug!("Content retrieved, {} blocks found", results.len());
        Ok(results)
    }

    // Extraer información relevante de una página
    fn extract_page_info(&self, page: &Value) -> Option<Value> {
        let properties = page.get("properties")?;
        
        let brand_name = properties.get("Brand Name")?
            .get("title")?
            .as_array()?
            .first()?
            .get("plain_text")?
            .as_str()?;
        
        let services = properties.get("Services")?
            .get("multi_select")?
            .as_array()?
            .iter()
            .filter_map(|opt| opt.get("name").and_then(|n| n.as_str()))
            .collect::<Vec<_>>();
        
        let description = properties.get("Description")
            .and_then(|d| d.get("rich_text"))
            .and_then(|rt| rt.as_array())
            .and_then(|arr| arr.first())
            .and_then(|t| t.get("plain_text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        
        let website = properties.get("Website")
            .and_then(|w| w.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("");

        let tagline = properties.get("Tagline")
            .and_then(|t| t.get("rich_text"))
            .and_then(|rt| rt.as_array())
            .and_then(|arr| arr.first())
            .and_then(|t| t.get("plain_text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let slug = properties.get("Slug")
            .and_then(|s| s.get("rich_text"))
            .and_then(|rt| rt.as_array())
            .and_then(|arr| arr.first())
            .and_then(|t| t.get("plain_text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        // Extraer URLs de imágenes
        let mut images = Vec::new();
        for i in 1..=10 {
            let key = format!("Image [{}]", i);
            if let Some(files) = properties.get(&key)
                .and_then(|f| f.get("files"))
                .and_then(|f| f.as_array()) {
                for file in files {
                    if let Some(url) = file.get("url").and_then(|u| u.as_str()) {
                        images.push(json!({
                            "id": i,
                            "url": url
                        }));
                    } else if let Some(url) = file.get("file").and_then(|f| f.get("url")).and_then(|u| u.as_str()) {
                        images.push(json!({
                            "id": i,
                            "url": url
                        }));
                    }
                }
            }
        }

        // Extraer imágenes especiales
        let special_images = [
            ("hero_image", "Hero Image"),
            ("cover", "Cover"),
            ("avatar", "Avatar"),
            ("square_image_1", "Image [7.1] square image"),
            ("square_image_2", "Image [7.2] square image")
        ];

        let mut media = json!({"images": images});
        if let Some(obj) = media.as_object_mut() {
            for (key, prop_name) in special_images.iter() {
                if let Some(files) = properties.get(prop_name)
                    .and_then(|f| f.get("files"))
                    .and_then(|f| f.as_array()) {
                    if let Some(file) = files.first() {
                        if let Some(url) = file.get("url").and_then(|u| u.as_str()) {
                            obj.insert(key.to_string(), json!({ "url": url }));
                        } else if let Some(url) = file.get("file").and_then(|f| f.get("url")).and_then(|u| u.as_str()) {
                            obj.insert(key.to_string(), json!({ "url": url }));
                        }
                    }
                }
            }
        }

        // Extraer URLs de videos
        let videos = json!({
            "video_1": properties.get("Video 1").and_then(|v| v.get("url")).and_then(|u| u.as_str()),
            "video_2": properties.get("Video 2").and_then(|v| v.get("url")).and_then(|u| u.as_str())
        });
        
        Some(json!({
            "id": page["id"].as_str()?,
            "name": brand_name,
            "services": services,
            "description": description,
            "website": website,
            "tagline": tagline,
            "slug": slug,
            "media": media,
            "videos": videos
        }))
    }
    
    // Consultar una base de datos
    pub async fn query_database(&self, database_id: &str, filter: Option<Value>, limit: Option<u32>) -> NotionResult<Vec<Value>> {
        let limit = limit.unwrap_or(100);
        debug!("Consultando base de datos: {} (límite: {})", database_id, limit);
        
        let mut payload = json!({
            "page_size": limit
        });
        
        if let Some(f) = filter {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("filter".to_string(), f);
            }
        }
        
        let response = self.client
            .post(&format!("{}/databases/{}/query", NOTION_BASE_URL, database_id))
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Error al consultar base de datos: {}", e);
                NotionMcpError::NotionApi(format!("Error en consulta de base de datos: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        let db_response: Value = response.json().await
            .map_err(|e| {
                error!("Error al parsear respuesta JSON: {}", e);
                NotionMcpError::JsonParse(e.to_string())
            })?;
        
        let results = db_response["results"].as_array()
            .ok_or_else(|| NotionMcpError::JsonParse("No se encontró campo 'results'".to_string()))?
            .iter()
            .filter_map(|page| self.extract_page_info(page))
            .collect::<Vec<_>>();
        
        debug!("Consulta completada, {} resultados encontrados", results.len());
        Ok(results)
    }

    // Crear una página
    pub async fn create_page(&self, parent_id: &str, properties: Value, content: Option<Vec<Value>>) -> NotionResult<Value> {
        debug!("Creando nueva página en parent_id: {}", parent_id);
        
        let is_database = parent_id.contains("-");
        
        let mut payload = json!({
            "parent": {
                if is_database { "database_id" } else { "page_id" }: parent_id
            },
            "properties": properties
        });
        
        if let Some(children) = content {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("children".to_string(), json!(children));
            }
        }
        
        let response = self.client
            .post(&format!("{}/pages", NOTION_BASE_URL))
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Error al crear página: {}", e);
                NotionMcpError::NotionApi(format!("Error al crear página: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        let page_response: Value = response.json().await
            .map_err(|e| {
                error!("Error al parsear respuesta JSON: {}", e);
                NotionMcpError::JsonParse(e.to_string())
            })?;
        
        debug!("Página creada correctamente: {}", page_response["id"].as_str().unwrap_or("unknown"));
        Ok(page_response)
    }

    // Actualizar una página
    pub async fn update_page(&self, page_id: &str, properties: Value) -> NotionResult<Value> {
        debug!("Actualizando página con ID: {}", page_id);
        
        let payload = json!({
            "properties": properties
        });
        
        let response = self.client
            .patch(&format!("{}/pages/{}", NOTION_BASE_URL, page_id))
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Error al actualizar página: {}", e);
                NotionMcpError::NotionApi(format!("Error al actualizar página: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        let page_response: Value = response.json().await
            .map_err(|e| {
                error!("Error al parsear respuesta JSON: {}", e);
                NotionMcpError::JsonParse(e.to_string())
            })?;
        
        debug!("Página actualizada correctamente");
        Ok(page_response)
    }

    // Convertir texto plano a bloques de Notion
    pub fn text_to_blocks(text: &str) -> Vec<Value> {
        text.split("\n\n")
            .map(|paragraph| {
                json!({
                    "object": "block",
                    "type": "paragraph",
                    "paragraph": {
                        "rich_text": [{
                            "type": "text",
                            "text": {
                                "content": paragraph
                            }
                        }]
                    }
                })
            })
            .collect()
    }

    // Extraer texto plano de bloques de Notion
    pub fn extract_text_from_blocks(blocks: &[Value]) -> String {
        blocks.iter()
            .filter_map(|block| {
                let block_type = block["type"].as_str()?;
                match block_type {
                    "paragraph" => {
                        let rich_text = block["paragraph"]["rich_text"].as_array()?;
                        let texts: Vec<String> = rich_text.iter()
                            .filter_map(|rt| rt["text"]["content"].as_str().map(|s| s.to_string()))
                            .collect();
                        Some(texts.join(""))
                    },
                    "heading_1" | "heading_2" | "heading_3" => {
                        let rich_text = block[block_type]["rich_text"].as_array()?;
                        let texts: Vec<String> = rich_text.iter()
                            .filter_map(|rt| rt["text"]["content"].as_str().map(|s| s.to_string()))
                            .collect();
                        Some(texts.join(""))
                    },
                    "bulleted_list_item" | "numbered_list_item" => {
                        let rich_text = block[block_type]["rich_text"].as_array()?;
                        let texts: Vec<String> = rich_text.iter()
                            .filter_map(|rt| rt["text"]["content"].as_str().map(|s| s.to_string()))
                            .collect();
                        Some(texts.join(""))
                    },
                    _ => None
                }
            })
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
