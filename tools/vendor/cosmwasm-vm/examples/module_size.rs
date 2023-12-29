use std::fs::File;
use std::io::Read;
use std::mem;

use clap::{Arg, Command};

use cosmwasm_vm::internals::{compile, make_compiling_engine};
use wasmer::{Engine, Module};

pub fn main() {
    let matches = Command::new("Module size estimation")
        .version("0.0.4")
        .author("Mauro Lacy <mauro@confio.gmbh>")
        .arg(
            Arg::new("WASM")
                .help("Wasm file to read and compile")
                .required(true)
                .index(1),
        )
        .get_matches();

    // File
    let path: &String = matches.get_one("WASM").expect("Error parsing file name");
    let mut file = File::open(path).unwrap();
    mem::drop(matches);

    // Read wasm
    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm).unwrap();
    mem::drop(file);

    // Report wasm size
    let wasm_size = wasm.len();
    println!("wasm size: {wasm_size} bytes");

    // Compile module
    let engine = make_compiling_engine(None);
    let module = compile(&engine, &wasm).unwrap();
    mem::drop(wasm);

    let serialized = module.serialize().unwrap();
    mem::drop(module);

    // Deserialize module
    let module = module_deserialize(&engine, &serialized);
    mem::drop(serialized);

    // Report (serialized) module size
    let serialized = module.serialize().unwrap();
    mem::drop(module);
    let ser_size = serialized.len();
    println!("module size (serialized): {ser_size} bytes");
    println!(
        "(serialized) module size ratio: {:.2}",
        ser_size as f32 / wasm_size as f32
    );
}

#[inline(never)]
fn module_deserialize(engine: &Engine, serialized: &[u8]) -> Module {
    unsafe { Module::deserialize(&engine, serialized) }.unwrap()
}
