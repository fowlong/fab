// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
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

test('toFabricMatrix converts point transforms to pixel space', () => {
  const transform: Matrix = [1, 0, 0, 1, 36, -18];
  const ptToPx = ptToPxMatrix(samplePage.heightPt);
  const expected = multiply(ptToPx, transform);
  const result = toFabricMatrix(samplePage as any, transform);
  assert.deepEqual(result, expected);
});

test('bboxPtToPx converts point bounding boxes into CSS pixels', () => {
  const bbox = [72, 648, 144, 720];
  const [left, top, width, height] = bboxPtToPx(samplePage as any, bbox as any);
  assert.equal(left, 72 / SCALE);
  assert.equal(top, (samplePage.heightPt - 720) / SCALE);
  assert.equal(width, (144 - 72) / SCALE);
  assert.equal(height, (720 - 648) / SCALE);
});

test('createFabricPlaceholder returns a styled fabric rect with metadata', () => {
  const canvas = new Canvas({ appendChild() {} } as any, {});
  const placeholder = createFabricPlaceholder(
    canvas as any,
    samplePage as any,
    sampleObject as any,
  ) as any;

  assert.equal(placeholder.options.fill, 'rgba(0,0,0,0)');
  assert.equal(placeholder.options.stroke, '#1d4ed8');
  assert.deepEqual(placeholder.metadata.data, { id: 't:99', pageIndex: 0 });
  assert.ok('width' in placeholder.options && 'height' in placeholder.options);
});
