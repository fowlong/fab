use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;

use crate::types::{DocumentIR, PageIR, PdfObject, PdfRef, TextObject, BtSpan, FontInfo};

const PLACEHOLDER_PDF_BASE64: &str = "JVBERi0xLjQKJeLjz9MKMSAwIG9iago8PAovVHlwZSAvQ2F0YWxvZwovUGFnZXMgMiAwIFIKPj4KZW5kb2JqCjIgMCBvYmoKPDwKL1R5cGUgL1BhZ2VzCi9LaWRzIFszIDAgUl0KL0NvdW50IDEKPj4KZW5kb2JqCjMgMCBvYmoKPDwKL1R5cGUgL1BhZ2UKL1BhcmVudCAyIDAgUgovTWVkaWFCb3ggWzAgMCA1OTUuMjc2IDg0MS44OV0KL0NvbnRlbnRzIDQgMCBSCj4+CmVuZG9iago0IDAgb2JqCjw8Ci9MZW5ndGggMTMKPj4Kc3RyZWFtCkJUIC9GMSAxMiBUZgooSGVsbG8pIFRqCkVUCmVuZHN0cmVhbQplbmRvYmoKNSAwIG9iago8PAovVHlwZSAvRm9udAovU3VidHlwZSAvVHlwZTEKL05hbWUgL0YxCi9CYXNlRm9udCAvSGVsdmV0aWNhCi9FbmNvZGluZyAvV2luQW5zaUVuY29kaW5nCj4+CmVuZG9iagp4cmVmCjAgNgowMDAwMDAwMDAwIDY1NTM1IGYgCjAwMDAwMDAwMTAgMDAwMDAgbiAKMDAwMDAwMDA4NSAwMDAwMCBuIAowMDAwMDAwMTc1IDAwMDAwIG4gCjAwMDAwMDAzMDIgMDAwMDAgbiAKMDAwMDAwMDM5MCAwMDAwMCBuIAp0cmFpbGVyCjw8Ci9TaXplIDYKL1Jvb3QgMSAwIFIKL0luZm8gNSAwIFIKPj4Kc3RhcnR4cmVmCjQyNgolJUVPRgo=";

#[derive(Default, Clone)]
pub struct PdfStore {
    inner: Arc<RwLock<HashMap<String, StoredDoc>>>,
}

struct StoredDoc {
    ir: DocumentIR,
    pdf_data_base64: String,
}

impl PdfStore {
    pub fn create_placeholder(&self) -> String {
        let mut guard = self.inner.write();
        let id = format!("doc-{}", guard.len() + 1);
        guard.insert(
            id.clone(),
            StoredDoc {
                ir: DocumentIR { pages: vec![placeholder_page()] },
                pdf_data_base64: PLACEHOLDER_PDF_BASE64.to_string(),
            },
        );
        id
    }

    pub fn ir_for(&self, id: &str) -> DocumentIR {
        self.inner
            .read()
            .get(id)
            .map(|doc| doc.ir.clone())
            .unwrap_or_else(|| DocumentIR { pages: vec![] })
    }

    pub fn updated_pdf(&self, id: &str) -> String {
        self.inner
            .read()
            .get(id)
            .map(|doc| doc.pdf_data_base64.clone())
            .unwrap_or_default()
    }
}

fn placeholder_page() -> PageIR {
    PageIR {
        index: 0,
        widthPt: 595.0,
        heightPt: 842.0,
        objects: vec![PdfObject::Text(TextObject {
            id: "t:1".into(),
            pdfRef: PdfRef { obj: 0, gen: 0 },
            btSpan: BtSpan {
                start: 0,
                end: 0,
                streamObj: 0,
            },
            Tm: [1.0, 0.0, 0.0, 1.0, 100.0, 700.0],
            font: FontInfo {
                resName: "F1".into(),
                size: 12.0,
                r#type: "Type0".into(),
            },
            unicode: "Placeholder".into(),
            glyphs: vec![],
            bbox: [100.0, 680.0, 200.0, 710.0],
        })],
    }
}
