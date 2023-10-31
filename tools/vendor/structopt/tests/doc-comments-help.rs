// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod utils;

use structopt::StructOpt;
use utils::*;

#[test]
fn doc_comments() {
    /// Lorem ipsum
    #[derive(StructOpt, PartialEq, Debug)]
    struct LoremIpsum {
        /// Fooify a bar
        /// and a baz
        #[structopt(short, long)]
        foo: bool,
    }

    let help = get_long_help::<LoremIpsum>();
    assert!(help.contains("Lorem ipsum"));
    assert!(help.contains("Fooify a bar and a baz"));
}

#[test]
fn help_is_better_than_comments() {
    /// Lorem ipsum
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// Fooify a bar
        #[structopt(short, long, help = "DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES")]
        foo: bool,
    }

    let help = get_long_help::<LoremIpsum>();
    assert!(help.contains("Dolor sit amet"));
    assert!(!help.contains("Lorem ipsum"));
    assert!(help.contains("DO NOT PASS A BAR"));
}

#[test]
fn empty_line_in_doc_comment_is_double_linefeed() {
    /// Foo.
    ///
    /// Bar
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "lorem-ipsum", no_version)]
    struct LoremIpsum {}

    let help = get_long_help::<LoremIpsum>();
    assert!(help.starts_with("lorem-ipsum \nFoo.\n\nBar\n\nUSAGE:"));
}

#[test]
fn field_long_doc_comment_both_help_long_help() {
    /// Lorem ipsumclap
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// Dot is removed from multiline comments.
        ///
        /// Long help
        #[structopt(long)]
        foo: bool,

        /// Dot is removed from one short comment.
        #[structopt(long)]
        bar: bool,
    }

    let short_help = get_help::<LoremIpsum>();
    let long_help = get_long_help::<LoremIpsum>();

    assert!(short_help.contains("Dot is removed from one short comment"));
    assert!(!short_help.contains("Dot is removed from one short comment."));
    assert!(short_help.contains("Dot is removed from multiline comments"));
    assert!(!short_help.contains("Dot is removed from multiline comments."));
    assert!(long_help.contains("Long help"));
    assert!(!short_help.contains("Long help"));
}

#[test]
fn top_long_doc_comment_both_help_long_help() {
    /// Lorem ipsumclap
    #[derive(StructOpt, Debug)]
    #[structopt(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        #[structopt(subcommand)]
        foo: SubCommand,
    }

    #[derive(StructOpt, Debug)]
    pub enum SubCommand {
        /// DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES
        ///
        /// Or something else
        Foo {
            #[structopt(help = "foo")]
            bars: Vec<String>,
        },
    }

    let short_help = get_help::<LoremIpsum>();
    let long_help = get_subcommand_long_help::<LoremIpsum>("foo");

    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}

#[test]
fn verbatim_doc_comment() {
    /// DANCE!
    ///
    ///                    ()
    ///                    |
    ///               (   ()   )
    ///     ) ________    //  )
    ///  ()  |\       \  //
    /// ( \\__ \ ______\//
    ///    \__) |       |
    ///      |  |       |
    ///       \ |       |
    ///        \|_______|
    ///        //    \\
    ///       ((     ||
    ///        \\    ||
    ///      ( ()    ||
    ///       (      () ) )
    #[derive(StructOpt, Debug)]
    #[structopt(verbatim_doc_comment)]
    struct SeeFigure1 {
        #[structopt(long)]
        foo: bool,
    }

    let help = get_long_help::<SeeFigure1>();
    let sample = r#"
                   ()
                   |
              (   ()   )
    ) ________    //  )
 ()  |\       \  //
( \\__ \ ______\//
   \__) |       |
     |  |       |
      \ |       |
       \|_______|
       //    \\
      ((     ||
       \\    ||
     ( ()    ||
      (      () ) )"#;

    assert!(help.contains(sample))
}

#[test]
fn verbatim_doc_comment_field() {
    #[derive(StructOpt, Debug)]
    struct App {
        /// This help ends in a period.
        #[structopt(long, verbatim_doc_comment)]
        foo: bool,
        /// This help does not end in a period.
        #[structopt(long)]
        bar: bool,
    }

    let help = get_long_help::<App>();
    let sample = r#"
        --bar        
            This help does not end in a period

        --foo        
            This help ends in a period."#;

    assert!(help.contains(sample))
}
