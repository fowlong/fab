import type { PatchOperation, PatchResponse, PdfOpenResponse, PdfState } from "./types";

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async open(file: File): Promise<PdfOpenResponse> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(`${this.baseUrl}/api/open`, {
      method: "POST",
      body: form,
    });
    if (!res.ok) {
      throw new Error(`Failed to open PDF: ${res.status}`);
    }
    return res.json();
  }

  async fetchIR(docId: string): Promise<PdfState> {
    const res = await fetch(`${this.baseUrl}/api/ir/${docId}`);
    if (!res.ok) {
      throw new Error(`Failed to load IR: ${res.status}`);
    }
    return res.json();
  }

  async sendPatch(docId: string, operations: PatchOperation[]): Promise<PatchResponse> {
    const res = await fetch(`${this.baseUrl}/api/patch/${docId}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(operations),
    });
    if (!res.ok) {
      throw new Error(`Patch failed: ${res.status}`);
    }
    return res.json();
  }

  async download(docId: string): Promise<Blob> {
    const res = await fetch(`${this.baseUrl}/api/pdf/${docId}`);
    if (!res.ok) {
      throw new Error(`Download failed: ${res.status}`);
    }
    return res.blob();
  }
}
