//! Abstract Syntax Tree for LaTeX expressions.

#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    /// Raw textual/alphanumeric content
    Text(String),
    /// A single symbol resolved to a Unicode character/string
    Symbol(String),
    /// A group enclosed in braces: `{ ... }`
    Group(Vec<MathNode>),
    /// Superscript: `base^{exponent}`
    Superscript {
        base: Option<Box<MathNode>>,
        exp: Box<MathNode>,
    },
    /// Subscript: `base_{subscript}`
    Subscript {
        base: Option<Box<MathNode>>,
        sub: Box<MathNode>,
    },
    /// Combined sub and super: `base_{sub}^{exp}`
    SubSup {
        base: Option<Box<MathNode>>,
        sub: Box<MathNode>,
        exp: Box<MathNode>,
    },
    /// Fraction: `\frac{numerator}{denominator}`
    Frac {
        num: Box<MathNode>,
        den: Box<MathNode>,
    },
    /// Square or n-th root: `\sqrt[n]{radicand}`
    Sqrt {
        index: Option<Box<MathNode>>,
        radicand: Box<MathNode>,
    },
    /// Binomial coefficient: `\binom{n}{k}`
    Binom {
        n: Box<MathNode>,
        k: Box<MathNode>,
    },
    /// Accent over a base: `\vec{x}`, `\hat{y}`
    Accent {
        accent: char,
        base: Box<MathNode>,
    },
    /// Delimited block: `\left( ... \right)`
    Delimited {
        left: String,
        body: Vec<MathNode>,
        right: String,
    },
    /// Matrix / Table environments: `\begin{matrix} ... \end{matrix}`
    Matrix {
        kind: MatrixKind,
        rows: Vec<Vec<Vec<MathNode>>>,
    },
    /// Boxed formula: `\boxed{...}`
    Boxed(Box<MathNode>),
    /// Overline / underline
    Overline(Box<MathNode>),
    Underline(Box<MathNode>),
    /// Underbrace: `\underbrace{body}_{label}`
    Underbrace {
        body: Box<MathNode>,
        label: Option<Box<MathNode>>,
    },
    /// Overbrace: `\overbrace{body}^{label}`
    Overbrace {
        body: Box<MathNode>,
        label: Option<Box<MathNode>>,
    },
    /// Plain space
    Space(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixKind {
    Plain,   // matrix
    Paren,   // pmatrix ( ... )
    Bracket, // bmatrix [ ... ]
    Brace,   // Bmatrix { ... }
    Vbar,    // vmatrix | ... |
    VVbar,   // Vmatrix || ... ||
    Cases,   // cases {
}
