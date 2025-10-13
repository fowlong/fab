const fileInput = document.querySelector('#htmlFileInput');
const dropZone = document.querySelector('#dropZone');
const previewFrame = document.querySelector('#previewFrame');
const statusMessage = document.querySelector('#statusMessage');
const printButton = document.querySelector('#printButton');
const clearButton = document.querySelector('#clearButton');

let loadedFile = null;
let previewURL = null;
let rawHtml = '';

function resetState() {
  loadedFile = null;
  rawHtml = '';
  if (previewURL) {
    URL.revokeObjectURL(previewURL);
    previewURL = null;
  }
  previewFrame.removeAttribute('src');
  previewFrame.srcdoc = '';
  statusMessage.textContent = 'No file loaded.';
  statusMessage.dataset.type = 'info';
  printButton.disabled = true;
  clearButton.disabled = true;
}

resetState();

function updateStatus(text, type = 'info') {
  statusMessage.textContent = text;
  statusMessage.dataset.type = type;
}

function isHtmlFile(file) {
  return /text\/html|\.html?$/.test(file.type) || /\.html?$/i.test(file.name);
}

async function loadFile(file) {
  if (!file) return;
  if (!isHtmlFile(file)) {
    updateStatus('Please choose a valid HTML file.', 'error');
    return;
  }

  loadedFile = file;
  updateStatus(`Loading “${file.name}”…`);
  try {
    rawHtml = await file.text();
    await renderPreview();
    clearButton.disabled = false;
    printButton.disabled = false;
    updateStatus(`Ready: ${file.name}`, 'success');
  } catch (error) {
    console.error(error);
    updateStatus('Unable to read the file. Please try again.', 'error');
  }
}

async function renderPreview() {
  if (!rawHtml) {
    previewFrame.srcdoc = '';
    previewFrame.removeAttribute('src');
    return;
  }

  if (previewURL) {
    URL.revokeObjectURL(previewURL);
    previewURL = null;
  }

  const blob = new Blob([rawHtml], { type: 'text/html' });
  previewURL = URL.createObjectURL(blob);
  previewFrame.src = previewURL;
}

async function printDocument() {
  if (!rawHtml) return;

  const printWindow = previewFrame.contentWindow;
  if (!printWindow) {
    updateStatus('Preview not ready yet. Please wait a moment.', 'error');
    return;
  }

  try {
    await waitForFrame(previewFrame);
    printWindow.focus();
    printWindow.print();
    updateStatus('Print dialog opened. Choose “Save as PDF” to export.', 'success');
  } catch (error) {
    console.error(error);
    updateStatus('Unable to open the print dialog. Try again.', 'error');
  }
}

function waitForFrame(frame) {
  return new Promise((resolve, reject) => {
    if (!frame) {
      reject(new Error('Frame not available'));
      return;
    }

    if (frame.contentDocument?.readyState === 'complete') {
      resolve();
      return;
    }

    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error('Timed out waiting for document to load'));
    }, 5000);

    function handleLoad() {
      cleanup();
      resolve();
    }

    function cleanup() {
      frame.removeEventListener('load', handleLoad);
      clearTimeout(timeout);
    }

    frame.addEventListener('load', handleLoad, { once: true });
  });
}

fileInput.addEventListener('change', (event) => {
  const [file] = event.target.files;
  loadFile(file);
});

printButton.addEventListener('click', () => {
  printDocument();
});

clearButton.addEventListener('click', () => {
  resetState();
  fileInput.value = '';
});

previewFrame.addEventListener('load', () => {
  if (!previewFrame.contentDocument) return;
  if (statusMessage.dataset.type !== 'success') {
    updateStatus(`Preview updated for ${loadedFile?.name ?? 'document'}.`, 'info');
  }
});

['dragenter', 'dragover'].forEach((eventName) => {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.add('dragover');
  });
});

['dragleave', 'drop'].forEach((eventName) => {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.remove('dragover');
  });
});

dropZone.addEventListener('drop', (event) => {
  const file = event.dataTransfer?.files?.[0];
  if (file) {
    loadFile(file);
  }
});

dropZone.addEventListener('click', () => {
  fileInput.click();
});

dropZone.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    fileInput.click();
  }
});

window.addEventListener('beforeunload', () => {
  if (previewURL) {
    URL.revokeObjectURL(previewURL);
  }
});
