import "./styles.css";
import { initialisePreview } from "./pdfPreview";
import { initialiseFabricOverlay } from "./fabricOverlay";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("#app container missing");
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <div class="controls">
        <button data-action="open">Open PDF</button>
        <button data-action="download">Download</button>
      </div>
      <div class="thumbnails" id="thumbnails"></div>
    </aside>
    <main class="editor" id="editor"></main>
  </div>
`;

const editor = document.getElementById("editor");
if (!editor) {
  throw new Error("Editor container missing");
}

initialisePreview(editor);
initialiseFabricOverlay(editor);
