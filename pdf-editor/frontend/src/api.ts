import type { DocumentIr, PatchOperation, PatchResponse } from './types';

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async open(file: File): Promise<string> {
    const formData = new FormData();
    formData.append('file', file);
    const response = await fetch(`${this.baseUrl}/api/open`, {
      method: 'POST',
      body: formData
    });
    if (!response.ok) {
      throw new Error('Failed to open PDF');
    }
    const payload: { docId: string } = await response.json();
    return payload.docId;
  }

  async fetchIr(docId: string): Promise<DocumentIr> {
    const response = await fetch(`${this.baseUrl}/api/ir/${encodeURIComponent(docId)}`);
    if (!response.ok) {
      throw new Error('Failed to fetch IR');
    }
    return (await response.json()) as DocumentIr;
  }

  async patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
    const response = await fetch(`${this.baseUrl}/api/patch/${encodeURIComponent(docId)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(ops)
    });
    if (!response.ok) {
      throw new Error('Patch failed');
    }
    return (await response.json()) as PatchResponse;
  }

  async download(docId: string): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/api/pdf/${encodeURIComponent(docId)}`);
    if (!response.ok) {
      throw new Error('Download failed');
    }
    return await response.blob();
  }
}
