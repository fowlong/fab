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

## Stage 2: Transform MVP

The project now provides a working end-to-end transform workflow for page 0. Users can select a PDF, manipulate Fabric.js overlays that track text runs and image XObjects, and download an incrementally updated PDF with the revised matrices. The backend caches parsed documents, rewrites only the affected content stream, and keeps the original bytes untouched.

### Backend

Start the Axum server:

```
cd backend
cargo run
```

The service listens on <http://localhost:8787> and exposes:

* `POST /api/open` – persist uploaded bytes under `/tmp/fab/<docId>.pdf` and return a `docId`.
* `GET /api/ir/:docId` – return the text/image IR for page 0, including `Tm`/`cm` matrices and byte spans.
* `POST /api/patch/:docId` – accept transform operations, update the relevant matrices, and append an incremental trailer while returning the updated PDF as a data URL.
* `GET /api/pdf/:docId` – stream the most recent PDF bytes for preview or download.

### Frontend

```
cd frontend
npm install
npm run dev
```

The Vite dev server runs on <http://localhost:5173>. Upload a single-page PDF, drag/rotate/scale the transparent controllers, and press “Download updated PDF” to save the rewritten file.

### Manual verification

1. Launch both servers with the commands above.
2. Upload a PDF containing at least one text block and one image.
3. Move the text overlay ~50 px right and ~20 px down, rotate about 10°, and scale slightly; download the PDF and confirm the text moved accordingly.
4. Repeat with the image overlay and verify that only the `cm` matrix changed.
5. Inspect the PDF content stream (for example with `qpdf --show-pages`) to check that only the relevant `Tm`/`cm` operator changed and no duplicate drawing commands remain.

All dependencies remain under permissive licences (MIT or Apache-2.0).
