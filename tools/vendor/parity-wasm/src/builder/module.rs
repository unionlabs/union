use super::{
	code::{self, FunctionBuilder, SignaturesBuilder},
	data, export, global, import,
	invoke::{Identity, Invoke},
	memory::{self, MemoryBuilder},
	table::{self, TableBuilder},
};
use crate::elements;
use alloc::vec::Vec;

/// Module builder
pub struct ModuleBuilder<F = Identity> {
	callback: F,
	module: ModuleScaffold,
}

/// Location of the internal module function
pub struct CodeLocation {
	/// Location (index in 'functions' section) of the signature
	pub signature: u32,
	/// Location (index in the 'code' section) of the body
	pub body: u32,
}

#[derive(Default, PartialEq)]
struct ModuleScaffold {
	pub types: elements::TypeSection,
	pub import: elements::ImportSection,
	pub functions: elements::FunctionSection,
	pub table: elements::TableSection,
	pub memory: elements::MemorySection,
	pub global: elements::GlobalSection,
	pub export: elements::ExportSection,
	pub start: Option<u32>,
	pub element: elements::ElementSection,
	pub code: elements::CodeSection,
	pub data: elements::DataSection,
	pub other: Vec<elements::Section>,
}

impl From<elements::Module> for ModuleScaffold {
	fn from(module: elements::Module) -> Self {
		let mut types: Option<elements::TypeSection> = None;
		let mut import: Option<elements::ImportSection> = None;
		let mut funcs: Option<elements::FunctionSection> = None;
		let mut table: Option<elements::TableSection> = None;
		let mut memory: Option<elements::MemorySection> = None;
		let mut global: Option<elements::GlobalSection> = None;
		let mut export: Option<elements::ExportSection> = None;
		let mut start: Option<u32> = None;
		let mut element: Option<elements::ElementSection> = None;
		let mut code: Option<elements::CodeSection> = None;
		let mut data: Option<elements::DataSection> = None;

		let mut other = Vec::new();
		let mut sections = module.into_sections();
		while let Some(section) = sections.pop() {
			match section {
				elements::Section::Type(sect) => {
					types = Some(sect);
				},
				elements::Section::Import(sect) => {
					import = Some(sect);
				},
				elements::Section::Function(sect) => {
					funcs = Some(sect);
				},
				elements::Section::Table(sect) => {
					table = Some(sect);
				},
				elements::Section::Memory(sect) => {
					memory = Some(sect);
				},
				elements::Section::Global(sect) => {
					global = Some(sect);
				},
				elements::Section::Export(sect) => {
					export = Some(sect);
				},
				elements::Section::Start(index) => {
					start = Some(index);
				},
				elements::Section::Element(sect) => {
					element = Some(sect);
				},
				elements::Section::Code(sect) => {
					code = Some(sect);
				},
				elements::Section::Data(sect) => {
					data = Some(sect);
				},
				section => other.push(section),
			}
		}

		ModuleScaffold {
			types: types.unwrap_or_default(),
			import: import.unwrap_or_default(),
			functions: funcs.unwrap_or_default(),
			table: table.unwrap_or_default(),
			memory: memory.unwrap_or_default(),
			global: global.unwrap_or_default(),
			export: export.unwrap_or_default(),
			start,
			element: element.unwrap_or_default(),
			code: code.unwrap_or_default(),
			data: data.unwrap_or_default(),
			other,
		}
	}
}

impl From<ModuleScaffold> for elements::Module {
	fn from(module: ModuleScaffold) -> Self {
		let mut sections = Vec::new();

		let types = module.types;
		if !types.types().is_empty() {
			sections.push(elements::Section::Type(types));
		}
		let import = module.import;
		if !import.entries().is_empty() {
			sections.push(elements::Section::Import(import));
		}
		let functions = module.functions;
		if !functions.entries().is_empty() {
			sections.push(elements::Section::Function(functions));
		}
		let table = module.table;
		if !table.entries().is_empty() {
			sections.push(elements::Section::Table(table));
		}
		let memory = module.memory;
		if !memory.entries().is_empty() {
			sections.push(elements::Section::Memory(memory));
		}
		let global = module.global;
		if !global.entries().is_empty() {
			sections.push(elements::Section::Global(global));
		}
		let export = module.export;
		if !export.entries().is_empty() {
			sections.push(elements::Section::Export(export));
		}
		if let Some(start) = module.start {
			sections.push(elements::Section::Start(start));
		}
		let element = module.element;
		if !element.entries().is_empty() {
			sections.push(elements::Section::Element(element));
		}
		let code = module.code;
		if !code.bodies().is_empty() {
			sections.push(elements::Section::Code(code));
		}
		let data = module.data;
		if !data.entries().is_empty() {
			sections.push(elements::Section::Data(data));
		}
		sections.extend(module.other);
		elements::Module::new(sections)
	}
}

