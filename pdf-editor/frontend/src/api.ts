import type { IrDocument, OpenResponse, PatchOperation, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export async function openPdf(file: File): Promise<OpenResponse> {
  const arrayBuffer = await file.arrayBuffer();
  const base64 = bufferToBase64(arrayBuffer);
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name: file.name, pdfBase64: base64 })
  });
  if (!response.ok) {
    throw new Error('Failed to open PDF');
  }
  return response.json();
}

export async function fetchIr(docId: string): Promise<IrDocument> {
  const response = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!response.ok) {
    throw new Error('Failed to fetch IR');
  }
  return response.json();
}

export async function postPatch(
  docId: string,
  ops: PatchOperation[]
): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops)
  });
  if (!response.ok) {
    throw new Error('Failed to apply patch');
  }
  return response.json();
}

function bufferToBase64(buffer: ArrayBuffer): string {
  let binary = '';
  const bytes = new Uint8Array(buffer);
  const len = bytes.byteLength;
  for (let i = 0; i < len; i += 1) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}
