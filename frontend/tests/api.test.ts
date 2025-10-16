// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { downloadPdf, fetchIR, openDocument, postPatch } from '../src/api';

test('openDocument posts multipart data', async (t) => {
  const file = new File(['test'], 'file.pdf', { type: 'application/pdf' });
  const calls: any[] = [];
  const response = { ok: true, status: 200, json: async () => ({ docId: 'doc-0001' }) };

  const restore = stubFetch((input, init) => {
    calls.push({ input, init });
    return Promise.resolve(response as any);
  });
  t.after(() => restore());

  const result = await openDocument(file);
  assert.deepEqual(result, { docId: 'doc-0001' });
  assert.equal(calls[0]?.init?.method, 'POST');
  assert.ok(calls[0]?.init?.body instanceof FormData);
});

test('fetchIR throws on non-ok responses', async (t) => {
  const restore = stubFetch(() => Promise.resolve({ ok: false, status: 500 } as any));
  t.after(() => restore());
  await assert.rejects(() => fetchIR('missing'), /500/);
});

test('postPatch serialises payloads as JSON', async (t) => {
  const ops = [{ op: 'setStyle', target: { page: 0, id: 't:1' }, style: { fillColor: [1, 0, 0] } }];
  const calls: any[] = [];
  const restore = stubFetch((input, init) => {
    calls.push({ input, init });
    return Promise.resolve({ ok: true, status: 200, json: async () => ({ ok: true }) } as any);
  });
  t.after(() => restore());

  const result = await postPatch('doc-1', ops as any);
  assert.deepEqual(result, { ok: true });
  assert.equal(calls[0]?.init?.headers?.['Content-Type'], 'application/json');
  assert.equal(calls[0]?.init?.body, JSON.stringify(ops));
});

test('downloadPdf returns the response blob', async (t) => {
  const blob = new Blob(['pdf'], { type: 'application/pdf' });
  const restore = stubFetch(() => Promise.resolve({ ok: true, status: 200, blob: async () => blob } as any));
  t.after(() => restore());

  const result = await downloadPdf('doc-1');
  assert.equal(result, blob);
});

function stubFetch(fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>) {
  const original = globalThis.fetch;
  globalThis.fetch = fn as any;
  return () => {
    globalThis.fetch = original;
  };
}
