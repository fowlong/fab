# Frontend

This package contains the Vite + TypeScript application that renders PDF pages and overlays Fabric.js controllers so that individual PDF content objects can be moved and edited.

## Stack

- [Vite](https://vitejs.dev/) (MIT) for development/build tooling
- [TypeScript](https://www.typescriptlang.org/) (Apache-2.0)
- [pdf.js](https://github.com/mozilla/pdf.js) via `pdfjs-dist` (Apache-2.0)
- [Fabric.js](http://fabricjs.com/) (MIT)

## Scripts

```
npm install
npm run dev
npm run build
npm run preview
```

Set `VITE_API_BASE` to point to the backend (default `http://localhost:8787`).

## Development status

The TypeScript modules provide scaffolding for:

- rendering PDF pages to `<canvas>` elements using pdf.js
- creating Fabric.js overlays that mirror the PDF layout
- translating between CSS pixels and PDF points
- mapping overlay objects to the backend IR
- submitting patch requests to the backend

The modules are intentionally incomplete but outline the intended control flow, data structures, and key functions to implement.
