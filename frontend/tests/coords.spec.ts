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

describe('coords utilities', () => {
  it('multiply combines affine matrices', () => {
    const translate: Matrix = [1, 0, 0, 1, 10, -5];
    const scale: Matrix = [2, 0, 0, 0.5, 0, 0];
    const combined = multiply(scale, translate);
    expect(combined).toEqual([2, 0, 0, 0.5, 20, -2.5]);
  });

  it('invert produces the matrix inverse', () => {
    const matrix: Matrix = [1.2, 0.1, -0.4, 0.9, 4, -7];
    const inverseMatrix = invert(matrix);
    const identity = multiply(matrix, inverseMatrix);
    expect(identity[0]).toBeCloseTo(1, 9);
    expect(identity[3]).toBeCloseTo(1, 9);
    expect(identity[1]).toBeCloseTo(0, 9);
    expect(identity[2]).toBeCloseTo(0, 9);
  });

  it('px <-> pt conversion produces identity when chained', () => {
    const pageHeight = 720;
    const pxToPt = pxToPtMatrix(pageHeight);
    const ptToPx = ptToPxMatrix(pageHeight);
    const identity = multiply(pxToPt, ptToPx);
    expect(identity[0]).toBeCloseTo(1, 9);
    expect(identity[3]).toBeCloseTo(1, 9);
  });

  it('fabricDeltaToPdfDelta converts translation into point space', () => {
    const pageHeight = 792;
    const fold: Matrix = [1, 0, 0, 1, 0, 0];
    const fnew: Matrix = [1, 0, 0, 1, 96, -96];
    const delta = fabricDeltaToPdfDelta(fold, fnew, pageHeight);
    expect(delta[4]).toBeCloseTo(72, 9);
    expect(delta[5]).toBeCloseTo(72, 9);
    expect(SCALE).toBeCloseTo(0.75, 9);
  });
});
