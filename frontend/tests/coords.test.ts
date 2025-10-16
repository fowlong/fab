// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import {
  SCALE,
  fabricDeltaToPdfDelta,
  invert,
  multiply,
  pxToPtMatrix,
  ptToPxMatrix,
  type Matrix,
} from '../src/coords';

test('multiply combines affine matrices', () => {
  const translate: Matrix = [1, 0, 0, 1, 10, -5];
  const scale: Matrix = [2, 0, 0, 0.5, 0, 0];
  const combined = multiply(scale, translate);
  assert.deepEqual(combined, [2, 0, 0, 0.5, 20, -2.5]);
});

test('invert produces the matrix inverse', () => {
  const matrix: Matrix = [1.2, 0.1, -0.4, 0.9, 4, -7];
  const inverse = invert(matrix);
  const identity = multiply(matrix, inverse);
  assert.ok(Math.abs(identity[0] - 1) < 1e-9);
  assert.ok(Math.abs(identity[3] - 1) < 1e-9);
  assert.ok(Math.abs(identity[1]) < 1e-9);
  assert.ok(Math.abs(identity[2]) < 1e-9);
});

test('px <-> pt conversion produces identity when chained', () => {
  const pageHeight = 720;
  const pxToPt = pxToPtMatrix(pageHeight);
  const ptToPx = ptToPxMatrix(pageHeight);
  const identity = multiply(pxToPt, ptToPx);
  assert.ok(Math.abs(identity[0] - 1) < 1e-9);
  assert.ok(Math.abs(identity[3] - 1) < 1e-9);
});

test('fabricDeltaToPdfDelta converts translation into point space', () => {
  const pageHeight = 792;
  const fold: Matrix = [1, 0, 0, 1, 0, 0];
  const fnew: Matrix = [1, 0, 0, 1, 96, -96];
  const delta = fabricDeltaToPdfDelta(fold, fnew, pageHeight);
  assert.ok(Math.abs(delta[4] - 72) < 1e-9);
  assert.ok(Math.abs(delta[5] - 72) < 1e-9);
  assert.ok(Math.abs(SCALE - 0.75) < 1e-9);
});
