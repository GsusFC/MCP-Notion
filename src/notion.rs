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
        // Validar el formato de la API key
        if !api_key.starts_with("ntn_") && !api_key.starts_with("secret_") {
            log::warn!("El formato de la API key de Notion no parece válido. Las claves actuales comienzan con 'ntn_'");
        }
        
        Self {
            client: Client::new(),
            api_key,
        }
    }
    
    // Validar la conexión a Notion
    pub async fn validate_connection(&self) -> NotionResult<bool> {
        debug!("Validando conexión a la API de Notion...");
        
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
                error!("Error al validar conexión con Notion: {}", e);
                NotionMcpError::Authentication(format!("Error de conexión: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            if status.as_u16() == 401 {
                error!("Error de autenticación: Token de API de Notion inválido");
                return Err(NotionMcpError::Authentication("Token de API de Notion inválido o expirado".to_string()));
            }
            
            error!("Error en respuesta de Notion ({}): {}", status, error_text);
            return Err(NotionMcpError::NotionApi(format!("Error HTTP {}: {}", status, error_text)));
        }
        
        debug!("Conexión a Notion validada correctamente");
        Ok(true)
    }

    // Headers para autenticación
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

    // Búsqueda en Notion
    pub async fn search(&self, query: &str, limit: Option<u32>) -> NotionResult<NotionSearchResponse> {
        let limit = limit.unwrap_or(10);
        debug!("Buscando en Notion: '{}' (límite: {})", query, limit);
        
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
                error!("Error al hacer búsqueda en Notion: {}", e);
                NotionMcpError::NotionApi(format!("Error en búsqueda: {}", e))
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
        
        debug!("Búsqueda completada, {} resultados encontrados", search_response.results.len());
        Ok(search_response)
    }

    // Obtener una página por ID
    pub async fn get_page(&self, page_id: &str) -> NotionResult<NotionPageResponse> {
        debug!("Obteniendo página con ID: {}", page_id);
        
        let response = self.client
            .get(&format!("{}/pages/{}", NOTION_BASE_URL, page_id))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| {
                error!("Error al obtener página: {}", e);
                NotionMcpError::NotionApi(format!("Error al obtener página: {}", e))
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
        
        debug!("Página obtenida correctamente: {}", page.id);
        Ok(page)
    }

    // Obtener contenido de una página
    pub async fn get_page_content(&self, page_id: &str) -> NotionResult<Vec<Value>> {
        debug!("Obteniendo contenido de página con ID: {}", page_id);
        
        let response = self.client
            .get(&format!("{}/blocks/{}/children", NOTION_BASE_URL, page_id))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| {
                error!("Error al obtener contenido de página: {}", e);
                NotionMcpError::NotionApi(format!("Error al obtener contenido: {}", e))
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
        
        debug!("Contenido obtenido, {} bloques encontrados", results.len());
        Ok(results)
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
            .clone();
        
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
