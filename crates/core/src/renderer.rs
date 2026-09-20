//! Unicode renderer for LaTeX MathNode AST.
//!
//! Handles:
//! - Inline rendering (compact 1D math representation with Unicode symbols)
//! - Multi-line Block Display rendering (2D brackets, matrices, cases, aligned formulas)

use crate::ast::{MathNode, MatrixKind};
use crate::symbols;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Inline,
    Block,
}

pub struct Renderer {
    mode: RenderMode,
}

impl Renderer {
    pub fn new(mode: RenderMode) -> Self {
        Self { mode }
    }

    pub fn render(&self, nodes: &[MathNode]) -> String {
        match self.mode {
            RenderMode::Inline => self.render_inline_nodes(nodes),
            RenderMode::Block => self.render_block_nodes(nodes),
        }
    }

    // =========================================================================
    // INLINE RENDERING (1D compact)
    // =========================================================================

    fn render_inline_nodes(&self, nodes: &[MathNode]) -> String {
        let mut out = String::new();
        for node in nodes {
            out.push_str(&self.render_inline_node(node));
        }
        normalize_spaces(&out)
    }

    fn render_inline_node(&self, node: &MathNode) -> String {
        match node {
            MathNode::Text(s) => s.clone(),
            MathNode::Symbol(s) => s.clone(),
            MathNode::Space(s) => s.clone(),
            MathNode::Group(children) => self.render_inline_nodes(children),

            MathNode::Superscript { base, exp } => {
                let base_str = base
                    .as_ref()
                    .map(|b| self.render_inline_node(b))
                    .unwrap_or_default();
                let exp_str = self.render_inline_node(exp);
                let converted_exp = to_unicode_sup(&exp_str);
                format!("{}{}", base_str, converted_exp)
            }

            MathNode::Subscript { base, sub } => {
                let base_str = base
                    .as_ref()
                    .map(|b| self.render_inline_node(b))
                    .unwrap_or_default();
                let sub_str = self.render_inline_node(sub);
                let converted_sub = to_unicode_sub(&sub_str);
                format!("{}{}", base_str, converted_sub)
            }

            MathNode::SubSup { base, sub, exp } => {
                let base_str = base
                    .as_ref()
                    .map(|b| self.render_inline_node(b))
                    .unwrap_or_default();
                let sub_str = self.render_inline_node(sub);
                let exp_str = self.render_inline_node(exp);
                format!("{}{}{}", base_str, to_unicode_sub(&sub_str), to_unicode_sup(&exp_str))
            }

            MathNode::Frac { num, den } => {
                let num_str = self.render_inline_node(num);
                let den_str = self.render_inline_node(den);
                let num_fmt = if num_str.chars().count() > 1 && !num_str.starts_with('(') {
                    format!("({})", num_str)
                } else {
                    num_str
                };
                let den_fmt = if den_str.chars().count() > 1 && !den_str.starts_with('(') {
                    format!("({})", den_str)
                } else {
                    den_str
                };
                format!("{}/{}", num_fmt, den_fmt)
            }

            MathNode::Sqrt { index, radicand } => {
                let rad_str = self.render_inline_node(radicand);
                let rad_fmt = if is_compound(&rad_str) {
                    format!("({})", rad_str)
                } else {
                    rad_str
                };
                if let Some(idx) = index {
                    let idx_str = self.render_inline_node(idx);
                    format!("{}√{}", to_unicode_sup(&idx_str), rad_fmt)
                } else {
                    format!("√{}", rad_fmt)
                }
            }

            MathNode::Binom { n, k } => {
                let n_str = self.render_inline_node(n);
                let k_str = self.render_inline_node(k);
                format!("C({}, {})", n_str, k_str)
            }

            MathNode::Accent { accent, base } => {
                let base_str = self.render_inline_node(base);
                let mut chars: Vec<char> = base_str.chars().collect();
                if let Some(first) = chars.first_mut() {
                    let mut s = String::new();
                    s.push(*first);
                    s.push(*accent);
                    s.extend(chars.iter().skip(1));
                    s
                } else {
                    accent.to_string()
                }
            }

            MathNode::Delimited { left, body, right } => {
                let body_str = self.render_inline_nodes(body);
                format!("{}{}{}", left, body_str, right)
            }

            MathNode::Boxed(inner) => {
                let inner_str = self.render_inline_node(inner);
                format!("| {} |", inner_str)
            }

            MathNode::Overline(inner) => {
                let inner_str = self.render_inline_node(inner);
                format!("¯({})¯", inner_str)
            }

            MathNode::Underline(inner) => {
                let inner_str = self.render_inline_node(inner);
                format!("_({})_", inner_str)
            }

            MathNode::Matrix { kind, rows } => {
                let (open, close) = match kind {
                    MatrixKind::Plain => ("", ""),
                    MatrixKind::Paren => ("(", ")"),
                    MatrixKind::Bracket => ("[", "]"),
                    MatrixKind::Brace => ("{", "}"),
                    MatrixKind::Vbar => ("|", "|"),
                    MatrixKind::VVbar => ("‖", "‖"),
                    MatrixKind::Cases => ("{", ""),
                };

                let row_strs: Vec<String> = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|cell| self.render_inline_nodes(cell))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .collect();

                format!("{}{}{}", open, row_strs.join("; "), close)
            }
        }
    }

    // =========================================================================
    // BLOCK RENDERING (2D formatted math)
    // =========================================================================

    fn render_block_nodes(&self, nodes: &[MathNode]) -> String {
        // If the expression contains matrices or cases, render high-fidelity 2D blocks
        for node in nodes {
            if let MathNode::Matrix { kind, rows } = node {
                return self.render_2d_matrix(*kind, rows);
            }
        }

        // Standard display block: centered / indented with clean inline rendering
        let rendered = self.render_inline_nodes(nodes);
        format!("  {}", rendered)
    }

    fn render_2d_matrix(&self, kind: MatrixKind, rows: &[Vec<Vec<MathNode>>]) -> String {
        if rows.is_empty() {
            return String::new();
        }

        let rendered_grid: Vec<Vec<String>> = rows
            .iter()
            .map(|r| {
                r.iter()
                    .map(|cell| self.render_inline_nodes(cell))
                    .collect()
            })
            .collect();

        let num_cols = rendered_grid.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths = vec![0; num_cols];

        for row in &rendered_grid {
            for (c_idx, cell) in row.iter().enumerate() {
                col_widths[c_idx] = col_widths[c_idx].max(cell.chars().count());
            }
        }

        let (top_l, mid_l, bot_l, top_r, mid_r, bot_r) = match kind {
            MatrixKind::Plain => (" ", " ", " ", " ", " ", " "),
            MatrixKind::Paren => ("⎛", "⎜", "⎝", "⎞", "⎟", "⎠"),
            MatrixKind::Bracket => ("⎡", "⎢", "⎣", "⎤", "⎥", "⎦"),
            MatrixKind::Brace => ("⎧", "⎨", "⎩", "⎫", "⎬", "⎭"),
            MatrixKind::Vbar => ("│", "│", "│", "│", "│", "│"),
            MatrixKind::VVbar => ("║", "║", "║", "║", "║", "║"),
            MatrixKind::Cases => ("⎧", "⎨", "⎩", "", "", ""),
        };

        let mut lines = Vec::new();
        let num_rows = rendered_grid.len();

        for (r_idx, row) in rendered_grid.iter().enumerate() {
            let left_delim = if num_rows == 1 {
                match kind {
                    MatrixKind::Paren => "(",
                    MatrixKind::Bracket => "[",
                    MatrixKind::Brace => "{",
                    MatrixKind::Vbar => "|",
                    MatrixKind::VVbar => "‖",
                    _ => "",
                }
            } else if r_idx == 0 {
                top_l
            } else if r_idx == num_rows - 1 {
                bot_l
            } else {
                mid_l
            };

            let right_delim = if num_rows == 1 {
                match kind {
                    MatrixKind::Paren => ")",
                    MatrixKind::Bracket => "]",
                    MatrixKind::Brace => "}",
                    MatrixKind::Vbar => "|",
                    MatrixKind::VVbar => "‖",
                    _ => "",
                }
            } else if r_idx == 0 {
                top_r
            } else if r_idx == num_rows - 1 {
                bot_r
            } else {
                mid_r
            };

            let mut row_cells = Vec::new();
            for (c_idx, cell) in row.iter().enumerate() {
                let w = col_widths[c_idx];
                let cell_len = cell.chars().count();
                let pad = " ".repeat(w.saturating_sub(cell_len));
                row_cells.push(format!("{}{}", cell, pad));
            }

            let content = row_cells.join("   ");
            lines.push(format!("  {} {} {}", left_delim, content, right_delim).trim_end().to_string());
        }

        lines.join("\n")
    }
}

