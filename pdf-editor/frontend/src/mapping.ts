import { fabric } from 'fabric';
import type { DocumentObjectIR, PageIR } from './types';
import { ptToPxMatrix } from './coords';

export function irObjectToFabric(object: DocumentObjectIR, page: PageIR): fabric.Object {
  const matrix = ptToPxMatrix(page.heightPt);
  const bboxWidth = object.bbox[2] - object.bbox[0];
  const bboxHeight = object.bbox[3] - object.bbox[1];

  const rect = new fabric.Rect({
    left: object.bbox[0],
    top: page.heightPt - object.bbox[3],
    width: bboxWidth,
    height: bboxHeight,
    fill: 'rgba(0,0,0,0)',
    stroke: '#1d4ed8',
    strokeWidth: 1,
    selectable: true,
    hasControls: true,
    objectCaching: false,
  });

  (rect as any)._fabricInitialMatrix = matrix as unknown as number[];
  return rect;
}
