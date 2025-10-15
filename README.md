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

Stage 2 of the editor delivers a working transform pipeline for the first page of a PDF. The backend parses the document with `lopdf`, extracts an intermediate representation for text runs and image XObjects, and applies `transform` patch operations by rewriting the page’s content stream in an incremental update. The frontend renders the page bitmap via `pdf.js`, overlays Fabric.js controllers for each selectable object, and translates user-driven drag/rotate/scale gestures into PDF-space matrices.

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

The Axum server starts on <http://localhost:8787> with CORS enabled for <http://localhost:5173>.

### Stage 2 manual test flow

1. Start the backend (`cargo run` in `backend/`).
2. Start the frontend dev server (`npm run dev` in `frontend/`).
3. Visit <http://localhost:5173>, select a single-page PDF that contains at least one text run and one image.
4. Drag, rotate, or scale one of the blue controllers. The frontend patches page 0 with a `transform` operation, the backend rewrites the relevant `Tm` or `cm` operator, and the updated PDF is re-rendered in-place.
5. Use the “Download updated PDF” button to save the incrementally updated document. Inspect the content stream to confirm that the original placement matrix has been pre-multiplied by the delta.

## Roadmap

* Extend extraction and patching to additional operator types and multi-page documents.
* Implement text editing and style patch operations.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
