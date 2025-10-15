import type { DocumentIR, PatchOperation, PatchResponse } from "./types";

export interface ApiClient {
  openLatest(): Promise<DocumentIR>;
  applyPatch(ops: PatchOperation[]): Promise<PatchResponse>;
}

export function createApiClient(base = import.meta.env.VITE_API_BASE ?? "http://localhost:8787"): ApiClient {
  const docIdKey = "pdf-editor:docId";

  async function requestDocId(): Promise<string> {
    const existing = sessionStorage.getItem(docIdKey);
    if (existing) {
      return existing;
    }
    const res = await fetch(`${base}/api/open`, { method: "POST" });
    if (!res.ok) {
      throw new Error(`Failed to open document: ${res.status}`);
    }
    const data = await res.json();
    if (!data.docId) {
      throw new Error("Backend did not return docId");
    }
    sessionStorage.setItem(docIdKey, data.docId);
    return data.docId;
  }

  return {
    async openLatest(): Promise<DocumentIR> {
      const docId = await requestDocId();
      const res = await fetch(`${base}/api/ir/${docId}`);
      if (!res.ok) {
        throw new Error(`Failed to fetch IR: ${res.status}`);
      }
      return (await res.json()) as DocumentIR;
    },
    async applyPatch(ops: PatchOperation[]): Promise<PatchResponse> {
      const docId = await requestDocId();
      const res = await fetch(`${base}/api/patch/${docId}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(ops),
      });
      if (!res.ok) {
        throw new Error(`Failed to apply patch: ${res.status}`);
      }
      return (await res.json()) as PatchResponse;
    },
  };
}
