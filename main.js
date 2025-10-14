const fileInput = document.getElementById("file-input");
const urlInput = document.getElementById("url-input");
const htmlInput = document.getElementById("html-input");
const loadForm = document.getElementById("load-form");
const fetchButton = document.getElementById("fetch-button");
const exportButton = document.getElementById("export-button");
const statusEl = document.getElementById("status");
const previewFrame = document.getElementById("preview-frame");

let currentSourceDescription = "";

function setStatus(message, type = "info") {
  statusEl.textContent = message;
  statusEl.dataset.type = type;
}

function enableExport(enabled) {
  exportButton.disabled = !enabled;
}

async function readFileAsText(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error);
    reader.onload = () => resolve(reader.result);
    reader.readAsText(file);
  });
}

function injectIntoIframe(html) {
  const iframeDoc = previewFrame.contentDocument;
  iframeDoc.open();
  iframeDoc.write(html);
  iframeDoc.close();
}

function loadHtml(html, description) {
  injectIntoIframe(html);
  currentSourceDescription = description;
  previewFrame.dataset.loaded = "true";
  enableExport(true);
  setStatus(`Loaded ${description}. Ready to export.`, "success");
}

loadForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  enableExport(false);
  setStatus("Loading…");

  try {
    if (fileInput.files?.length) {
      const file = fileInput.files[0];
      const text = await readFileAsText(file);
      loadHtml(text, `file \"${file.name}\"`);
      return;
    }

    if (htmlInput.value.trim()) {
      loadHtml(htmlInput.value, "pasted HTML");
      return;
    }

    if (urlInput.value.trim()) {
      const url = urlInput.value.trim();
      const response = await fetch(url, { mode: "cors" });
      if (!response.ok) {
        throw new Error(`Unable to fetch HTML (HTTP ${response.status})`);
      }
      const text = await response.text();
      loadHtml(text, `URL ${url}`);
      return;
    }

    setStatus("Please select a file, paste HTML, or provide a URL to load.", "error");
  } catch (error) {
    console.error(error);
    setStatus(`Error loading HTML: ${error.message}`, "error");
  }
});

fetchButton.addEventListener("click", async () => {
  if (!urlInput.value.trim()) {
    setStatus("Enter a URL to fetch.", "error");
    return;
  }

  enableExport(false);
  setStatus("Fetching HTML…");

  try {
    const url = urlInput.value.trim();
    const response = await fetch(url, { mode: "cors" });
    if (!response.ok) {
      throw new Error(`Unable to fetch HTML (HTTP ${response.status})`);
    }
    const text = await response.text();
    htmlInput.value = text;
    loadHtml(text, `URL ${url}`);
  } catch (error) {
    console.error(error);
    setStatus(`Error fetching URL: ${error.message}`, "error");
  }
});

exportButton.addEventListener("click", () => {
  if (previewFrame.dataset.loaded !== "true") {
    setStatus("Load HTML into the preview before exporting.", "error");
    return;
  }

  setStatus(`Opening print dialog for ${currentSourceDescription}…`);

  // Ensure styles have applied before printing.
  previewFrame.contentWindow?.focus();
  previewFrame.contentWindow?.print();
});

previewFrame.addEventListener("load", () => {
  // On load, ensure the iframe size matches the content width for better previewing.
  const iframeDocument = previewFrame.contentDocument;
  if (!iframeDocument) {
    return;
  }

  const htmlElement = iframeDocument.documentElement;
  htmlElement.style.overflow = "auto";
});

setStatus("Choose an input method to begin.");
