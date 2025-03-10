use clap::Parser;

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
    let _args = Args::parse();
}
