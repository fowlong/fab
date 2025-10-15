import type { DocumentIR, OpenResponse, PatchOp, PatchResponse } from "./types";

const API_BASE = import.meta.env?.VITE_API_BASE ?? "http://localhost:8787";

export async function uploadPdf(file: File): Promise<OpenResponse> {
  const formData = new FormData();
  formData.append("file", file);
  const res = await fetch(`${API_BASE}/api/open`, {
    method: "POST",
    body: formData,
  });
  if (!res.ok) {
    throw new Error(`Failed to upload PDF: ${res.status}`);
  }
  return res.json();
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const res = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!res.ok) {
    throw new Error(`Failed to load IR: ${res.status}`);
  }
  return res.json();
}

export async function postPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const res = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(ops),
  });
  if (!res.ok) {
    throw new Error(`Failed to apply patch: ${res.status}`);
  }
  return res.json();
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error(`Failed to download PDF: ${res.status}`);
  }
  return res.blob();
}
