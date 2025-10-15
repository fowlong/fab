# Backend

The backend is a Rust server built with `axum`. It exposes REST endpoints for opening PDFs, retrieving an intermediate representation (IR), applying patches, and downloading the incrementally updated document.

## Commands

```bash
cargo run         # start the development server on http://localhost:8787
cargo test        # run backend unit tests
```

Configuration is handled with environment variables:

- `DATA_DIR` – where uploaded PDFs and incremental revisions are stored (defaults to an in-memory store if unset).
- `BIND_ADDR` – server bind address (defaults to `127.0.0.1:8787`).
