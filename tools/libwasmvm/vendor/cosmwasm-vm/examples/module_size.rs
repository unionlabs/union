use std::fs::File;
use std::io::Read;
use std::mem;

use clap::{App, Arg};

use cosmwasm_vm::internals::compile;
use cosmwasm_vm::internals::make_runtime_store;
use cosmwasm_vm::Size;
use wasmer::Module;

pub fn main() {
    let matches = App::new("Module size estimation")
        .version("0.0.3")
        .author("Mauro Lacy <mauro@lacy.com.es>")
        .arg(
            Arg::with_name("WASM")
                .help("Wasm file to read and compile")
                .required(true)
                .index(1),
        )
        .get_matches();

    // File
    let path = matches.value_of("WASM").expect("Error parsing file name");
    let mut file = File::open(path).unwrap();
    mem::drop(matches);

    // Read wasm
    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm).unwrap();
    mem::drop(file);

    // Report wasm size
    let wasm_size = wasm.len();
    println!("wasm size: {} bytes", wasm_size);

    let memory_limit = Some(Size::mebi(10));

    // Compile module
    let module = module_compile(&wasm, memory_limit);
    mem::drop(wasm);

    // Report loupe size
    let loupe_size = loupe::size_of_val(&module);
    println!("module size (loupe): {} bytes", loupe_size);

    let serialized = module.serialize().unwrap();
    mem::drop(module);

    // Deserialize module
    let module = module_deserialize(&serialized, memory_limit);
    mem::drop(serialized);

    // Report (serialized) module size
    let serialized = module.serialize().unwrap();
    mem::drop(module);
    let ser_size = serialized.len();
    println!("module size (serialized): {} bytes", ser_size);
    println!(
        "(loupe) module size ratio: {:.2}",
        loupe_size as f32 / wasm_size as f32
    );
    println!(
        "(serialized) module size ratio: {:.2}",
        ser_size as f32 / wasm_size as f32
    );
}

#[inline(never)]
fn module_compile(wasm: &[u8], memory_limit: Option<Size>) -> Module {
    compile(wasm, memory_limit, &[]).unwrap()
}

#[inline(never)]
fn module_deserialize(serialized: &[u8], memory_limit: Option<Size>) -> Module {
    // Deserialize using make_runtime_store()
    let store = make_runtime_store(memory_limit);
    unsafe { Module::deserialize(&store, serialized) }.unwrap()
}
