use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotionMcpError {
    #[error("Error en la API de Notion: {0}")]
    NotionApi(String),
    
    #[error("Error de transporte: {0}")]
    Transport(String),
    
    #[error("Error interno del servidor: {0}")]
    Server(String),
    
    #[error("Parámetros inválidos: {0}")]
    InvalidParams(String),
    
    #[error("Método no encontrado: {0}")]
    MethodNotFound(String),
    
    #[error("Recurso no encontrado: {0}")]
    ResourceNotFound(String),
    
    #[error("Error de autenticación: {0}")]
    Authentication(String),
    
    #[error("Error al analizar JSON: {0}")]
    JsonParse(String),
    
    #[error("Error desconocido: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for NotionMcpError {
    fn from(error: reqwest::Error) -> Self {
        NotionMcpError::NotionApi(error.to_string())
    }
}

impl From<serde_json::Error> for NotionMcpError {
    fn from(error: serde_json::Error) -> Self {
        NotionMcpError::JsonParse(error.to_string())
    }
}

pub type NotionResult<T> = std::result::Result<T, NotionMcpError>;
