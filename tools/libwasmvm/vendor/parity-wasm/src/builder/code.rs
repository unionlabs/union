use super::{
	invoke::{Identity, Invoke},
	misc::{ValueTypeBuilder, ValueTypesBuilder},
};
use crate::elements;
use alloc::vec::Vec;

/// Signature template description
pub enum Signature {
	TypeReference(u32),
	Inline(elements::FunctionType),
}

/// Signature builder
pub struct SignatureBuilder<F = Identity> {
	callback: F,
	signature: elements::FunctionType,
}

impl SignatureBuilder {
	/// New signature builder
	pub fn new() -> Self {
		SignatureBuilder::with_callback(Identity)
	}
}

impl Default for SignatureBuilder {
	fn default() -> Self {
		Self::new()
	}
}
impl<F> SignatureBuilder<F>
where
	F: Invoke<elements::FunctionType>,
{
	/// New builder with callback function specified
	pub fn with_callback(callback: F) -> Self {
		SignatureBuilder { callback, signature: elements::FunctionType::default() }
	}

	/// Add argument to signature builder
	pub fn with_param(mut self, value_type: elements::ValueType) -> Self {
		self.signature.params_mut().push(value_type);
		self
	}

	/// Add multiple arguments to signature builder
	pub fn with_params<I>(mut self, value_types: I) -> Self
	where
		I: IntoIterator<Item = elements::ValueType>,
	{
		self.signature.params_mut().extend(value_types);
		self
	}

	/// Start build new argument
	pub fn param(self) -> ValueTypeBuilder<Self> {
		ValueTypeBuilder::with_callback(self)
	}

	/// Start build multiple arguments
	pub fn params(self) -> ValueTypesBuilder<Self> {
		ValueTypesBuilder::with_callback(self)
	}

	/// Add result to signature builder
	pub fn with_result(mut self, value_type: elements::ValueType) -> Self {
		self.signature.results_mut().push(value_type);
		self
	}

	/// Add multiple results to signature builder
	pub fn with_results<I>(mut self, value_types: I) -> Self
	where
		I: IntoIterator<Item = elements::ValueType>,
	{
		self.signature.results_mut().extend(value_types);
		self
	}

	/// Start building new result
	pub fn result(self) -> ValueTypeBuilder<Self> {
		ValueTypeBuilder::with_callback(self)
	}

	/// Start building multiple results
	pub fn results(self) -> ValueTypeBuilder<Self> {
		ValueTypeBuilder::with_callback(self)
	}

	/// Finish current builder
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.signature)
	}

	/// Finish current builder returning intermediate `Signature` struct
	pub fn build_sig(self) -> Signature {
		Signature::Inline(self.signature)
	}
}

impl<F, I> Invoke<I> for SignatureBuilder<F>
where
	F: Invoke<elements::FunctionType>,
	I: IntoIterator<Item = elements::ValueType>,
{
	type Result = Self;

	fn invoke(self, args: I) -> Self {
		self.with_params(args)
	}
}

impl<F> Invoke<elements::ValueType> for SignatureBuilder<F>
where
	F: Invoke<elements::FunctionType>,
{
	type Result = Self;

	fn invoke(self, arg: elements::ValueType) -> Self {
		self.with_result(arg)
	}
}

/// Type (signature) reference builder (for function/import/indirect call)
pub struct TypeRefBuilder<F = Identity> {
	callback: F,
	type_ref: u32,
}

impl<F> TypeRefBuilder<F>
where
	F: Invoke<u32>,
{
	/// New builder chained with specified callback
	pub fn with_callback(callback: F) -> Self {
		TypeRefBuilder { callback, type_ref: 0 }
	}

	/// Set/override of type reference
	pub fn val(mut self, val: u32) -> Self {
		self.type_ref = val;
		self
	}

	/// Finish current builder
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.type_ref)
	}
}

