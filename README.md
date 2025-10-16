# PDF Editor

This project delivers a minimal end-to-end workflow for rewriting text and image transforms inside a PDF without rasterising the page. A Rust (Axum) backend parses the document with `lopdf`, exposes a lightweight intermediate representation (IR), applies transform patches, and writes an incremental update. A Vite + TypeScript frontend renders page 0 with `pdf.js`, overlays Fabric.js controllers, and pushes deltas back to the server.

## Requirements

- Rust 1.75+ with the default toolchain
- Node.js 18+

All bundled dependencies use permissive licences (Apache-2.0 or MIT).

## Stage 2 workflow

### 1. Start the backend

```bash
cd backend
cargo run
```

The server listens on <http://localhost:8787> with CORS enabled for <http://localhost:5173>.

### 2. Start the frontend

```bash
cd frontend
npm install
npm run dev
```

Open <http://localhost:5173> in a browser.

### 3. Exercise the transforms

1. Pick a one-page PDF that contains at least one text object and one image XObject.
2. Upload it via the “Select PDF” control.
3. Drag, rotate, or scale the text controller – a single transform patch is sent to the backend and the PDF preview refreshes from the incremental save.
4. Repeat the process with the image controller.
5. Use the “Download updated PDF” button to fetch the latest revision.

The IR (page 0 only for now) lists text runs and image placements with their matrices and byte spans. Patching left-multiplies the recorded matrix with the delta supplied by Fabric, rewrites the content stream, and appends a new content stream object plus trailer.

## Repository layout

```
backend/   Rust Axum service (PDF loader, IR extraction, patching, incremental writer)
frontend/  Vite + TypeScript client (pdf.js preview, Fabric overlay)
e2e/       Sample assets and future integration tests
```

## Development notes

- Backend routes:
  - `POST /api/open` – upload a PDF (multipart or base64 JSON), persist to `/tmp/fab/<docId>.pdf`, parse, and cache the IR.
  - `GET /api/ir/:docId` – return the cached IR for page 0.
  - `POST /api/patch/:docId` – apply transform patches, emit incremental PDF bytes, and return a data URI for the updated file.
  - `GET /api/pdf/:docId` – stream the latest bytes.
- The IR caches per document, so follow-up patches do not trigger a full reparse until after the incremental write succeeds.
- Fabric controllers carry their original Fabric matrix (`F0`). Each modification computes `ΔF`, maps it into PDF space, and posts a single `transform` op.

Future stages will add text editing, vector path support, richer selection tooling, and automated integration tests.
