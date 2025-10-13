(() => {
  const fileInput = document.getElementById('file-input');
  const urlInput = document.getElementById('url-input');
  const rawHtml = document.getElementById('raw-html');
  const loadUrlButton = document.getElementById('load-url');
  const loadRawButton = document.getElementById('load-raw');
  const printButton = document.getElementById('print');
  const clearButton = document.getElementById('clear');
  const previewFrame = document.getElementById('preview-frame');

  let currentObjectUrl = null;

  const enablePrint = (enabled) => {
    printButton.disabled = !enabled;
  };

  const revokeCurrentUrl = () => {
    if (currentObjectUrl) {
      URL.revokeObjectURL(currentObjectUrl);
      currentObjectUrl = null;
    }
  };

  const loadHtmlContent = (htmlText) => {
    revokeCurrentUrl();
    const blob = new Blob([htmlText], { type: 'text/html' });
    currentObjectUrl = URL.createObjectURL(blob);
    previewFrame.src = currentObjectUrl;
    enablePrint(true);
  };

  const showError = (message) => {
    window.alert(message);
  };

  fileInput.addEventListener('change', () => {
    const [file] = fileInput.files || [];
    if (!file) {
      return;
    }

    if (!file.type && !/\.html?$/i.test(file.name)) {
      showError('Please select an HTML document.');
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      loadHtmlContent(reader.result);
    };
    reader.onerror = () => {
      showError('Unable to read the selected file.');
    };
    reader.readAsText(file);
  });

  loadUrlButton.addEventListener('click', async () => {
    const url = urlInput.value.trim();
    if (!url) {
      showError('Enter a valid URL.');
      return;
    }

    enablePrint(false);
    try {
      const response = await fetch(url, { mode: 'cors' });
      if (!response.ok) {
        throw new Error(`Request failed (${response.status})`);
      }
      const html = await response.text();
      loadHtmlContent(html);
    } catch (error) {
      showError(
        'Unable to load the requested URL. It may not allow cross-origin requests. ' +
          'Try downloading the HTML file and loading it locally instead.\n\nDetails: ' +
          error.message
      );
      enablePrint(false);
    }
  });

  loadRawButton.addEventListener('click', () => {
    const html = rawHtml.value.trim();
    if (!html) {
      showError('Paste HTML markup before loading.');
      return;
    }
    loadHtmlContent(html);
  });

  clearButton.addEventListener('click', () => {
    fileInput.value = '';
    urlInput.value = '';
    rawHtml.value = '';
    enablePrint(false);
    revokeCurrentUrl();
    previewFrame.removeAttribute('src');
  });

  printButton.addEventListener('click', () => {
    if (!previewFrame.contentWindow) {
      showError('Nothing to print yet. Load an HTML document first.');
      return;
    }
    previewFrame.contentWindow.focus();
    previewFrame.contentWindow.print();
  });
})();
