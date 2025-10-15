# PDF Editor

This repository hosts an experimental PDF editor that combines a Fabric.js overlay with a PDF.js bitmap underlay to offer direct
manipulation of true PDF content streams. The long-term goal is a full incremental editor that rewrites text, image, and vector
path objects in-place without rasterisation.

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

The `frontend` folder contains the Vite + TypeScript application that renders PDF pages with `pdf.js` and draws a Fabric.js overlay
for interactive editing. The `backend` folder contains an Axum-based Rust service that parses PDFs via `lopdf`, exposes them
as an intermediate representation (IR), and applies JSON patch operations to produce incremental PDF updates. The `shared` folder
holds JSON schema files that document the contracts exchanged between the frontend and backend. The `e2e` folder is reserved
for integration tests and sample fixtures.

The codebase is licensed under Apache-2.0. All third-party dependencies are limited to permissive licences (MIT or Apache-2.0).

## Stage 2 status

Stage 2 delivers a minimum viable transform workflow:

* The backend persists uploaded PDFs, extracts text runs and image XObjects for page 0, and streams incremental updates after transform patches.
* The frontend renders page 0 via pdf.js, draws Fabric.js controllers for each selectable object, and issues transform patches when controllers are moved, rotated, or scaled.
* The updated PDF can be downloaded directly from the backend once patches are applied.

## Getting started

### Prerequisites

* Node.js 18+
* Rust toolchain (stable)

### Backend

```
cd backend
cargo run
```

The Axum server listens on <http://localhost:8787>. CORS is configured for <http://localhost:5173> so the frontend can access the API.

### Frontend

```
cd frontend
npm install
npm run dev
```

The Vite dev server runs on <http://localhost:5173>.

### Stage 2 test steps

1. Start the backend (`cargo run`) and the frontend (`npm run dev`).
2. Visit <http://localhost:5173>, upload a one-page PDF that contains at least one text run and one image.
3. Drag, rotate, or scale a controller in the overlay. The backend patches the PDF content stream and returns the updated document.
4. Use the download button to fetch the revised PDF and verify the transformed objects in a local viewer.

## Roadmap

* Expand the IR to include path objects and richer font metadata.
* Implement text editing and appearance patches alongside transforms.
* Add automated end-to-end tests in `/e2e` that exercise representative editing scenarios.

Contributions are welcome via pull requests.
