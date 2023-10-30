use super::invoke::{Identity, Invoke};
use crate::elements;
use alloc::vec::Vec;

/// Table definition
#[derive(Debug, PartialEq, Default)]
pub struct TableDefinition {
	/// Minimum length
	pub min: u32,
	/// Maximum length, if any
	pub max: Option<u32>,
	/// Element segments, if any
	pub elements: Vec<TableEntryDefinition>,
}

/// Table elements entry definition
#[derive(Debug, PartialEq)]
pub struct TableEntryDefinition {
	/// Offset initialization expression
	pub offset: elements::InitExpr,
	/// Values of initialization
	pub values: Vec<u32>,
}

/// Table builder
pub struct TableBuilder<F = Identity> {
	callback: F,
	table: TableDefinition,
}

impl TableBuilder {
	/// New table builder
	pub fn new() -> Self {
		TableBuilder::with_callback(Identity)
	}
}

impl Default for TableBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> TableBuilder<F>
where
	F: Invoke<TableDefinition>,
{
	/// New table builder with callback in chained context
	pub fn with_callback(callback: F) -> Self {
		TableBuilder { callback, table: Default::default() }
	}

	/// Set/override minimum length
	pub fn with_min(mut self, min: u32) -> Self {
		self.table.min = min;
		self
	}

	/// Set/override maximum length
	pub fn with_max(mut self, max: Option<u32>) -> Self {
		self.table.max = max;
		self
	}

	/// Generate initialization expression and element values on specified index
	pub fn with_element(mut self, index: u32, values: Vec<u32>) -> Self {
		self.table.elements.push(TableEntryDefinition {
			offset: elements::InitExpr::new(vec![
				elements::Instruction::I32Const(index as i32),
				elements::Instruction::End,
			]),
			values,
		});
		self
	}

	/// Finalize current builder spawning resulting struct
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.table)
	}
}
