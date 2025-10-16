import { afterEach, describe, expect, it } from 'vitest';
import { installDom, TestElement } from './dom-stub';
import { FabricOverlayManager } from '../src/fabricOverlay';

describe('FabricOverlayManager', () => {
  let restoreDom: (() => void) | undefined;

  afterEach(() => {
    restoreDom?.();
    restoreDom = undefined;
  });

  it('populates overlays for each page', () => {
    restoreDom = installDom();
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

    expect(canvases.length).toBe(1);
    expect(canvases[0].objects.length).toBe(1);
    expect(canvases[0].renderCount).toBe(1);
    expect(wrappers[0].children.length).toBe(1);
  });

  it('resets prior overlays when repopulating', () => {
    restoreDom = installDom();
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

    expect(firstCanvas.disposed).toBe(true);
    expect(canvases.length).toBe(2);
    expect(wrappers[0].children.length).toBe(1);
  });
});
