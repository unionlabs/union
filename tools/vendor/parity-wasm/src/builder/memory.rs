use super::invoke::{Identity, Invoke};
use crate::elements;
use alloc::vec::Vec;

/// Memory definition struct
#[derive(Debug, PartialEq)]
pub struct MemoryDefinition {
	/// Minimum memory size
	pub min: u32,
	/// Maximum memory size
	pub max: Option<u32>,
	/// Memory data segments (static regions)
	pub data: Vec<MemoryDataDefinition>,
}

/// Memory static region entry definition
#[derive(Debug, PartialEq)]
pub struct MemoryDataDefinition {
	/// Segment initialization expression for offset
	pub offset: elements::InitExpr,
	/// Raw bytes of static region
	pub values: Vec<u8>,
}

/// Memory and static regions builder
pub struct MemoryBuilder<F = Identity> {
	callback: F,
	memory: MemoryDefinition,
}

impl MemoryBuilder {
	/// New memory builder
	pub fn new() -> Self {
		MemoryBuilder::with_callback(Identity)
	}
}

impl Default for MemoryBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> MemoryBuilder<F>
where
	F: Invoke<MemoryDefinition>,
{
	/// New memory builder with callback (in chained context)
	pub fn with_callback(callback: F) -> Self {
		MemoryBuilder { callback, memory: Default::default() }
	}

	/// Set/override minimum size
	pub fn with_min(mut self, min: u32) -> Self {
		self.memory.min = min;
		self
	}

	/// Set/override maximum size
	pub fn with_max(mut self, max: Option<u32>) -> Self {
		self.memory.max = max;
		self
	}

	/// Push new static region with initialized offset expression and raw bytes
	pub fn with_data(mut self, index: u32, values: Vec<u8>) -> Self {
		self.memory.data.push(MemoryDataDefinition {
			offset: elements::InitExpr::new(vec![
				elements::Instruction::I32Const(index as i32),
				elements::Instruction::End,
			]),
			values,
		});
		self
	}

	/// Finalize current builder, spawning resulting struct
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.memory)
	}
}

impl Default for MemoryDefinition {
	fn default() -> Self {
		MemoryDefinition { min: 1, max: None, data: Vec::new() }
	}
}
