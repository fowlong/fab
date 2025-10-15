# PDF Editor

This repository hosts an experimental PDF editor that combines a Fabric.js overlay with a PDF.js bitmap underlay to offer direct manipulation of true PDF content streams. The long-term goal is a full incremental editor that rewrites text, image, and vector path objects in-place without rasterisation.

## Project layout

```
pdf-editor/
  LICENSE
  README.md
  /frontend
  /backend
  /shared
  /e2e
```

The `frontend` folder contains the Vite + TypeScript application that renders PDF pages with `pdf.js` and draws a Fabric.js overlay for interactive editing. The `backend` folder contains an Axum-based Rust service that parses PDFs via `lopdf`, exposes them as an intermediate representation (IR), and applies JSON patch operations to produce incremental PDF updates. The `shared` folder holds JSON schema files that document the contracts exchanged between the frontend and backend. The `e2e` folder is reserved for integration tests and sample fixtures.

The codebase is licensed under Apache-2.0. All third-party dependencies are limited to permissive licences as listed in the project plan (MIT or Apache-2.0).

## Current status

Stage 2 ships a minimal but functional transform workflow:

### Frontend

* Renders page 0 of an uploaded PDF with `pdf.js`.
* Uses Fabric.js controllers to drag, scale, and rotate text runs and image XObjects.
* Converts Fabric transforms back into PDF point-space matrices before calling the backend.

### Backend

* Persists uploaded PDFs to `/tmp/fab` and keeps an in-memory document store.
* Tokenises page content streams to build an IR that exposes text and image placements.
* Applies transform patches by rewriting `Tm`/`cm` matrices and appending an incremental PDF revision.

## Getting started

### Prerequisites

* Node.js 18+
* Rust toolchain (stable)

### Frontend

```
cd frontend
npm install
npm run dev
```

The development server listens on <http://localhost:5173>.

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://localhost:8787> and serves the `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId` routes.

## Stage 2 verification

1. Start the backend (`cargo run` in `backend/`).
2. Start the frontend dev server (`npm run dev` in `frontend/`).
3. Visit <http://localhost:5173>, upload a one-page PDF that includes text and an image.
4. Drag, rotate, and scale the overlays; each interaction triggers a transform patch.
5. Use the “Download updated PDF” button to fetch the incrementally saved document and inspect the updated `Tm`/`cm` commands.

## Roadmap

* Extend the IR to support inline text editing and vector paths.
* Produce incremental updates for text edits and styling patches.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
