const fileInput = document.getElementById('fileInput');
const htmlTextarea = document.getElementById('htmlTextarea');
const loadMarkupButton = document.getElementById('loadMarkup');
const exportButton = document.getElementById('exportButton');
const statusMessage = document.getElementById('statusMessage');
const previewFrame = document.getElementById('previewFrame');
const documentName = document.getElementById('documentName');
const printStyleTemplate = document.getElementById('printStyleTemplate');

let currentDocument = null;

function setStatus(message, tone = 'info') {
  statusMessage.textContent = message;
  statusMessage.dataset.tone = tone;
}

function clearStatus() {
  statusMessage.textContent = '';
  delete statusMessage.dataset.tone;
}

function sanitizeFilename(name) {
  if (!name) return '';
  return name.replace(/\s+/g, ' ').trim();
}

function loadHtmlIntoPreview(html, name = 'Untitled document') {
  previewFrame.srcdoc = html;
  previewFrame.scrollTo(0, 0);
  documentName.textContent = sanitizeFilename(name);
  currentDocument = { html, name };
  exportButton.disabled = false;
  setStatus(`Loaded ${name}. Ready to export.`, 'success');
}

function readFileAsText(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener('load', () => resolve(reader.result));
    reader.addEventListener('error', () => reject(reader.error ?? new Error('Failed to read file.')));
    reader.readAsText(file);
  });
}

fileInput?.addEventListener('change', async (event) => {
  const file = event.target.files?.[0];
  if (!file) {
    clearStatus();
    return;
  }

  if (!file.type.includes('html') && !file.name.endsWith('.html')) {
    setStatus('Please select a valid HTML file.', 'error');
    return;
  }

  try {
    setStatus('Reading file…');
    const text = await readFileAsText(file);
    loadHtmlIntoPreview(text, file.name);
  } catch (error) {
    console.error(error);
    setStatus(`Could not load file: ${error.message}`, 'error');
  }
});

loadMarkupButton?.addEventListener('click', () => {
  const markup = htmlTextarea.value.trim();
  if (!markup) {
    setStatus('Paste HTML markup before loading it.', 'error');
    return;
  }

  loadHtmlIntoPreview(markup, 'Pasted markup');
});

function waitFor(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function exportCurrentDocument() {
  if (!currentDocument?.html) {
    setStatus('Load an HTML document before exporting.', 'error');
    return;
  }

  exportButton.disabled = true;
  setStatus('Preparing document for printing…');

  const printFrame = document.createElement('iframe');
  printFrame.setAttribute('sandbox', 'allow-modals allow-same-origin allow-scripts');
  printFrame.style.position = 'fixed';
  printFrame.style.width = '0';
  printFrame.style.height = '0';
  printFrame.style.border = '0';
  printFrame.style.opacity = '0';
  printFrame.title = 'Print preview frame';
  document.body.appendChild(printFrame);

  const cleanup = () => {
    setTimeout(() => {
      if (printFrame.parentNode) {
        printFrame.remove();
      }
    }, 1000);
    exportButton.disabled = false;
  };

  try {
    const loadPromise = new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error('Timed out while preparing the print preview.'));
      }, 8000);

      printFrame.addEventListener(
        'load',
        () => {
          clearTimeout(timer);
          resolve();
        },
        { once: true }
      );
    });

    printFrame.srcdoc = currentDocument.html;
    await loadPromise;

    if (printStyleTemplate?.content) {
      try {
        const clone = document.importNode(printStyleTemplate.content, true);
        printFrame.contentDocument.head.appendChild(clone);
      } catch (error) {
        console.warn('Could not apply print styles', error);
      }
    }

    await waitFor(200);

    setStatus('Opening print dialog…');

    printFrame.contentWindow.focus();
    printFrame.contentWindow.print();
  } catch (error) {
    console.error(error);
    setStatus(`Export failed: ${error.message}`, 'error');
    cleanup();
    return;
  }

  waitFor(1000).finally(() => {
    setStatus(`Print dialog opened for ${sanitizeFilename(currentDocument.name) || 'document'}.`, 'success');
    cleanup();
  });
}

exportButton?.addEventListener('click', exportCurrentDocument);

htmlTextarea?.addEventListener('input', () => {
  if (!htmlTextarea.value.trim()) {
    return;
  }
  clearStatus();
});
