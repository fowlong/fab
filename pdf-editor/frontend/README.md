# Frontend

The frontend is a Vite + TypeScript application that renders PDF pages via
`pdf.js` and overlays editable controllers powered by `Fabric.js`.

## Development

```bash
npm install
npm run dev
```

Set the `VITE_API_BASE` environment variable to point at the backend server if
it is not running on `http://localhost:8787`.
