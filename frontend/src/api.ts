import type { DocumentIR, PatchOperation, PatchResponse } from "./types";

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async open(file: File): Promise<{ docId: string }> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(new URL("/api/open", this.baseUrl), {
      method: "POST",
      body: form
    });
    if (!res.ok) {
      throw new Error(`Failed to open PDF: ${res.status}`);
    }
    return res.json();
  }

  async fetchIr(docId: string): Promise<DocumentIR> {
    const res = await fetch(new URL(`/api/ir/${docId}`, this.baseUrl));
    if (!res.ok) {
      throw new Error(`Failed to fetch IR: ${res.status}`);
    }
    return res.json();
  }

  async patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
    const res = await fetch(new URL(`/api/patch/${docId}`, this.baseUrl), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ops)
    });
    if (!res.ok) {
      throw new Error(`Failed to apply patch: ${res.status}`);
    }
    return res.json();
  }

  async download(docId: string): Promise<Blob> {
    const res = await fetch(new URL(`/api/pdf/${docId}`, this.baseUrl));
    if (!res.ok) {
      throw new Error(`Failed to download PDF: ${res.status}`);
    }
    return res.blob();
  }
}
