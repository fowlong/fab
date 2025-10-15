# PDF Editor (MVP scaffold)

This repository contains the scaffolding for a browser-based PDF editor that keeps
vector content editable. The system is split into a TypeScript/Vite frontend that
uses pdf.js and Fabric.js for rendering and interaction, and a Rust backend that
parses, patches, and writes PDFs incrementally using `lopdf`, `harfbuzz-rs`, and
`ttf-parser`.

The code base is organised as follows:

```
pdf-editor/
  frontend/   # Vite + TypeScript UI, pdf.js preview, Fabric overlay
  backend/    # Axum HTTP server + PDF processing pipeline
  shared/     # JSON schema definitions shared between client and server
  e2e/        # Sample assets and HTTP test snippets
```

See the per-directory README sections below for more information on the
responsibilities of each layer. This scaffold focuses on establishing project
structure, shared types, and placeholders for the core flow so the next steps can
concentrate on implementing the PDF editing logic described in the project brief.

## Frontend

The frontend is built with Vite and TypeScript. It renders PDF pages with pdf.js
as a bitmap underlay and uses Fabric.js to provide interactive controllers for
PDF objects. When users transform or edit objects, the frontend sends patch
requests to the backend and renders updated PDFs returned by the server.

Key modules under `frontend/src/`:

- `main.ts` bootstraps the application and coordinates loading the current
  document.
- `pdfPreview.ts` wraps pdf.js rendering into canvases per page.
- `fabricOverlay.ts` configures Fabric.js and translates user edits into patch
  operations.
- `mapping.ts`, `coords.ts`, and `types.ts` provide shared math and type helpers
  that keep the Fabric overlay in sync with the PDF coordinate space.
- `api.ts` centralises calls to the backend.

## Backend

The backend is an Axum application that keeps PDFs in memory and applies patches
incrementally.

The `pdf` module contains the core subsystems:

- `loader.rs` for ingesting PDFs and caching them per session/document ID.
- `extract.rs` for parsing PDF content streams into an intermediate
  representation (IR) that the frontend consumes.
- `content.rs` for tokenising operators and operands inside page content
  streams.
- `patch.rs` for applying high-level patch operations to the IR and PDF
  structure.
- `write.rs` for producing incremental updates with correct cross-reference
  sections.
- `fonts/` for shaping text (`shape.rs`), creating subsets (`subset.rs`), and
  embedding resources (`embed.rs`).

The `types.rs` file mirrors the shared JSON structures so the backend can serialise
and deserialise IR and patch payloads.

Utility helpers under `util/` (e.g. `matrix.rs`, `bbox.rs`) encapsulate affine math
and bounding-box calculations shared across modules.

## Shared schemas

The `shared/schema` directory contains JSON Schema documents that define the IR
and patch payloads. These schemas are used during development to validate that
both the frontend and backend stay aligned on data contracts.

## End-to-end assets

The `e2e` directory holds an example PDF (`sample.pdf`, to be replaced with a
real test document) and a `tests.http` file showing example REST requests for use
with VS Code's REST Client extension or `curl`.

## Getting started

### Backend

```
cd backend
cargo run
```

The development server listens on `http://localhost:8787`.

### Frontend

```
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` to interact with the editor UI. When running the
frontend locally, you can set the `VITE_API_BASE` environment variable to point
at the backend server (defaults to `http://localhost:8787`).

## Licence

This project is licensed under the Apache License, Version 2.0. See the
[LICENCE](./LICENSE) file for details.
