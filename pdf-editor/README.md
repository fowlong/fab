# PDF Editor

A proof-of-concept browser-based PDF editor that combines a pdf.js rendered preview with a Fabric.js overlay for direct manipulation of PDF content streams. The backend is written in Rust using axum and lopdf to parse, patch, and incrementally rewrite PDF files.

## Project structure

```
pdf-editor/
  LICENSE
  README.md
  frontend/
  backend/
  shared/
  e2e/
```

See the individual `README.md` files in each package for development instructions.
