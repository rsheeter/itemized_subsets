use clap::Parser;
use cli::{fonts::Fonts, graphemes, itemize};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The string we want to itemize
    #[arg(short, long)]
    text: String,

    /// The language to prefer when breaking ties, particularly crucial for CJK due to Han unification
    #[arg(short, long, default_value = "")]
    lang: String,

    /// Where to look for fonts, such as a Google Fonts github path
    #[arg(short, long)]
    fonts_dir: String,
}

fn main() {
    let args = Args::parse();
    let fonts = Fonts::from_dir(&args.fonts_dir);
    itemize(&args.text, fonts);

    for s in graphemes(&args.text) {
        println!("{} codepoints: {s}", s.chars().count());
    }
}
