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

Stage 2 introduces a functioning end-to-end slice:

* The backend accepts real PDFs, parses page 0 streams, exposes a text/image IR, applies transform patches, and emits incremental updates without rewriting the original bytes.
* The frontend renders the updated PDF via pdf.js, draws Fabric controllers aligned to the IR, and pushes transform deltas back to the backend whenever the user drags, rotates, or scales an overlay.
* Text runs gain a synthetic `Tm` matrix when needed so follow-up transforms always work against an explicit matrix.

### Frontend

* Vanilla TypeScript + Vite bundle that targets the Axum backend running on <http://localhost:8787> (CORS enabled for the Vite dev server on port 5173).
* `pdfPreview.ts` renders page 0 via pdf.js and exposes page dimensions for the overlay coordinate transforms.
* `fabricOverlay.ts` builds Fabric controllers for text runs and image XObjects, converts Fabric matrices into PDF-space deltas, and calls the backend with a single `transform` patch.

### Backend

* Axum 0.7 server with multipart and base64 JSON upload support at `POST /api/open`.
* `GET /api/ir/:docId` returns the parsed IR for page 0 (text + image objects only).
* `POST /api/patch/:docId` applies transform patches by rewriting either the `Tm` in a BT…ET span or the closest `cm` before a `Do` call, writing a new content stream via an incremental update, and returning a data-URL of the updated PDF.
* `GET /api/pdf/:docId` streams the latest bytes from `/tmp/fab/<docId>.pdf`.

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

Open <http://localhost:5173> in the browser. The app connects to the Axum backend via CORS and lets you upload a PDF, manipulate overlays, and download the rewritten file.

### Backend

```
cd backend
cargo run
```

The server listens on <http://localhost:8787> and persists uploaded PDFs to `/tmp/fab/<docId>.pdf`. Incremental updates append to the original bytes, so inspecting the tail of the file reveals the new stream and xref subsection.

### Stage 2 manual test

1. Start the backend (`cargo run`) and the frontend dev server (`npm run dev` inside `frontend`).
2. Load a one-page PDF containing text and at least one image.
3. Drag a text overlay ~50px to the right and ~20px down, rotate by ~10°, and scale by roughly 1.1×.
4. Click the download button and inspect the PDF: the text should appear in the new position/orientation and the BT…ET block must contain a single updated `Tm` command (inserted if it was missing previously).
5. Drag/rotate/scale the image overlay. Download again and confirm the image moves while the XObject resource stays untouched (only the nearest `cm` changes).
6. Ensure the downloaded PDF size only increases modestly (incremental update at the tail) and that the backend log reports successful patch application.

## Development environment

Open the repository in the provided [Development Container](https://containers.dev/) configuration to get a reproducible toolchain with:

* Rust (stable) plus the `wasm32-unknown-unknown` target and [`cargo-watch`](https://github.com/watchexec/cargo-watch).
* Node.js LTS with `pnpm` installed.
* `poppler-utils` (`pdftoppm`) and Ghostscript for generating PDF image diffs.

> [!TIP]
> On Windows hosts outside of the Dev Container, install the PDF tooling with Chocolatey: `choco install poppler`.

## Roadmap

* Implement real PDF parsing in `backend/src/pdf/extract.rs`.
* Produce incremental updates for transform, text edit, and style patches.
* Complete the Fabric overlay controller logic and inline text editing UX.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
