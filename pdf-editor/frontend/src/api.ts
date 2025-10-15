import type { IRDocument, PatchOperation } from "./types";

export interface ApiClient {
  open(data: ArrayBuffer): Promise<string>;
  fetchIR(docId: string): Promise<IRDocument>;
  patch(docId: string, ops: PatchOperation[]): Promise<void>;
}

const DEFAULT_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";

export function createApiClient(baseUrl: string = DEFAULT_BASE): ApiClient {
  return {
    async open(data: ArrayBuffer) {
      const response = await fetch(`${baseUrl}/api/open`, {
        method: "POST",
        body: data,
      });
      if (!response.ok) {
        throw new Error("Failed to open PDF");
      }
      const payload = await response.json();
      return payload.docId as string;
    },

    async fetchIR(docId: string) {
      const response = await fetch(`${baseUrl}/api/ir/${docId}`);
      if (!response.ok) {
        throw new Error("Failed to load IR");
      }
      return (await response.json()) as IRDocument;
    },

    async patch(docId: string, ops: PatchOperation[]) {
      const response = await fetch(`${baseUrl}/api/patch/${docId}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(ops),
      });
      if (!response.ok) {
        throw new Error("Patch failed");
      }
      await response.json();
    },
  };
}
