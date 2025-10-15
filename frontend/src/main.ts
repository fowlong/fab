import { initialisePdfPreview } from "./pdfPreview";
import { initialiseFabricOverlay } from "./fabricOverlay";
import { createApiClient } from "./api";
import { attachGlobalStyles } from "./styles";

const root = document.getElementById("app");
if (!root) {
  throw new Error("#app container missing");
}

attachGlobalStyles();

const status = document.createElement("div");
status.className = "status-bar";
status.textContent = "No document loaded";
root.appendChild(status);

const canvasHost = document.createElement("div");
canvasHost.className = "canvas-host";
root.appendChild(canvasHost);

const previewCanvas = document.createElement("canvas");
previewCanvas.className = "pdf-preview";
canvasHost.appendChild(previewCanvas);

const overlayCanvas = document.createElement("canvas");
overlayCanvas.id = "fabric-overlay";
overlayCanvas.className = "fabric-overlay";
canvasHost.appendChild(overlayCanvas);

const api = createApiClient();

initialisePdfPreview({ canvas: previewCanvas, api, onLoaded: (doc) => {
  status.textContent = `Loaded document with ${doc.pages.length} page(s)`;
  initialiseFabricOverlay({ canvas: overlayCanvas, document: doc, api });
}}).catch((err) => {
  status.textContent = `Failed to load document: ${err instanceof Error ? err.message : String(err)}`;
  console.error(err);
});
