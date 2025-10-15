import type { DocumentIR, PatchOp, PatchResponse } from "./types";

type OpenResponse = {
  docId: string;
  ir: DocumentIR;
};

const API_BASE: string = __API_BASE__;

async function jsonFetch<T>(url: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(url, init);
  if (!res.ok) {
    throw new Error(`Request failed: ${res.status} ${res.statusText}`);
  }
  return (await res.json()) as T;
}

export async function loadInitialDocument(file: File): Promise<OpenResponse> {
  const form = new FormData();
  form.append("file", file);
  return await jsonFetch<OpenResponse>(`${API_BASE}/api/open`, {
    method: "POST",
    body: form
  });
}

export async function postPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  return await jsonFetch<PatchResponse>(`${API_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(ops)
  });
}

export function registerDownloadHandler(button: HTMLButtonElement, docId: string) {
  button.onclick = () => {
    const link = document.createElement("a");
    link.href = `${API_BASE}/api/pdf/${encodeURIComponent(docId)}`;
    link.download = "edited.pdf";
    link.click();
  };
}