impl ModuleBuilder {
	/// New empty module builder
	pub fn new() -> Self {
		ModuleBuilder::with_callback(Identity)
	}
}

impl Default for ModuleBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	/// New module builder with bound callback
	pub fn with_callback(callback: F) -> Self {
		ModuleBuilder { callback, module: Default::default() }
	}

	/// Builder from raw module
	pub fn with_module(mut self, module: elements::Module) -> Self {
		self.module = module.into();
		self
	}

	/// Fill module with sections from iterator
	pub fn with_sections<I>(mut self, sections: I) -> Self
	where
		I: IntoIterator<Item = elements::Section>,
	{
		self.module.other.extend(sections);
		self
	}

	/// Add additional section
	pub fn with_section(mut self, section: elements::Section) -> Self {
		self.module.other.push(section);
		self
	}

	/// Binds to the type section, creates additional types when required
	pub fn with_signatures(mut self, bindings: code::SignatureBindings) -> Self {
		self.push_signatures(bindings);
		self
	}

	/// Push stand-alone function definition, creating sections, signature and code blocks
	/// in corresponding sections.
	/// `FunctionDefinition` can be build using `builder::function` builder
	pub fn push_function(&mut self, func: code::FunctionDefinition) -> CodeLocation {
		let signature = func.signature;
		let body = func.code;

		let type_ref = self.resolve_type_ref(signature);

		self.module.functions.entries_mut().push(elements::Func::new(type_ref));
		let signature_index = self.module.functions.entries_mut().len() as u32 - 1;
		self.module.code.bodies_mut().push(body);
		let body_index = self.module.code.bodies_mut().len() as u32 - 1;

		if func.is_main {
			self.module.start = Some(body_index);
		}

		CodeLocation { signature: signature_index, body: body_index }
	}

	/// Push linear memory region
	pub fn push_memory(&mut self, mut memory: memory::MemoryDefinition) -> u32 {
		let entries = self.module.memory.entries_mut();
		entries.push(elements::MemoryType::new(memory.min, memory.max));
		let memory_index = (entries.len() - 1) as u32;
		for data in memory.data.drain(..) {
			self.module.data.entries_mut().push(elements::DataSegment::new(
				memory_index,
				Some(data.offset),
				data.values,
			))
		}
		memory_index
	}

	/// Push table
	pub fn push_table(&mut self, mut table: table::TableDefinition) -> u32 {
		let entries = self.module.table.entries_mut();
		entries.push(elements::TableType::new(table.min, table.max));
		let table_index = (entries.len() - 1) as u32;
		for entry in table.elements.drain(..) {
			self.module.element.entries_mut().push(elements::ElementSegment::new(
				table_index,
				Some(entry.offset),
				entry.values,
			))
		}
		table_index
	}

	/// Push global.
	pub fn push_global(&mut self, global: elements::GlobalEntry) -> u32 {
		let entries = self.module.global.entries_mut();
		entries.push(global);
		entries.len() as u32 - 1
	}

	fn resolve_type_ref(&mut self, signature: code::Signature) -> u32 {
		match signature {
			code::Signature::Inline(func_type) => {
				if let Some(existing_entry) =
					self.module.types.types().iter().enumerate().find(|(_idx, t)| {
						let elements::Type::Function(ref existing) = t;
						*existing == func_type
					}) {
					return existing_entry.0 as u32
				}
				self.module.types.types_mut().push(elements::Type::Function(func_type));
				self.module.types.types().len() as u32 - 1
			},
			code::Signature::TypeReference(type_ref) => type_ref,
		}
	}

	/// Push one function signature, returning it's calling index.
	/// Can create corresponding type in type section.
	pub fn push_signature(&mut self, signature: code::Signature) -> u32 {
		self.resolve_type_ref(signature)
	}

	/// Push signatures in the module, returning corresponding indices of pushed signatures
	pub fn push_signatures(&mut self, signatures: code::SignatureBindings) -> Vec<u32> {
		signatures.into_iter().map(|binding| self.resolve_type_ref(binding)).collect()
	}

	/// Push import entry to module. Note that this does not update calling indices in
	/// function bodies.
	pub fn push_import(&mut self, import: elements::ImportEntry) -> u32 {
		self.module.import.entries_mut().push(import);
		// todo: actually update calling addresses in function bodies
		// todo: also batch push

		self.module.import.entries_mut().len() as u32 - 1
	}

	/// Push export entry to module.
	pub fn push_export(&mut self, export: elements::ExportEntry) -> u32 {
		self.module.export.entries_mut().push(export);
		self.module.export.entries_mut().len() as u32 - 1
	}

	/// Add new function using dedicated builder
	pub fn function(self) -> FunctionBuilder<Self> {
		FunctionBuilder::with_callback(self)
	}

	/// Add new linear memory using dedicated builder
	pub fn memory(self) -> MemoryBuilder<Self> {
		MemoryBuilder::with_callback(self)
	}

	/// Add new table using dedicated builder
	pub fn table(self) -> TableBuilder<Self> {
		TableBuilder::with_callback(self)
	}

	/// Define functions section
	pub fn functions(self) -> SignaturesBuilder<Self> {
		SignaturesBuilder::with_callback(self)
	}

	/// With inserted export entry
	pub fn with_export(mut self, entry: elements::ExportEntry) -> Self {
		self.module.export.entries_mut().push(entry);
		self
	}

	/// With inserted import entry
	pub fn with_import(mut self, entry: elements::ImportEntry) -> Self {
		self.module.import.entries_mut().push(entry);
		self
	}

	/// Import entry builder
	/// # Examples
	/// ```
	/// use parity_wasm::builder::module;
	///
	/// let module = module()
	///    .import()
	///        .module("env")
	///        .field("memory")
	///        .external().memory(256, Some(256))
	///        .build()
	///    .build();
	///
	/// assert_eq!(module.import_section().expect("import section to exist").entries().len(), 1);
	/// ```
	pub fn import(self) -> import::ImportBuilder<Self> {
		import::ImportBuilder::with_callback(self)
	}

	/// With global variable
	pub fn with_global(mut self, global: elements::GlobalEntry) -> Self {
		self.module.global.entries_mut().push(global);
		self
	}

	/// With table
	pub fn with_table(mut self, table: elements::TableType) -> Self {
		self.module.table.entries_mut().push(table);
		self
	}

	/// Export entry builder
	/// # Examples
	/// ```
	/// use parity_wasm::builder::module;
	/// use parity_wasm::elements::Instruction::*;
	///
	/// let module = module()
	///    .global()
	///         .value_type().i32()
	///         .init_expr(I32Const(0))
	///         .build()
	///    .export()
	///        .field("_zero")
	///        .internal().global(0)
	///        .build()
	///    .build();
	///
	/// assert_eq!(module.export_section().expect("export section to exist").entries().len(), 1);
	/// ```
	pub fn export(self) -> export::ExportBuilder<Self> {
		export::ExportBuilder::with_callback(self)
	}

	/// Glboal entry builder
	/// # Examples
	/// ```
	/// use parity_wasm::builder::module;
	/// use parity_wasm::elements::Instruction::*;
	///
	/// let module = module()
	///    .global()
	///         .value_type().i32()
	///         .init_expr(I32Const(0))
	///         .build()
	///    .build();
	///
	/// assert_eq!(module.global_section().expect("global section to exist").entries().len(), 1);
	/// ```
	pub fn global(self) -> global::GlobalBuilder<Self> {
		global::GlobalBuilder::with_callback(self)
	}

	/// Add data segment to the builder
	pub fn with_data_segment(mut self, segment: elements::DataSegment) -> Self {
		self.module.data.entries_mut().push(segment);
		self
	}

	/// Data entry builder
	pub fn data(self) -> data::DataSegmentBuilder<Self> {
		data::DataSegmentBuilder::with_callback(self)
	}

	/// Build module (final step)
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.module.into())
	}
}

