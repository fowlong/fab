# PDF Editor (MVP scaffold)

This repository contains an initial scaffold for a browser-based PDF editor that combines a pdf.js preview with a Fabric.js overlay and a Rust backend powered by axum and lopdf. The aim is to build a true PDF editor that rewrites objects in-place instead of raster overlays.

## Project layout

```
pdf-editor/
  LICENSE
  README.md
  frontend/
    index.html
    package.json
    tsconfig.json
    vite.config.ts
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
        write.rs
        content.rs
        extract.rs
        patch.rs
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

Each folder currently provides minimal, compilable building blocks so future work can focus on implementing full PDF parsing, editing, and rendering behaviour.

## Development

- **Frontend**: A Vite-based TypeScript project that mounts a pdf.js underlay and a Fabric.js overlay. The current implementation is a lightweight stub that demonstrates how modules interact and defines key utilities (coordinate conversion, IR mapping stubs, and API wrappers).
- **Backend**: A Rust workspace with axum routing and placeholder modules for document loading, IR extraction, patch application, and incremental writing. All modules include skeletal implementations and `todo!()` markers to guide future development while keeping the crate compilable.

### Running the dev servers

```
# Backend
cd backend
cargo run

# Frontend (separate terminal)
cd frontend
npm install
npm run dev
```

Environment variable `VITE_API_BASE` can be used to point the frontend at a different backend host.

## Licensing

The entire project is distributed under the Apache-2.0 license. Dependencies referenced in manifests are limited to permissively licensed projects (MIT or Apache-2.0).
