# PDF Editor MVP

This repository contains the beginnings of a browser-based PDF editor that renders a PDF using pdf.js and uses Fabric.js as a controller overlay. Changes that the user performs on the overlay are sent to a Rust backend that rewrites the PDF content streams incrementally using `lopdf`.

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

See inline `README` files in the frontend and backend folders for instructions on how to run each part.
