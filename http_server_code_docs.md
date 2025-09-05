# HTTP Server with Router - Code Documentation

## Module Structure

```
src/
├── main.rs              // Server initialization and startup
├── routes/
│   └── mod.rs          // Route definitions and handlers
└── server/
    ├── mod.rs          // Module re-exports
    ├── http.rs         // HTTP request/response structures
    ├── router.rs       // Routing logic and handler dispatch
    └── handler.rs      // Connection handling and request processing
```

## HTTP Module (`src/server/http.rs