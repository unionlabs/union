/// Vtables are a collection of free function. Those functions are created in Go and
/// then assigned via function pointers that Rust can call. The functions do not
/// change during the lifetime of the object.
///
/// Since the functions are free, a single function is used for multiple instances.
/// In fact, in Go we create a single global vtable variable holding and owning the
/// function the vtables in Rust point to.
///
/// The `Vtable` trait is created to find those vtables troughout the codebase.
///
/// ## Nullability
///
/// Since all functions are represented as a function pointer, they are naturally
/// nullable. I.e. in Go we can always set them to `nil`. For this reason the functions
/// are all wrapped in Options, e.g.
///
/// ```
/// # use wasmvm::UnmanagedVector;
/// # struct IteratorReference;
/// # struct gas_meter_t;
/// pub struct IteratorVtable {
///     pub next: Option<
///         extern "C" fn(
///             iterator: IteratorReference,
///             gas_meter: *mut gas_meter_t,
///             gas_used: *mut u64,
///             key_out: *mut UnmanagedVector,
///             value_out: *mut UnmanagedVector,
///             err_msg_out: *mut UnmanagedVector,
///         ) -> i32,
///     >,
///     // ...
/// }
/// ```
///
/// Or to say it in the words of [the Rust documentation](https://doc.rust-lang.org/nomicon/ffi.html#the-nullable-pointer-optimization):
///
/// > So `Option<extern "C" fn(c_int) -> c_int>` is a correct way to represent a nullable function pointer using the C ABI (corresponding to the C type `int (*)(int))`.
///
/// Since all vtable fields are nullable, we can easily demand them
/// to have a Default implementation. This sometimes is handy when a
/// type is created in Rust and then filled in Go.
///
/// Missing vtable fields always indicate a lifecycle bug in wasmvm and
/// should be treated with a panic.
pub trait Vtable: Default {}
