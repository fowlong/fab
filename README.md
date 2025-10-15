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

The `frontend` folder contains the Vite + TypeScript application that renders PDF pages with `pdf.js` and draws a Fabric.js
overlay for interactive editing. The `backend` folder contains an Axum-based Rust service that parses PDFs via `lopdf`, exposes
them as an intermediate representation (IR), and applies JSON patch operations to produce incremental PDF updates. The `shared`
folder holds JSON schema files that document the contracts exchanged between the frontend and backend. The `e2e` folder is
reserved for integration tests and sample fixtures.

The codebase is licensed under Apache-2.0. All third-party dependencies are limited to permissive licences as listed in the
project plan (MIT or Apache-2.0).

## Stage 2 status

The Stage 2 milestone wires the scaffold into a working end-to-end transform loop. The backend parses a real PDF, extracts the
page-0 IR, applies transform patches to text/image objects, and performs an incremental save. The frontend renders page 0 with
pdf.js, overlays Fabric.js controllers, and posts transform deltas back to the service.

### Frontend

* Vite configuration for a TypeScript entry point.
* Strict TypeScript modules for coordinate math, Fabric.js interaction, and REST bindings.
* Vanilla UI that renders pdf.js canvases and real overlay controllers driven by the backend IR.

### Backend

* Axum HTTP server with CORS configured for <http://localhost:5173>.
* PDF loader, content stream tokeniser, IR extractor for page 0, patch applier, and incremental writer.
* In-memory document store that persists uploads to `/tmp/fab`, caches the IR, and serves incremental revisions.

### Prerequisites

* Node.js 18+
* Rust toolchain (stable)

### Backend

```
cd backend
cargo run
```

The server listens on <http://localhost:8787> and exposes the following routes:

* `POST /api/open` – accept a multipart or base64 JSON payload, persist the PDF, and return `{ docId }`.
* `GET /api/ir/:docId` – return the page-0 IR with text and image objects.
* `POST /api/patch/:docId` – apply transform ops, write an incremental update, and return the updated PDF as a data URI.
* `GET /api/pdf/:docId` – stream the latest revision.

### Frontend

```
cd frontend
npm install
npm run dev
```

Vite serves the UI at <http://localhost:5173>. Upload a one-page PDF that contains text and an image, then drag/rotate/scale the
overlay controllers. Each modification posts a transform to the backend, re-renders the PDF underlay, and refreshes the IR.

### Manual test flow

1. Start the backend and frontend as described above.
2. Upload a PDF with a text run and an image.
3. Drag the text overlay 50&nbsp;px to the right, rotate it slightly, and release – the underlay refreshes with the new placement.
4. Drag or scale the image overlay – the PDF is patched in-place and can be downloaded via the sidebar button.
5. Use the download button to fetch the incrementally updated PDF (`doc-XXXX.pdf`) from the backend.

Contributions are welcome via pull requests.
