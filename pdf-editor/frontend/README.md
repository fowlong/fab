# Frontend

The frontend is a Vite + TypeScript single page application. It renders a PDF
preview using `pdf.js` and overlays Fabric.js controllers for interactive
editing. The implementation currently provides placeholder canvases and stubs
for delta transforms, awaiting integration with the backend IR and patch API.

## Development

```bash
npm install
npm run dev
```

The dev server runs on <http://localhost:5173>. Configure the backend URL via
`VITE_API_BASE`.
