use latex_to_unicode::{latex_to_unicode_block, transform_markdown};

#[test]
fn test_huge_matrix_100_rows() {
    let mut matrix_code = String::from("\\begin{pmatrix}\n");
    for i in 1..=100 {
        matrix_code.push_str(&format!("a_{{{}}} & b_{{{}}} & c_{{{}}}", i, i, i));
        if i < 100 {
            matrix_code.push_str(" \\\\\n");
        } else {
            matrix_code.push_str("\n");
        }
    }
    matrix_code.push_str("\\end{pmatrix}");

    let res = latex_to_unicode_block(&matrix_code);
    assert!(res.contains("⎛"));
    assert!(res.contains("⎝"));
    assert!(res.contains("a₁"));
    assert!(res.contains("a₁₀₀"));
    assert!(res.contains("c₁₀₀"));
}

#[test]
fn test_all_delimiters_variants() {
    let doc = r#"
Document with all math delimiter variants:
1. Bracket display:
\[
\int_0^1 x^2 dx = \frac{1}{3}
\]

2. Paren inline: \( \alpha + \beta = 42 \)

3. Dollar inline: $ \gamma \leq 10 $

4. Double dollar display:
$$
\sum_{k=1}^n k = \frac{n(n+1)}{2}
$$
"#;

    let res = transform_markdown(doc);
    assert!(res.contains("∫₀¹ x² dx = 1/3"));
    assert!(res.contains("α + β = 42"));
    assert!(res.contains("γ ≤ 10"));
    assert!(res.contains("∑ₖ₌₁ⁿ k = (n(n+1))/2"));
}

#[test]
fn test_aligned_environment_long_proof() {
    let proof = r#"
\begin{aligned}
(x + y)^3 &= (x + y)(x + y)^2 \\
&= (x + y)(x^2 + 2xy + y^2) \\
&= x^3 + 2x^2y + xy^2 + x^2y + 2xy^2 + y^3 \\
&= x^3 + 3x^2y + 3xy^2 + y^3
\end{aligned}
"#;

    let res = latex_to_unicode_block(proof);
    assert!(res.contains("(x + y)³"));
    assert!(res.contains("x³ + 3x²y + 3xy² + y³"));
}
