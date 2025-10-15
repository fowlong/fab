import type { DocumentIR, PatchOp, PatchResponse } from './types';

declare const __API_BASE__: string;

type OpenResponse = {
  docId: string;
};

const API_BASE: string = typeof __API_BASE__ !== 'undefined' ? __API_BASE__ : 'http://localhost:8787';

export async function openDocument(file: File): Promise<{ docId: string; ir: DocumentIR }>
{
  const body = new FormData();
  body.append('file', file);

  const openResponse = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body
  });

  if (!openResponse.ok) {
    throw new Error('Failed to open document');
  }
  const openJson = (await openResponse.json()) as OpenResponse;

  const irResponse = await fetch(`${API_BASE}/api/ir/${openJson.docId}`);
  if (!irResponse.ok) {
    throw new Error('Failed to load IR');
  }
  const irJson = (await irResponse.json()) as DocumentIR;
  return { docId: openJson.docId, ir: irJson };
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const res = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!res.ok) {
    throw new Error('Failed to fetch IR');
  }
  return (await res.json()) as DocumentIR;
}

export async function applyPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const res = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(ops)
  });

  if (!res.ok) {
    throw new Error('Failed to apply patch');
  }

  return (await res.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error('Failed to download PDF');
  }
  return await res.blob();
}
