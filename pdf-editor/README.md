# PDF Editor

A split frontend/backend prototype for a true PDF editor that lets users move, transform, and edit native PDF objects instead of flattening them into bitmaps. The frontend uses pdf.js for rendering and Fabric.js for the interaction overlay, while the backend rewrites the PDF content streams using Rust, axum, and lopdf.

## Features

* Upload a PDF and inspect the parsed intermediate representation (IR) per page.
* Render each page with pdf.js and display controller widgets with Fabric.js.
* Apply move/scale/rotate transforms, text edits, and style changes via an incremental PDF patch API.
* Download the latest incrementally saved PDF at any time.

The code base is intentionally modular so future stretch goals—like path node editing, z-order changes, and form XObject lifting—can be layered on without rewriting the core.

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
      util/
        matrix.rs
        bbox.rs
      pdf/
        mod.rs
        loader.rs
        extract.rs
        content.rs
        patch.rs
        write.rs
        fonts/
          mod.rs
          shape.rs
          subset.rs
          embed.rs
  shared/
    schema/
      ir.schema.json
      patch.schema.json
  e2e/
    sample.pdf
    tests.http
```

## Getting started

### Backend

```
cd backend
cargo run
```

The development server listens on `http://localhost:8787` by default. Set `PDF_EDITOR_DATA_DIR` to change the directory used for persisted PDFs.

### Frontend

```
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173`. The frontend expects the backend at `http://localhost:8787`; configure a different URL via `VITE_API_BASE_URL`.

## Data contracts

Canonical JSON schema definitions for the IR and patch protocol live under `shared/schema`. The TypeScript and Rust code both reference these to keep the contracts consistent.

## Licensing

All code in this repository is released under the Apache-2.0 license. Third-party dependencies are limited to permissive licenses (MIT or Apache-2.0).
