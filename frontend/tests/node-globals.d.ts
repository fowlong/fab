// @ts-nocheck
declare const Buffer: any;
type Buffer = any;

declare module 'node:stream/web' {
  export type ReadableStream<T = any> = any;
}

declare module 'stream' {
  export type Readable = any;
}

declare module 'http' {
  export type RequestOptions = any;
}

declare module 'jsdom' {
  export type DOMWindow = any;
}

declare module 'node:fs' {
  export function readFileSync(path: string, encoding: string): string;
}

declare module 'node:path' {
  export function join(...parts: string[]): string;
  export function resolve(...parts: string[]): string;
  export function dirname(path: string): string;
}

declare module 'node:url' {
  export function fileURLToPath(url: string | URL): string;
}
