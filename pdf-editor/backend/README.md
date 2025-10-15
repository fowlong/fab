# Backend

The backend is an axum-based web server that exposes a JSON API for parsing and
mutating PDF documents. The current implementation focuses on scaffolding the
major components described in the project plan and leaves clear extension points
for the future incremental writer.

## Running the server

```bash
cargo run
```

The server listens on `http://localhost:8787` and serves the following routes:

- `POST /api/open` — upload a PDF file and receive a `docId`.
- `GET /api/ir/:docId` — retrieve the cached intermediate representation.
- `POST /api/patch/:docId` — apply an array of patch operations.
- `GET /api/pdf/:docId` — download the latest PDF bytes.

> **Note:** The current MVP returns placeholder IR/patch results while the PDF
> parsing and writing pipeline is still under construction. The public API is in
> place so the frontend can be developed in parallel.
