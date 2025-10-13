const localFileInput = document.getElementById("localFile");
const loadLocalButton = document.getElementById("loadLocal");
const remoteUrlInput = document.getElementById("remoteUrl");
const loadRemoteButton = document.getElementById("loadRemote");
const exportButton = document.getElementById("export");
const previewFrame = document.getElementById("previewFrame");
function setPreview(html) {
  previewFrame.srcdoc = html;
  exportButton.disabled = false;
}

function showMessage(message) {
  exportButton.disabled = true;
  const doc = previewFrame.contentDocument;
  if (!doc) {
    previewFrame.srcdoc = `<p>${message}</p>`;
    return;
  }

  doc.open();
  doc.write(`<p>${message}</p>`);
  doc.close();
}

function showLoading(message = "Loading document…") {
  exportButton.disabled = true;
  previewFrame.srcdoc = `<div class="loading-state">${message}</div>`;
}

async function loadLocalFile() {
  const [file] = localFileInput.files;
  if (!file) {
    showMessage("Please choose a local HTML file first.");
    return;
  }

  try {
    showLoading("Reading local file…");
    const text = await file.text();
    setPreview(text);
  } catch (error) {
    console.error(error);
    showMessage("Unable to read the selected file. Please try again.");
  }
}

async function loadRemoteUrl() {
  const url = remoteUrlInput.value.trim();
  if (!url) {
    showMessage("Enter a valid URL to load.");
    return;
  }

  try {
    showLoading("Fetching remote document…");
    const response = await fetch(url, { credentials: "omit" });
    if (!response.ok) {
      throw new Error(`${response.status} ${response.statusText}`);
    }
    const html = await response.text();
    setPreview(html);
  } catch (error) {
    console.error(error);
    showMessage(`Failed to load the document. ${error.message}`);
  }
}

function exportToPdf() {
  if (!previewFrame.contentWindow) {
    showMessage("Load a document before exporting.");
    return;
  }

  try {
    previewFrame.contentWindow.focus();
    previewFrame.contentWindow.print();
  } catch (error) {
    console.error(error);
    showMessage("Export failed. Check the browser console for details.");
  }
}

loadLocalButton.addEventListener("click", loadLocalFile);
loadRemoteButton.addEventListener("click", loadRemoteUrl);
exportButton.addEventListener("click", exportToPdf);

window.addEventListener("message", (event) => {
  if (event.data === "print") {
    exportToPdf();
  }
});

showMessage("Load a document to begin.");
