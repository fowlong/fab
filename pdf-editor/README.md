# PDF Editor

This project provides a browser-based PDF editor where the bitmap preview rendered by pdf.js is paired with a Fabric.js overlay. The overlay exposes controllers for the native PDF objects so that move, scale, rotate, colour, opacity and text edits manipulate the underlying PDF content streams directly.

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
      util/
        bbox.rs
        matrix.rs
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

## Getting started

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

The backend server listens on <http://localhost:8787>. Configure the frontend `FRONTEND_API_BASE` environment variable if it runs on a different origin.

## Licensing

The repository is distributed under the terms of the Apache License 2.0. All dependencies mentioned in the roadmap are MIT or Apache licensed.
