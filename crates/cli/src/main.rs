use clap::Parser;
use std::io::{self, Read};

#[derive(Parser, Debug)]
#[command(
    name = "latex-to-unicode",
    author = "Ivann H. KAMDEM POUOKAM <kapoivha@gmail.com>",
    version = "0.1.0",
    about = "Renders LaTeX mathematical formulas into clean, high-fidelity Unicode text in CLI."
)]
struct Args {
    /// LaTeX expression to convert (if omitted, reads from standard input)
    #[arg(value_name = "EXPRESSION")]
    expression: Option<String>,

    /// Transform an entire markdown document by replacing math blocks
    #[arg(short, long)]
    markdown: bool,

    /// Force 2D display block rendering mode
    #[arg(short, long)]
    block: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let input = match args.expression {
        Some(expr) => expr,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    if args.markdown {
        let output = latex_to_unicode::transform_markdown(&input);
        print!("{}", output);
    } else if args.block {
        let output = latex_to_unicode::latex_to_unicode_block(&input);
        println!("{}", output);
    } else {
        let output = latex_to_unicode::latex_to_unicode(&input);
        println!("{}", output);
    }

    Ok(())
}
