# PDF Editor

A proof-of-concept visual PDF editor that combines a Fabric.js controller overlay with pdf.js rendering. Users can drag, rotate, and edit true PDF objects, with a Rust backend performing incremental updates to the PDF content streams and resources.

## Features (MVP)

- Upload PDFs and inspect their intermediate representation (IR) per page
- Render accurate page previews using pdf.js
- Manipulate text, images, and vector paths through Fabric.js controller widgets
- Apply transforms, edit text runs, and tweak colour/opacity
- Receive incremental PDF updates streamed from the backend without full rewrites

## Project Structure

```
pdf-editor/
  LICENSE
  README.md
  frontend/
    index.html
    package.json
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

## Getting Started

### Frontend

```
cd frontend
npm install
npm run dev
```

### Backend

```
cd backend
cargo run
```

The backend listens on `http://localhost:8787` and exposes the `/api/*` endpoints consumed by the frontend.

## Licence

Apache License 2.0. See [LICENSE](./LICENSE).
