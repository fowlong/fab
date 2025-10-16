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

Stage&nbsp;2 delivers a minimum viable transform pipeline. Page&nbsp;0 text runs and image XObjects are extracted into an IR, rendered with a Fabric.js controller, and patched back into the PDF by concatenating new matrices into an incremental update tail. The backend persists documents to `/tmp/fab`, caches the parsed IR, and rewrites the relevant content stream when a controller is moved.

### Frontend

* Strict-mode TypeScript Vite app that renders page&nbsp;0 with `pdfjs-dist`.
* Fabric.js overlay controllers for text and image objects that translate drag/scale/rotate gestures into PDF-space matrices.
* API bindings for `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId`, including a download helper.

### Backend

* Axum 0.7 server with CORS configured for <http://localhost:5173>.
* PDF loader using `lopdf`, a content tokeniser with byte spans, IR extraction for page&nbsp;0 text/image objects, and transform patch handling.
* Incremental writer built on `lopdf::IncrementalDocument` that appends a new content stream object and updates the page dictionary.

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

Visit <http://localhost:5173>, choose a PDF, and drag the overlay controllers to update the live preview.

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://localhost:8787>. It accepts multipart or base64 JSON payloads at `/api/open`, serves IR snapshots, applies transform patches, and streams the incrementally updated PDF.

### Stage 2 manual test

1. Start the backend (`cargo run`) and frontend (`npm run dev`).
2. Load a one-page PDF containing text and an image. The overlay rectangles should appear over each object.
3. Drag, rotate, and scale the text controller; the preview updates after the patch response.
4. Repeat for the image controller.
5. Download the updated PDF and inspect the content stream to confirm a single updated `Tm` or `cm` command.

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
