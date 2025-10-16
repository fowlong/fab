// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { installDom, TestElement } from './dom-stub';
import { FabricOverlayManager } from '../src/fabricOverlay';

test('FabricOverlayManager populates overlays for each page', (t) => {
  const restoreDom = installDom();
  t.after(() => restoreDom());

  const wrappers = [new TestElement('div')];
  const canvases = (globalThis as any).__fabricCreatedCanvases as any[];
  canvases.length = 0;
  const manager = new FabricOverlayManager();
  manager.populate(
    {
      pages: [
        {
          index: 0,
          widthPt: 595.276,
          heightPt: 841.89,
          objects: [
            {
              id: 't:42',
              kind: 'text',
              pdfRef: { obj: 1, gen: 0 },
              btSpan: { start: 0, end: 1, streamObj: 1 },
              Tm: [1, 0, 0, 1, 0, 0],
              font: { resName: 'F1', size: 12, type: 'Type0' },
              unicode: 'Hello',
              glyphs: [],
              bbox: [0, 0, 100, 50],
            },
          ],
        },
      ],
    },
    wrappers as unknown as HTMLElement[],
    [{ width: 400, height: 500 }],
  );
  assert.equal(canvases.length, 1);
  assert.equal(canvases[0].objects.length, 1);
  assert.equal(canvases[0].renderCount, 1);
  assert.equal(wrappers[0].children.length, 1);
});

test('FabricOverlayManager resets prior overlays when repopulating', (t) => {
  const restoreDom = installDom();
  t.after(() => restoreDom());

  const wrappers = [new TestElement('div')];
  const canvases = (globalThis as any).__fabricCreatedCanvases as any[];
  canvases.length = 0;
  const manager = new FabricOverlayManager();

  manager.populate(
    {
      pages: [
        {
          index: 0,
          widthPt: 595.276,
          heightPt: 841.89,
          objects: [],
        },
      ],
    },
    wrappers as unknown as HTMLElement[],
    [{ width: 400, height: 500 }],
  );

  const firstCanvas = canvases[0];

  manager.populate(
    {
      pages: [
        {
          index: 0,
          widthPt: 595.276,
          heightPt: 841.89,
          objects: [],
        },
      ],
    },
    wrappers as unknown as HTMLElement[],
    [{ width: 400, height: 500 }],
  );

  assert.equal(firstCanvas.disposed, true);
  assert.equal(canvases.length, 2);
  assert.equal(wrappers[0].children.length, 1);
});
