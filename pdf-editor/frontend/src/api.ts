import type { DocumentIR, PatchOperation } from './types';

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async openDocument(file: File): Promise<{ docId: string }> {
    const formData = new FormData();
    formData.append('file', file);
    const response = await fetch(new URL('/api/open', this.baseUrl), {
      method: 'POST',
      body: formData,
    });
    if (!response.ok) {
      throw new Error(`Failed to open document: ${response.statusText}`);
    }
    return response.json();
  }

  async fetchIR(docId: string): Promise<DocumentIR> {
    const response = await fetch(new URL(`/api/ir/${docId}`, this.baseUrl));
    if (!response.ok) {
      throw new Error(`Failed to fetch IR: ${response.statusText}`);
    }
    return response.json();
  }

  async sendPatch(docId: string, ops: PatchOperation[]): Promise<void> {
    const response = await fetch(new URL(`/api/patch/${docId}`, this.baseUrl), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(ops),
    });
    if (!response.ok) {
      throw new Error(`Patch failed: ${response.statusText}`);
    }
  }

  async download(docId: string): Promise<Blob> {
    const response = await fetch(new URL(`/api/pdf/${docId}`, this.baseUrl));
    if (!response.ok) {
      throw new Error(`Failed to download PDF: ${response.statusText}`);
    }
    return response.blob();
  }
}
