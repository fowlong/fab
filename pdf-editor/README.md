# Structured PDF Editor

This project implements a real PDF editor with a Fabric.js control overlay.
Users edit individual PDF content objects visually and the backend updates the
underlying content streams without rasterising the document.

## Project layout

```
pdf-editor/
  LICENSE                    # Apache-2.0
  README.md                  # this file
  frontend/                  # Vite + TypeScript application
  backend/                   # Axum + Rust services
  shared/schema/             # JSON schema contracts
  e2e/                       # fixtures and manual tests
```

### Frontend

The frontend is a Vite project written in TypeScript. It renders PDF pages using
`pdf.js` as a bitmap underlay and overlays Fabric.js controllers that map
1:1 to PDF objects. User interactions produce delta transforms or edit
operations that are forwarded to the backend.

Key source files:

- `src/main.ts` – application entry point and UI wiring.
- `src/pdfPreview.ts` – renders PDF pages with `pdf.js`.
- `src/fabricOverlay.ts` – creates and maintains Fabric.js controllers.
- `src/mapping.ts` – mapping between IR objects and Fabric objects.
- `src/coords.ts` – shared coordinate conversion helpers (points ↔ pixels).
- `src/api.ts` – REST client for the backend API.
- `src/types.ts` – TypeScript definitions that mirror backend types.

### Backend

The backend exposes a JSON/HTTP API that mirrors the specification:

- `POST /api/open` – open and cache a PDF.
- `GET /api/ir/:docId` – return the current intermediate representation.
- `POST /api/patch/:docId` – apply edits and return an incrementally updated PDF.
- `GET /api/pdf/:docId` – download the latest PDF bytes.

Implementation modules:

- `pdf/loader.rs` – load and cache PDFs on disk.
- `pdf/content.rs` – tokenise content streams (operators, operands).
- `pdf/extract.rs` – extract the IR from content streams.
- `pdf/patch.rs` – apply JSON patch operations to the IR.
- `pdf/write.rs` – write incremental updates to the PDF file.
- `pdf/fonts/*` – font shaping, subsetting, and embedding.
- `types.rs` – shared Rust structs for API payloads.
- `util/*` – geometry helpers (matrices, bounding boxes).

### Shared Schemas

JSON Schema files that mirror the REST API contracts:

- `shared/schema/ir.schema.json`
- `shared/schema/patch.schema.json`

These can be consumed by code generators or used for runtime validation.

### End-to-end fixtures

- `e2e/sample.pdf` – sample document with known object structure.
- `e2e/tests.http` – example REST requests (for VS Code REST Client or curl).

## Development

### Prerequisites

- Node.js 18+
- Rust toolchain (stable)

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The development server runs at <http://localhost:5173>. Set the environment
variable `VITE_API_BASE` to override the backend base URL if required.

### Backend

```bash
cd backend
cargo run
```

The server listens on <http://localhost:8787> by default. Configuration options
are exposed via environment variables; see inline documentation in `main.rs`.

### Testing

Manual REST workflows can be tried using the HTTP file under `e2e/tests.http`.
Automated tests will be added alongside the evolving implementation.

## Licensing

All code in this repository is licensed under Apache-2.0. Dependencies are
restricted to permissive licences as noted in the project specification.
