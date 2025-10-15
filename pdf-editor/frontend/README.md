# Frontend

This folder contains the Vite-powered single page application. The UI renders a
pdf.js bitmap underlay with a Fabric.js interaction layer that communicates with
the Rust backend through a small API client.

## Available scripts

```bash
npm install      # install dependencies
npm run dev      # start the Vite dev server
npm run build    # produce a production build
npm run preview  # preview the production build
npm run typecheck
```

Environment variable `VITE_API_BASE` can be set to point at the backend API.
