# PDF Editor

This repository hosts an experimental PDF editor that combines a Fabric.js overlay with a pdf.js bitmap underlay to offer direct manipulation of true PDF content streams. The long-term goal is a full incremental editor that rewrites text, image, and vector path objects in-place without rasterisation.

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

Stage 2 delivers end-to-end transform editing for text runs and image XObjects on page 0. The backend extracts the live IR from the PDF, rewrites the content stream matrices (text `Tm`, image `cm`) in an incremental save, and returns an updated data URL for immediate preview. The frontend renders page 0 with pdf.js, builds draggable Fabric controllers, and dispatches transform patches as the user moves, rotates, or scales objects.

### Frontend

* Vite-powered TypeScript app with strict mode enabled.
* pdf.js underlay rendering for page 0 and Fabric.js overlay controllers for text and image objects.
* API bindings for `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId`.
* Matrix utilities for converting between px and pt spaces and translating Fabric deltas into PDF deltas.

### Backend

* Axum 0.7 server with multipart/base64 upload handling, CORS configured for `http://localhost:5173`.
* lopdf-driven IR extraction for page 0 text runs and image XObjects, including span metadata for precise patching.
* Transform patching that splices updated matrices into decompressed content streams and appends incremental xref sections.
* Document store that persists uploaded PDFs to `/tmp/fab/<docId>.pdf` and caches the parsed IR for quick reads.

## Stage 2 workflow

### Prerequisites

* Node.js 18+
* Rust toolchain (stable)

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://localhost:8787> with CORS open to the Vite dev server.

### Frontend

```
cd frontend
npm install
npm run dev
```

The Vite dev server runs on <http://localhost:5173> and proxies API calls to the backend.

### Test steps

1. Start both servers as outlined above.
2. Visit <http://localhost:5173> and upload a one-page PDF that contains at least one text run and one image.
3. Drag the translucent controller for the text object 50px to the right and 20px down, rotate it by ~10°, and scale it slightly. The underlay refreshes once the backend replies with an incremental update.
4. Perform a similar drag/rotate/scale on the image controller.
5. Use the “Download updated PDF” button to save the incrementally rewritten file. Inspect the content stream to confirm that the relevant `Tm`/`cm` matrices are updated and no duplicate drawing commands remain.

## Roadmap

* Extend IR extraction and patching to additional operators (paths, multiple pages).
* Add inline text editing and font management.
* Introduce automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
