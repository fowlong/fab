const fileInput = document.getElementById('file-input');
const baseUrlInput = document.getElementById('base-url');
const stylesheetInput = document.getElementById('stylesheet');
const previewButton = document.getElementById('preview-button');
const printButton = document.getElementById('print-button');
const statusEl = document.getElementById('status');
const previewPlaceholder = document.getElementById('preview-placeholder');
const previewFrame = document.getElementById('preview-frame');

let selectedFile = null;
let lastPreviewHtml = '';

function escapeHtml(value) {
  return value.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function escapeStyle(css) {
  return css.replace(/<\/(style)/gi, '<\\/$1');
}

function buildPreviewHtml(content, baseHref, extraCss) {
  let additions = '';
  if (baseHref) {
    additions += `<base href="${escapeHtml(baseHref)}">`;
  }
  if (extraCss.trim()) {
    additions += `<style>${escapeStyle(extraCss)}</style>`;
  }

  if (!additions) {
    return content;
  }

  if (/<head[^>]*>/i.test(content)) {
    return content.replace(/<head[^>]*>/i, (match) => `${match}${additions}`);
  }

  // If there's no explicit head element, inject one at the top of the document.
  const hasHtml = /<html[^>]*>/i.test(content);
  if (hasHtml) {
    return content.replace(/<html[^>]*>/i, (match) => `${match}<head>${additions}</head>`);
  }

  return `<!DOCTYPE html><html><head>${additions}</head><body>${content}</body></html>`;
}

async function readFileAsText(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error || new Error('Failed to read file'));
    reader.readAsText(file);
  });
}

function setStatus(message, tone = 'info') {
  statusEl.textContent = message;
  statusEl.dataset.tone = tone;
}

function setPreviewVisibility(visible) {
  previewFrame.hidden = !visible;
  previewPlaceholder.hidden = visible;
}

async function updatePreview() {
  if (!selectedFile) {
    return;
  }

  try {
    setStatus('Loading document…');
    printButton.disabled = true;
    const rawHtml = await readFileAsText(selectedFile);
    const baseHref = baseUrlInput.value.trim();
    const extraCss = stylesheetInput.value;
    lastPreviewHtml = buildPreviewHtml(rawHtml, baseHref, extraCss);
    previewFrame.srcdoc = lastPreviewHtml;
  } catch (error) {
    console.error(error);
    setStatus(`Unable to load file: ${error.message}`, 'error');
  }
}

function enableControls(enabled) {
  previewButton.disabled = !enabled;
  printButton.disabled = !enabled;
}

fileInput.addEventListener('change', () => {
  const [file] = fileInput.files;
  selectedFile = file || null;
  lastPreviewHtml = '';
  setPreviewVisibility(false);
  printButton.disabled = true;
  previewButton.disabled = !selectedFile;
  if (!selectedFile) {
    setStatus('Waiting for a file…');
  } else {
    setStatus(`Selected ${selectedFile.name}. Click “Load preview”.`);
  }
});

previewButton.addEventListener('click', async () => {
  if (!selectedFile) {
    return;
  }
  await updatePreview();
});

previewFrame.addEventListener('load', () => {
  if (!lastPreviewHtml) {
    return;
  }
  setPreviewVisibility(true);
  enableControls(true);
  setStatus('Preview ready. Use “Export to PDF” to open the print dialog.', 'success');
});

printButton.addEventListener('click', () => {
  if (!lastPreviewHtml || !previewFrame.contentWindow) {
    return;
  }
  previewFrame.contentWindow.focus();
  previewFrame.contentWindow.print();
});

// Support drag-and-drop for convenience.
document.addEventListener('dragover', (event) => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy';
  }
});

document.addEventListener('drop', (event) => {
  event.preventDefault();
  const items = event.dataTransfer?.files;
  if (!items || !items.length) {
    return;
  }
  fileInput.files = items;
  fileInput.dispatchEvent(new Event('change'));
});

setPreviewVisibility(false);
