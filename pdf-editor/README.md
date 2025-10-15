# PDF Editor

This repository contains an end-to-end prototype for a PDF editor that keeps
PDF content in its native vector representation. A Fabric.js overlay acts as the
interactive controller for content extracted from the PDF, while a Rust backend
tracks incremental edits and rewrites the underlying content streams.

## Project layout

```
pdf-editor/
  LICENSE               # Apache-2.0 license
  README.md             # Project overview & instructions
  frontend/             # Vite + TypeScript application
  backend/              # Rust Axum service
  shared/schema/        # JSON schema describing API contracts
  e2e/                  # End-to-end samples & REST collection
```

## Getting started

### Backend

```
cd backend
cargo run
```

This launches the Axum server on <http://localhost:8787>. The backend exposes
`/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId`.

### Frontend

```
cd frontend
npm install
npm run dev
```

By default Vite serves the app on <http://localhost:5173>. Set the
`FRONTEND_API_BASE` environment variable if the backend runs on another host.

## Development notes

* All dependencies use permissive licenses (MIT or Apache-2.0).
* The backend performs incremental PDF updates rather than rewriting whole
  documents.
* JSON schema in `shared/schema` describes the intermediate representation and
  patch protocol shared between the frontend and backend.
