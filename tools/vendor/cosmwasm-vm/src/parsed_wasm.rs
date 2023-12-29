use wasmer::wasmparser::{
    Export, Import, MemoryType, Parser, TableType, ValidPayload, Validator, WasmFeatures,
};

use crate::VmResult;

/// A parsed and validated wasm module.
/// It keeps track of the parts that are important for our static analysis and compatibility checks.
#[derive(Debug)]
pub struct ParsedWasm<'a> {
    pub version: u32,
    pub exports: Vec<Export<'a>>,
    pub imports: Vec<Import<'a>>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
}

impl<'a> ParsedWasm<'a> {
    pub fn parse(wasm: &'a [u8]) -> VmResult<Self> {
        let mut validator = Validator::new_with_features(WasmFeatures {
            deterministic_only: true,
            component_model: false,
            simd: false,
            relaxed_simd: false,
            threads: false,
            multi_memory: false,
            memory64: false,
            ..Default::default()
        });

        let mut this = Self {
            version: 0,
            exports: vec![],
            imports: vec![],
            tables: vec![],
            memories: vec![],
        };

        let mut fun_allocations = Default::default();
        for p in Parser::new(0).parse_all(wasm) {
            let p = p?;
            // validate the payload
            if let ValidPayload::Func(fv, body) = validator.payload(&p)? {
                // also validate function bodies
                let mut fun_validator = fv.into_validator(fun_allocations);
                fun_validator.validate(&body)?;
                fun_allocations = fun_validator.into_allocations();
            }

            match p {
                wasmer::wasmparser::Payload::Version { num, .. } => this.version = num,
                wasmer::wasmparser::Payload::ImportSection(i) => {
                    this.imports = i.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                wasmer::wasmparser::Payload::TableSection(t) => {
                    this.tables = t.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                wasmer::wasmparser::Payload::MemorySection(m) => {
                    this.memories = m.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                wasmer::wasmparser::Payload::ExportSection(e) => {
                    this.exports = e.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                _ => {} // ignore everything else
            }
        }

        Ok(this)
    }
}
