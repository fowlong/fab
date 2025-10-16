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

This milestone implements the minimum viable pipeline for transforming text runs and image XObjects in a PDF content stream. The backend loads a document, builds an intermediate representation for page 0, and applies transform patches by rewriting the relevant `Tm` or `cm` matrices inside an incremental update. The frontend renders page 0 with `pdf.js`, draws Fabric.js controllers, and relays drag/rotate/scale gestures back to the backend as matrix deltas.

### Frontend

* Vite-powered TypeScript application without a framework runtime.
* `pdf.js` underlay paired with Fabric.js overlays that expose draggable controllers for each text run or image.
* Coordinate utilities that convert between pixel space and PDF points so Fabric gestures become precise matrix deltas.
* API bindings for `/api/open`, `/api/ir`, `/api/patch`, and `/api/pdf`, including support for downloading the latest revision.

### Backend

* Axum server with CORS configured for the Vite dev server.
* PDF loader based on `lopdf` that tokenises page-0 content streams, derives a lightweight IR, and caches stream operations.
* Patch handler that left-multiplies text `Tm` matrices or image `cm` matrices, rewrites the affected stream, and appends a new incremental revision.
* Incremental writer built on `lopdf::IncrementalDocument` so the updated PDF preserves the original bytes with a small tail.

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

The development server listens on <http://localhost:5173>.

### Backend

```
cd backend
cargo run
```

The Axum server starts on <http://localhost:8787>.

### Stage 2 verification steps

1. Start the backend (`cargo run`) and the Vite dev server (`npm run dev`).
2. Open <http://localhost:5173> and click “Load sample” to fetch the bundled one-page PDF.
3. Drag the text controller 50&nbsp;px to the right and 20&nbsp;px down, rotate it roughly 10°, and scale it slightly (≈1.1×). The underlay re-renders and the controller remains in sync.
4. Drag and rotate the image controller. The preview refreshes with the image in the new pose.
5. Click “Download PDF” and open the file in an external viewer; the text and image appear with the updated transforms. Inspecting the content stream shows the updated `Tm`/`cm` matrices appended via incremental save.

## Development environment

Open the repository in the provided [Development Container](https://containers.dev/) configuration to get a reproducible toolchain with:

* Rust (stable) plus the `wasm32-unknown-unknown` target and [`cargo-watch`](https://github.com/watchexec/cargo-watch).
* Node.js LTS with `pnpm` installed.
* `poppler-utils` (`pdftoppm`) and Ghostscript for generating PDF image diffs.

> [!TIP]
> On Windows hosts outside of the Dev Container, install the PDF tooling with Chocolatey: `choco install poppler`.

## Roadmap

* Extend the IR to cover additional operators (paths, form XObjects, text edits).
* Implement incremental updates for text editing and style changes.
* Polish the Fabric overlay UX (keyboard nudging, snapping, better hit-testing).
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
