# PDF Editor

This project is an end-to-end prototype of a true PDF editor that keeps PDF
content in vector form. It consists of a browser-based frontend (Vite, TypeScript,
Fabric.js, pdf.js) and a Rust backend (axum, lopdf, harfbuzz-rs) that rewrites
PDF content streams incrementally.

## Project layout

```
pdf-editor/
  frontend/         # Vite app that renders the PDF and overlay controllers
  backend/          # Axum server for PDF parsing, patching, and incremental saves
  shared/schema/    # JSON schema definitions for IR and patch contracts
  e2e/              # Sample PDFs and manual API tests
```

See the `frontend/README.md` and `backend/README.md` (to be written) for details
about running each half of the stack.
