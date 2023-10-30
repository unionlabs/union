// See https://github.com/TeXitoi/structopt/issues/354

use structopt::StructOpt;

#[test]
fn we_need_syn_full() {
    #[allow(unused)]
    #[derive(Debug, StructOpt, Clone)]
    struct Args {
        #[structopt(
            short = "c",
            long = "colour",
            help = "Output colouring",
            default_value = "auto",
            possible_values = &["always", "auto", "never"]
        )]
        colour: String,
    }
}
