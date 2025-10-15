import { fabric } from 'fabric';
import type { ApiClient } from './api';
import type { PdfPreviewContext } from './pdfPreview';
import { fabricDeltaToPdfDelta } from './coords';
import type { DocumentObjectIR, PatchOperation } from './types';
import { irObjectToFabric } from './mapping';

export interface FabricOverlayManagerOptions {
  preview: PdfPreviewContext;
  api: ApiClient;
  docId: string;
}

interface OverlayInstance {
  canvas: fabric.Canvas;
  objectsById: Map<string, fabric.Object>;
}

export function createFabricOverlayManager(options: FabricOverlayManagerOptions) {
  const overlays: OverlayInstance[] = options.preview.pages.map((page) => {
    const overlayCanvas = options.preview.container.querySelector<HTMLCanvasElement>(
      `.overlay-canvas[data-page-index="${page.index}"]`,
    );
    if (!overlayCanvas) {
      throw new Error(`Missing overlay canvas for page ${page.index}`);
    }
    const canvas = new fabric.Canvas(overlayCanvas, {
      selection: false,
      preserveObjectStacking: true,
    });
    canvas.setDimensions({ width: overlayCanvas.width, height: overlayCanvas.height });
    const objectsById = new Map<string, fabric.Object>();
    return { canvas, objectsById };
  });

  for (const page of options.preview.pages) {
    const overlay = overlays[page.index];
    for (const object of page.objects) {
      const fabricObject = irObjectToFabric(object, page);
      overlay.canvas.add(fabricObject);
      overlay.objectsById.set(object.id, fabricObject);
      attachTransformHandler({ overlay, object, pageHeight: page.heightPt, options });
    }
  }
}

function attachTransformHandler(params: {
  overlay: OverlayInstance;
  object: DocumentObjectIR;
  pageHeight: number;
  options: FabricOverlayManagerOptions;
}) {
  const { overlay, object, pageHeight, options } = params;
  const fabricObj = overlay.objectsById.get(object.id);
  if (!fabricObj) return;

  fabricObj.on('modified', async () => {
    const originalMatrix = (fabricObj as any)._fabricInitialMatrix ?? fabricObj.calcTransformMatrix();
    const currentMatrix = fabricObj.calcTransformMatrix();
    const delta = fabricDeltaToPdfDelta(originalMatrix, currentMatrix, pageHeight);

    const op: PatchOperation = {
      op: 'transform',
      kind: object.kind,
      target: { page: object.pageIndex, id: object.id },
      deltaMatrixPt: delta,
    };

    try {
      await options.api.sendPatch(options.docId, [op]);
    } catch (error) {
      console.error('Failed to apply transform', error);
    }
  });
}
