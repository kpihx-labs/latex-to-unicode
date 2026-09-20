//! Core entry point and high-level conversion APIs.

pub mod ast;
pub mod parser;
pub mod renderer;
pub mod symbols;

use parser::Parser;
use renderer::{RenderMode, Renderer};

/// Strip standard LaTeX delimiters: `$ ... $`, `$$ ... $$`, `\[ ... \]`, `\( ... \)`.
pub fn strip_delimiters(latex: &str) -> &str {
    let trimmed = latex.trim();
    if (trimmed.starts_with("$$") && trimmed.ends_with("$$"))
        || (trimmed.starts_with("\\[") && trimmed.ends_with("\\]"))
    {
        let inner = &trimmed[2..trimmed.len() - 2];
        inner.trim()
    } else if (trimmed.starts_with('$') && trimmed.ends_with('$'))
        || (trimmed.starts_with("\\(") && trimmed.ends_with("\\)"))
    {
        let start_len = if trimmed.starts_with('$') { 1 } else { 2 };
        let end_len = if trimmed.ends_with('$') { 1 } else { 2 };
        let inner = &trimmed[start_len..trimmed.len() - end_len];
        inner.trim()
    } else {
        trimmed
    }
}

/// Convert a single inline LaTeX math formula into clean Unicode text.
pub fn latex_to_unicode(latex: &str) -> String {
    let raw = strip_delimiters(latex);
    let mut parser = Parser::new(raw);
    let nodes = parser.parse();
    let renderer = Renderer::new(RenderMode::Inline);
    renderer.render(&nodes)
}

/// Convert a display block LaTeX formula into 2D / multi-line formatted Unicode text.
pub fn latex_to_unicode_block(latex: &str) -> String {
    let raw = strip_delimiters(latex);
    let mut parser = Parser::new(raw);
    let nodes = parser.parse();
    let renderer = Renderer::new(RenderMode::Block);
    renderer.render(&nodes)
}

/// Transform entire markdown documents by finding and converting math blocks ($$, $, \[, \() into Unicode.
pub fn transform_markdown(markdown: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = markdown.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // 1. Check for \[ ... \] display block math (multi-line supported)
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '[' {
            let start = i;
            i += 2;
            let mut found_end = false;
            let mut end = i;

            while i + 1 < chars.len() {
                if chars[i] == '\\' && chars[i + 1] == ']' {
                    found_end = true;
                    end = i;
                    i += 2;
                    break;
                }
                i += 1;
            }

            if found_end {
                let formula: String = chars[start + 2..end].iter().collect();
                let converted = latex_to_unicode_block(&formula);
                result.push('\n');
                result.push_str(&converted);
                result.push('\n');
            } else {
                result.extend(&chars[start..]);
                break;
            }
            continue;
        }

        // 2. Check for \( ... \) inline math
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '(' {
            let start = i;
            i += 2;
            let mut found_end = false;
            let mut end = i;

            while i + 1 < chars.len() {
                if chars[i] == '\\' && chars[i + 1] == ')' {
                    found_end = true;
                    end = i;
                    i += 2;
                    break;
                }
                if chars[i] == '\n' {
                    break;
                }
                i += 1;
            }

            if found_end {
                let formula: String = chars[start + 2..end].iter().collect();
                let converted = latex_to_unicode(&formula);
                result.push_str(&converted);
            } else {
                result.push(chars[start]);
                i = start + 1;
            }
            continue;
        }

        // 3. Check for $$ display math (multi-line supported)
        if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '$' {
            let start = i;
            i += 2; // skip opening $$
            let mut found_end = false;
            let mut end = i;

            while i + 1 < chars.len() {
                if chars[i] == '$' && chars[i + 1] == '$' && (i == 0 || chars[i - 1] != '\\') {
                    found_end = true;
                    end = i;
                    i += 2;
                    break;
                }
                i += 1;
            }

            if found_end {
                let formula: String = chars[start + 2..end].iter().collect();
                let converted = latex_to_unicode_block(&formula);
                result.push('\n');
                result.push_str(&converted);
                result.push('\n');
            } else {
                result.extend(&chars[start..]);
                break;
            }
            continue;
        }

        // 4. Check for $ inline math
        if chars[i] == '$' && (i == 0 || chars[i - 1] != '\\') {
            let start = i;
            i += 1; // skip opening $
            let mut found_end = false;
            let mut end = i;

            while i < chars.len() {
                if chars[i] == '$' && chars[i - 1] != '\\' {
                    found_end = true;
                    end = i;
                    i += 1;
                    break;
                }
                if chars[i] == '\n' {
                    // Inline math does not cross single lines unless $$
                    break;
                }
                i += 1;
            }

            if found_end {
                let formula: String = chars[start + 1..end].iter().collect();
                let converted = latex_to_unicode(&formula);
                result.push_str(&converted);
            } else {
                result.push(chars[start]);
                i = start + 1;
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_symbols() {
        assert_eq!(latex_to_unicode(r"E = mc^2"), "E = mc²");
        assert_eq!(latex_to_unicode(r"\alpha + \beta = \gamma"), "α + β = γ");
    }

    #[test]
    fn test_integral_and_fractions() {
        let input = r"\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}";
        let res = latex_to_unicode(input);
        assert_eq!(res, "∫₀^∞ e⁻ˣ² dx = (√π)/2");
    }

    #[test]
    fn test_sub_sup() {
        let input = r"\sum_{i=0}^{n} x_i \cdot \vec{v} \approx \alpha \pm \beta";
        let res = latex_to_unicode(input);
        assert_eq!(res, "∑ᵢ₌₀ⁿ xᵢ · v⃗ ≈ α ± β");
    }

    #[test]
    fn test_alphabets() {
        assert_eq!(
            latex_to_unicode(r"\forall x \in \mathbb{R}, \exists y \in \mathbb{C}"),
            "∀ x ∈ ℝ, ∃ y ∈ ℂ"
        );
        assert_eq!(
            latex_to_unicode(r"\mathcal{H} \otimes \mathcal{F}"),
            "ℋ ⊗ ℱ"
        );
        assert_eq!(
            latex_to_unicode(r"\mathfrak{g} \oplus \mathfrak{h}"),
            "𝔤 ⊕ 𝔥"
        );
    }

    #[test]
    fn test_nested_fractions() {
        let input = r"\frac{a + \frac{1}{b}}{c + d}";
        let res = latex_to_unicode(input);
        assert_eq!(res, "(a + 1/b)/(c + d)");
    }

    #[test]
    fn test_2d_matrix_rendering() {
        let input = r"\begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}";
        let res = latex_to_unicode_block(input);
        assert!(res.contains("⎛"));
        assert!(res.contains("1"));
        assert!(res.contains("0"));
        assert!(res.contains("⎠"));
    }

    #[test]
    fn test_markdown_transformation_multi_line() {
        let md = "Here is an equation:\n$$\n\\int_a^b f(x)dx = F(b) - F(a)\n$$\nDone!";
        let out = transform_markdown(md);
        assert!(out.contains("∫ₐᵇ f(x)dx = F(b) - F(a)"));
    }
}
