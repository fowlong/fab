# PDF Editor

A proof-of-concept browser-based PDF editor that combines a pdf.js bitmap preview with a Fabric.js controller overlay to drive direct PDF content stream updates performed by a Rust backend.

## Features (MVP)
- Upload a PDF and inspect an intermediate representation (IR) of each page.
- Manipulate text, image, and path objects via a Fabric.js overlay and push incremental patches to the backend.
- Edit text with HarfBuzz shaping and font subsetting for missing glyphs.
- Apply colour and opacity changes that map to proper PDF graphics state updates.
- Download incrementally-saved PDFs that preserve original objects when possible.

## Project layout
```
pdf-editor/
  LICENSE
  README.md
  frontend/
  backend/
  shared/
  e2e/
```

Each subdirectory contains its own README or documentation that expands on the implementation details.
