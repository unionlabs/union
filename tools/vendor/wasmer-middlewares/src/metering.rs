//! `metering` is a middleware for tracking how many operators are
//! executed in total and putting a limit on the total number of
//! operators executed. The WebAssemblt instance execution is stopped
//! when the limit is reached.
//!
//! # Example
//!
//! [See the `metering` detailed and complete
//! example](https://github.com/wasmerio/wasmer/blob/master/examples/metering.rs).

use loupe::{MemoryUsage, MemoryUsageTracker};
use std::convert::TryInto;
use std::fmt;
use std::mem;
use std::sync::{Arc, Mutex};
use wasmer::wasmparser::{Operator, Type as WpType, TypeOrFuncType as WpTypeOrFuncType};
use wasmer::{
    ExportIndex, FunctionMiddleware, GlobalInit, GlobalType, Instance, LocalFunctionIndex,
    MiddlewareError, MiddlewareReaderState, ModuleMiddleware, Mutability, Type,
};
use wasmer_types::{GlobalIndex, ModuleInfo};

#[derive(Clone, MemoryUsage)]
struct MeteringGlobalIndexes(GlobalIndex, GlobalIndex);

impl MeteringGlobalIndexes {
    /// The global index in the current module for remaining points.
    fn remaining_points(&self) -> GlobalIndex {
        self.0
    }

    /// The global index in the current module for a boolean indicating whether points are exhausted
    /// or not.
    /// This boolean is represented as a i32 global:
    ///   * 0: there are remaining points
    ///   * 1: points have been exhausted
    fn points_exhausted(&self) -> GlobalIndex {
        self.1
    }
}

impl fmt::Debug for MeteringGlobalIndexes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeteringGlobalIndexes")
            .field("remaining_points", &self.remaining_points())
            .field("points_exhausted", &self.points_exhausted())
            .finish()
    }
}

/// The module-level metering middleware.
///
/// # Panic
///
/// An instance of `Metering` should _not_ be shared among different
/// modules, since it tracks module-specific information like the
/// global index to store metering state. Attempts to use a `Metering`
/// instance from multiple modules will result in a panic.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use wasmer::{wasmparser::Operator, CompilerConfig};
/// use wasmer_middlewares::Metering;
///
/// fn create_metering_middleware(compiler_config: &mut dyn CompilerConfig) {
///     // Let's define a dummy cost function,
///     // which counts 1 for all operators.
///     let cost_function = |_operator: &Operator| -> u64 { 1 };
///
///     // Let's define the initial limit.
///     let initial_limit = 10;
///
///     // Let's creating the metering middleware.
///     let metering = Arc::new(Metering::new(
///         initial_limit,
///         cost_function
///     ));
///
///     // Finally, let's push the middleware.
///     compiler_config.push_middleware(metering);
/// }
/// ```
pub struct Metering<F: Fn(&Operator) -> u64 + Send + Sync> {
    /// Initial limit of points.
    initial_limit: u64,

    /// Function that maps each operator to a cost in "points".
    cost_function: Arc<F>,

    /// The global indexes for metering points.
    global_indexes: Mutex<Option<MeteringGlobalIndexes>>,
}

/// The function-level metering middleware.
pub struct FunctionMetering<F: Fn(&Operator) -> u64 + Send + Sync> {
    /// Function that maps each operator to a cost in "points".
    cost_function: Arc<F>,

    /// The global indexes for metering points.
    global_indexes: MeteringGlobalIndexes,

    /// Accumulated cost of the current basic block.
    accumulated_cost: u64,
}

/// Represents the type of the metering points, either `Remaining` or
/// `Exhausted`.
///
/// # Example
///
/// See the [`get_remaining_points`] function to get an example.
#[derive(Debug, PartialEq)]
pub enum MeteringPoints {
    /// The given number of metering points is left for the execution.
    /// If the value is 0, all points are consumed but the execution
    /// was not terminated.
    Remaining(u64),

