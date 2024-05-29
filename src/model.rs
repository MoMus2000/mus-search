use std::collections::HashMap;

type TF = HashMap<String, usize>;
type IDF = HashMap<TF, usize>;

pub struct Model {
    pub tf: TF,
    pub idf: IDF
}

impl Model {
    // Add functions to include, remove documents to index etc
}

pub fn term_freq(term: &str, n:usize, document: &TF) -> f32{
    let a = document.get(term).cloned().unwrap_or(0) as f32;
    let b = n as f32;
    a / b
}

pub fn inverse_document_freq(term: &str, n:usize, doc_freq: &TF) -> f32 {
    let d = n as f32;
    let m = doc_freq.get(term).cloned().unwrap_or(1) as f32;
    (d / m).log10()
}