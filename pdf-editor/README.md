# PDF Editor

This repository contains a proof-of-concept visual PDF editor. The frontend uses Vite, TypeScript, pdf.js, and Fabric.js to render PDF pages and interactive overlays. The backend is written in Rust with axum, lopdf, harfbuzz-rs, and ttf-parser to parse, manipulate, and incrementally rewrite PDF files. All dependencies carry permissive licenses, and the project is distributed under the Apache-2.0 license.

## Structure

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

Each directory contains placeholder files that outline the planned implementation. The code focuses on structure and documentation so that future work can fill in the core editing capabilities.
