# Frontend

This package hosts the Vite + TypeScript single-page application that renders
the PDF preview (via pdf.js) and an interactive Fabric.js overlay for editing
PDF objects.

## Getting started

```bash
cd frontend
npm install
npm run dev
```

The development server defaults to http://localhost:5173. Set the environment
variable `VITE_API_BASE` when the backend is running on a different origin
(defaults to `http://localhost:8787`).

## Scripts

- `npm run dev` – start Vite in development mode with HMR.
- `npm run build` – produce a production build.
- `npm run preview` – locally preview the production build.
- `npm run lint` – perform a type-check only compilation.

## Architecture notes

The source modules mirror the data flow described in the project scope:

- `pdfPreview.ts` uses pdf.js to render each PDF page into stacked `<canvas>`
  elements that act as the bitmap underlay.
- `fabricOverlay.ts` initialises Fabric.js and instantiates controller objects
  for the PDF IR nodes.
- `coords.ts` contains coordinate conversion helpers between Fabric canvas
  coordinates (CSS pixels) and PDF user space (points).
- `mapping.ts` synchronises Fabric controllers with IR objects.
- `api.ts` provides minimal fetch wrappers for the backend routes.
- `types.ts` defines TypeScript types that mirror the backend JSON payloads.
- `styles.css` contains lightweight layout styling for the MVP editor shell.

The modules are currently scaffolded with placeholder logic so the project can
build cleanly while backend functionality is completed.
