# Backend

The backend is a Rust/axum service that receives PDFs, exposes an
intermediate representation (IR), and accepts patch operations that update the
underlying PDF content streams. The current codebase focuses on scaffolding:
state management, REST routes, and module layout for future PDF editing
features.

## Development

```bash
cargo run
```

The server listens on <http://localhost:8787>. Endpoints:

- `POST /api/open` — upload a PDF file (binary body) and receive a `docId`.
- `GET /api/ir/:docId` — fetch the parsed IR (currently placeholder data).
- `POST /api/patch/:docId` — apply patch operations (returns 501 until
  implemented).
- `GET /api/pdf/:docId` — download the latest PDF bytes.

## Next steps

- Implement true PDF parsing via `lopdf` in `pdf::extract`.
- Flesh out `pdf::content` for tokenisation and rewriting of content streams.
- Add font shaping/subsetting logic under `pdf::fonts`.
