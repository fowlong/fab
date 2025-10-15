import "./styles.css";
import { initPdfPreview } from "./pdfPreview";
import { initFabricOverlay } from "./fabricOverlay";
import { createApiClient } from "./api";
import type { EditorState } from "./types";

const app = document.getElementById("app");
if (!app) {
  throw new Error("App root not found");
}

app.innerHTML = `
  <main class="layout">
    <section class="sidebar">
      <input type="file" id="file-input" accept="application/pdf" />
      <div id="thumbnail-list"></div>
    </section>
    <section class="editor">
      <div id="page-container"></div>
    </section>
  </main>
`;

const fileInput = document.getElementById("file-input") as HTMLInputElement;
const pageContainer = document.getElementById("page-container");

if (!fileInput || !pageContainer) {
  throw new Error("Missing editor elements");
}

const api = createApiClient();
const state: EditorState = {
  docId: null,
  pages: [],
  fabricOverlays: new Map(),
};

fileInput.addEventListener("change", async (event) => {
  const files = (event.target as HTMLInputElement).files;
  if (!files || files.length === 0) {
    return;
  }

  const file = files[0];
  const buffer = await file.arrayBuffer();
  const docId = await api.open(buffer);
  state.docId = docId;

  const ir = await api.fetchIR(docId);
  state.pages = ir.pages;

  const preview = await initPdfPreview(pageContainer, ir.pages);
  initFabricOverlay(state, preview, api);
});
