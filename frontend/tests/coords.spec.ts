import { describe, expect, it } from 'vitest';
import {
  SCALE,
  fabricDeltaToPdfDelta,
  invert,
  multiply,
  pxToPtMatrix,
  ptToPxMatrix,
  type Matrix,
} from '../src/coords';

describe('coords helpers', () => {
  it('multiplies affine matrices in the correct order', () => {
    const translate: Matrix = [1, 0, 0, 1, 10, -5];
    const scale: Matrix = [2, 0, 0, 0.5, 0, 0];
    const combined = multiply(scale, translate);

    expect(combined).toEqual([2, 0, 0, 0.5, 20, -2.5]);
  });

  it('computes matrix inverses', () => {
    const matrix: Matrix = [1.2, 0.1, -0.4, 0.9, 4, -7];
    const inverse = invert(matrix);
    const identity = multiply(matrix, inverse);

    expect(identity[0]).toBeCloseTo(1, 12);
    expect(identity[3]).toBeCloseTo(1, 12);
    expect(identity[1]).toBeCloseTo(0, 12);
    expect(identity[2]).toBeCloseTo(0, 12);
    expect(identity[4]).toBeCloseTo(0, 12);
    expect(identity[5]).toBeCloseTo(0, 12);
  });

  it('produces identity when converting px -> pt -> px', () => {
    const pageHeight = 720;
    const pxToPt = pxToPtMatrix(pageHeight);
    const ptToPx = ptToPxMatrix(pageHeight);
    const identity = multiply(pxToPt, ptToPx);

    expect(identity[0]).toBeCloseTo(1, 12);
    expect(identity[3]).toBeCloseTo(1, 12);
    expect(identity[4]).toBeCloseTo(0, 12);
    expect(identity[5]).toBeCloseTo(0, 12);
  });

  it('converts fabric deltas into PDF space', () => {
    const pageHeight = 792;
    const fold: Matrix = [1, 0, 0, 1, 0, 0];
    const fnew: Matrix = [1, 0, 0, 1, 96, -96];

    const delta = fabricDeltaToPdfDelta(fold, fnew, pageHeight);

    expect(delta[4]).toBeCloseTo(72, 12);
    expect(delta[5]).toBeCloseTo(72, 12);
    expect(SCALE).toBeCloseTo(0.75, 12);
  });
});
