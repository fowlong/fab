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

The Stage 2 milestone introduces end-to-end transform editing for page 0 of any uploaded PDF. The backend now extracts a lightweight IR for text runs (`BT … ET`) and image XObjects, exposes it via `/api/ir/:docId`, and rewrites content streams incrementally when the frontend posts transform patches. The frontend renders the live PDF underlay with `pdf.js`, overlays Fabric.js controllers for every selectable object, and converts drag/rotate/scale gestures into PDF-space matrices.

### Frontend

* Strict TypeScript build driven by Vite.
* `pdfPreview.ts` renders page 0 with `pdf.js` and keeps the bitmap in sync after every patch.
* `fabricOverlay.ts` maps IR objects to translucent controllers, captures user transforms, and reverts safely if the patch request fails.
* `coords.ts` contains shared affine helpers (px ↔︎ pt conversion, multiply, inverse, Fabric-to-PDF delta conversion).
* A minimal UI that wires file upload, status messaging, download, and per-op patch dispatch.

### Backend

* Axum 0.7 server with CORS configured for <http://localhost:5173>.
* `pdf/content.rs` tokenises content streams with byte spans and unit tests covering the operators used in this stage.
* `pdf/extract.rs` resolves page 0, tracks graphics state, and produces both JSON IR and in-memory caches for patching.
* `pdf/patch.rs` rewrites text `Tm` and image `cm` entries, inserts missing `Tm` tokens, and composes incremental updates via `pdf/write.rs`.
* `pdf/write.rs` appends new objects and trailers without disturbing the original bytes so file diffs stay incremental.

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

The development server listens on <http://localhost:5173>. Upload a one-page PDF containing at least one text run and one image. Blue translucent controllers appear over each object—drag, rotate, or scale them to post transform patches.

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://localhost:8787>. It accepts multipart uploads at `/api/open`, returns the extracted IR via `/api/ir/:docId`, applies transform patches with `/api/patch/:docId`, and streams the latest PDF bytes from `/api/pdf/:docId`.

### Manual test plan

1. Start both dev servers as described above.
2. Upload a one-page PDF (text + image). The IR overlays should appear immediately.
3. Drag a text overlay ~50 px to the right, 20 px down, rotate a few degrees, and scale slightly. Download the PDF—its content stream should contain a single updated `Tm`, and the visual output should mirror the on-screen controller.
4. Apply a similar transform to an image overlay. The downloaded PDF should show the image in the new position with an updated `cm` entry.
5. Inspect the PDF tail in a hex editor—an incremental `xref` and trailer should be appended without rewriting the original bytes.

## Roadmap

* Extend the IR to cover multi-page documents and additional object kinds (paths, annotations).
* Implement text editing and style patches with proper font management.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
