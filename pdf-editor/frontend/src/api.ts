import type { IrDocument, PatchOp, PatchResponse } from "./types";

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";

export async function openDocument(file: File): Promise<{ docId: string }> {
  const form = new FormData();
  form.append("file", file);
  const res = await fetch(`${API_BASE}/api/open`, {
    method: "POST",
    body: form
  });
  if (!res.ok) {
    throw new Error(`Failed to open PDF (${res.status})`);
  }
  return res.json();
}

export async function fetchIr(docId: string): Promise<IrDocument> {
  const res = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch IR (${res.status})`);
  }
  return res.json();
}

export async function sendPatch(
  docId: string,
  ops: PatchOp[]
): Promise<PatchResponse> {
  const res = await fetch(`${API_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(ops)
  });
  if (!res.ok) {
    throw new Error(`Failed to patch PDF (${res.status})`);
  }
  return res.json();
}

export async function download(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!res.ok) {
    throw new Error(`Failed to download PDF (${res.status})`);
  }
  return res.blob();
}