impl<F> Invoke<elements::FunctionSection> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, section: elements::FunctionSection) -> Self {
		self.with_section(elements::Section::Function(section))
	}
}

impl<F> Invoke<code::SignatureBindings> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, bindings: code::SignatureBindings) -> Self {
		self.with_signatures(bindings)
	}
}

impl<F> Invoke<code::FunctionDefinition> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, def: code::FunctionDefinition) -> Self {
		let mut b = self;
		b.push_function(def);
		b
	}
}

impl<F> Invoke<memory::MemoryDefinition> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, def: memory::MemoryDefinition) -> Self {
		let mut b = self;
		b.push_memory(def);
		b
	}
}

impl<F> Invoke<table::TableDefinition> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, def: table::TableDefinition) -> Self {
		let mut b = self;
		b.push_table(def);
		b
	}
}

impl<F> Invoke<elements::ImportEntry> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, entry: elements::ImportEntry) -> Self::Result {
		self.with_import(entry)
	}
}

impl<F> Invoke<elements::ExportEntry> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, entry: elements::ExportEntry) -> Self::Result {
		self.with_export(entry)
	}
}

impl<F> Invoke<elements::GlobalEntry> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, entry: elements::GlobalEntry) -> Self::Result {
		self.with_global(entry)
	}
}

