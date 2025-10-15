/// Token parsing utilities for PDF content streams.
/// The MVP implementation exposes just enough scaffolding to satisfy the
/// incremental update pipeline, without yet performing full tokenisation.
#[derive(Debug, Clone)]
pub struct ContentSpan {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ContentStream {
  pub bytes: Vec<u8>,
}

impl ContentStream {
  pub fn new(bytes: Vec<u8>) -> Self {
    Self { bytes }
  }

  pub fn replace_span(&mut self, span: &ContentSpan, replacement: &[u8]) {
    self.bytes.splice(span.start..span.end, replacement.iter().copied());
  }
}
