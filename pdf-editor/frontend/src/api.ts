import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = ((): string => {
  const env = (window as typeof window & { FRONTEND_API_BASE?: string }).FRONTEND_API_BASE;
  return env ?? import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';
})();

export interface OpenResponse {
  docId: string;
}

async function request<T>(input: RequestInfo, init?: RequestInit): Promise<T> {
  const res = await fetch(input, init);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`HTTP ${res.status}: ${text}`);
  }
  return (await res.json()) as T;
}

export async function openDocument(file: File): Promise<OpenResponse> {
  const body = new FormData();
  body.append('file', file);
  return request<OpenResponse>(`${API_BASE}/api/open`, {
    method: 'POST',
    body,
  });
}

export async function loadIR(docId: string): Promise<DocumentIR> {
  return request<DocumentIR>(`${API_BASE}/api/ir/${docId}`);
}

export async function sendPatch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  return request<PatchResponse>(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops),
  });
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`);
  }
  return res.blob();
}
