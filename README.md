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

Stage two of the project delivers a working end-to-end transform editor. The backend parses PDF content streams, emits an intermediate representation for page zero, and applies transform patches by rewriting matrices inside new incremental revisions. The frontend renders page zero with `pdf.js`, overlays Fabric.js controllers on text runs and image XObjects, and posts transform patches when the user drags, rotates, or scales an object. Each patch results in a fresh incremental revision that can be downloaded.

## Getting started

### Prerequisites

* Node.js 18+
* Rust toolchain (stable)

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://127.0.0.1:8787>. CORS is configured for the Vite dev server on port 5173.

### Frontend

```
cd frontend
npm install
npm run dev
```

The frontend dev server runs on <http://localhost:5173>.

### Manual test plan

1. Start both servers as outlined above.
2. Open <http://localhost:5173> in a browser.
3. Click “Load sample document” to fetch the bundled one-page PDF containing a text run and an image.
4. Drag the blue text controller roughly 50 px right and 20 px down, rotate it ~10°, and apply a slight scale. The PDF preview should re-render with the updated placement once the patch succeeds.
5. Drag and rotate the image controller; the image should move in the PDF underlay after the patch completes.
6. Use the “Download current PDF” button to retrieve the incrementally saved document and confirm that the updated transforms are present when opened in an external viewer.

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
