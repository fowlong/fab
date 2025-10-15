# PDF Editor

A proof-of-concept PDF editor that combines a Fabric.js overlay with pdf.js previews and a Rust backend powered by axum and lopdf. The application is designed to incrementally update PDF content streams rather than rasterising overlays.

## Project layout

```
pdf-editor/
├─ LICENSE
├─ README.md
├─ frontend/
│  ├─ index.html
│  ├─ vite.config.ts
│  ├─ package.json
│  └─ src/
│     ├─ main.ts
│     ├─ pdfPreview.ts
│     ├─ fabricOverlay.ts
│     ├─ api.ts
│     ├─ mapping.ts
│     ├─ coords.ts
│     ├─ types.ts
│     └─ styles.css
├─ backend/
│  ├─ Cargo.toml
│  └─ src/
│     ├─ main.rs
│     ├─ types.rs
│     ├─ pdf/
│     │  ├─ mod.rs
│     │  ├─ loader.rs
│     │  ├─ extract.rs
│     │  ├─ patch.rs
│     │  ├─ write.rs
│     │  ├─ content.rs
│     │  └─ fonts/
│     │     ├─ mod.rs
│     │     ├─ shape.rs
│     │     ├─ subset.rs
│     │     └─ embed.rs
│     └─ util/
│        ├─ matrix.rs
│        └─ bbox.rs
├─ shared/
│  └─ schema/
├─ e2e/
│  ├─ sample.pdf
│  └─ tests.http
└─ geneve.html
```

The current backend exposes stubbed endpoints that accept file uploads, store them in memory and provide placeholder responses for incremental patching. Frontend modules outline the data flow between pdf.js rendering, Fabric.js overlays and the patch API.

## Getting started

### Frontend

```bash
cd frontend
npm install
npm run dev
```

### Backend

```bash
cd backend
cargo run
```

The development server listens on <http://localhost:8787>. Configure the frontend with `FRONTEND_API_BASE=http://localhost:8787` if you proxy requests.

## Roadmap

- Implement full PDF content stream parsing and IR extraction in Rust.
- Wire Fabric.js transform events to patch requests that rewrite PDF matrices.
- Integrate HarfBuzz shaping and font subsetting for text edits.
- Expand e2e assets and automated tests for regression coverage.
