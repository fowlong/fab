import { afterEach, describe, expect, it, vi } from 'vitest';
import { downloadPdf, fetchIR, openDocument, postPatch } from '../src/api';

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  vi.restoreAllMocks();
});

describe('api helpers', () => {
  it('posts multipart data when opening documents', async () => {
    const file = new File(['test'], 'file.pdf', { type: 'application/pdf' });
    const json = vi.fn().mockResolvedValue({ docId: 'doc-0001' });
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 200, json } as any);
    globalThis.fetch = fetchMock as any;

    const result = await openDocument(file);

    expect(result).toEqual({ docId: 'doc-0001' });
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [, init] = fetchMock.mock.calls[0] ?? [];
    expect(init?.method).toBe('POST');
    expect(init?.body).toBeInstanceOf(FormData);
  });

  it('throws when fetching IR fails', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: false, status: 500 } as any);
    globalThis.fetch = fetchMock as any;

    await expect(fetchIR('missing')).rejects.toThrow(/500/);
  });

  it('serialises patch payloads as JSON', async () => {
    const ops = [
      { op: 'setStyle', target: { page: 0, id: 't:1' }, style: { fillColor: [1, 0, 0] } },
    ];
    const json = vi.fn().mockResolvedValue({ ok: true });
    const fetchMock = vi
      .fn()
      .mockResolvedValue({ ok: true, status: 200, json } as any);
    globalThis.fetch = fetchMock as any;

    const result = await postPatch('doc-1', ops as any);

    expect(result).toEqual({ ok: true });
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, init] = fetchMock.mock.calls[0] ?? [];
    expect(String(url)).toContain('/api/patch/doc-1');
    expect(init?.headers?.['Content-Type']).toBe('application/json');
    expect(init?.body).toBe(JSON.stringify(ops));
  });

  it('returns blobs when downloading PDFs', async () => {
    const blob = new Blob(['pdf'], { type: 'application/pdf' });
    const fetchMock = vi
      .fn()
      .mockResolvedValue({ ok: true, status: 200, blob: () => Promise.resolve(blob) } as any);
    globalThis.fetch = fetchMock as any;

    const result = await downloadPdf('doc-1');

    expect(result).toBe(blob);
  });
});
