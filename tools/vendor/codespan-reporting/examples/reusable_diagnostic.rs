use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::termcolor::StandardStream;
use codespan_reporting::term::{self, ColorArg};
use std::ops::Range;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub struct Opts {
    #[structopt(long = "color",
        parse(try_from_str),
        default_value = "auto",
        possible_values = ColorArg::VARIANTS,
        case_insensitive = true
    )]
    color: ColorArg,
}

fn main() -> anyhow::Result<()> {
    let file = SimpleFile::new(
        "main.rs",
        unindent::unindent(
            r#"
                fn main() {
                    let foo: i32 = "hello, world";
                    foo += 1;
                }
            "#,
        ),
    );

    let errors = [
        Error::MismatchType(
            Item::new(20..23, "i32"),
            Item::new(31..45, "\"hello, world\""),
        ),
        Error::MutatingImmutable(Item::new(20..23, "foo"), Item::new(51..59, "foo += 1")),
    ];

    let opts = Opts::from_args();
    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::term::Config::default();
    for diagnostic in errors.iter().map(Error::report) {
        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    }

    Ok(())
}

/// An error enum that represent all possible errors within your program
enum Error {
    MismatchType(Item, Item),
    MutatingImmutable(Item, Item),
}

impl Error {
    fn report(&self) -> Diagnostic<()> {
        match self {
            Error::MismatchType(left, right) => Diagnostic::error()
                .with_code("E0308")
                .with_message("mismatch types")
                .with_labels(vec![
                    Label::primary((), right.range.clone()).with_message(format!(
                        "Expected `{}`, found: `{}`",
                        left.content, right.content,
                    )),
                    Label::secondary((), left.range.clone()).with_message("expected due to this"),
                ]),
            Error::MutatingImmutable(original, mutating) => Diagnostic::error()
                .with_code("E0384")
                .with_message(format!(
                    "cannot mutate immutable variable `{}`",
                    original.content,
                ))
                .with_labels(vec![
                    Label::secondary((), original.range.clone()).with_message(unindent::unindent(
                        &format!(
                            r#"
                                first assignment to `{0}`
                                help: make this binding mutable: `mut {0}`
                            "#,
                            original.content,
                        ),
                    )),
                    Label::primary((), mutating.range.clone())
                        .with_message("cannot assign twice to immutable variable"),
                ]),
        }
    }
}

/// An item in the source code to be used in the `Error` enum.
/// In a more complex program it could also contain a `files::FileId` to handle errors that occur inside multiple files.
struct Item {
    range: Range<usize>,
    content: String,
}

impl Item {
    fn new(range: Range<usize>, content: impl Into<String>) -> Item {
        let content = content.into();
        Item { range, content }
    }
}
