# PDF Editor MVP

This directory provides the groundwork for a true, vector-aware PDF editor. The
frontend renders PDF pages with `pdf.js` and exposes interactive controllers
with `fabric.js`. The backend parses, patches, and serialises PDFs using Rust
and permissively licensed libraries such as `lopdf` and `harfbuzz-rs`.

The initial implementation focuses on establishing the application structure,
API contracts, and developer tooling so the incremental PDF editing features can
be built iteratively.

## Project layout

```
pdf-editor/
  LICENSE                 # Apache-2.0
  README.md               # This file
  frontend/               # Vite + TypeScript app
  backend/                # axum-based Rust service
  shared/schema/          # JSON schemas for IR/Patch contracts
  e2e/                    # Sample assets and HTTP request fixtures
```

## Getting started

### Frontend

```
cd frontend
npm install
npm run dev
```

The development server listens on <http://localhost:5173>. Set the `VITE_API_BASE`
environment variable to override the backend URL if the server is not running on
`http://localhost:8787`.

### Backend

```
cd backend
cargo run
```

The backend listens on <http://localhost:8787> and exposes the following routes:

- `POST /api/open` – upload a PDF and receive a document identifier.
- `GET /api/ir/:docId` – fetch a simplified intermediate representation.
- `POST /api/patch/:docId` – submit patch operations.
- `GET /api/pdf/:docId` – download the latest PDF bytes.

## Current status

- The frontend provides a skeleton UI with upload, page preview, and overlay
  scaffolding.
- The backend stores uploaded PDFs in-memory, returns a placeholder IR, and
  echoes incremental updates so that the full editing pipeline can be developed
  incrementally.
- Shared JSON schemas document the IR and patch formats used by the API.
- An example PDF and HTTP request collection (for VS Code REST Client or curl)
  live under `e2e/`.

## Contributing

All code is released under the Apache-2.0 licence. External dependencies are
restricted to permissively licensed libraries in line with the project
requirements.
