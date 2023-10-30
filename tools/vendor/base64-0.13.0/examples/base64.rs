use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

use base64::{read, write};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum CharacterSet {
    Standard,
    UrlSafe,
}

impl Default for CharacterSet {
    fn default() -> Self {
        CharacterSet::Standard
    }
}

impl Into<base64::Config> for CharacterSet {
    fn into(self) -> base64::Config {
        match self {
            CharacterSet::Standard => base64::STANDARD,
            CharacterSet::UrlSafe => base64::URL_SAFE,
        }
    }
}

impl FromStr for CharacterSet {
    type Err = String;
    fn from_str(s: &str) -> Result<CharacterSet, String> {
        match s {
            "standard" => Ok(CharacterSet::Standard),
            "urlsafe" => Ok(CharacterSet::UrlSafe),
            _ => Err(format!("charset '{}' unrecognized", s)),
        }
    }
}

/// Base64 encode or decode FILE (or standard input), to standard output.
#[derive(Debug, StructOpt)]
struct Opt {
    /// decode data
    #[structopt(short = "d", long = "decode")]
    decode: bool,
    /// The character set to choose. Defaults to the standard base64 character set.
    /// Supported character sets include "standard" and "urlsafe".
    #[structopt(long = "charset")]
    charset: Option<CharacterSet>,
    /// The file to encode/decode.
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let stdin;
    let mut input: Box<dyn Read> = match opt.file {
        None => {
            stdin = io::stdin();
            Box::new(stdin.lock())
        }
        Some(ref f) if f.as_os_str() == "-" => {
            stdin = io::stdin();
            Box::new(stdin.lock())
        }
        Some(f) => Box::new(File::open(f).unwrap()),
    };
    let config = opt.charset.unwrap_or_default().into();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let r = if opt.decode {
        let mut decoder = read::DecoderReader::new(&mut input, config);
        io::copy(&mut decoder, &mut stdout)
    } else {
        let mut encoder = write::EncoderWriter::new(&mut stdout, config);
        io::copy(&mut input, &mut encoder)
    };
    if let Err(e) = r {
        eprintln!(
            "Base64 {} failed with {}",
            if opt.decode { "decode" } else { "encode" },
            e
        );
        process::exit(1);
    }
}
