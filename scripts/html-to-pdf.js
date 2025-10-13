const fileInput = document.getElementById("html-file");
const baseUrlInput = document.getElementById("base-url");
const exportButton = document.getElementById("export");
const statusEl = document.getElementById("status");
const previewFrame = document.getElementById("preview-frame");
const previewEmpty = document.getElementById("preview-empty");
const pageFormatSelect = document.getElementById("page-format");
const orientationSelect = document.getElementById("orientation");
const printBackgroundCheckbox = document.getElementById("print-background");

let currentHtml = "";
let currentFileName = "document";

const PAGE_FORMAT_MAP = {
  a4: "a4",
  letter: "letter",
  legal: "legal",
  a3: "a3",
};

const dropTargets = [document.body];
dropTargets.forEach((target) => {
  target.addEventListener("dragover", (event) => {
    if (hasHtml(event)) {
      event.preventDefault();
      event.dataTransfer.dropEffect = "copy";
    }
  });

  target.addEventListener("drop", (event) => {
    if (hasHtml(event)) {
      event.preventDefault();
      const file = [...event.dataTransfer.files].find((item) => item.type === "text/html");
      if (file) {
        loadFile(file);
      }
    }
  });
});

fileInput.addEventListener("change", () => {
  const [file] = fileInput.files || [];
  if (!file) {
    return;
  }
  loadFile(file);
});

exportButton.addEventListener("click", async () => {
  if (!currentHtml) {
    return;
  }

  toggleBusy(true, "Preparing PDF…");

  try {
    const doc = previewFrame.contentDocument;
    if (!doc) {
      throw new Error("Preview document is not ready.");
    }

    const clone = doc.documentElement.cloneNode(true);

    const baseHref = baseUrlInput.value.trim();
    if (baseHref) {
      ensureBaseElement(clone, baseHref);
    }

    const options = {
      margin: 0,
      filename: `${currentFileName}.pdf`,
      pagebreak: { mode: ["css", "legacy"] },
      image: { type: "jpeg", quality: 0.98 },
      html2canvas: {
        useCORS: true,
        scale: 2,
        backgroundColor: printBackgroundCheckbox.checked ? undefined : "#ffffff",
        logging: false,
      },
      jsPDF: {
        unit: "in",
        format: PAGE_FORMAT_MAP[pageFormatSelect.value] ?? "letter",
        orientation: orientationSelect.value,
      },
    };

    if (!printBackgroundCheckbox.checked) {
      clone.querySelectorAll("*").forEach((el) => {
        el.style.background = el.style.backgroundColor = "";
      });
    }

    await html2pdf().set(options).from(clone).save();
    toggleBusy(false, "PDF ready! Saved using browser download dialog.");
  } catch (error) {
    console.error(error);
    toggleBusy(false, `Unable to export: ${error.message}`);
  }
});

function loadFile(file) {
  const reader = new FileReader();
  toggleBusy(true, `Loading “${file.name}”…`);

  reader.addEventListener("error", () => {
    toggleBusy(false, "Failed to read the file.");
  });

  reader.addEventListener("load", () => {
    currentHtml = String(reader.result);
    currentFileName = file.name.replace(/\.[^.]+$/, "") || "document";
    updatePreview(currentHtml);
    toggleBusy(false, "Document loaded. Adjust settings and export when ready.");
  });

  reader.readAsText(file);
}

function updatePreview(html) {
  previewEmpty.hidden = true;
  previewFrame.hidden = false;

  const baseHref = baseUrlInput.value.trim();
  const docHtml = injectBaseHref(html, baseHref);
  previewFrame.srcdoc = docHtml;

  exportButton.disabled = false;
}

function injectBaseHref(html, baseHref) {
  if (!baseHref) {
    return html;
  }

  if (/<base\s+/i.test(html)) {
    return html;
  }

  const headMatch = html.match(/<head[^>]*>/i);
  const baseTag = `<base href="${baseHref}">`;
  if (headMatch) {
    return html.replace(headMatch[0], `${headMatch[0]}\n    ${baseTag}`);
  }
  return `<head>\n    ${baseTag}\n  </head>\n${html}`;
}

function ensureBaseElement(root, href) {
  const head = root.querySelector("head");
  if (!head) {
    return;
  }

  let base = head.querySelector("base");
  if (!base) {
    base = root.ownerDocument.createElement("base");
    head.insertBefore(base, head.firstChild);
  }
  base.setAttribute("href", href);
}

function toggleBusy(isBusy, message) {
  exportButton.disabled = isBusy || !currentHtml;
  exportButton.textContent = isBusy ? "Working…" : "Export to PDF";
  statusEl.textContent = message;
}

function hasHtml(event) {
  return [...(event.dataTransfer?.types || [])].includes("text/html") ||
    [...(event.dataTransfer?.files || [])].some((file) => file.type === "text/html");
}
