# HTML to PDF Exporter UI

This project provides a simple browser-based interface for loading an HTML document and exporting it to PDF using the browser's built-in print-to-PDF functionality.

## Features

- Load HTML from a local file, remote URL, or pasted source code.
- Preview the rendered HTML inside the page.
- Trigger the browser print dialog to export the preview to a PDF file.

## Usage

1. Open `index.html` in your browser.
2. Load HTML using one of the available input methods.
3. Once the preview is ready, click **Export to PDF** to open the print dialog.
4. Choose **Save as PDF** (or the equivalent option in your browser) to export the document.

> ⚠️ Note: When loading HTML from external URLs, the remote server must permit cross-origin requests. Otherwise the browser will block the fetch for security reasons.
