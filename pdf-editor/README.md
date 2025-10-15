# PDF Editor

This repository contains a proof-of-concept browser-based PDF editor that keeps
PDF content in vector form. The frontend renders PDF pages using `pdf.js` and
exposes editable overlays using `Fabric.js`. The backend is an Axum-based Rust
server that parses PDFs with `lopdf`, rewrites content streams, and returns
incremental updates.

## Structure

```
pdf-editor/
  LICENSE
  README.md
  frontend/
  backend/
  shared/
  e2e/
```

Refer to `frontend/README.md` and `backend/README.md` (if present) for more
details on running each part of the stack.
