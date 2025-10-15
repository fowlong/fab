import type { DocumentIR, PatchOp } from "./types";

const DEFAULT_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";

interface OpenResponse {
  docId: string;
  ir: DocumentIR;
  pdfDataUrl: string;
}

export class ApiClient {
  constructor(private readonly base = DEFAULT_BASE) {}

  async open(file: File): Promise<{ docId: string; ir: DocumentIR }> {
    const formData = new FormData();
    formData.append("file", file);

    const res = await fetch(`${this.base}/api/open`, {
      method: "POST",
      body: formData
    });
    if (!res.ok) {
      throw new Error(`Failed to open PDF: ${res.status}`);
    }
    const data = (await res.json()) as OpenResponse;
    return { docId: data.docId, ir: { ...data.ir, pdfDataUrl: data.pdfDataUrl } };
  }

  async patch(docId: string, ops: PatchOp[]): Promise<void> {
    const res = await fetch(`${this.base}/api/patch/${docId}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ops)
    });
    if (!res.ok) {
      throw new Error(`Failed to apply patch: ${res.status}`);
    }
  }

  async download(docId: string): Promise<Blob> {
    const res = await fetch(`${this.base}/api/pdf/${docId}`);
    if (!res.ok) {
      throw new Error(`Failed to download PDF: ${res.status}`);
    }
    return await res.blob();
  }
}
