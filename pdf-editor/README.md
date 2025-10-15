# PDF Editor

A full-stack PDF editor that combines a Fabric.js overlay with pdf.js rendering to allow true vector editing of PDF content streams. The project is licensed under the Apache License 2.0 and is intentionally scoped to use permissive third-party dependencies only.

## Project layout

```
pdf-editor/
  LICENSE
  README.md
  frontend/
    index.html
    vite.config.ts
    package.json
    tsconfig.json
    src/
      main.ts
      pdfPreview.ts
      fabricOverlay.ts
      api.ts
      mapping.ts
      coords.ts
      types.ts
      styles.css
  backend/
    Cargo.toml
    src/
      main.rs
      types.rs
      pdf/
        mod.rs
        loader.rs
        write.rs
        content.rs
        extract.rs
        patch.rs
        fonts/
          mod.rs
          shape.rs
          subset.rs
          embed.rs
      util/
        matrix.rs
        bbox.rs
  shared/
    schema/
      ir.schema.json
      patch.schema.json
  e2e/
    sample.pdf
    tests.http
```

## Getting started

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The Vite dev server runs on http://localhost:5173. Set the `VITE_API_BASE` environment variable if the backend is hosted elsewhere.

### Backend

```bash
cd backend
cargo run
```

The Axum server listens on http://localhost:8787 and exposes the REST endpoints under `/api/*`.

## Goals

* Parse PDFs on the backend and expose an intermediate representation (IR) that captures text runs, images, and vector paths per page.
* Render PDF pages in the browser using pdf.js as a bitmap underlay while mirroring the IR objects as interactive Fabric.js controllers.
* When a user manipulates an object in Fabric, emit JSON patches describing the delta transform or content change. The backend rewrites the original PDF content streams and returns an incrementally updated document.
* Maintain original vector data—no raster overlays.

## Status

This repository currently contains the initial scaffolding for the frontend and backend along with JSON schemas and sample assets. Implementation work can build on top of this structure.
