# PDF Editor (MVP scaffold)

This repository tracks the in-progress implementation of a browser-based PDF editor that keeps the original vector/text content intact. The frontend renders PDF pages with [`pdf.js`](https://github.com/mozilla/pdf.js) and overlays [`Fabric.js`](http://fabricjs.com/) controllers to provide WYSIWYG transforms. The backend is a Rust service built on [`axum`](https://github.com/tokio-rs/axum) that parses PDFs with [`lopdf`](https://github.com/J-F-Liu/lopdf), applies edits, and serialises incremental updates.

> **Status:** This commit establishes the initial project structure, type definitions, and stub API flows for further development. The application is not feature complete yet, but the scaffolding aligns with the architecture described in the project brief.

## Project layout

```
pdf-editor/
├── LICENSE
├── README.md
├── backend
│   ├── Cargo.toml
│   └── src
│       ├── main.rs
│       ├── types.rs
│       ├── util
│       │   ├── bbox.rs
│       │   └── matrix.rs
│       └── pdf
│           ├── mod.rs
│           ├── loader.rs
│           ├── write.rs
│           ├── content.rs
│           ├── extract.rs
│           ├── patch.rs
│           └── fonts
│               ├── mod.rs
│               ├── shape.rs
│               ├── subset.rs
│               └── embed.rs
├── frontend
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── src
│       ├── main.ts
│       ├── pdfPreview.ts
│       ├── fabricOverlay.ts
│       ├── api.ts
│       ├── mapping.ts
│       ├── coords.ts
│       ├── types.ts
│       └── styles.css
├── shared
│   └── schema
│       ├── ir.schema.json
│       └── patch.schema.json
└── e2e
    ├── sample.pdf
    └── tests.http
```

## Getting started

### Backend

```bash
cd backend
cargo run
```

The server listens on `http://localhost:8787`. The current implementation exposes placeholder endpoints for `/api/open`, `/api/ir/:docId`, `/api/patch/:docId`, and `/api/pdf/:docId`.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The Vite dev server hosts the UI at `http://localhost:5173`. Configure `VITE_API_BASE` if the backend runs on a different origin.

## Development roadmap

* Flesh out PDF content parsing in `backend/src/pdf/extract.rs` and content rewriting in `backend/src/pdf/write.rs`.
* Implement Harfbuzz-driven shaping and font subsetting for `editText` operations.
* Complete the Fabric overlay interaction hooks in `frontend/src/fabricOverlay.ts`.
* Expand the JSON Schema definitions in `shared/schema` and connect automated validation.
* Add end-to-end tests under `e2e/` once the MVP flows are functional.

## Licensing

The project uses the Apache-2.0 license. All third-party dependencies listed in this scaffold are compatible permissive licenses (MIT or Apache-2.0).