impl<F> Invoke<elements::DataSegment> for ModuleBuilder<F>
where
	F: Invoke<elements::Module>,
{
	type Result = Self;

	fn invoke(self, segment: elements::DataSegment) -> Self {
		self.with_data_segment(segment)
	}
}

/// Start new module builder
/// # Examples
///
/// ```
/// use parity_wasm::builder;
///
/// let module = builder::module()
///     .function()
///         .signature().param().i32().build()
///         .body().build()
///         .build()
///     .build();
///
/// assert_eq!(module.type_section().expect("type section to exist").types().len(), 1);
/// assert_eq!(module.function_section().expect("function section to exist").entries().len(), 1);
/// assert_eq!(module.code_section().expect("code section to exist").bodies().len(), 1);
/// ```
pub fn module() -> ModuleBuilder {
	ModuleBuilder::new()
}

/// Start builder to extend existing module
pub fn from_module(module: elements::Module) -> ModuleBuilder {
	ModuleBuilder::new().with_module(module)
}

#[cfg(test)]
mod tests {

	use super::module;
	use crate::elements;

	#[test]
	fn smoky() {
		let module = module().build();
		assert_eq!(module.sections().len(), 0);
	}

	#[test]
	fn functions() {
		let module = module()
			.function()
			.signature()
			.param()
			.i32()
			.build()
			.body()
			.build()
			.build()
			.build();

		assert_eq!(module.type_section().expect("type section to exist").types().len(), 1);
		assert_eq!(
			module.function_section().expect("function section to exist").entries().len(),
			1
		);
		assert_eq!(module.code_section().expect("code section to exist").bodies().len(), 1);
	}

	#[test]
	fn export() {
		let module = module().export().field("call").internal().func(0).build().build();

		assert_eq!(module.export_section().expect("export section to exist").entries().len(), 1);
	}

	#[test]
	fn global() {
		let module = module()
			.global()
			.value_type()
			.i64()
			.mutable()
			.init_expr(elements::Instruction::I64Const(5))
			.build()
			.build();

		assert_eq!(module.global_section().expect("global section to exist").entries().len(), 1);
	}

	#[test]
	fn data() {
		let module = module()
			.data()
			.offset(elements::Instruction::I32Const(16))
			.value(vec![0u8, 15, 10, 5, 25])
			.build()
			.build();

		assert_eq!(module.data_section().expect("data section to exist").entries().len(), 1);
	}

	#[test]
	fn reuse_types() {
		let module = module()
			.function()
			.signature()
			.param()
			.i32()
			.build()
			.body()
			.build()
			.build()
			.function()
			.signature()
			.param()
			.i32()
			.build()
			.body()
			.build()
			.build()
			.build();

		assert_eq!(module.type_section().expect("type section failed").types().len(), 1);
	}
}
