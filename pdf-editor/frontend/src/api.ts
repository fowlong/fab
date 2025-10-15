import type { DocumentIR, PatchOperation } from "./types";

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async openDocument(file: File): Promise<string> {
    const formData = new FormData();
    formData.append("file", file);

    const res = await fetch(new URL("/api/open", this.baseUrl), {
      method: "POST",
      body: formData
    });

    if (!res.ok) {
      throw new Error(`Failed to open document: ${res.status}`);
    }

    const data = (await res.json()) as { docId: string };
    return data.docId;
  }

  async fetchIR(docId: string): Promise<DocumentIR> {
    const res = await fetch(new URL(`/api/ir/${docId}`, this.baseUrl));
    if (!res.ok) throw new Error(`Failed to load IR: ${res.status}`);
    return (await res.json()) as DocumentIR;
  }

  resolvePdfUrl(docId: string): string {
    return new URL(`/api/pdf/${docId}`, this.baseUrl).toString();
  }

  async sendPatch(docId: string, ops: PatchOperation[]) {
    const res = await fetch(new URL(`/api/patch/${docId}`, this.baseUrl), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ops)
    });
    if (!res.ok) throw new Error(`Patch failed: ${res.status}`);
    return res.json();
  }
}
