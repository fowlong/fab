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

The Stage 2 milestone delivers an interactive transform MVP capable of opening a PDF, exposing a lightweight intermediate representation (IR) for page 0, and rewriting matrices for text runs and image XObjects. The backend performs incremental saves so the original bytes are preserved while the appended tail reflects the edits.

### Frontend

* Vite-powered TypeScript app compiled in strict mode.
* `pdf.js` renders the page underlay while Fabric.js draws transparent controllers above each editable object.
* Dragging, rotating, or scaling an overlay issues a transform patch whose delta matrix is converted from Fabric space into PDF space before hitting the backend.

### Backend

* Axum service written in Rust 2021 edition with permissive dependency licences only.
* Content stream tokeniser with unit tests that tracks BT…ET, text operators, cm, and Do instructions.
* IR extraction for page 0 covering text runs and image XObjects with bounding boxes and base matrices.
* Patch executor that left-multiplies matrices for text Tm operators or image cm operators, then emits an incremental trailer via `write.rs`.

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

The development server listens on <http://localhost:5173>. Uploading a document renders page 0 via `pdf.js`, overlays Fabric controllers for each text run and image XObject, and syncs transform patches with the backend in real time.

### Backend

```
cd backend
cargo run
```

The Axum server starts on <http://localhost:8787>. The `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId` routes operate on real PDFs, caching parsed IR data and persisting incremental updates to `/tmp/fab/<docId>.pdf`.

## Roadmap

Stage 3 will broaden the editor beyond the transform MVP:

* Extend IR extraction and caching across all pages with lazy loading.
* Support editing additional object kinds (paths, annotations) and inline text content changes.
* Provide richer UX affordances such as keyboard nudging, snapping, and selection outlines.
* Add automated end-to-end tests in `/e2e` that cover representative editing scenarios.

Contributions are welcome via pull requests.

## Stage 2

Follow the steps below to exercise the transform MVP:

1. Start the backend service:
   ```bash
   cd backend
   cargo run
   ```
   If `cargo` cannot fetch crates because of a proxy returning 403, retry once the network permits. The project has no custom build scripts.
2. Launch the frontend development server:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```
3. Visit <http://localhost:5173>, upload a one-page PDF that contains text and an image, and wait for the IR overlay to appear.
4. Drag the text controller roughly 50 px to the right and 20 px downward, rotate by ~10°, and scale to ~110%. Repeat the interaction on the image controller.
5. Download the updated PDF and inspect the `Tm` or `cm` operators in the content stream—each should be updated rather than duplicated, and the incremental trailer should be visible at the end of the file.
