use super::invoke::{Identity, Invoke};
use crate::elements;
use alloc::{borrow::ToOwned, string::String};

/// Import builder
pub struct ImportBuilder<F = Identity> {
	callback: F,
	module: String,
	field: String,
	binding: elements::External,
}

impl ImportBuilder {
	/// New import builder
	pub fn new() -> Self {
		ImportBuilder::with_callback(Identity)
	}
}

impl Default for ImportBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> ImportBuilder<F> {
	/// New import builder with callback (in chained context)
	pub fn with_callback(callback: F) -> Self {
		ImportBuilder {
			callback,
			module: String::new(),
			field: String::new(),
			binding: elements::External::Function(0),
		}
	}

	/// Set/override module name
	pub fn module(mut self, name: &str) -> Self {
		self.module = name.to_owned();
		self
	}

	/// Set/override field name
	pub fn field(mut self, name: &str) -> Self {
		self.field = name.to_owned();
		self
	}

	/// Set/override both module name and field name
	pub fn path(self, module: &str, field: &str) -> Self {
		self.module(module).field(field)
	}

	/// Set/override external mapping for this import
	pub fn with_external(mut self, external: elements::External) -> Self {
		self.binding = external;
		self
	}

	/// Start new external mapping builder
	pub fn external(self) -> ImportExternalBuilder<Self> {
		ImportExternalBuilder::with_callback(self)
	}
}

impl<F> ImportBuilder<F>
where
	F: Invoke<elements::ImportEntry>,
{
	/// Finalize current builder spawning the resulting struct
	pub fn build(self) -> F::Result {
		self.callback
			.invoke(elements::ImportEntry::new(self.module, self.field, self.binding))
	}
}

impl<F> Invoke<elements::External> for ImportBuilder<F> {
	type Result = Self;
	fn invoke(self, val: elements::External) -> Self {
		self.with_external(val)
	}
}

/// Import to external mapping builder
pub struct ImportExternalBuilder<F = Identity> {
	callback: F,
	binding: elements::External,
}

impl<F> ImportExternalBuilder<F>
where
	F: Invoke<elements::External>,
{
	/// New import to external mapping builder with callback (in chained context)
	pub fn with_callback(callback: F) -> Self {
		ImportExternalBuilder { callback, binding: elements::External::Function(0) }
	}

	/// Function mapping with type reference
	pub fn func(mut self, index: u32) -> F::Result {
		self.binding = elements::External::Function(index);
		self.callback.invoke(self.binding)
	}

	/// Memory mapping with specified limits
	pub fn memory(mut self, min: u32, max: Option<u32>) -> F::Result {
		self.binding = elements::External::Memory(elements::MemoryType::new(min, max));
		self.callback.invoke(self.binding)
	}

	/// Table mapping with specified limits
	pub fn table(mut self, min: u32, max: Option<u32>) -> F::Result {
		self.binding = elements::External::Table(elements::TableType::new(min, max));
		self.callback.invoke(self.binding)
	}

	/// Global mapping with speciifed type and mutability
	pub fn global(mut self, value_type: elements::ValueType, is_mut: bool) -> F::Result {
		self.binding = elements::External::Global(elements::GlobalType::new(value_type, is_mut));
		self.callback.invoke(self.binding)
	}
}

/// New builder for import entry
pub fn import() -> ImportBuilder {
	ImportBuilder::new()
}

#[cfg(test)]
mod tests {
	use super::import;

	#[test]
	fn example() {
		let entry =
			import().module("env").field("memory").external().memory(256, Some(256)).build();

		assert_eq!(entry.module(), "env");
		assert_eq!(entry.field(), "memory");
	}
}
