# PDF Editor (MVP scaffold)

This repository contains a work-in-progress implementation of a true PDF editing experience built with permissively licensed tooling end-to-end.

- **Frontend**: Vite + TypeScript, pdf.js for bitmap preview, Fabric.js for the interactive overlay.
- **Backend**: Rust with axum, lopdf for PDF parsing, harfbuzz-rs and ttf-parser for text shaping and font embedding.
- **Shared schemas**: JSON Schema definitions for the Intermediate Representation (IR) and patch protocol exchanged between the frontend and backend.

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

Each subfolder contains its own README snippets explaining purpose where relevant. The backend and frontend folders can be developed and deployed independently while sharing the same API contracts.

## Getting started

1. Install the toolchains:
   - Node.js 18+ (for Vite dev server)
   - Rust stable (for the axum backend)
2. Install frontend dependencies and start the dev server:

   ```bash
   cd frontend
   npm install
   npm run dev
   ```

3. In another terminal, start the backend:

   ```bash
   cd backend
   cargo run
   ```

4. Visit the frontend (default `http://localhost:5173`) and upload `e2e/sample.pdf` to explore the placeholder experience.

> **Note**: The current code base focuses on providing a well-structured scaffold. It renders sample IR data and exposes stub endpoints that will later be extended to perform full PDF rewriting. The architecture and type definitions closely follow the design brief so future incremental work can focus on the core algorithms without reworking the project layout.

## Licensing

The entire repository is licensed under the Apache License 2.0. Individual third-party dependencies are restricted to permissive licenses only (MIT / Apache 2.0 compatible) as outlined in the design goals.
