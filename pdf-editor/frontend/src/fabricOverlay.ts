import { fabric } from 'fabric';
import type { IrObject, PageIR, PatchOp } from './types';
import { fabricDeltaToPdfDelta, type Matrix } from './coords';
import { mapPageObjects } from './mapping';
import { sendPatch, downloadPdf } from './api';

interface OverlayOptions {
  pages: PageIR[];
  docId: string;
  preview: { canvases: HTMLCanvasElement[] };
}

interface OverlayHandle {
  downloadLatest(): Promise<Blob | null>;
}

interface OverlayObjectMeta {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
  object: IrObject;
}

export function initFabricOverlay({ pages, docId, preview }: OverlayOptions): OverlayHandle {
  const overlays: fabric.Canvas[] = [];
  const mappings = mapPageObjects(pages);
  const container = document.querySelector<HTMLDivElement>('#page-stack');
  if (!container) {
    throw new Error('Missing page stack container');
  }

  const objectMeta = new Map<string, OverlayObjectMeta>();

  for (const [index, canvasEl] of preview.canvases.entries()) {
    const overlayCanvas = document.createElement('canvas');
    overlayCanvas.width = canvasEl.width;
    overlayCanvas.height = canvasEl.height;
    overlayCanvas.className = 'fabric-overlay';
    overlayCanvas.style.position = 'absolute';
    overlayCanvas.style.left = '0';
    overlayCanvas.style.top = '0';
    overlayCanvas.style.pointerEvents = 'auto';
    canvasEl.parentElement?.appendChild(overlayCanvas);

    const overlay = new fabric.Canvas(overlayCanvas, {
      selection: false,
      skipTargetFind: false,
    });
    overlay.setBackgroundColor('rgba(0,0,0,0)', () => {});
    overlay.selection = false;
    overlays.push(overlay);

    for (const mapping of mappings.filter((m) => m.pageIndex === index)) {
      const rect = new fabric.Rect({
        left: mapping.bboxPx[0],
        top: mapping.bboxPx[1],
        width: mapping.bboxPx[2] - mapping.bboxPx[0],
        height: mapping.bboxPx[3] - mapping.bboxPx[1],
        stroke: '#2b6cb0',
        strokeWidth: 1,
        fill: 'rgba(0,0,0,0)',
        selectable: true,
        hasBorders: true,
        hasControls: true,
        objectCaching: false,
        transparentCorners: false,
        lockScalingFlip: true,
      });
      rect.set('data', { id: mapping.id });
      rect.setControlsVisibility({ mtr: true });
      rect.on('modified', async () => {
        await handleModified(rect, docId, pages[index]);
      });
      overlay.add(rect);
      objectMeta.set(mapping.id, {
        id: mapping.id,
        pageIndex: index,
        baseMatrix: mapping.initialMatrixPx,
        object: mapping.object,
      });
    }
  }

  async function handleModified(obj: fabric.Object, docIdParam: string, page: PageIR) {
    const data = obj.get('data') as { id?: string } | undefined;
    if (!data?.id) {
      return;
    }
    const meta = objectMeta.get(data.id);
    if (!meta) {
      return;
    }
    const raw = obj.calcTransformMatrix();
    const transform: Matrix = [raw[0], raw[1], raw[2], raw[3], raw[4], raw[5]];
    const delta = fabricDeltaToPdfDelta(meta.baseMatrix, transform, page.heightPt);

    const ops: PatchOp[] = [
      {
        op: 'transform',
        target: { page: meta.pageIndex, id: meta.id },
        deltaMatrixPt: delta,
        kind: meta.object.kind,
      },
    ];

    try {
      await sendPatch(docIdParam, ops);
    } catch (error) {
      console.error('Failed to send patch', error);
    }
  }

  return {
    async downloadLatest() {
      try {
        return await downloadPdf(docId);
      } catch (error) {
        console.error('Failed to download PDF', error);
        return null;
      }
    },
  };
}
