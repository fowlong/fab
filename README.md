# PDF Editor MVP

This repository contains a proof-of-concept for an in-browser PDF editor built with permissively licensed tools. The frontend uses Vite, TypeScript, Fabric.js, and pdf.js to present a live editing experience over a PDF page bitmap. The backend is implemented in Rust with axum and lopdf to parse, patch, and incrementally update PDF content streams without rasterising the source material.

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
    tsconfig.node.json
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

See the individual `README` sections in the frontend and backend directories for development instructions.
