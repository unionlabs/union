//! Various builders to generate/alter wasm components

mod code;
mod data;
mod export;
mod global;
mod import;
mod invoke;
mod memory;
mod misc;
mod module;
mod table;

pub use self::{
	code::{
		function, signature, signatures, FuncBodyBuilder, FunctionBuilder, FunctionDefinition,
		SignatureBuilder, SignaturesBuilder, TypeRefBuilder,
	},
	data::DataSegmentBuilder,
	export::{export, ExportBuilder, ExportInternalBuilder},
	global::{global, GlobalBuilder},
	import::{import, ImportBuilder},
	invoke::Identity,
	memory::MemoryBuilder,
	module::{from_module, module, CodeLocation, ModuleBuilder},
	table::{TableBuilder, TableDefinition, TableEntryDefinition},
};
