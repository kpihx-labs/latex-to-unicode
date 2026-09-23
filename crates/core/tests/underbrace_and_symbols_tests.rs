use latex_to_unicode::{latex_to_unicode, latex_to_unicode_block, transform_markdown};

#[test]
fn test_underbrace_inline_and_block() {
    let inline = latex_to_unicode(r"\underbrace{\sum_{k=0}^{q} \frac{q!}{k!}}_{S_1}");
    assert!(
        !inline.contains("\\underbrace"),
        "literal \\underbrace must not leak: {inline}"
    );
    assert!(inline.contains('⏟'), "expected underbrace glyph: {inline}");
    assert!(inline.contains('Σ') || inline.contains('∑'), "{inline}");

    let block = latex_to_unicode_block(
        r"\underbrace{\sum_{k=0}^{q} \frac{q!}{k!}}_{S_1} + \underbrace{\sum_{k=q+1}^{\infty} \frac{q!}{k!}}_{R}",
    );
    assert!(
        !block.contains("\\underbrace"),
        "literal \\underbrace must not leak in block: {block}"
    );
    assert!(block.contains('─') || block.contains('⏟'), "{block}");
    assert!(block.contains('S') || block.contains('₁'), "{block}");
    assert!(block.contains('R'), "{block}");
}

#[test]
fn test_overbrace() {
    let res = latex_to_unicode(r"\overbrace{a+b}^{sum}");
    assert!(!res.contains("\\overbrace"), "{res}");
    assert!(res.contains('⏞'), "{res}");
}

#[test]
fn test_mid_bigl_blacksquare() {
    assert_eq!(latex_to_unicode(r"p \mid n"), "p ∣ n");
    assert_eq!(latex_to_unicode(r"\bigl( a \bigr)"), "( a )");
    assert_eq!(latex_to_unicode(r"\Bigl[ F \Bigr]"), "[ F ]");
    assert_eq!(latex_to_unicode(r"\blacksquare"), "■");
    assert_eq!(latex_to_unicode(r"\square"), "□");
    assert_eq!(latex_to_unicode(r"\qed"), "∎");
}

#[test]
fn test_partial_subscript_q_plus_one() {
    let res = latex_to_unicode(r"\sum_{k=q+1}^{\infty}");
    assert!(
        !res.contains("_(k=q+1)"),
        "should partially convert, not full ASCII fallback: {res}"
    );
    assert!(res.contains('∑'), "{res}");
    // Digits / = / + convert; q stays ASCII
    assert!(res.contains('q'), "{res}");
    assert!(res.contains('₊') || res.contains('+'), "{res}");
}

#[test]
fn test_frac_no_spurious_parens() {
    assert_eq!(latex_to_unicode(r"\frac{p^2}{q^2}"), "p²/q²");
    assert_eq!(latex_to_unicode(r"\sqrt{2}=\frac{p}{q}"), "√2=p/q");
    assert_eq!(latex_to_unicode(r"\frac{a+b}{c}"), "(a+b)/c");
}

#[test]
fn test_binom_block_2d() {
    let res = latex_to_unicode_block(r"\binom{n}{k}");
    assert!(res.contains('⎛'), "{res}");
    assert!(res.contains('⎝'), "{res}");
    assert!(res.contains('n'), "{res}");
    assert!(res.contains('k'), "{res}");
}

#[test]
fn test_markdown_e_proof_style() {
    let md = r#"
Step:
$$
q! \cdot e = \underbrace{\sum_{k=0}^{q} \frac{q!}{k!}}_{S_1} + \underbrace{\sum_{k=q+1}^{\infty} \frac{q!}{k!}}_{R}
$$
And $p \mid n^2$ with $\bigl(a\bigr)$ and $\blacksquare$.
"#;
    let out = transform_markdown(md);
    assert!(!out.contains("\\underbrace"), "{out}");
    assert!(!out.contains("\\mid"), "{out}");
    assert!(!out.contains("\\bigl"), "{out}");
    assert!(!out.contains("\\blacksquare"), "{out}");
    assert!(out.contains('∣'), "{out}");
    assert!(out.contains('■'), "{out}");
}
