# PDF Editor

This repository hosts an experimental PDF editor that couples a Fabric.js overlay with a pdf.js bitmap underlay so that people
can drag, rotate, and scale real PDF objects without rasterising the original content streams. The backend applies each
interaction as an incremental rewrite, preserving the original bytes and appending a new xref section.

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

The `frontend` folder contains the Vite + TypeScript application that renders page 0 of a PDF with `pdfjs-dist` and positions a
Fabric.js canvas on top for interactive guides. The `backend` folder exposes an Axum service that parses PDFs with `lopdf`,
extracts a lightweight IR, and applies transform patches by splicing updated content streams before writing an incremental
update. The `shared` folder stores JSON schema documentation, and `e2e` is reserved for future integration fixtures.

All source code is licensed under Apache-2.0. Third-party dependencies are limited to permissive licences.

## Current status

Stage 2 delivers an end-to-end transform MVP for text runs and image XObjects on page 0.

### Backend

* Stores uploaded PDFs in `/tmp/fab/<docId>.pdf` alongside an in-memory cache of parsed documents and extracted IR.
* Implements `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId` with CORS enabled for
  `http://localhost:5173`.
* Tokenises content streams, tracks graphics state, and emits an IR that captures BT/ET spans, text matrices, image cm blocks,
  and coarse bounding boxes.
* Applies `transform` patches by left-multiplying Tm or cm matrices, writing new stream objects, and appending an incremental
  update so the original bytes remain untouched.

### Frontend

* Renders page 0 into a `<canvas class="pdf-underlay">` using `pdfjs-dist`.
* Builds a Fabric overlay with transparent controllers for each text run or image from the IR, mirroring the PDF transform.
* Converts Fabric drag/rotate/scale gestures into PDF-space matrices and posts `transform` patches back to the backend.
* Allows users to download the incrementally saved PDF straight from the backend.

## Stage 2 manual verification

1. **Start the backend**
   ```bash
   cd backend
   cargo run
   ```

2. **Start the frontend**
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

3. **Exercise the editor**
   * Visit <http://localhost:5173>.
   * Upload a single-page PDF that contains both text and an image.
   * Drag, rotate, and scale the blue controllers; each modification sends a patch and re-renders the underlay.
   * Use the download button to fetch the updated PDF and confirm the content stream now contains the adjusted Tm/cm values
     while the XObject entries remain unchanged.

These steps validate the incremental rewrite flow for the Stage 2 milestone. Future milestones will extend the IR, add
selection polish, and integrate automated tests.
