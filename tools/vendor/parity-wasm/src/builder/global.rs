use super::{
	invoke::{Identity, Invoke},
	misc::ValueTypeBuilder,
};

use crate::elements;

/// Global builder
pub struct GlobalBuilder<F = Identity> {
	callback: F,
	value_type: elements::ValueType,
	is_mutable: bool,
	init_expr: elements::InitExpr,
}

impl GlobalBuilder {
	/// New global builder
	pub fn new() -> Self {
		GlobalBuilder::with_callback(Identity)
	}
}

impl Default for GlobalBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> GlobalBuilder<F> {
	/// New global builder with callback (in chained context)
	pub fn with_callback(callback: F) -> Self {
		GlobalBuilder {
			callback,
			value_type: elements::ValueType::I32,
			init_expr: elements::InitExpr::empty(),
			is_mutable: false,
		}
	}

	/// Set/override resulting global type
	pub fn with_type(mut self, value_type: elements::ValueType) -> Self {
		self.value_type = value_type;
		self
	}

	/// Set mutabilty to true
	pub fn mutable(mut self) -> Self {
		self.is_mutable = true;
		self
	}

	/// Set initialization expression instruction for this global (`end` instruction will be added automatically)
	pub fn init_expr(mut self, instruction: elements::Instruction) -> Self {
		self.init_expr = elements::InitExpr::new(vec![instruction, elements::Instruction::End]);
		self
	}

	/// Start value type builder
	pub fn value_type(self) -> ValueTypeBuilder<Self> {
		ValueTypeBuilder::with_callback(self)
	}
}

impl<F> GlobalBuilder<F>
where
	F: Invoke<elements::GlobalEntry>,
{
	/// Finalize current builder spawning resulting struct
	pub fn build(self) -> F::Result {
		self.callback.invoke(elements::GlobalEntry::new(
			elements::GlobalType::new(self.value_type, self.is_mutable),
			self.init_expr,
		))
	}
}

impl<F> Invoke<elements::ValueType> for GlobalBuilder<F> {
	type Result = Self;
	fn invoke(self, the_type: elements::ValueType) -> Self {
		self.with_type(the_type)
	}
}

/// New builder for export entry
pub fn global() -> GlobalBuilder {
	GlobalBuilder::new()
}

#[cfg(test)]
mod tests {
	use super::global;
	use crate::elements;

	#[test]
	fn example() {
		let entry = global().value_type().i32().build();
		assert_eq!(entry.global_type().content_type(), elements::ValueType::I32);
		assert!(!entry.global_type().is_mutable());
	}
}
