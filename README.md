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

The Stage&nbsp;2 milestone implements a true PDF rewrite loop for page&nbsp;0 text runs and image XObjects. The backend now parses the page content stream, exposes a structured intermediate representation, and applies transform patches by left-multiplying the `Tm`/`cm` matrices before writing an incremental update. The frontend renders the live PDF with `pdf.js`, overlays Fabric.js controllers, and posts transform patches whenever the user drags, rotates, or scales a handle.

### Frontend

* Vite + strict TypeScript application without a framework runtime.
* `pdf.js` underlay that renders page&nbsp;0 into a bitmap canvas.
* Fabric.js overlay that exposes draggable controllers for text runs and images; controller movement is converted back into PDF point-space matrices via `coords.ts`.
* API client for `/api/open`, `/api/ir`, `/api/patch`, and `/api/pdf` endpoints plus download helper.

### Backend

* Axum 0.7 server with CORS configured for <http://localhost:5173>.
* `pdf::content` tokenizer with byte-span tracking (unit-tested for `cm`, `BT/ET`, `Tj/TJ`, and `Do`).
* `pdf::extract` IR builder that resolves page&nbsp;0 streams, graphics state, text state, and image placements.
* `pdf::patch` transform logic that rewrites `Tm`/`cm` tokens, splices updated bytes, and uses `pdf::write` to emit an incremental update via `lopdf::IncrementalDocument`.
* In-memory document store backed by `/tmp/fab/<docId>.pdf` snapshots.

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

Visit <http://localhost:5173>, choose a PDF with a text run and an image on page&nbsp;0, and the app will render the page with draggable controllers.

### Backend

```
cd backend
cargo run
```

The service listens on <http://localhost:8787>. The frontend uploads PDFs through `/api/open`, queries `/api/ir/:docId` for the structured representation, posts transform patches to `/api/patch/:docId`, and downloads the incrementally updated file via `/api/pdf/:docId`.

### Stage&nbsp;2 verification steps

1. Start the backend (`cargo run`) and frontend (`npm run dev`).
2. Load a one-page PDF that contains at least one text run within a `BT…ET` block and one image XObject.
3. Drag a text controller ~50&nbsp;px right and 20&nbsp;px down, rotate it slightly, and release. The backend updates the `Tm` entry and the viewer re-renders the page.
4. Repeat for the image controller to confirm the surrounding `cm` operator is rewritten.
5. Use the “Download current PDF” button to fetch the incrementally saved PDF. Inspect the content stream to verify a single updated `Tm`/`cm` and no duplicate drawings.

All third-party dependencies remain under permissive licences.

### Troubleshooting

* `cargo test` currently fails in the default sandbox because outbound HTTPS proxying to crates.io returns HTTP 403. Re-run once network access is restored or use a cached registry mirror.
* `npm run build` may print `sh: 1: vite: Permission denied` when the bundled Windows binary lands without the executable bit. Reinstall dependencies on Linux (`rm -rf node_modules && npm install`) or invoke `npx vite build` to use the platform-specific shim.

## Roadmap

* Implement real PDF parsing in `backend/src/pdf/extract.rs`.
* Produce incremental updates for transform, text edit, and style patches.
* Complete the Fabric overlay controller logic and inline text editing UX.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
