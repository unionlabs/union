// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

mod options {
    use super::StructOpt;

    #[derive(Debug, StructOpt)]
    pub struct Options {
        #[structopt(subcommand)]
        pub subcommand: super::subcommands::SubCommand,
    }
}

mod subcommands {
    use super::StructOpt;

    #[derive(Debug, StructOpt)]
    pub enum SubCommand {
        /// foo
        Foo {
            /// foo
            bars: Vec<String>,
        },
    }
}
