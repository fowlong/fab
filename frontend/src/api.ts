import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const DEFAULT_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export async function openDocument(file: File): Promise<{ docId: string }> {
  const formData = new FormData();
  formData.set('file', file);
  const response = await fetch(`${DEFAULT_BASE}/api/open`, {
    method: 'POST',
    body: formData,
  });
  if (!response.ok) {
    throw new Error(`open failed: ${response.status}`);
  }
  return response.json();
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${DEFAULT_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`IR fetch failed: ${response.status}`);
  }
  return response.json();
}

export async function postPatch(
  docId: string,
  ops: PatchOperation[],
): Promise<PatchResponse> {
  const response = await fetch(`${DEFAULT_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error(`patch failed: ${response.status}`);
  }
  return response.json();
}

export async function downloadPdf(docId: string): Promise<void> {
  const response = await fetch(`${DEFAULT_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`download failed: ${response.status}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${docId}.pdf`;
  link.click();
  URL.revokeObjectURL(url);
}

export async function fetchPdfBytes(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${DEFAULT_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`fetch pdf failed: ${response.status}`);
  }
  return response.arrayBuffer();
}
