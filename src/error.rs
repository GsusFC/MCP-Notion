use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotionMcpError {
    #[error("Notion API error: {0}")]
    NotionApi(String),
    
    #[error("Transport error: {0}")]
    Transport(String),
    
    #[error("Internal server error: {0}")]
    Server(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("JSON parsing error: {0}")]
    JsonParse(String),
    
    #[error("Unknown error: {0}")]
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
