use clap::Parser;
use cli::{graphemes, itemize};

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
}

fn main() {
    let args = Args::parse();

    itemize(&args.text);

    for s in graphemes(&args.text) {
        println!("{} codepoints: {s}", s.chars().count());
    }
}
