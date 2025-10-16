import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import * as ts from 'typescript';

const compilerOptions = {
  module: ts.ModuleKind.ES2020,
  target: ts.ScriptTarget.ES2020,
  esModuleInterop: true,
  moduleResolution: ts.ModuleResolutionKind.NodeJs,
};

const mockMap = new Map([
  ['fabric', path.resolve('tests/mocks/fabric.ts')],
  ['pdfjs-dist', path.resolve('tests/mocks/pdfjs-dist.ts')],
  ['pdfjs-dist/build/pdf.worker?url', path.resolve('tests/mocks/pdfjs-dist/build/pdf.worker?url.ts')],
  ['vitest', path.resolve('tests/vitest-shim.mjs')],
]);

export async function resolve(specifier, context, defaultResolve) {
  const mapped = mockMap.get(specifier);
  if (mapped) {
    return { shortCircuit: true, url: pathToFileURL(mapped).href };
  }

  if (specifier.startsWith('.') && context.parentURL?.endsWith('.ts')) {
    const url = new URL(specifier, context.parentURL);
    if (!url.pathname.endsWith('.ts')) {
      url.pathname += '.ts';
    }
    return { shortCircuit: true, url: url.href };
  }

  return defaultResolve(specifier, context, defaultResolve);
}

export async function load(url, context, defaultLoad) {
  if (url.endsWith('.ts')) {
    const source = await readFile(fileURLToPath(url), 'utf8');
    const transpiled = ts.transpileModule(source, {
      compilerOptions,
      fileName: fileURLToPath(url),
    });
    return {
      format: 'module',
      source: transpiled.outputText,
      shortCircuit: true,
    };
  }

  return defaultLoad(url, context, defaultLoad);
}
