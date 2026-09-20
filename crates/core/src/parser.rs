//! Recursive-descent parser for mathematical LaTeX.
//!
//! Transforms a LaTeX token stream into a clean `MathNode` AST.

use crate::ast::{MathNode, MatrixKind};
use crate::symbols;

pub struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Vec<MathNode> {
        let mut nodes = Vec::new();
        while self.pos < self.chars.len() {
            if let Some(node) = self.parse_node() {
                // Post-process sub/superscripts chaining
                nodes.push(node);
            }
        }
        self.collapse_sub_sup(nodes)
    }

    fn parse_node(&mut self) -> Option<MathNode> {
        let c = self.peek()?;

        match c {
            '\\' => {
                self.advance();
                self.parse_command()
            }
            '{' => {
                self.advance();
                let children = self.parse_until_brace();
                Some(MathNode::Group(children))
            }
            '}' => {
                // Stray closing brace, consume
                self.advance();
                None
            }
            '^' => {
                self.advance();
                self.skip_whitespace();
                let arg = self.parse_single_arg()?;
                Some(MathNode::Superscript {
                    base: None,
                    exp: Box::new(arg),
                })
            }
            '_' => {
                self.advance();
                self.skip_whitespace();
                let arg = self.parse_single_arg()?;
                Some(MathNode::Subscript {
                    base: None,
                    sub: Box::new(arg),
                })
            }
            ' ' | '\t' | '\n' | '\r' => {
                self.advance();
                Some(MathNode::Space(" ".to_string()))
            }
            _ => {
                self.advance();
                Some(MathNode::Text(c.to_string()))
            }
        }
    }

    fn parse_until_brace(&mut self) -> Vec<MathNode> {
        let mut nodes = Vec::new();
        while let Some(c) = self.peek() {
            if c == '}' {
                self.advance();
                break;
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            }
        }
        self.collapse_sub_sup(nodes)
    }

    fn parse_single_arg(&mut self) -> Option<MathNode> {
        self.skip_whitespace();
        let c = self.peek()?;
        if c == '{' {
            self.advance();
            let nodes = self.parse_until_brace();
            Some(MathNode::Group(nodes))
        } else if c == '\\' {
            self.advance();
            self.parse_command()
        } else {
            self.advance();
            Some(MathNode::Text(c.to_string()))
        }
    }

    fn parse_command(&mut self) -> Option<MathNode> {
        let mut name = String::new();

        // Special single char commands (e.g. \, \; \: \!)
        if let Some(c) = self.peek() {
            if !c.is_alphabetic() {
                self.advance();
                name.push(c);
            } else {
                while let Some(ch) = self.peek() {
                    if ch.is_alphabetic() {
                        self.advance();
                        name.push(ch);
                    } else {
                        break;
                    }
                }
            }
        }

        match name.as_str() {
            "frac" | "dfrac" | "tfrac" => {
                let num = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                let den = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(MathNode::Frac {
                    num: Box::new(num),
                    den: Box::new(den),
                })
            }
            "sqrt" => {
                self.skip_whitespace();
                let mut index = None;
                if self.peek() == Some('[') {
                    self.advance();
                    let mut idx_nodes = Vec::new();
                    while let Some(ch) = self.peek() {
                        if ch == ']' {
                            self.advance();
                            break;
                        }
                        if let Some(n) = self.parse_node() {
                            idx_nodes.push(n);
                        }
                    }
                    index = Some(Box::new(MathNode::Group(idx_nodes)));
                }
                let radicand = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(MathNode::Sqrt {
                    index,
                    radicand: Box::new(radicand),
                })
            }
            "binom" | "tbinom" | "dbinom" => {
                let n = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                let k = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(MathNode::Binom {
                    n: Box::new(n),
                    k: Box::new(k),
                })
            }
            "boxed" => {
                let inner = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(MathNode::Boxed(Box::new(inner)))
            }
            "vec" | "hat" | "widehat" | "bar" | "dot" | "ddot" | "tilde" | "widetilde" => {
                if let Some(acc) = symbols::lookup_combining_accent(&name) {
                    let base = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                    Some(MathNode::Accent {
                        accent: acc,
                        base: Box::new(base),
                    })
                } else {
                    Some(MathNode::Text(name))
                }
            }
            "mathbb" => {
                let arg = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(self.apply_alphabet_transform(arg, symbols::mathbb))
            }
            "mathcal" => {
                let arg = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(self.apply_alphabet_transform(arg, symbols::mathcal))
            }
            "mathfrak" => {
                let arg = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(self.apply_alphabet_transform(arg, symbols::mathfrak))
            }
            "text" | "mathrm" | "mathit" | "mathbf" | "operatorname" => {
                let arg = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(arg)
            }
            "left" => {
                self.skip_whitespace();
                let left_delim = self.advance().map(|c| c.to_string()).unwrap_or_default();
                let body = self.parse_until_right();
                let right_delim = self.parse_right_delimiter();
                Some(MathNode::Delimited {
                    left: left_delim,
                    body,
                    right: right_delim,
                })
            }
            "begin" => {
                let env_name = self.parse_env_name();
                self.parse_environment(&env_name)
            }
            "overline" => {
                let arg = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(MathNode::Overline(Box::new(arg)))
            }
            "underline" => {
                let arg = self.parse_single_arg().unwrap_or(MathNode::Text("".into()));
                Some(MathNode::Underline(Box::new(arg)))
            }
            _ => {
                if let Some(sym) = symbols::lookup_symbol(&name) {
                    Some(MathNode::Symbol(sym.to_string()))
                } else {
                    Some(MathNode::Text(format!("\\{}", name)))
                }
            }
        }
    }

    fn parse_until_right(&mut self) -> Vec<MathNode> {
        let mut nodes = Vec::new();
        while self.pos < self.chars.len() {
            if self.peek() == Some('\\') {
                // Check if following is "right"
                let saved_pos = self.pos;
                self.advance();
                let mut cmd = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphabetic() {
                        self.advance();
                        cmd.push(c);
                    } else {
                        break;
                    }
                }
                if cmd == "right" {
                    break;
                } else {
                    self.pos = saved_pos;
                }
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            }
        }
        self.collapse_sub_sup(nodes)
    }

    fn parse_right_delimiter(&mut self) -> String {
        self.skip_whitespace();
        self.advance().map(|c| c.to_string()).unwrap_or_default()
    }

    fn parse_env_name(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() == Some('{') {
            self.advance();
            let mut name = String::new();
            while let Some(c) = self.peek() {
                if c == '}' {
                    self.advance();
                    break;
                }
                self.advance();
                name.push(c);
            }
            name
        } else {
            String::new()
        }
    }

    fn parse_environment(&mut self, env_name: &str) -> Option<MathNode> {
        let kind = match env_name {
            "matrix" | "aligned" | "align" | "align*" | "split" | "gather" => MatrixKind::Plain,
            "pmatrix" => MatrixKind::Paren,
            "bmatrix" => MatrixKind::Bracket,
            "Bmatrix" => MatrixKind::Brace,
            "vmatrix" => MatrixKind::Vbar,
            "Vmatrix" => MatrixKind::VVbar,
            "cases" => MatrixKind::Cases,
            _ => MatrixKind::Plain,
        };

        let mut rows: Vec<Vec<Vec<MathNode>>> = Vec::new();
        let mut current_row: Vec<Vec<MathNode>> = Vec::new();
        let mut current_cell: Vec<MathNode> = Vec::new();

        while self.pos < self.chars.len() {
            // Check for row separator \\ or \end{env_name}
            if self.peek() == Some('\\') {
                let saved_pos = self.pos;
                self.advance();
                if self.peek() == Some('\\') {
                    self.advance();
                    // Double backslash = row separator
                    current_row.push(current_cell);
                    current_cell = Vec::new();
                    rows.push(current_row);
                    current_row = Vec::new();
                    continue;
                }

                let mut cmd = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphabetic() {
                        self.advance();
                        cmd.push(c);
                    } else {
                        break;
                    }
                }
                if cmd == "end" {
                    let end_env = self.parse_env_name();
                    if end_env == env_name {
                        break;
                    }
                } else {
                    self.pos = saved_pos;
                }
            }

            if self.peek() == Some('&') {
                self.advance();
                current_row.push(current_cell);
                current_cell = Vec::new();
                continue;
            }

            if let Some(node) = self.parse_node() {
                current_cell.push(node);
            }
        }

        if !current_cell.is_empty() || !current_row.is_empty() {
            current_row.push(current_cell);
            rows.push(current_row);
        }

        Some(MathNode::Matrix { kind, rows })
    }

    fn apply_alphabet_transform<F>(&self, node: MathNode, transform: F) -> MathNode
    where
        F: Fn(char) -> Option<char> + Copy,
    {
        match node {
            MathNode::Text(s) => {
                let res: String = s.chars().map(|c| transform(c).unwrap_or(c)).collect();
                MathNode::Symbol(res)
            }
            MathNode::Group(children) => {
                let new_children = children
                    .into_iter()
                    .map(|child| self.apply_alphabet_transform(child, transform))
                    .collect();
                MathNode::Group(new_children)
            }
            other => other,
        }
    }

    fn collapse_sub_sup(&self, nodes: Vec<MathNode>) -> Vec<MathNode> {
        let mut result: Vec<MathNode> = Vec::new();

        for node in nodes {
            match node {
                MathNode::Superscript { base: None, exp } => {
                    if let Some(last) = result.pop() {
                        match last {
                            MathNode::Subscript { base, sub } => {
                                result.push(MathNode::SubSup { base, sub, exp });
                            }
                            other => {
                                result.push(MathNode::Superscript {
                                    base: Some(Box::new(other)),
                                    exp,
                                });
                            }
                        }
                    } else {
                        result.push(MathNode::Superscript { base: None, exp });
                    }
                }
                MathNode::Subscript { base: None, sub } => {
                    if let Some(last) = result.pop() {
                        match last {
                            MathNode::Superscript { base, exp } => {
                                result.push(MathNode::SubSup { base, sub, exp });
                            }
                            other => {
                                result.push(MathNode::Subscript {
                                    base: Some(Box::new(other)),
                                    sub,
                                });
                            }
                        }
                    } else {
                        result.push(MathNode::Subscript { base: None, sub });
                    }
                }
                other => result.push(other),
            }
        }

        result
    }
}
