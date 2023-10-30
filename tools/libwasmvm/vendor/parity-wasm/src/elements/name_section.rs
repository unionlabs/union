use crate::io;
use alloc::string::String;

use super::{
	index_map::IndexMap, Deserialize, Error, Module, Serialize, Type, VarUint32, VarUint7,
};

const NAME_TYPE_MODULE: u8 = 0;
const NAME_TYPE_FUNCTION: u8 = 1;
const NAME_TYPE_LOCAL: u8 = 2;

/// Debug name information.
#[derive(Clone, Debug, PartialEq)]
pub struct NameSection {
	/// Module name subsection.
	module: Option<ModuleNameSubsection>,

	/// Function name subsection.
	functions: Option<FunctionNameSubsection>,

	/// Local name subsection.
	locals: Option<LocalNameSubsection>,
}

impl NameSection {
	/// Creates a new name section.
	pub fn new(
		module: Option<ModuleNameSubsection>,
		functions: Option<FunctionNameSubsection>,
		locals: Option<LocalNameSubsection>,
	) -> Self {
		Self { module, functions, locals }
	}

	/// Module name subsection of this section.
	pub fn module(&self) -> Option<&ModuleNameSubsection> {
		self.module.as_ref()
	}

	/// Module name subsection of this section (mutable).
	pub fn module_mut(&mut self) -> &mut Option<ModuleNameSubsection> {
		&mut self.module
	}

	/// Functions name subsection of this section.
	pub fn functions(&self) -> Option<&FunctionNameSubsection> {
		self.functions.as_ref()
	}

	/// Functions name subsection of this section (mutable).
	pub fn functions_mut(&mut self) -> &mut Option<FunctionNameSubsection> {
		&mut self.functions
	}

	/// Local name subsection of this section.
	pub fn locals(&self) -> Option<&LocalNameSubsection> {
		self.locals.as_ref()
	}

	/// Local name subsection of this section (mutable).
	pub fn locals_mut(&mut self) -> &mut Option<LocalNameSubsection> {
		&mut self.locals
	}
}

impl NameSection {
	/// Deserialize a name section.
	pub fn deserialize<R: io::Read>(module: &Module, rdr: &mut R) -> Result<Self, Error> {
		let mut module_name: Option<ModuleNameSubsection> = None;
		let mut function_names: Option<FunctionNameSubsection> = None;
		let mut local_names: Option<LocalNameSubsection> = None;

		while let Ok(raw_subsection_type) = VarUint7::deserialize(rdr) {
			let subsection_type = raw_subsection_type.into();
			// deserialize the section size
			let size: usize = VarUint32::deserialize(rdr)?.into();

			match subsection_type {
				NAME_TYPE_MODULE => {
					if module_name.is_some() {
						return Err(Error::DuplicatedNameSubsections(NAME_TYPE_FUNCTION))
					}
					module_name = Some(ModuleNameSubsection::deserialize(rdr)?);
				},

				NAME_TYPE_FUNCTION => {
					if function_names.is_some() {
						return Err(Error::DuplicatedNameSubsections(NAME_TYPE_FUNCTION))
					}
					function_names = Some(FunctionNameSubsection::deserialize(module, rdr)?);
				},

				NAME_TYPE_LOCAL => {
					if local_names.is_some() {
						return Err(Error::DuplicatedNameSubsections(NAME_TYPE_LOCAL))
					}
					local_names = Some(LocalNameSubsection::deserialize(module, rdr)?);
				},

				_ => {
					// Consume the entire subsection size and drop it. This allows other sections to still be
					// consumed if there are any.
					let mut buf = vec![0; size];
					rdr.read(&mut buf)?;
				},
			};
		}

		Ok(Self { module: module_name, functions: function_names, locals: local_names })
	}
}

impl Serialize for NameSection {
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Error> {
		fn serialize_subsection<W: io::Write>(
			wtr: &mut W,
			name_type: u8,
			name_payload: &[u8],
		) -> Result<(), Error> {
			VarUint7::from(name_type).serialize(wtr)?;
			VarUint32::from(name_payload.len()).serialize(wtr)?;
			wtr.write(name_payload).map_err(Into::into)
		}

		if let Some(module_name_subsection) = self.module {
			let mut buffer = vec![];
			module_name_subsection.serialize(&mut buffer)?;
			serialize_subsection(wtr, NAME_TYPE_MODULE, &buffer)?;
		}

		if let Some(function_name_subsection) = self.functions {
			let mut buffer = vec![];
			function_name_subsection.serialize(&mut buffer)?;
			serialize_subsection(wtr, NAME_TYPE_FUNCTION, &buffer)?;
		}

		if let Some(local_name_subsection) = self.locals {
			let mut buffer = vec![];
			local_name_subsection.serialize(&mut buffer)?;
			serialize_subsection(wtr, NAME_TYPE_LOCAL, &buffer)?;
		}

		Ok(())
	}
}

/// The name of this module.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleNameSubsection {
	name: String,
}

impl ModuleNameSubsection {
	/// Create a new module name section with the specified name.
	pub fn new<S: Into<String>>(name: S) -> ModuleNameSubsection {
		ModuleNameSubsection { name: name.into() }
	}

	/// The name of this module.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// The name of this module (mutable).
	pub fn name_mut(&mut self) -> &mut String {
		&mut self.name
	}
}

impl Serialize for ModuleNameSubsection {
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Error> {
		self.name.serialize(wtr)
	}
}

impl Deserialize for ModuleNameSubsection {
	type Error = Error;

