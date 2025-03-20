# MCP Notion

Servidor MCP (Model Context Protocol) para integración con Notion API. Proporciona endpoints HTTP para interactuar con bases de datos y páginas de Notion.

## 🚀 Características

- Búsqueda en Notion
- Obtención de páginas y contenido
- Consulta de bases de datos
- Soporte para CORS
- Manejo de errores robusto
- Logging integrado

## 📋 Requisitos Previos

- Rust (última versión estable)
- Token de API de Notion
- Base de datos o páginas en Notion para interactuar

## 🔧 Instalación

1. Clonar el repositorio:
```bash
git clone https://github.com/GsusFC/MCP-Notion.git
cd MCP-Notion
```

2. Configurar variables de entorno:
```bash
cp .env.example .env
# Editar .env y añadir tu NOTION_API_KEY
```

3. Compilar y ejecutar:
```bash
cargo build
cargo run
```

El servidor se iniciará en `http://localhost:3004` por defecto.

## 🔌 API Endpoints

### Búsqueda
```http
POST /api/search
Content-Type: application/json

{
    "query": "término de búsqueda",
    "limit": 10
}
```

### Obtener Página
```http
GET /api/get_page/{page_id}
```

### Obtener Contenido de Página
```http
GET /api/get_page_content/{page_id}
```

### Consultar Base de Datos
```http
POST /api/query_database
Content-Type: application/json

{
    "database_id": "tu-database-id",
    "page_size": 100
}
```

## ⚙️ Configuración

Variables de entorno disponibles:

- `NOTION_API_KEY`: Token de API de Notion (requerido)
- `MCP_PORT`: Puerto del servidor (default: 3004)
- `RUST_LOG`: Nivel de logging (default: info)

## 🔍 Ejemplos de Uso

### Búsqueda Simple
```bash
curl -X POST http://localhost:3004/api/search \
  -H "Content-Type: application/json" \
  -d '{"query": "", "limit": 5}'
```

### Consultar Base de Datos
```bash
curl -X POST http://localhost:3004/api/query_database \
  -H "Content-Type: application/json" \
  -d '{"database_id": "tu-database-id", "page_size": 10}'
```

## 🤝 Contribuir

1. Fork el proyecto
2. Crear una rama para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abrir un Pull Request

## 📝 Licencia

Este proyecto está bajo la Licencia MIT - ver el archivo [LICENSE](LICENSE) para más detalles.

## ✨ Agradecimientos

- Equipo de Notion por su excelente API
- Comunidad de Rust por las herramientas y librerías
