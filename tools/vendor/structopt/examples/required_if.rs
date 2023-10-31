//! How to use `required_if` with structopt.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     required_if -o <out-type> [FILE]
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -o <out-type>        Where to write the output: to `stdout` or `file`
//!
//! ARGS:
//!     <FILE>    File name: only required when `out-type` is set to `file`
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialEq)]
struct Opt {
    /// Where to write the output: to `stdout` or `file`
    #[structopt(short)]
    out_type: String,

    /// File name: only required when `out-type` is set to `file`
    #[structopt(name = "FILE", required_if("out-type", "file"))]
    file_name: Option<String>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_out_type_file_without_file_name_returns_err() {
        let opt = Opt::from_iter_safe(&["test", "-o", "file"]);
        let err = opt.unwrap_err();
        assert_eq!(err.kind, clap::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_opt_out_type_file_with_file_name_returns_ok() {
        let opt = Opt::from_iter_safe(&["test", "-o", "file", "filename"]);
        let opt = opt.unwrap();
        assert_eq!(
            opt,
            Opt {
                out_type: "file".into(),
                file_name: Some("filename".into()),
            }
        );
    }
}
