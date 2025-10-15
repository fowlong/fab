# PDF Editor

This repository hosts an experimental PDF editor that combines a Fabric.js overlay with a PDF.js bitmap underlay to offer direct manipulation of true PDF content streams. The current milestone delivers end-to-end support for translating, rotating, and scaling text runs and image XObjects on the first page of a document. Edits are persisted via incremental saves so the original bytes remain intact and the updated file can be downloaded immediately.

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

The `frontend` folder contains the Vite + TypeScript application that renders page bitmaps with `pdf.js` and draws a Fabric.js overlay for interactive editing. The `backend` folder exposes an Axum-based Rust service that parses PDFs via `lopdf`, produces an intermediate representation (IR), and applies JSON patch operations to update content streams. The `shared` folder is reserved for protocol artefacts and `e2e` holds fixtures for end-to-end testing.

All third-party dependencies are limited to permissive licences (MIT or Apache-2.0).

## Stage 2 capabilities

* Upload a PDF and persist the original bytes to a temporary working copy.
* Generate an IR for page 0 that lists text runs (with their `Tm` matrices) and image XObjects (with their `cm` matrices).
* Display a Fabric.js overlay above the pdf.js rendering so that each IR entry can be dragged, rotated, or scaled.
* Translate Fabric transforms back into PDF space and patch the content stream by rewriting the relevant `Tm` or `cm` operator.
* Append updates incrementally so the original bytes remain untouched and the file tail shows the new xref section.
* Download the updated PDF with the new placement applied.

## Getting started

### Prerequisites

* Node.js 18+
* Rust (stable toolchain)

### Backend (Axum + lopdf)

```
cd backend
cargo run
```

The server listens on <http://localhost:8787>. CORS is configured for the Vite dev server on port 5173. Key routes:

* `POST /api/open` – accept a PDF (multipart form-data or JSON with base64) and return a `docId`.
* `GET /api/ir/:docId` – return the IR for page 0 with text and image objects.
* `POST /api/patch/:docId` – apply transform patches, update the PDF incrementally, and return a data URL for the revised file.
* `GET /api/pdf/:docId` – stream the latest bytes for preview/download.

### Frontend (Vite + Fabric.js)

```
cd frontend
npm install
npm run dev
```

The development server runs on <http://localhost:5173>. Open the page, select a PDF, and the first page will render with draggable controllers for each text run and image.

### Manual verification

1. Start the backend (`cargo run`) and the frontend (`npm run dev`).
2. Upload a single-page PDF containing at least one text run and one image.
3. Drag a text overlay 50 px right and ~20 px down, apply a slight rotation, and scale it slightly. The canvas should update once the patch response is returned.
4. Drag and rotate an image overlay.
5. Use the *Download updated PDF* button and open the file in a viewer – both the text and image should reflect the new placement. Inspect the content stream to confirm that only the relevant `Tm`/`cm` operators changed and that the tail contains a fresh incremental xref section.

## Roadmap

* Extend the IR and patcher to cover additional operators (paths, colour, and text editing).
* Add keyboard nudge controls and snapping behaviours to the Fabric overlay.
* Build automated end-to-end tests in `/e2e` that exercise the full upload → edit → download loop.

Contributions are welcome via pull requests.