fn is_compound(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Check if contains binary operators outside parentheses
    trimmed.contains(' ')
        || trimmed.contains('+')
        || trimmed.contains('-')
        || trimmed.contains('=')
        || trimmed.contains('/')
        || trimmed.contains('·')
        || trimmed.contains('×')
}

fn to_unicode_sup(s: &str) -> String {
    if s == "∞" {
        return "^∞".to_string();
    }
    if s == "+∞" {
        return "⁺^∞".to_string();
    }
    if s == "-∞" {
        return "⁻^∞".to_string();
    }
    let mut res = String::new();
    let mut all_converted = true;

    for c in s.chars() {
        if let Some(sup) = symbols::to_superscript(c) {
            res.push(sup);
        } else {
            match c {
                '²' => res.push('²'),
                '³' => res.push('³'),
                '¹' => res.push('¹'),
                '⁰' => res.push('⁰'),
                '⁴' => res.push('⁴'),
                '⁵' => res.push('⁵'),
                '⁶' => res.push('⁶'),
                '⁷' => res.push('⁷'),
                '⁸' => res.push('⁸'),
                '⁹' => res.push('⁹'),
                '⁻' => res.push('⁻'),
                '⁺' => res.push('⁺'),
                _ => {
                    all_converted = false;
                    break;
                }
            }
        }
    }

    if all_converted {
        res
    } else {
        format!("^({})", s)
    }
}

fn to_unicode_sub(s: &str) -> String {
    if s == "∞" {
        return "_∞".to_string();
    }
    if s == "+∞" {
        return "₊_∞".to_string();
    }
    if s == "-∞" {
        return "₋_∞".to_string();
    }
    let mut res = String::new();
    let mut all_converted = true;

    for c in s.chars() {
        if let Some(sub) = symbols::to_subscript(c) {
            res.push(sub);
        } else {
            all_converted = false;
            break;
        }
    }

    if all_converted {
        res
    } else {
        format!("_({})", s)
    }
}

fn normalize_spaces(s: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}
