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
