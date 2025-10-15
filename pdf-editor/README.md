# PDF Editor MVP

This repository hosts an experimental browser-based PDF editor. The application
combines a Fabric.js controller overlay with pdf.js rendering to edit PDF content
streams directly on the server using a Rust backend.

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

### Backend

```
cd backend
cargo run
```

The server listens on `http://localhost:8787`. API contracts are documented in
`shared/schema/` and sample requests are available in `e2e/tests.http`.

### Frontend

```
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` in a browser. Set `FRONTEND_API_BASE` in
`.env.local` if the backend runs on a different host or port.

## Licensing

All code in this repository is provided under the Apache-2.0 license. External
dependencies are chosen to use permissive licenses compatible with Apache-2.0.
