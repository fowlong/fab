# PDF Editor MVP

This repository contains an experimental PDF editor that keeps PDF content as vector/text data rather than flattening to bitmaps. The project is designed around a Vite + TypeScript frontend that uses pdf.js for preview rendering and Fabric.js for interactive editing overlays, and a Rust backend built on axum and lopdf.

## Project layout

```
pdf-editor/
  LICENSE
  README.md
  frontend/
    index.html
    vite.config.ts
    package.json
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

```
cd frontend
npm install
npm run dev
```

The development server defaults to http://localhost:5173/. If the backend runs on a non-default host/port you can set `VITE_API_BASE` when starting Vite.

### Backend

```
cd backend
cargo run
```

The server listens on http://localhost:8787 and serves REST endpoints under `/api/*` for document management and editing operations.

### End-to-end samples

The `e2e/tests.http` file contains sample REST requests that can be used with VS Code's REST Client extension or `curl`. The `e2e/sample.pdf` file is a placeholder PDF for manual testing.

## Licensing

The project uses the Apache-2.0 license, which can be found in `LICENSE`. All dependencies are selected to be permissively licensed (MIT, Apache-2.0, or similar) to avoid copyleft obligations.
