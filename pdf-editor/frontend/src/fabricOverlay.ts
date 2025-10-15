import { fabric } from 'fabric';
import type { DocumentIR, PageIR, PdfObject } from './types';
import { fabricDeltaToPdfDelta } from './coords';
import { sendPatch } from './api';
import { PagePreview } from './pdfPreview';

interface OverlayContext {
  docId: string;
  ir: DocumentIR;
  previews: PagePreview[];
}

interface OverlayMeta {
  id: string;
  page: number;
  baseMatrix: number[];
}

export function createOverlayController(ctx: OverlayContext) {
  ctx.previews.forEach((preview) => {
    const canvasEl = document.createElement('canvas');
    canvasEl.width = preview.canvas.width;
    canvasEl.height = preview.canvas.height;
    canvasEl.className = 'fabric-overlay';
    preview.canvas.insertAdjacentElement('afterend', canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      uniformScaling: true,
    });

    const page = ctx.ir.pages.find((p) => p.index === preview.pageIndex);
    if (!page) return;
    addPageObjects(canvas, page, ctx);
  });
}

function addPageObjects(canvas: fabric.Canvas, page: PageIR, ctx: OverlayContext) {
  page.objects.forEach((obj) => {
    const rect = new fabric.Rect({
      left: obj.bbox[0],
      top: page.heightPt - obj.bbox[3],
      width: obj.bbox[2] - obj.bbox[0],
      height: obj.bbox[3] - obj.bbox[1],
      fill: 'rgba(0,0,0,0)',
      stroke: obj.kind === 'text' ? '#4C6EF5' : '#F59F00',
      strokeWidth: 1,
      selectable: true,
      hasControls: true,
    });
    rect.set({
      data: {
        overlay: {
          id: obj.id,
          page: page.index,
          baseMatrix: [...rect.calcTransformMatrix()],
        } as OverlayMeta,
      },
    });

    rect.on('modified', () => handleModified(rect, obj, page, ctx));
    canvas.add(rect);
  });
}

function handleModified(target: fabric.Rect, obj: PdfObject, page: PageIR, ctx: OverlayContext) {
  const meta = target.data?.overlay as OverlayMeta | undefined;
  if (!meta) return;
  const current = target.calcTransformMatrix();
  const delta = fabricDeltaToPdfDelta(meta.baseMatrix, current, page.heightPt);
  void sendPatch(ctx.docId, [
    {
      op: 'transform',
      kind: obj.kind,
      target: { page: page.index, id: obj.id },
      deltaMatrixPt: delta,
    },
  ]);
  meta.baseMatrix = [...current];
}
