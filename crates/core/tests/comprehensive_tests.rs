use latex_to_unicode::{latex_to_unicode, latex_to_unicode_block, transform_markdown};

#[test]
fn test_advanced_greek_and_logic() {
    assert_eq!(
        latex_to_unicode(
            r"\forall x \in \mathbb{R}, \exists y \in \mathbb{C} \implies x \leq y \land y \neq x"
        ),
        "∀ x ∈ ℝ, ∃ y ∈ ℂ ⟹ x ≤ y ∧ y ≠ x"
    );
    assert_eq!(
        latex_to_unicode(r"\alpha \times \beta \cdot \gamma \div \delta \approx \pm \infty"),
        "α × β · γ ÷ δ ≈ ± ∞"
    );
}

#[test]
fn test_complex_derivatives_and_integrals() {
    assert_eq!(
        latex_to_unicode(
            r"\oint_{\partial \Omega} \vec{F} \cdot d\vec{r} = \iint_\Omega (\nabla \times \vec{F}) \cdot d\vec{S}"
        ),
        "∮_(∂ Ω) F⃗ · dr⃗ = ∬_(Ω) (∇ × F⃗) · dS⃗"
    );
}

#[test]
fn test_binomial_and_boxed() {
    assert_eq!(
        latex_to_unicode(r"\binom{n}{k} = \frac{n!}{k!(n-k)!}"),
        "C(n, k) = n!/(k!(n-k)!)"
    );
    assert_eq!(
        latex_to_unicode(r"\boxed{e^{i\pi} + 1 = 0}"),
        "| e^(iπ) + 1 = 0 |"
    );
}

#[test]
fn test_limits_and_sums() {
    assert_eq!(
        latex_to_unicode(r"\lim_{x \to 0} \frac{\sin x}{x} = 1"),
        "lim_(x → 0) (sin x)/x = 1"
    );
    assert_eq!(
        latex_to_unicode(r"\sum_{n=1}^\infty \frac{1}{n^2} = \frac{\pi^2}{6}"),
        "∑ₙ₌₁^∞ 1/n² = π²/6"
    );
}

#[test]
fn test_nth_roots() {
    assert_eq!(latex_to_unicode(r"\sqrt{x^2 + y^2}"), "√(x² + y²)");
    assert_eq!(latex_to_unicode(r"\sqrt[3]{8} = 2"), "³√8 = 2");
}

#[test]
fn test_cases_environment() {
    let cases = r"\begin{cases} x & \text{if } x > 0 \\ -x & \text{otherwise} \end{cases}";
    let res = latex_to_unicode_block(cases);
    assert!(res.contains("⎧"));
    assert!(res.contains("⎩"));
    assert!(res.contains("x"));
    assert!(res.contains("-x"));
}

#[test]
fn test_markdown_full_document_stream() {
    let md = r#"
# Calculus Note

Let $f: \mathbb{R} \to \mathbb{R}$ be defined by:
$$
f(x) = \sum_{k=0}^{\infty} \frac{x^k}{k!}
$$

We notice that:
$f'(x) = f(x)$ and $f(0) = 1$.

Hence:
$$
\lim_{x \to \infty} f(x) = \infty
$$
"#;

    let res = transform_markdown(md);
    assert!(res.contains("f: ℝ → ℝ"));
    assert!(res.contains("f'(x) = f(x)"));
    assert!(res.contains("lim_(x → ∞) f(x) = ∞"));
}
