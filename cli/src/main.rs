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
    let args = Args::parse();
    let fonts = Fonts::from_dir(&args.fonts_dir);

    let familyset = Familyset::from_fonts_xml();
    let fallbacks = familyset.fallbacks();

    let mut contains = 0;
    let mut missing = 0;
    for fallback in fallbacks {
        for font in fallback.fonts.iter() {
            if fonts.contains(&FontIdentifier::Filename(font.filename.as_str().into())) {
                contains += 1;
            } else {
                missing += 1;
                eprintln!("Unable to locate {}", font.filename);
            }
        }
    }

    itemize(&args.text, fonts);

    println!("{contains}/{} fallback fonts located", contains + missing);

    for s in graphemes(&args.text) {
        println!("{} codepoints: {s}", s.chars().count());
    }
}
