# Frontend

This Vite + TypeScript frontend renders a PDF preview using `pdf.js` and paints Fabric.js rectangles over each IR object returned by the backend.

## Scripts

```bash
npm install
npm run dev
```

The dev server expects the backend to run on `http://localhost:8787`. Configure an alternate API base by setting `VITE_API_BASE`.
