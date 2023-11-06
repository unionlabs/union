use std::ffi::OsString;

use clap::Parser;
use unionlabs::{parse_wasm_client_type, WasmClientType};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Args {
    file_path: OsString,
    expected_client_type: WasmClientType,
}

fn main() {
    let args = Args::parse();

    let bz = std::fs::read(args.file_path).unwrap();

    match parse_wasm_client_type(bz) {
        Ok(Some(ty)) => assert_eq!(ty, args.expected_client_type),
        Ok(None) => panic!("file does not contain a wasm client type"),
        Err(err) => panic!("{err}"),
    }
}
