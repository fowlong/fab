# PDF Editor

This repository hosts an experimental PDF editor that combines a Fabric.js overlay with a PDF.js bitmap underlay to offer direct
manipulation of true PDF content streams. The long-term goal is a full incremental editor that rewrites text, image, and vector
path objects in-place without rasterisation.

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

The `frontend` folder contains the Vite + TypeScript application that renders PDF pages with `pdf.js` and draws a Fabric.js overlay
for interactive editing. The `backend` folder contains an Axum-based Rust service that parses PDFs via `lopdf`, exposes them
as an intermediate representation (IR), and applies JSON patch operations to produce incremental PDF updates. The `shared` folder
holds JSON schema files that document the contracts exchanged between the frontend and backend. The `e2e` folder is reserved
for integration tests and sample fixtures.

The codebase is licensed under Apache-2.0. All third-party dependencies are limited to permissive licences as listed in the project plan.

## Current status

The current build supports transforming text runs and image XObjects on page 0 of a PDF. The backend extracts a lightweight IR that
tracks object matrices and byte spans, applies `transform` patch operations by rewriting the corresponding content stream tokens, and
saves the result as an incremental update. The frontend renders the page with pdf.js, overlays Fabric controllers, and submits delta
matrices whenever the user drags, scales, or rotates a control. After each patch the refreshed PDF can be downloaded from the UI.

### Frontend

* Vite configuration for a strict TypeScript entry point.
* Coordinate helpers that convert between CSS pixels and PDF points while preserving orientation.
* A PdfPreview class that renders page 0 with pdf.js and exposes the page height in points to the overlay.
* A FabricOverlay controller that mirrors IR objects as Fabric rectangles, converts Fabric deltas into PDF deltas, and invokes the backend.
* Minimal UI with status messaging, upload and download actions, and automatic preview refresh after each patch.

### Backend

* Axum HTTP server with CORS enabled for <http://localhost:5173>.
* Endpoints for opening a PDF (`/api/open`), fetching the IR for page 0 (`/api/ir/:docId`), applying transform patches (`/api/patch/:docId`),
  and streaming the latest PDF bytes (`/api/pdf/:docId`).
* Content stream tokenizer that tracks byte spans for BT/ET, Tm, cm, and Do operators.
* IR extraction that resolves page 0, flattens the graphics state, and records text runs plus image XObjects.
* Patch engine that rewrites the target token, injects missing Tm operators when required, and performs incremental saves via `lopdf::IncrementalDocument`.

## Stage 2 instructions

1. **Backend** – start the API service:
   ```bash
   cd backend
   cargo run
   ```
   The server listens on <http://localhost:8787>.

2. **Frontend** – install dependencies and launch Vite:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```
   Open <http://localhost:5173> in your browser.

3. **Test the transform workflow**:
   * Upload a one-page PDF that includes both text and an image.
   * Drag, rotate, or scale the text controller – the backend applies the delta by updating the text matrix (`Tm`).
   * Drag, rotate, or scale the image controller – the backend updates the corresponding `cm` matrix.
   * Use the “Download updated PDF” button to retrieve the incrementally saved revision.
   * Inspect the content stream to confirm that only the relevant matrix has been updated and the original bytes remain intact ahead of the incremental tail.

Contributions are welcome via pull requests.
