import { DocumentIR } from "./types";

const DEFAULT_IR: DocumentIR = {
  pages: [
    {
      index: 0,
      widthPt: 595.276,
      heightPt: 841.89,
      objects: []
    }
  ]
};

export async function loadInitialDocument(): Promise<DocumentIR> {
  // Placeholder implementation until backend is wired up.
  return DEFAULT_IR;
}
