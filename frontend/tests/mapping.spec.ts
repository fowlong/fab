import { describe, expect, it } from 'vitest';
import { toFabricMatrix, bboxPtToPx, createFabricPlaceholder } from '../src/mapping';
import { ptToPxMatrix, multiply, SCALE, type Matrix } from '../src/coords';
import { Canvas } from 'fabric';

const samplePage = {
  index: 0,
  widthPt: 595.276,
  heightPt: 841.89,
  objects: [],
};

const sampleObject = {
  id: 't:99',
  kind: 'text',
  bbox: [72, 700, 144, 760],
};

describe('mapping helpers', () => {
  it('toFabricMatrix converts point transforms to pixel space', () => {
    const transform: Matrix = [1, 0, 0, 1, 36, -18];
    const ptToPx = ptToPxMatrix(samplePage.heightPt);
    const expected = multiply(ptToPx, transform);
    const result = toFabricMatrix(samplePage as any, transform);
    expect(result).toEqual(expected);
  });

  it('bboxPtToPx converts point bounding boxes into CSS pixels', () => {
    const bbox = [72, 648, 144, 720];
    const [left, top, width, height] = bboxPtToPx(samplePage as any, bbox as any);
    expect(left).toBeCloseTo(72 / SCALE, 9);
    expect(top).toBeCloseTo((samplePage.heightPt - 720) / SCALE, 9);
    expect(width).toBeCloseTo((144 - 72) / SCALE, 9);
    expect(height).toBeCloseTo((720 - 648) / SCALE, 9);
  });

  it('createFabricPlaceholder returns a styled fabric rect with metadata', () => {
    const canvas = new Canvas({ appendChild() {} } as any, {});
    const placeholder = createFabricPlaceholder(
      canvas as any,
      samplePage as any,
      sampleObject as any,
    ) as any;

    expect(placeholder.options.fill).toBe('rgba(0,0,0,0)');
    expect(placeholder.options.stroke).toBe('#1d4ed8');
    expect(placeholder.metadata.data).toEqual({ id: 't:99', pageIndex: 0 });
    expect(placeholder.options).toHaveProperty('width');
    expect(placeholder.options).toHaveProperty('height');
  });
});
