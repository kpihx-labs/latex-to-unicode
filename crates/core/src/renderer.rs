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
                format!(
                    "{}{}{}",
                    base_str,
                    to_unicode_sub(&sub_str),
                    to_unicode_sup(&exp_str)
                )
            }

            MathNode::Frac { num, den } => {
                let num_str = self.render_inline_node(num);
                let den_str = self.render_inline_node(den);
                // Parens only when the side is a compound expression (operators / spaces)
                let num_fmt = if is_compound(&num_str) && !num_str.starts_with('(') {
                    format!("({})", num_str)
                } else {
                    num_str
                };
                let den_fmt = if is_compound(&den_str) && !den_str.starts_with('(') {
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
                // Compact choose notation; block mode upgrades to 2D paren form
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

            MathNode::Underbrace { body, label } => {
                let body_str = self.render_inline_node(body);
                match label {
                    Some(lab) => {
                        let lab_str = self.render_inline_node(lab);
                        format!("{}⏟{}", body_str, to_unicode_sub(&lab_str))
                    }
                    None => format!("{}⏟", body_str),
                }
            }

            MathNode::Overbrace { body, label } => {
                let body_str = self.render_inline_node(body);
                match label {
                    Some(lab) => {
                        let lab_str = self.render_inline_node(lab);
                        format!("{}⏞{}", body_str, to_unicode_sup(&lab_str))
                    }
                    None => format!("⏞{}", body_str),
                }
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
        // Matrices / cases: high-fidelity 2D
        for node in nodes {
            if let MathNode::Matrix { kind, rows } = node {
                return self.render_2d_matrix(*kind, rows);
            }
        }

        // Single underbrace / overbrace / binom / frac → dedicated 2D
        if nodes.len() == 1 {
            match &nodes[0] {
                MathNode::Underbrace { body, label } => {
                    return self.render_2d_brace(body, label.as_deref(), true);
                }
                MathNode::Overbrace { body, label } => {
                    return self.render_2d_brace(body, label.as_deref(), false);
                }
                MathNode::Binom { n, k } => {
                    return self.render_2d_binom(n, k);
                }
                MathNode::Frac { num, den } => {
                    return self.render_2d_frac(num, den);
                }
                _ => {}
            }
        }

        // Mixed expression containing underbrace/overbrace: expand those nodes 2D
        if nodes
            .iter()
            .any(|n| matches!(n, MathNode::Underbrace { .. } | MathNode::Overbrace { .. }))
        {
            return self.render_block_with_braces(nodes);
        }

        // Standard display block: indented inline rendering
        let rendered = self.render_inline_nodes(nodes);
        format!("  {}", rendered)
    }

    /// 2D underbrace (under=true) or overbrace (under=false).
    fn render_2d_brace(&self, body: &MathNode, label: Option<&MathNode>, under: bool) -> String {
        let body_str = self.render_inline_node(body);
        let label_str = label
            .map(|l| self.render_inline_node(l))
            .unwrap_or_default();
        let body_w = visual_width(&body_str);
        let label_w = visual_width(&label_str);
        let width = body_w.max(label_w).max(1);

        let body_line = center_pad(&body_str, width);
        let brace_line = if width <= 3 {
            center_pad(&if under { "⏟" } else { "⏞" }.to_string(), width)
        } else {
            std::iter::repeat('─').take(width).collect()
        };
        let label_line = if label_str.is_empty() {
            None
        } else {
            Some(center_pad(&label_str, width))
        };

        let mut lines = Vec::new();
        if under {
            lines.push(format!("  {}", body_line));
            lines.push(format!("  {}", brace_line));
            if let Some(l) = label_line {
                lines.push(format!("  {}", l));
            }
        } else {
            if let Some(l) = label_line {
                lines.push(format!("  {}", l));
            }
            lines.push(format!("  {}", brace_line));
            lines.push(format!("  {}", body_line));
        }
        lines.join("\n")
    }

    fn render_2d_binom(&self, n: &MathNode, k: &MathNode) -> String {
        let n_str = self.render_inline_node(n);
        let k_str = self.render_inline_node(k);
        let w = visual_width(&n_str).max(visual_width(&k_str));
        format!(
            "  ⎛ {} ⎞\n  ⎝ {} ⎠",
            center_pad(&n_str, w),
            center_pad(&k_str, w)
        )
    }

    fn render_2d_frac(&self, num: &MathNode, den: &MathNode) -> String {
        let num_str = self.render_inline_node(num);
        let den_str = self.render_inline_node(den);
        let w = visual_width(&num_str).max(visual_width(&den_str)).max(1);
        let bar: String = std::iter::repeat('─').take(w).collect();
        format!(
            "  {}\n  {}\n  {}",
            center_pad(&num_str, w),
            bar,
            center_pad(&den_str, w)
        )
    }

    /// Block render when the expression mixes braces with other tokens.
    fn render_block_with_braces(&self, nodes: &[MathNode]) -> String {
        enum Part {
            Inline(String),
            Brace {
                body: String,
                bar: String,
                label: String,
                under: bool,
            },
        }

        let mut parts: Vec<Part> = Vec::new();
        for node in nodes {
            match node {
                MathNode::Underbrace { body, label } => {
                    let body_str = self.render_inline_node(body);
                    let label_str = label
                        .as_ref()
                        .map(|l| self.render_inline_node(l))
                        .unwrap_or_default();
                    let w = visual_width(&body_str).max(visual_width(&label_str)).max(1);
                    let bar: String = if w <= 3 {
                        center_pad("⏟", w)
                    } else {
                        std::iter::repeat('─').take(w).collect()
                    };
                    parts.push(Part::Brace {
                        body: center_pad(&body_str, w),
                        bar,
                        label: center_pad(&label_str, w),
                        under: true,
                    });
                }
                MathNode::Overbrace { body, label } => {
                    let body_str = self.render_inline_node(body);
                    let label_str = label
                        .as_ref()
                        .map(|l| self.render_inline_node(l))
                        .unwrap_or_default();
                    let w = visual_width(&body_str).max(visual_width(&label_str)).max(1);
                    let bar: String = if w <= 3 {
                        center_pad("⏞", w)
                    } else {
                        std::iter::repeat('─').take(w).collect()
                    };
                    parts.push(Part::Brace {
                        body: center_pad(&body_str, w),
                        bar,
                        label: center_pad(&label_str, w),
                        under: false,
                    });
                }
                other => {
                    let s = self.render_inline_node(other);
                    if s.is_empty() {
                        continue;
                    }
                    // Merge consecutive inlines
                    if let Some(Part::Inline(prev)) = parts.last_mut() {
                        prev.push_str(&s);
                    } else {
                        parts.push(Part::Inline(s));
                    }
                }
            }
        }

        if !parts.iter().any(|p| matches!(p, Part::Brace { .. })) {
            return format!("  {}", self.render_inline_nodes(nodes));
        }

        let all_under = parts.iter().all(|p| match p {
            Part::Brace { under, .. } => *under,
            Part::Inline(_) => true,
        });

        let mut body_parts = Vec::new();
        let mut bar_parts = Vec::new();
        let mut label_parts = Vec::new();

        for part in &parts {
            match part {
                Part::Inline(s) => {
                    let w = visual_width(s);
                    body_parts.push(s.clone());
                    bar_parts.push(" ".repeat(w));
                    label_parts.push(" ".repeat(w));
                }
                Part::Brace {
                    body, bar, label, ..
                } => {
                    body_parts.push(body.clone());
                    bar_parts.push(bar.clone());
                    label_parts.push(label.clone());
                }
            }
        }

        let body_line = body_parts.join("");
        let bar_line = bar_parts.join("");
        let label_line = label_parts.join("");

        if all_under {
            format!("  {}\n  {}\n  {}", body_line, bar_line, label_line)
        } else {
            format!("  {}\n  {}\n  {}", label_line, bar_line, body_line)
        }
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
            lines.push(
                format!("  {} {} {}", left_delim, content, right_delim)
                    .trim_end()
                    .to_string(),
            );
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

    // Phrases with spaces/arrows: all-or-nothing (keeps lim_(x → 0) readable)
    let use_partial = !(s.contains(' ') || s.contains('→') || s.contains('←') || s.contains('⟶'));

    let mut res = String::new();
    let mut all_converted = true;
    let mut any_converted = false;

    for c in s.chars() {
        if let Some(sub) = symbols::to_subscript(c) {
            res.push(sub);
            any_converted = true;
        } else if use_partial {
            // Leave ASCII letters without a Unicode subscript form as-is (e.g. q in k=q+1)
            res.push(c);
            all_converted = false;
        } else {
            all_converted = false;
            break;
        }
    }

    if all_converted {
        res
    } else if use_partial && any_converted {
        res
    } else {
        format!("_({})", s)
    }
}

fn visual_width(s: &str) -> usize {
    s.chars().count()
}

fn center_pad(s: &str, width: usize) -> String {
    let w = visual_width(s);
    if w >= width {
        return s.to_string();
    }
    let pad = width - w;
    let left = pad / 2;
    let right = pad - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
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