/// Multiple signatures builder
pub struct SignaturesBuilder<F = Identity> {
	callback: F,
	section: Vec<Signature>,
}

impl SignaturesBuilder {
	/// New empty functions section builder
	pub fn new() -> Self {
		SignaturesBuilder::with_callback(Identity)
	}
}

impl Default for SignaturesBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> SignaturesBuilder<F> {
	/// New builder chained with specified callback
	pub fn with_callback(callback: F) -> Self {
		SignaturesBuilder { callback, section: Vec::new() }
	}

	/// Push new signature into the builder output
	pub fn with_signature(mut self, signature: Signature) -> Self {
		self.section.push(signature);
		self
	}

	/// Start building new signature with `TypeRefBuilder`
	pub fn type_ref(self) -> TypeRefBuilder<Self> {
		TypeRefBuilder::with_callback(self)
	}
}

impl<F> SignaturesBuilder<F>
where
	F: Invoke<SignatureBindings>,
{
	/// Start building new signature with dedicated builder
	pub fn signature(self) -> SignatureBuilder<Self> {
		SignatureBuilder::with_callback(self)
	}
}

impl<F> Invoke<elements::FunctionType> for SignaturesBuilder<F> {
	type Result = Self;

	fn invoke(self, signature: elements::FunctionType) -> Self {
		self.with_signature(Signature::Inline(signature))
	}
}

impl<F> Invoke<u32> for SignaturesBuilder<F> {
	type Result = Self;

	fn invoke(self, type_ref: u32) -> Self {
		self.with_signature(Signature::TypeReference(type_ref))
	}
}

impl<F> SignaturesBuilder<F>
where
	F: Invoke<elements::FunctionSection>,
{
	/// Finalize builder spawning element
	pub fn build(self) -> F::Result {
		let mut result = elements::FunctionSection::default();
		for f in self.section.into_iter() {
			if let Signature::TypeReference(type_ref) = f {
				result.entries_mut().push(elements::Func::new(type_ref));
			} else {
				unreachable!(); // never possible with current generics impl-s
			}
		}
		self.callback.invoke(result)
	}
}

/// Signature bindings
pub type SignatureBindings = Vec<Signature>;

impl<F> SignaturesBuilder<F>
where
	F: Invoke<SignatureBindings>,
{
	/// Bind signature list
	pub fn bind(self) -> F::Result {
		self.callback.invoke(self.section)
	}
}

/// Function body (code) builder
pub struct FuncBodyBuilder<F = Identity> {
	callback: F,
	body: elements::FuncBody,
}

impl<F> FuncBodyBuilder<F> {
	/// New body (code) builder given the chain callback
	pub fn with_callback(callback: F) -> Self {
		FuncBodyBuilder {
			callback,
			body: elements::FuncBody::new(Vec::new(), elements::Instructions::empty()),
		}
	}
}

impl<F> FuncBodyBuilder<F>
where
	F: Invoke<elements::FuncBody>,
{
	/// Set/override entirely with FuncBody struct
	pub fn with_func(mut self, func: elements::FuncBody) -> Self {
		self.body = func;
		self
	}

	/// Extend function local list with new entries
	pub fn with_locals<I>(mut self, locals: I) -> Self
	where
		I: IntoIterator<Item = elements::Local>,
	{
		self.body.locals_mut().extend(locals);
		self
	}

	/// Set code of the function
	pub fn with_instructions(mut self, instructions: elements::Instructions) -> Self {
		*self.body.code_mut() = instructions;
		self
	}

	/// Finish current builder spawning resulting struct
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.body)
	}
}

/// Function definition (extended structure to specify function entirely, incl. signature, mainness and code)
pub struct FunctionDefinition {
	/// Is this function is start function
	pub is_main: bool,
	/// Signature description
	pub signature: Signature,
	/// Body (code) of the function
	pub code: elements::FuncBody,
}

impl Default for FunctionDefinition {
	fn default() -> Self {
		FunctionDefinition {
			is_main: false,
			signature: Signature::TypeReference(0),
			code: elements::FuncBody::empty(),
		}
	}
}

