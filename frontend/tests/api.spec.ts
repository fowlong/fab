import { afterEach, describe, expect, it, vi } from 'vitest';
import { downloadPdf, fetchIR, openDocument, postPatch } from '../src/api';

describe('api client', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('openDocument posts multipart data', async () => {
    const file = new File(['test'], 'file.pdf', { type: 'application/pdf' });
    const response = { ok: true, status: 200, json: async () => ({ docId: 'doc-0001' }) };
    const fetchMock = vi.fn().mockResolvedValue(response as any);
    vi.stubGlobal('fetch', fetchMock);

    const result = await openDocument(file);
    expect(result).toEqual({ docId: 'doc-0001' });
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [, init] = fetchMock.mock.calls[0] ?? [];
    expect(init?.method).toBe('POST');
    expect(init?.body).toBeInstanceOf(FormData);
  });

  it('fetchIR throws on non-ok responses', async () => {
    const response = { ok: false, status: 500 };
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response as any));

    await expect(fetchIR('missing')).rejects.toThrow(/500/);
  });

  it('postPatch serialises payloads as JSON', async () => {
    const ops = [{ op: 'setStyle', target: { page: 0, id: 't:1' }, style: { fillColor: [1, 0, 0] } }];
    const fetchMock = vi
      .fn()
      .mockResolvedValue({ ok: true, status: 200, json: async () => ({ ok: true }) } as any);
    vi.stubGlobal('fetch', fetchMock);

    const result = await postPatch('doc-1', ops as any);
    expect(result).toEqual({ ok: true });
    const [, init] = fetchMock.mock.calls[0] ?? [];
    expect(init?.headers?.['Content-Type']).toBe('application/json');
    expect(init?.body).toBe(JSON.stringify(ops));
  });

  it('downloadPdf returns the response blob', async () => {
    const blob = new Blob(['pdf'], { type: 'application/pdf' });
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, status: 200, blob: async () => blob } as any));

    const result = await downloadPdf('doc-1');
    expect(result).toBe(blob);
  });
});
