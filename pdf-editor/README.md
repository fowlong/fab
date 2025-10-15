# PDF Editor

This project provides a proof-of-concept PDF editor with a Fabric.js overlay
controlling native PDF content. The application is split into a Vite-based
TypeScript frontend and a Rust backend powered by Axum and lopdf.

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

The development server listens on `http://localhost:8787` by default.

### Frontend

```
cd frontend
npm install
npm run dev
```

Then open `http://localhost:5173` in a browser. Configure
`FRONTEND_API_BASE=http://localhost:8787` to connect to the backend if required.

## License

Apache-2.0. See [LICENSE](LICENSE).
