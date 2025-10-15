import type { DocumentIR, PatchOperation } from './types';

type OpenResponse = {
  docId: string;
  ir: DocumentIR;
};

export async function loadInitialDocument(file: File): Promise<OpenResponse> {
  const formData = new FormData();
  formData.append('file', file);
  const response = await fetch('/api/open', {
    method: 'POST',
    body: formData
  });
  if (!response.ok) {
    throw new Error('Failed to open PDF');
  }
  return response.json();
}

export async function postPatch(docId: string, ops: PatchOperation[]) {
  const response = await fetch(`/api/patch/${docId}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(ops)
  });
  if (!response.ok) {
    throw new Error('Failed to post patch');
  }
  return response.json();
}
