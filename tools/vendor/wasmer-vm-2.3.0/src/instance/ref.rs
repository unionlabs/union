use super::Instance;
use loupe::{MemoryUsage, MemoryUsageTracker};
use std::alloc::Layout;
use std::convert::TryFrom;
use std::mem;
use std::ptr::{self, NonNull};
use std::sync::{Arc, Weak};

/// Dynamic instance allocation.
///
/// This structure has a C representation because `Instance` is
/// dynamically-sized, and the `instance` field must be last.
///
/// This `InstanceRef` must be freed with [`InstanceInner::deallocate_instance`]
/// if and only if it has been set correctly. The `Drop` implementation of
/// [`InstanceInner`] calls its `deallocate_instance` method without
/// checking if this property holds, only when `Self.strong` is equal to 1.
#[derive(Debug)]
#[repr(C)]
struct InstanceInner {
    /// The layout of `Instance` (which can vary).
    instance_layout: Layout,

    /// The `Instance` itself. It must be the last field of
    /// `InstanceRef` since `Instance` is dyamically-sized.
    ///
    /// `Instance` must not be dropped manually by Rust, because it's
    /// allocated manually with `alloc` and a specific layout (Rust
    /// would be able to drop `Instance` itself but it will imply a
    /// memory leak because of `alloc`).
    ///
    /// No one in the code has a copy of the `Instance`'s
    /// pointer. `Self` is the only one.
    instance: NonNull<Instance>,
}

impl InstanceInner {
    /// Deallocate `Instance`.
    ///
    /// # Safety
    ///
    /// `Self.instance` must be correctly set and filled before being
    /// dropped and deallocated.
    unsafe fn deallocate_instance(&mut self) {
        let instance_ptr = self.instance.as_ptr();

        ptr::drop_in_place(instance_ptr);
        std::alloc::dealloc(instance_ptr as *mut u8, self.instance_layout);
    }

    /// Get a reference to the `Instance`.
    #[inline]
    pub(crate) fn as_ref(&self) -> &Instance {
        // SAFETY: The pointer is properly aligned, it is
        // “dereferencable”, it points to an initialized memory of
        // `Instance`, and the reference has the lifetime `'a`.
        unsafe { self.instance.as_ref() }
    }

    #[inline]
    pub(super) fn as_mut(&mut self) -> &mut Instance {
        unsafe { self.instance.as_mut() }
    }
}

impl PartialEq for InstanceInner {
    /// Two `InstanceInner` are equal if and only if
    /// `Self.instance` points to the same location.
    fn eq(&self, other: &Self) -> bool {
        self.instance == other.instance
    }
}

impl Drop for InstanceInner {
    /// Drop the `InstanceInner`.
    fn drop(&mut self) {
        unsafe { Self::deallocate_instance(self) };
    }
}

/// TODO: Review this super carefully.
unsafe impl Send for InstanceInner {}
unsafe impl Sync for InstanceInner {}

impl MemoryUsage for InstanceInner {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.instance_layout.size_of_val(tracker)
            - mem::size_of_val(&self.instance_layout)
            + self.as_ref().size_of_val(tracker)
            - mem::size_of_val(&self.instance)
    }
}

/// An `InstanceRef` is responsible to properly deallocate,
/// and to give access to an `Instance`, in such a way that `Instance`
/// is unique, can be shared, safely, across threads, without
/// duplicating the pointer in multiple locations. `InstanceRef`
/// must be the only “owner” of an `Instance`.
///
/// Consequently, one must not share `Instance` but
/// `InstanceRef`. It acts like an Atomically Reference Counter
/// to `Instance`. In short, `InstanceRef` is roughly a
/// simplified version of `std::sync::Arc`.
///
/// Note for the curious reader: [`InstanceAllocator::new`]
/// and [`InstanceHandle::new`] will respectively allocate a proper
/// `Instance` and will fill it correctly.
///
/// A little bit of background: The initial goal was to be able to
/// share an [`Instance`] between an [`InstanceHandle`] and the module
/// exports, so that one can drop a [`InstanceHandle`] but still being
/// able to use the exports properly.
#[derive(Debug, PartialEq, Clone, MemoryUsage)]
pub struct InstanceRef(Arc<InstanceInner>);

