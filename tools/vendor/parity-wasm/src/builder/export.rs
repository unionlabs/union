use super::invoke::{Identity, Invoke};
use crate::elements;
use alloc::{borrow::ToOwned, string::String};

/// Export entry builder
pub struct ExportBuilder<F = Identity> {
	callback: F,
	field: String,
	binding: elements::Internal,
}

impl ExportBuilder {
	/// New export builder
	pub fn new() -> Self {
		ExportBuilder::with_callback(Identity)
	}
}

impl Default for ExportBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> ExportBuilder<F> {
	/// New export entry builder in the specified chained context
	pub fn with_callback(callback: F) -> Self {
		ExportBuilder { callback, field: String::new(), binding: elements::Internal::Function(0) }
	}

	/// Set the field name of the export entry
	pub fn field(mut self, field: &str) -> Self {
		self.field = field.to_owned();
		self
	}

	/// Specify the internal module mapping for this entry
	pub fn with_internal(mut self, external: elements::Internal) -> Self {
		self.binding = external;
		self
	}

	/// Start the internal builder for this export entry
	pub fn internal(self) -> ExportInternalBuilder<Self> {
		ExportInternalBuilder::with_callback(self)
	}
}

impl<F> ExportBuilder<F>
where
	F: Invoke<elements::ExportEntry>,
{
	/// Finalize export entry builder spawning the resulting struct
	pub fn build(self) -> F::Result {
		self.callback.invoke(elements::ExportEntry::new(self.field, self.binding))
	}
}

impl<F> Invoke<elements::Internal> for ExportBuilder<F> {
	type Result = Self;
	fn invoke(self, val: elements::Internal) -> Self {
		self.with_internal(val)
	}
}

/// Internal mapping builder for export entry
pub struct ExportInternalBuilder<F = Identity> {
	callback: F,
	binding: elements::Internal,
}

impl<F> ExportInternalBuilder<F>
where
	F: Invoke<elements::Internal>,
{
	/// New export entry internal mapping for the chained context
	pub fn with_callback(callback: F) -> Self {
		ExportInternalBuilder { callback, binding: elements::Internal::Function(0) }
	}

	/// Map to function by index
	pub fn func(mut self, index: u32) -> F::Result {
		self.binding = elements::Internal::Function(index);
		self.callback.invoke(self.binding)
	}

	/// Map to memory
	pub fn memory(mut self, index: u32) -> F::Result {
		self.binding = elements::Internal::Memory(index);
		self.callback.invoke(self.binding)
	}

	/// Map to table
	pub fn table(mut self, index: u32) -> F::Result {
		self.binding = elements::Internal::Table(index);
		self.callback.invoke(self.binding)
	}

	/// Map to global
	pub fn global(mut self, index: u32) -> F::Result {
		self.binding = elements::Internal::Global(index);
		self.callback.invoke(self.binding)
	}
}

/// New builder for export entry
pub fn export() -> ExportBuilder {
	ExportBuilder::new()
}

#[cfg(test)]
mod tests {
	use super::export;

	#[test]
	fn example() {
		let entry = export().field("memory").internal().memory(0).build();
		assert_eq!(entry.field(), "memory");
	}
}
