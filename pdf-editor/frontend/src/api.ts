import type { DocumentIr, PatchOperation, PatchResponse } from './types';

const jsonHeaders = { 'Content-Type': 'application/json' } as const;

export async function openDocument(name?: string): Promise<string> {
  const res = await fetch('/api/open', {
    method: 'POST',
    headers: jsonHeaders,
    body: JSON.stringify({ name }),
  });
  if (!res.ok) {
    throw new Error('Failed to open document');
  }
  const data = (await res.json()) as { doc_id: string };
  return data.doc_id;
}

export async function fetchIr(docId: string): Promise<DocumentIr> {
  const res = await fetch(`/api/ir/${docId}`);
  if (!res.ok) {
    throw new Error('Failed to load IR');
  }
  return (await res.json()) as DocumentIr;
}

export async function sendPatchOperations(
  docId: string,
  ops: PatchOperation[],
): Promise<PatchResponse> {
  const res = await fetch(`/api/patch/${docId}`, {
    method: 'POST',
    headers: jsonHeaders,
    body: JSON.stringify(ops),
  });
  if (!res.ok) {
    throw new Error('Failed to apply patch');
  }
  return (await res.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error('Failed to download PDF');
  }
  return await res.blob();
}
