//! Minimal TTF subsetting placeholder.

/// Represents a subsetted font program ready for embedding.
#[derive(Debug, Clone)]
pub struct SubsetFont {
    pub name: String,
    pub data: Vec<u8>,
}

impl SubsetFont {
    #[must_use]
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }
}

/// Creates a dummy subset font using the provided family name.
#[must_use]
pub fn subset_font(family: &str) -> SubsetFont {
    SubsetFont::new(format!("FAKE+{}", family.replace(' ', "")), vec![])
}
