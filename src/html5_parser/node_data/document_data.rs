#[derive(Debug, PartialEq, Clone)]
pub struct DocumentData {}

impl Default for DocumentData {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentData {
    pub(crate) fn new() -> Self {
        DocumentData {}
    }
}
