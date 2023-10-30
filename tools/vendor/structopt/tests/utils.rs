#![allow(unused)]

use structopt::StructOpt;

pub fn get_help<T: StructOpt>() -> String {
    let mut output = Vec::new();
    <T as StructOpt>::clap().write_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    eprintln!("\n%%% HELP %%%:=====\n{}\n=====\n", output);
    eprintln!("\n%%% HELP (DEBUG) %%%:=====\n{:?}\n=====\n", output);

    output
}

pub fn get_long_help<T: StructOpt>() -> String {
    let mut output = Vec::new();
    <T as StructOpt>::clap()
        .write_long_help(&mut output)
        .unwrap();
    let output = String::from_utf8(output).unwrap();

    eprintln!("\n%%% LONG_HELP %%%:=====\n{}\n=====\n", output);
    eprintln!("\n%%% LONG_HELP (DEBUG) %%%:=====\n{:?}\n=====\n", output);

    output
}

pub fn get_subcommand_long_help<T: StructOpt>(subcmd: &str) -> String {
    let output = <T as StructOpt>::clap()
        .get_matches_from_safe(vec!["test", subcmd, "--help"])
        .expect_err("")
        .message;

    eprintln!(
        "\n%%% SUBCOMMAND `{}` HELP %%%:=====\n{}\n=====\n",
        subcmd, output
    );
    eprintln!(
        "\n%%% SUBCOMMAND `{}` HELP (DEBUG) %%%:=====\n{:?}\n=====\n",
        subcmd, output
    );

    output
}