    /// The execution was terminated because the metering points were
    /// exhausted.  You can recover from this state by setting the
    /// points via [`set_remaining_points`] and restart the execution.
    Exhausted,
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> Metering<F> {
    /// Creates a `Metering` middleware.
    pub fn new(initial_limit: u64, cost_function: F) -> Self {
        Self {
            initial_limit,
            cost_function: Arc::new(cost_function),
            global_indexes: Mutex::new(None),
        }
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> fmt::Debug for Metering<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metering")
            .field("initial_limit", &self.initial_limit)
            .field("cost_function", &"<function>")
            .field("global_indexes", &self.global_indexes)
            .finish()
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync + 'static> ModuleMiddleware for Metering<F> {
    /// Generates a `FunctionMiddleware` for a given function.
    fn generate_function_middleware(&self, _: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionMetering {
            cost_function: self.cost_function.clone(),
            global_indexes: self.global_indexes.lock().unwrap().clone().unwrap(),
            accumulated_cost: 0,
        })
    }

    /// Transforms a `ModuleInfo` struct in-place. This is called before application on functions begins.
    fn transform_module_info(&self, module_info: &mut ModuleInfo) {
        let mut global_indexes = self.global_indexes.lock().unwrap();

        if global_indexes.is_some() {
            panic!("Metering::transform_module_info: Attempting to use a `Metering` middleware from multiple modules.");
        }

        // Append a global for remaining points and initialize it.
        let remaining_points_global_index = module_info
            .globals
            .push(GlobalType::new(Type::I64, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I64Const(self.initial_limit as i64));

        module_info.exports.insert(
            "wasmer_metering_remaining_points".to_string(),
            ExportIndex::Global(remaining_points_global_index),
        );

        // Append a global for the exhausted points boolean and initialize it.
        let points_exhausted_global_index = module_info
            .globals
            .push(GlobalType::new(Type::I32, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I32Const(0));

        module_info.exports.insert(
            "wasmer_metering_points_exhausted".to_string(),
            ExportIndex::Global(points_exhausted_global_index),
        );

        *global_indexes = Some(MeteringGlobalIndexes(
            remaining_points_global_index,
            points_exhausted_global_index,
        ))
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync + 'static> MemoryUsage for Metering<F> {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.global_indexes.size_of_val(tracker)
            - mem::size_of_val(&self.global_indexes)
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> fmt::Debug for FunctionMetering<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionMetering")
            .field("cost_function", &"<function>")
            .field("global_indexes", &self.global_indexes)
            .finish()
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> FunctionMiddleware for FunctionMetering<F> {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        // Get the cost of the current operator, and add it to the accumulator.
        // This needs to be done before the metering logic, to prevent operators like `Call` from escaping metering in some
        // corner cases.
        self.accumulated_cost += (self.cost_function)(&operator);

        // Possible sources and targets of a branch. Finalize the cost of the previous basic block and perform necessary checks.
        match operator {
            Operator::Loop { .. } // loop headers are branch targets
            | Operator::End // block ends are branch targets
            | Operator::Else // "else" is the "end" of an if branch
            | Operator::Br { .. } // branch source
            | Operator::BrTable { .. } // branch source
            | Operator::BrIf { .. } // branch source
            | Operator::Call { .. } // function call - branch source
            | Operator::CallIndirect { .. } // function call - branch source
            | Operator::Return // end of function - branch source
            => {
                if self.accumulated_cost > 0 {
                    state.extend(&[
                        // if unsigned(globals[remaining_points_index]) < unsigned(self.accumulated_cost) { throw(); }
                        Operator::GlobalGet { global_index: self.global_indexes.remaining_points().as_u32() },
                        Operator::I64Const { value: self.accumulated_cost as i64 },
                        Operator::I64LtU,
                        Operator::If { ty: WpTypeOrFuncType::Type(WpType::EmptyBlockType) },
                        Operator::I32Const { value: 1 },
                        Operator::GlobalSet { global_index: self.global_indexes.points_exhausted().as_u32() },
                        Operator::Unreachable,
                        Operator::End,

                        // globals[remaining_points_index] -= self.accumulated_cost;
                        Operator::GlobalGet { global_index: self.global_indexes.remaining_points().as_u32() },
                        Operator::I64Const { value: self.accumulated_cost as i64 },
                        Operator::I64Sub,
                        Operator::GlobalSet { global_index: self.global_indexes.remaining_points().as_u32() },
                    ]);

                    self.accumulated_cost = 0;
                }
            }
            _ => {}
        }
        state.push_operator(operator);

        Ok(())
    }
}

/// Get the remaining points in an [`Instance`][wasmer::Instance].
///
/// Note: This can be used in a headless engine after an ahead-of-time
/// compilation as all required state lives in the instance.
///
/// # Panic
///
/// The [`Instance`][wasmer::Instance) must have been processed with
/// the [`Metering`] middleware at compile time, otherwise this will
/// panic.
///
/// # Example
///
/// ```rust
/// use wasmer::Instance;
/// use wasmer_middlewares::metering::{get_remaining_points, MeteringPoints};
///
/// /// Check whether the instance can continue to run based on the
/// /// number of remaining points.
/// fn can_continue_to_run(instance: &Instance) -> bool {
///     matches!(get_remaining_points(instance), MeteringPoints::Remaining(points) if points > 0)
/// }
/// ```
pub fn get_remaining_points(instance: &Instance) -> MeteringPoints {
    let exhausted: i32 = instance
        .exports
        .get_global("wasmer_metering_points_exhausted")
        .expect("Can't get `wasmer_metering_points_exhausted` from Instance")
        .get()
        .try_into()
        .expect("`wasmer_metering_points_exhausted` from Instance has wrong type");

    if exhausted > 0 {
        return MeteringPoints::Exhausted;
    }

    let points = instance
        .exports
        .get_global("wasmer_metering_remaining_points")
        .expect("Can't get `wasmer_metering_remaining_points` from Instance")
        .get()
        .try_into()
        .expect("`wasmer_metering_remaining_points` from Instance has wrong type");

    MeteringPoints::Remaining(points)
}

/// Set the new provided remaining points in an
/// [`Instance`][wasmer::Instance].
///
/// Note: This can be used in a headless engine after an ahead-of-time
/// compilation as all required state lives in the instance.
///
/// # Panic
///
/// The given [`Instance`][wasmer::Instance] must have been processed
/// with the [`Metering`] middleware at compile time, otherwise this
/// will panic.
///
/// # Example
///
/// ```rust
/// use wasmer::Instance;
/// use wasmer_middlewares::metering::set_remaining_points;
///
/// fn update_remaining_points(instance: &Instance) {
///     // The new limit.
///     let new_limit = 10;
///
///     // Update the remaining points to the `new_limit`.
///     set_remaining_points(instance, new_limit);
/// }
/// ```
pub fn set_remaining_points(instance: &Instance, points: u64) {
    instance
        .exports
        .get_global("wasmer_metering_remaining_points")
        .expect("Can't get `wasmer_metering_remaining_points` from Instance")
        .set(points.into())
        .expect("Can't set `wasmer_metering_remaining_points` in Instance");

    instance
        .exports
        .get_global("wasmer_metering_points_exhausted")
        .expect("Can't get `wasmer_metering_points_exhausted` from Instance")
        .set(0i32.into())
        .expect("Can't set `wasmer_metering_points_exhausted` in Instance");
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use wasmer::{imports, wat2wasm, CompilerConfig, Cranelift, Module, Store, Universal};

    fn cost_function(operator: &Operator) -> u64 {
        match operator {
            Operator::LocalGet { .. } | Operator::I32Const { .. } => 1,
            Operator::I32Add { .. } => 2,
            _ => 0,
        }
    }

    fn bytecode() -> Vec<u8> {
        wat2wasm(
            br#"
            (module
            (type $add_t (func (param i32) (result i32)))
            (func $add_one_f (type $add_t) (param $value i32) (result i32)
                local.get $value
                i32.const 1
                i32.add)
            (export "add_one" (func $add_one_f)))
            "#,
        )
        .unwrap()
        .into()
    }

    #[test]
    fn get_remaining_points_works() {
        let metering = Arc::new(Metering::new(10, cost_function));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(metering.clone());
        let store = Store::new(&Universal::new(compiler_config).engine());
        let module = Module::new(&store, bytecode()).unwrap();

        // Instantiate
        let instance = Instance::new(&module, &imports! {}).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(10)
        );

        // First call
        //
        // Calling add_one costs 4 points. Here are the details of how it has been computed:
        // * `local.get $value` is a `Operator::LocalGet` which costs 1 point;
        // * `i32.const` is a `Operator::I32Const` which costs 1 point;
        // * `i32.add` is a `Operator::I32Add` which costs 2 points.
        let add_one = instance
            .exports
            .get_function("add_one")
            .unwrap()
            .native::<i32, i32>()
            .unwrap();
        add_one.call(1).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(6)
        );

        // Second call
        add_one.call(1).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(2)
        );

        // Third call fails due to limit
        assert!(add_one.call(1).is_err());
        assert_eq!(get_remaining_points(&instance), MeteringPoints::Exhausted);
    }

    #[test]
    fn set_remaining_points_works() {
        let metering = Arc::new(Metering::new(10, cost_function));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(metering.clone());
        let store = Store::new(&Universal::new(compiler_config).engine());
        let module = Module::new(&store, bytecode()).unwrap();

        // Instantiate
        let instance = Instance::new(&module, &imports! {}).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(10)
        );
        let add_one = instance
            .exports
            .get_function("add_one")
            .unwrap()
            .native::<i32, i32>()
            .unwrap();

        // Increase a bit to have enough for 3 calls
        set_remaining_points(&instance, 12);

        // Ensure we can use the new points now
        add_one.call(1).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(8)
        );

        add_one.call(1).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(4)
        );

        add_one.call(1).unwrap();
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(0)
        );

        assert!(add_one.call(1).is_err());
        assert_eq!(get_remaining_points(&instance), MeteringPoints::Exhausted);

        // Add some points for another call
        set_remaining_points(&instance, 4);
        assert_eq!(
            get_remaining_points(&instance),
            MeteringPoints::Remaining(4)
        );
    }
}
