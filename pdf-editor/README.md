# PDF Editor

This repository contains the scaffolding for a browser-based PDF editor that performs true content editing by rewriting PDF content streams. The project is structured as a monorepo with separate frontend and backend workspaces plus shared schemas and sample assets.

## Project layout

```
pdf-editor/
  LICENSE
  README.md
  frontend/
  backend/
  shared/
  e2e/
```

The frontend is a Vite + TypeScript application that combines `pdf.js` for rendering pages and `fabric.js` for interactive overlays. The backend is an Axum-based Rust server that parses, patches, and serialises PDF documents using permissively licensed crates.

## Getting started

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The dev server runs on http://localhost:5173 by default. Set `FRONTEND_API_BASE` in `.env` to point to the backend when the origin differs.

### Backend

```bash
cd backend
cargo run
```

The server listens on http://localhost:8787 and exposes REST endpoints for opening PDFs, retrieving the intermediate representation, applying patches, and downloading the latest revision.

## Licence

The entire repository is released under the [Apache License 2.0](LICENSE).
