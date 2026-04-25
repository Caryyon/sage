# SAGE REST API

## Base URL
```
http://localhost:19175/api/v1
```

## Endpoints

### Chat
```http
POST /chat
Content-Type: application/json

{
  "message": "What is SAGE?",
  "history": [],
  "max_tokens": 1000
}
```

### Node Status
```http
GET /node/status
```

### Brain Stats
```http
GET /brain/stats
```

### Knowledge Search
```http
POST /knowledge/search
Content-Type: application/json

{
  "query": "neural networks",
  "max_results": 5
}
```
