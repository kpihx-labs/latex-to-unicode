use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn latex_to_unicode(latex: &str) -> String {
    latex_to_unicode::latex_to_unicode(latex)
}

#[wasm_bindgen]
pub fn latex_to_unicode_block(latex: &str) -> String {
    latex_to_unicode::latex_to_unicode_block(latex)
}

#[wasm_bindgen]
pub fn transform_markdown(markdown: &str) -> String {
    latex_to_unicode::transform_markdown(markdown)
}
