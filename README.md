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

The current commit provides initial scaffolding for the frontend and backend projects together with strongly typed interfaces for the IR and patch protocol. Full PDF parsing, shaping, and editing logic are stubbed but the core routes and client wiring are ready for incremental feature development.

### Frontend

* Vite configuration for a TypeScript entry point.
* Shared utility modules for coordinate math, Fabric.js mapping helpers, and API bindings.
* React-free vanilla TypeScript app that renders pdf.js canvases and a Fabric overlay placeholder.

### Backend

* `cargo` workspace with Axum HTTP server skeleton.
* Placeholder modules for PDF parsing, content tokenisation, patching, and font handling.
* Data structures mirroring the JSON contracts shared with the frontend.

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

The development server listens on <http://localhost:5173>. For now it renders placeholder canvases because the backend does not yet implement PDF parsing.

### Backend

```
cd backend
cargo run
```

The Axum server starts on <http://localhost:8787>. The `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId` routes are stubbed and return mock data.

## Roadmap

* Implement real PDF parsing in `backend/src/pdf/extract.rs`.
* Produce incremental updates for transform, text edit, and style patches.
* Complete the Fabric overlay controller logic and inline text editing UX.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
