use crate::vmcontext::VMGlobalDefinition;
use loupe::MemoryUsage;
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::Mutex;
use thiserror::Error;
use wasmer_types::{GlobalType, Mutability, Type, Value, WasmValueType};

#[derive(Debug, MemoryUsage)]
/// A Global instance
pub struct Global {
    ty: GlobalType,
    // TODO: this box may be unnecessary
    vm_global_definition: Box<UnsafeCell<VMGlobalDefinition>>,
    // used to synchronize gets/sets
    lock: Mutex<()>,
}

/// # Safety
/// This is safe to send between threads because there is no-thread specific logic.
/// TODO: look into other reasons that make something not `Send`
unsafe impl Send for Global {}
/// # Safety
/// This is safe to share between threads because it uses a `Mutex` internally.
unsafe impl Sync for Global {}

/// Error type describing things that can go wrong when operating on Wasm Globals.
#[derive(Error, Debug, Clone, PartialEq, Hash)]
pub enum GlobalError {
    /// The error returned when attempting to set an immutable global.
    #[error("Attempted to set an immutable global")]
    ImmutableGlobalCannotBeSet,

    /// The error returned when attempting to operate on a global as a specific type
    /// that differs from the global's own type.
    #[error("Attempted to operate on a global of type {expected} as a global of type {found}")]
    IncorrectType {
        /// The type that the global is.
        expected: Type,
        /// The type that we were asked to use it as.
        found: Type,
    },
}

impl Global {
    /// Create a new, zero bit-pattern initialized global from a [`GlobalType`].
    pub fn new(global_type: GlobalType) -> Self {
        Self {
            ty: global_type,
            vm_global_definition: Box::new(UnsafeCell::new(VMGlobalDefinition::new())),
            lock: Mutex::new(()),
        }
    }

    /// Get the type of the global.
    pub fn ty(&self) -> &GlobalType {
        &self.ty
    }

    /// Get a pointer to the underlying definition used by the generated code.
    pub fn vmglobal(&self) -> NonNull<VMGlobalDefinition> {
        let ptr = self.vm_global_definition.as_ref() as *const UnsafeCell<VMGlobalDefinition>
            as *const VMGlobalDefinition as *mut VMGlobalDefinition;
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// Get a value from the global.
    // TODO(reftypes): the `&dyn Any` here for `Store` is a work-around for the fact
    // that `Store` is defined in `API` when we need it earlier. Ideally this should
    // be removed.
    pub fn get<T: WasmValueType>(&self, store: &dyn std::any::Any) -> Value<T> {
        let _global_guard = self.lock.lock().unwrap();
        unsafe {
            let definition = &*self.vm_global_definition.get();
            match self.ty().ty {
                Type::I32 => Value::I32(definition.to_i32()),
                Type::I64 => Value::I64(definition.to_i64()),
                Type::F32 => Value::F32(definition.to_f32()),
                Type::F64 => Value::F64(definition.to_f64()),
                Type::V128 => Value::V128(definition.to_u128()),
                Type::ExternRef => Value::ExternRef(definition.to_externref().into()),
                Type::FuncRef => {
                    let p = definition.to_u128() as i128;
                    if p as usize == 0 {
                        Value::FuncRef(None)
                    } else {
                        let v = T::read_value_from(store, &p);
                        Value::FuncRef(Some(v))
                    }
                }
            }
        }
    }

    /// Set a value for the global.
    ///
    /// # Safety
    /// The caller should check that the `val` comes from the same store as this global.
    pub unsafe fn set<T: WasmValueType>(&self, val: Value<T>) -> Result<(), GlobalError> {
        let _global_guard = self.lock.lock().unwrap();
        if self.ty().mutability != Mutability::Var {
            return Err(GlobalError::ImmutableGlobalCannotBeSet);
        }
        if val.ty() != self.ty().ty {
            return Err(GlobalError::IncorrectType {
                expected: self.ty.ty,
                found: val.ty(),
            });
        }
        self.set_unchecked(val)
    }

    /// Set a value from the global (unchecked)
    ///
    /// # Safety
    /// The caller should check that the `val` comes from the same store as this global.
    /// The caller should also ensure that this global is synchronized. Otherwise, use
    /// `set` instead.
    pub unsafe fn set_unchecked<T: WasmValueType>(&self, val: Value<T>) -> Result<(), GlobalError> {
        // ideally we'd use atomics for the global value rather than needing to lock it
        let definition = &mut *self.vm_global_definition.get();
        match val {
            Value::I32(i) => *definition.as_i32_mut() = i,
            Value::I64(i) => *definition.as_i64_mut() = i,
            Value::F32(f) => *definition.as_f32_mut() = f,
            Value::F64(f) => *definition.as_f64_mut() = f,
            Value::V128(x) => *definition.as_bytes_mut() = x.to_ne_bytes(),
            Value::ExternRef(r) => {
                let extern_ref = definition.as_externref_mut();
                extern_ref.ref_drop();
                *extern_ref = r.into()
            }
            Value::FuncRef(None) => *definition.as_u128_mut() = 0,
            Value::FuncRef(Some(r)) => {
                r.write_value_to(definition.as_u128_mut() as *mut u128 as *mut i128)
            }
        }
        Ok(())
    }
}