impl InstanceRef {
    /// Create a new `InstanceRef`. It allocates nothing. It fills
    /// nothing. The `Instance` must be already valid and
    /// filled.
    ///
    /// # Safety
    ///
    /// `instance` must a non-null, non-dangling, properly aligned,
    /// and correctly initialized pointer to `Instance`. See
    /// [`InstanceAllocator`] for an example of how to correctly use
    /// this API.
    pub(super) unsafe fn new(instance: NonNull<Instance>, instance_layout: Layout) -> Self {
        Self(Arc::new(InstanceInner {
            instance_layout,
            instance,
        }))
    }

    /// Get a reference to the `Instance`.
    #[inline]
    pub(crate) fn as_ref(&self) -> &Instance {
        (&*self.0).as_ref()
    }

    /// Only succeeds if ref count is 1.
    #[inline]
    pub(super) fn as_mut(&mut self) -> Option<&mut Instance> {
        Some(Arc::get_mut(&mut self.0)?.as_mut())
    }

    /// Like [`InstanceRef::as_mut`] but always succeeds.
    /// May cause undefined behavior if used improperly.
    ///
    /// # Safety
    /// It is the caller's responsibility to ensure exclusivity and synchronization of the
    /// instance before calling this function. No other pointers to any Instance data
    /// should be dereferenced for the lifetime of the returned `&mut Instance`.
    #[inline]
    pub(super) unsafe fn as_mut_unchecked(&mut self) -> &mut Instance {
        let ptr: *mut InstanceInner = Arc::as_ptr(&self.0) as *mut _;
        (&mut *ptr).as_mut()
    }
}

/// A weak instance ref. This type does not keep the underlying `Instance` alive
/// but can be converted into a full `InstanceRef` if the underlying `Instance` hasn't
/// been deallocated.
#[derive(Debug, Clone)]
pub struct WeakInstanceRef(Weak<InstanceInner>);

impl PartialEq for WeakInstanceRef {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl WeakInstanceRef {
    /// Try to convert into a strong, `InstanceRef`.
    pub fn upgrade(&self) -> Option<InstanceRef> {
        let inner = self.0.upgrade()?;
        Some(InstanceRef(inner))
    }
}

impl MemoryUsage for WeakInstanceRef {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + if let Some(ir) = self.upgrade() {
                ir.size_of_val(tracker)
            } else {
                0
            }
    }
}

/// An `InstanceRef` that may or may not be keeping the `Instance` alive.
///
/// This type is useful for types that conditionally must keep / not keep the
/// underlying `Instance` alive. For example, to prevent cycles in `WasmerEnv`s.
#[derive(Debug, Clone, PartialEq, MemoryUsage)]
pub enum WeakOrStrongInstanceRef {
    /// A weak instance ref.
    Weak(WeakInstanceRef),
    /// A strong instance ref.
    Strong(InstanceRef),
}

impl WeakOrStrongInstanceRef {
    /// Tries to upgrade weak references to a strong reference, returning None
    /// if it can't be done.
    pub fn upgrade(&self) -> Option<Self> {
        match self {
            Self::Weak(weak) => weak.upgrade().map(Self::Strong),
            Self::Strong(strong) => Some(Self::Strong(strong.clone())),
        }
    }

    /// Clones self into a weak reference.
    pub fn downgrade(&self) -> Self {
        match self {
            Self::Weak(weak) => Self::Weak(weak.clone()),
            Self::Strong(strong) => Self::Weak(WeakInstanceRef(Arc::downgrade(&strong.0))),
        }
    }
}

impl TryFrom<WeakOrStrongInstanceRef> for InstanceRef {
    type Error = &'static str;
    fn try_from(value: WeakOrStrongInstanceRef) -> Result<Self, Self::Error> {
        match value {
            WeakOrStrongInstanceRef::Strong(strong) => Ok(strong),
            WeakOrStrongInstanceRef::Weak(weak) => {
                weak.upgrade().ok_or("Failed to upgrade weak reference")
            }
        }
    }
}

impl From<WeakOrStrongInstanceRef> for WeakInstanceRef {
    fn from(value: WeakOrStrongInstanceRef) -> Self {
        match value {
            WeakOrStrongInstanceRef::Strong(strong) => Self(Arc::downgrade(&strong.0)),
            WeakOrStrongInstanceRef::Weak(weak) => weak,
        }
    }
}

impl From<WeakInstanceRef> for WeakOrStrongInstanceRef {
    fn from(value: WeakInstanceRef) -> Self {
        Self::Weak(value)
    }
}

impl From<InstanceRef> for WeakOrStrongInstanceRef {
    fn from(value: InstanceRef) -> Self {
        Self::Strong(value)
    }
}
