import type { ApiClient, IrDocument } from "./types";

interface OpenResponse {
  docId: string;
}

interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
}

function baseUrl() {
  return import.meta.env.VITE_API_BASE ?? "http://localhost:8787";
}

export function createApiClient(): ApiClient {
  return {
    async openDocument(file: File) {
      const formData = new FormData();
      formData.append("file", file);
      const res = await fetch(`${baseUrl()}/api/open`, {
        method: "POST",
        body: formData,
      });
      if (!res.ok) {
        throw new Error("Failed to open PDF");
      }
      const json = (await res.json()) as OpenResponse;
      return json.docId;
    },
    async loadIr(docId: string) {
      const res = await fetch(`${baseUrl()}/api/ir/${docId}`);
      if (!res.ok) {
        throw new Error("Failed to load IR");
      }
      return (await res.json()) as IrDocument;
    },
    async patch(docId: string, ops: unknown[]) {
      const res = await fetch(`${baseUrl()}/api/patch/${docId}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(ops),
      });
      if (!res.ok) {
        throw new Error("Patch failed");
      }
      const json = (await res.json()) as PatchResponse;
      if (!json.ok) {
        throw new Error("Patch response reported failure");
      }
    },
    async download(docId: string) {
      const res = await fetch(`${baseUrl()}/api/pdf/${docId}`);
      if (!res.ok) {
        throw new Error("Failed to download PDF");
      }
      return await res.blob();
    },
  };
}