	fn deserialize<R: io::Read>(rdr: &mut R) -> Result<ModuleNameSubsection, Error> {
		let name = String::deserialize(rdr)?;
		Ok(ModuleNameSubsection { name })
	}
}

/// The names of the functions in this module.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FunctionNameSubsection {
	names: NameMap,
}

impl FunctionNameSubsection {
	/// A map from function indices to names.
	pub fn names(&self) -> &NameMap {
		&self.names
	}

	/// A map from function indices to names (mutable).
	pub fn names_mut(&mut self) -> &mut NameMap {
		&mut self.names
	}

	/// Deserialize names, making sure that all names correspond to functions.
	pub fn deserialize<R: io::Read>(
		module: &Module,
		rdr: &mut R,
	) -> Result<FunctionNameSubsection, Error> {
		let names = IndexMap::deserialize(module.functions_space(), rdr)?;
		Ok(FunctionNameSubsection { names })
	}
}

impl Serialize for FunctionNameSubsection {
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Error> {
		self.names.serialize(wtr)
	}
}

/// The names of the local variables in this module's functions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalNameSubsection {
	local_names: IndexMap<NameMap>,
}

impl LocalNameSubsection {
	/// A map from function indices to a map from variables indices to names.
	pub fn local_names(&self) -> &IndexMap<NameMap> {
		&self.local_names
	}

	/// A map from function indices to a map from variables indices to names
	/// (mutable).
	pub fn local_names_mut(&mut self) -> &mut IndexMap<NameMap> {
		&mut self.local_names
	}

	/// Deserialize names, making sure that all names correspond to local
	/// variables.
	pub fn deserialize<R: io::Read>(
		module: &Module,
		rdr: &mut R,
	) -> Result<LocalNameSubsection, Error> {
		let max_entry_space = module.functions_space();

		let max_signature_args = module
			.type_section()
			.map(|ts| {
				ts.types()
					.iter()
					.map(|x| {
						let Type::Function(ref func) = *x;
						func.params().len()
					})
					.max()
					.unwrap_or(0)
			})
			.unwrap_or(0);

		let max_locals = module
			.code_section()
			.map(|cs| {
				cs.bodies()
					.iter()
					.map(|f| f.locals().iter().map(|l| l.count() as usize).sum())
					.max()
					.unwrap_or(0)
			})
			.unwrap_or(0);

		let max_space = max_signature_args + max_locals;

		let deserialize_locals = |_: u32, rdr: &mut R| IndexMap::deserialize(max_space, rdr);

		let local_names = IndexMap::deserialize_with(max_entry_space, &deserialize_locals, rdr)?;
		Ok(LocalNameSubsection { local_names })
	}
}

impl Serialize for LocalNameSubsection {
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Error> {
		self.local_names.serialize(wtr)
	}
}

/// A map from indices to names.
pub type NameMap = IndexMap<String>;

#[cfg(test)]
mod tests {
	use super::*;

	// A helper function for the tests. Serialize a section, deserialize it,
	// and make sure it matches the original.
	fn serialize_test(original: NameSection) -> Vec<u8> {
		let mut buffer = vec![];
		original.serialize(&mut buffer).expect("serialize error");
		buffer
		// todo: add deserialization to this test
	}

	#[test]
	fn serialize_module_name() {
		let module_name_subsection = ModuleNameSubsection::new("my_mod");
		let original = NameSection::new(Some(module_name_subsection), None, None);
		serialize_test(original);
	}

	#[test]
	fn serialize_function_names() {
		let mut function_name_subsection = FunctionNameSubsection::default();
		function_name_subsection.names_mut().insert(0, "hello_world".to_string());
		let name_section = NameSection::new(None, Some(function_name_subsection), None);
		serialize_test(name_section);
	}

	#[test]
	fn serialize_local_names() {
		let mut local_name_subsection = LocalNameSubsection::default();
		let mut locals = NameMap::default();
		locals.insert(0, "msg".to_string());
		local_name_subsection.local_names_mut().insert(0, locals);

		let name_section = NameSection::new(None, None, Some(local_name_subsection));
		serialize_test(name_section);
	}

	#[test]
	fn serialize_all_subsections() {
		let module_name_subsection = ModuleNameSubsection::new("ModuleNameSubsection");

		let mut function_name_subsection = FunctionNameSubsection::default();
		function_name_subsection.names_mut().insert(0, "foo".to_string());
		function_name_subsection.names_mut().insert(1, "bar".to_string());

		let mut local_name_subsection = LocalNameSubsection::default();
		let mut locals = NameMap::default();
		locals.insert(0, "msg1".to_string());
		locals.insert(1, "msg2".to_string());
		local_name_subsection.local_names_mut().insert(0, locals);

		let name_section = NameSection::new(
			Some(module_name_subsection),
			Some(function_name_subsection),
			Some(local_name_subsection),
		);
		serialize_test(name_section);
	}

	#[test]
	fn deserialize_local_names() {
		let module = super::super::deserialize_file("./res/cases/v1/names_with_imports.wasm")
			.expect("Should be deserialized")
			.parse_names()
			.expect("Names to be parsed");

		let name_section = module.names_section().expect("name_section should be present");
		let local_names = name_section.locals().expect("local_name_section should be present");

		let locals = local_names.local_names().get(0).expect("entry #0 should be present");
		assert_eq!(locals.get(0).expect("entry #0 should be present"), "abc");

		let locals = local_names.local_names().get(1).expect("entry #1 should be present");
		assert_eq!(locals.get(0).expect("entry #0 should be present"), "def");
	}
}
