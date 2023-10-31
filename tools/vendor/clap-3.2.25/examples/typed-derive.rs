use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)] // requires `derive` feature
struct Args {
    /// Implicitly using `std::str::FromStr`
    #[clap(short = 'O', value_parser)]
    optimization: Option<usize>,

    /// Allow invalid UTF-8 paths
    #[clap(short = 'I', value_parser, value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    include: Option<std::path::PathBuf>,

    /// Handle IP addresses
    #[clap(long, value_parser)]
    bind: Option<std::net::IpAddr>,

    /// Allow human-readable durations
    #[clap(long, value_parser)]
    sleep: Option<humantime::Duration>,

    /// Hand-written parser for tuples
    #[clap(short = 'D', value_parser = parse_key_val::<String, i32>)]
    defines: Vec<(String, i32)>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