/// Function definition builder
pub struct FunctionBuilder<F = Identity> {
	callback: F,
	func: FunctionDefinition,
}

impl FunctionBuilder {
	/// New function builder
	pub fn new() -> Self {
		FunctionBuilder::with_callback(Identity)
	}
}

impl Default for FunctionBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl<F> FunctionBuilder<F>
where
	F: Invoke<FunctionDefinition>,
{
	/// New function builder with chained callback
	pub fn with_callback(callback: F) -> Self {
		FunctionBuilder { callback, func: Default::default() }
	}

	/// Set that this function is main entry point
	pub fn main(mut self) -> Self {
		self.func.is_main = true;
		self
	}

	/// Start signature builder of the function
	pub fn signature(self) -> SignatureBuilder<Self> {
		SignatureBuilder::with_callback(self)
	}

	/// Override current signature entirely with new one from known struct
	pub fn with_signature(mut self, signature: Signature) -> Self {
		self.func.signature = signature;
		self
	}

	/// Start code (body) builder
	pub fn body(self) -> FuncBodyBuilder<Self> {
		FuncBodyBuilder::with_callback(self)
	}

	/// Set body (code) for this function
	pub fn with_body(mut self, body: elements::FuncBody) -> Self {
		self.func.code = body;
		self
	}

	/// Finalize current builder spawning resulting struct in the callback
	pub fn build(self) -> F::Result {
		self.callback.invoke(self.func)
	}
}

impl<F> Invoke<elements::FunctionType> for FunctionBuilder<F>
where
	F: Invoke<FunctionDefinition>,
{
	type Result = Self;

	fn invoke(self, signature: elements::FunctionType) -> Self {
		self.with_signature(Signature::Inline(signature))
	}
}

impl<F> Invoke<u32> for FunctionBuilder<F>
where
	F: Invoke<FunctionDefinition>,
{
	type Result = Self;

	fn invoke(self, type_ref: u32) -> Self {
		self.with_signature(Signature::TypeReference(type_ref))
	}
}

impl<F> Invoke<elements::FuncBody> for FunctionBuilder<F>
where
	F: Invoke<FunctionDefinition>,
{
	type Result = Self;

	fn invoke(self, body: elements::FuncBody) -> Self::Result {
		self.with_body(body)
	}
}

/// New builder of signature list
pub fn signatures() -> SignaturesBuilder {
	SignaturesBuilder::new()
}

/// New signature builder
pub fn signature() -> SignatureBuilder {
	SignatureBuilder::new()
}

/// New builder of function (signature & body)
pub fn function() -> FunctionBuilder {
	FunctionBuilder::new()
}

#[cfg(test)]
mod tests {

	use super::{function, signatures};
	use crate::elements;

	#[test]
	fn example() {
		let result = signatures().type_ref().val(1).build().build();

		assert_eq!(result.entries().len(), 1);

		let result = signatures()
			.signature()
			.param()
			.i32()
			.param()
			.i32()
			.result()
			.i64()
			.build()
			.bind();

		assert_eq!(result.len(), 1);
	}

	#[test]
	fn func_example() {
		let func = function()
			.signature()
			.param()
			.i32()
			.result()
			.i32()
			.build()
			.body()
			.with_instructions(elements::Instructions::empty())
			.build()
			.build();

		assert_eq!(func.code.locals().len(), 0);
		assert_eq!(func.code.code().elements().len(), 1);
	}

	#[test]
	fn func_example_multi_result() {
		let func = function()
			.signature()
			.param()
			.i32()
			.result()
			.i32()
			.result()
			.i32()
			.build()
			.body()
			.with_instructions(elements::Instructions::empty())
			.build()
			.build();

		assert_eq!(func.code.locals().len(), 0);
		assert_eq!(func.code.code().elements().len(), 1);
	}
}
