import type { DocumentIR, PatchOperation, PatchResponse } from "./types";

interface OpenResponse {
  docId: string;
}

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl = import.meta.env.VITE_API_BASE ?? "http://localhost:8787") {
    this.baseUrl = baseUrl.replace(/\/$/, "");
  }

  async open(file: File): Promise<OpenResponse> {
    const formData = new FormData();
    formData.append("file", file);

    const res = await fetch(`${this.baseUrl}/api/open`, {
      method: "POST",
      body: formData,
    });

    if (!res.ok) {
      throw new Error(`Open failed: ${res.status}`);
    }

    return res.json();
  }

  async fetchIR(docId: string): Promise<DocumentIR> {
    const res = await fetch(`${this.baseUrl}/api/ir/${docId}`);
    if (!res.ok) {
      throw new Error(`IR fetch failed: ${res.status}`);
    }
    return res.json();
  }

  async patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
    const res = await fetch(`${this.baseUrl}/api/patch/${docId}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ops),
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
