#[macro_use]
extern crate criterion;

use anyhow::Result;
use criterion::Criterion;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use wasmparser::{DataKind, ElementKind, Parser, Payload, Validator, WasmFeatures};

/// A benchmark input.
pub struct BenchmarkInput {
    /// The path to the benchmark file important for handling errors.
    pub path: PathBuf,
    /// The encoded Wasm module that is run by the benchmark.
    pub wasm: Vec<u8>,
}

impl BenchmarkInput {
    /// Creates a new benchmark input.
    pub fn new(test_path: PathBuf, encoded_wasm: Vec<u8>) -> Self {
        Self {
            path: test_path,
            wasm: encoded_wasm,
        }
    }
}

/// Returns a vector of all found benchmark input files under the given directory.
///
/// Benchmark input files can be `.wat` or `.wast` formatted files.
/// For `.wast` files we pull out all the module directives and run them in the benchmarks.
fn collect_test_files(path: &Path, list: &mut Vec<BenchmarkInput>) -> Result<()> {
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_test_files(&path, list)?;
            continue;
        }
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("wasm") => {
                let wasm = fs::read(&path)?;
                list.push(BenchmarkInput::new(path, wasm));
            }
            Some("wat") | Some("txt") => {
                if let Ok(wasm) = wat::parse_file(&path) {
                    list.push(BenchmarkInput::new(path, wasm));
                }
            }
            Some("wast") => {
                let contents = fs::read_to_string(&path)?;
                let buf = match wast::parser::ParseBuffer::new(&contents) {
                    Ok(buf) => buf,
                    Err(_) => continue,
                };
                let wast: wast::Wast<'_> = match wast::parser::parse(&buf) {
                    Ok(wast) => wast,
                    Err(_) => continue,
                };
                for directive in wast.directives {
                    match directive {
                        wast::WastDirective::Module(mut module) => {
                            let wasm = module.encode()?;
                            list.push(BenchmarkInput::new(path.clone(), wasm));
                        }
                        _ => continue,
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

/// Reads the input given the Wasm parser or validator.
///
/// The `path` specifies which benchmark input file we are currently operating on
/// so that we can report better errors in case of failures.
fn read_all_wasm(wasm: &[u8]) -> Result<()> {
    use Payload::*;
    for item in Parser::new(0).parse_all(wasm) {
        match item? {
            TypeSection(s) => {
                for item in s {
                    item?;
                }
            }
            ImportSection(s) => {
                for item in s {
                    item?;
                }
            }
            AliasSection(s) => {
                for item in s {
                    item?;
                }
            }
            InstanceSection(s) => {
                for item in s {
                    for arg in item?.args()? {
                        arg?;
                    }
                }
            }
            FunctionSection(s) => {
                for item in s {
                    item?;
                }
            }
            TableSection(s) => {
                for item in s {
                    item?;
                }
            }
            MemorySection(s) => {
                for item in s {
                    item?;
                }
            }
            TagSection(s) => {
                for item in s {
                    item?;
                }
            }
            GlobalSection(s) => {
                for item in s {
                    for op in item?.init_expr.get_operators_reader() {
                        op?;
                    }
                }
            }
            ExportSection(s) => {
                for item in s {
                    item?;
                }
            }
            ElementSection(s) => {
                for item in s {
                    let item = item?;
                    if let ElementKind::Active { init_expr, .. } = item.kind {
                        for op in init_expr.get_operators_reader() {
                            op?;
                        }
                    }
                    for op in item.items.get_items_reader()? {
                        op?;
                    }
                }
            }
            DataSection(s) => {
                for item in s {
                    let item = item?;
                    if let DataKind::Active { init_expr, .. } = item.kind {
                        for op in init_expr.get_operators_reader() {
                            op?;
                        }
                    }
                }
            }
            CodeSectionEntry(body) => {
                for local in body.get_locals_reader()? {
                    local?;
                }
                for op in body.get_operators_reader()? {
                    op?;
                }
            }

            Version { .. }
            | StartSection { .. }
            | DataCountSection { .. }
            | UnknownSection { .. }
            | CustomSection { .. }
            | CodeSectionStart { .. }
            | ModuleSectionStart { .. }
            | ModuleSectionEntry { .. }
            | End => {}
        }
    }
    Ok(())
}

/// Returns the default benchmark inputs that are proper `wasmparser` benchmark
/// test inputs.
fn collect_benchmark_inputs() -> Vec<BenchmarkInput> {
    let mut ret = Vec::new();
    collect_test_files("../../tests".as_ref(), &mut ret).unwrap();
    return ret;
}

fn it_works_benchmark(c: &mut Criterion) {
    let mut inputs = collect_benchmark_inputs();
    // Filter out all benchmark inputs that fail to parse via `wasmparser`.
    inputs.retain(|input| read_all_wasm(input.wasm.as_slice()).is_ok());
    c.bench_function("it works benchmark", move |b| {
        b.iter(|| {
            for input in &mut inputs {
                read_all_wasm(input.wasm.as_slice()).unwrap();
            }
        })
    });
}

fn validate_benchmark(c: &mut Criterion) {
    fn validator() -> Validator {
        let mut ret = Validator::new();
        ret.wasm_features(WasmFeatures {
            reference_types: true,
            multi_value: true,
            simd: true,
            relaxed_simd: true,
            exceptions: true,
            module_linking: true,
            bulk_memory: true,
            threads: true,
            tail_call: true,
            multi_memory: true,
            memory64: true,
            extended_const: true,
            deterministic_only: false,
            mutable_global: true,
            saturating_float_to_int: true,
            sign_extension: true,
        });
        return ret;
    }
    let mut inputs = collect_benchmark_inputs();
    // Filter out all benchmark inputs that fail to validate via `wasmparser`.
    inputs.retain(|input| validator().validate_all(&input.wasm).is_ok());
    c.bench_function("validate benchmark", move |b| {
        b.iter(|| {
            for input in &mut inputs {
                validator().validate_all(&input.wasm).unwrap();
            }
        })
    });
}

criterion_group!(benchmark, it_works_benchmark, validate_benchmark);
criterion_main!(benchmark);
