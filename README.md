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

Stage 2 of the prototype delivers a working end-to-end transform loop:

* The backend can accept a PDF upload, extract page&nbsp;0 into a lightweight IR (text runs and image XObjects only), and apply transform patches by rewriting the page content stream. Updates are saved incrementally using `lopdf` so the original bytes are preserved.
* The frontend renders page&nbsp;0 with `pdf.js`, overlays Fabric.js controllers for each IR object, and converts drag/rotate/scale edits into `transform` patches. After the backend responds, the preview is refreshed so the underlay always reflects the live PDF.
* All routes expose real data and CORS is enabled for <http://localhost:5173> so the Vite dev server can talk to the Axum API.

## Getting started

### Prerequisites

* Node.js 18+
* Rust toolchain (stable)

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://127.0.0.1:8787>. Routes:

* `POST /api/open` – accept a multipart upload or base64 JSON payload, persist the PDF to `/tmp/fab/<docId>.pdf`, parse, and cache the IR.
* `GET /api/ir/:docId` – return the cached IR for page&nbsp;0.
* `POST /api/patch/:docId` – apply transform patches and respond with a data URL for the updated PDF.
* `GET /api/pdf/:docId` – stream the latest bytes.

### Frontend

```
cd frontend
npm install
npm run dev
```

The Vite dev server runs on <http://localhost:5173>. Visit the URL after the backend is running, upload a PDF, and manipulate the overlay handles to send real patches.

### Stage 2 manual test

1. Start the backend (`cargo run`) and frontend (`npm run dev`).
2. Open <http://localhost:5173>, upload a single-page PDF containing text and an image.
3. Drag a text handle 50&nbsp;px to the right and 20&nbsp;px down, rotate roughly 10°, and scale it slightly.
4. Drag or rotate the image handle.
5. Use the “Download current PDF” button and inspect the downloaded file — the content stream should show updated `Tm`/`cm` entries and no duplicate objects. The incremental tail should be visible when the PDF is opened in a hex viewer.

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
