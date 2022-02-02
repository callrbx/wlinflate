use std::{
    io::{self, stdout, BufWriter, Write},
    path::PathBuf,
};

use structopt::StructOpt;
use wlinflate::Wordlist;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "wlinflate",
    author = "icon",
    about = "simple tool to expand a wordlist with prepends, appends, extensions, and substitutions"
)]
struct Args {
    #[structopt(short = "v", long = "verbose", help = "enable basic logging")]
    verbose: bool,
    #[structopt(short = "p", long = "prepend", help = "prepend wordlist words (csv)")]
    prepend: Option<String>,
    #[structopt(short = "a", long = "append", help = "append wordlist words (csv)")]
    append: Option<String>,
    #[structopt(short = "x", long = "extensions", help = "extensions to search (csv)")]
    extensions: Option<String>,
    #[structopt(
        short = "s",
        long = "swap",
        help = "swap in for entries that contain {SWAP} (csv)"
    )]
    swap: Option<String>,

    #[structopt(
        short = "w",
        long = "wordlist",
        help = "path to wordlist",
        parse(from_os_str)
    )]
    wordlist: PathBuf,
    #[structopt(short = "o", long = "output", help = "output file", parse(from_os_str))]
    outfile: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Args::from_args();
    let mut count: usize = 0;
    let file;
    let stdout = stdout();
    let stdout_lock = stdout.lock();

    let mut writer: Box<dyn Write> = match args.outfile {
        None => Box::new(BufWriter::new(stdout_lock)),
        Some(filename) => {
            file = std::fs::File::create(filename)?;
            Box::new(BufWriter::new(file))
        }
    };

    let wl = Wordlist::new(
        &args.wordlist,
        args.prepend,
        args.append,
        args.swap,
        args.extensions,
    );

    if args.verbose {
        println!("[*] Orginal Wordlist Size: {}", wl.base_count);
        println!("[*] Estimated Inflated Size: {}", wl.total_count);
    }

    for word in wl {
        writer.write(word.as_bytes())?;
        writer.write(b"\n")?;
        count += 1;
    }

    writer.flush()?;

    if args.verbose {
        println!("[*] Inflated Wordlist Size: {}", count);
    }

    return Ok(());
}
