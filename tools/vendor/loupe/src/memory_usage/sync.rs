#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;
use std::sync::{
    atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
        AtomicU64, AtomicU8, AtomicUsize,
    },
    Arc, Mutex, RwLock, Weak,
};

macro_rules! impl_memory_usage_for_numeric {
    ( $type:ty ) => {
        impl MemoryUsage for $type {
            fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
                mem::size_of_val(self)
            }
        }
    };

    ( $( $type:ty ),+ $(,)* ) => {
        $( impl_memory_usage_for_numeric!( $type ); )+
    }
}

impl_memory_usage_for_numeric!(
    AtomicBool,
    AtomicI8,
    AtomicI16,
    AtomicI32,
    AtomicI64,
    AtomicIsize,
    AtomicU8,
    AtomicU16,
    AtomicU32,
    AtomicU64,
    AtomicUsize,
);

impl<T> MemoryUsage for Arc<T>
where
    T: MemoryUsage + ?Sized,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + if tracker.track(Arc::as_ptr(self) as *const ()) {
                self.as_ref().size_of_val(tracker)
            } else {
                0
            }
    }
}

impl<T> MemoryUsage for Weak<T>
where
    T: MemoryUsage + ?Sized,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + if tracker.track(Weak::as_ptr(self) as *const ()) {
                Weak::upgrade(self)
                    .map(|arc| arc.as_ref().size_of_val(tracker))
                    .unwrap_or(0)
            } else {
                0
            }
    }
}

impl<T> MemoryUsage for Mutex<T>
where
    T: MemoryUsage + ?Sized,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.lock().unwrap().size_of_val(tracker)
    }
}

impl<T> MemoryUsage for RwLock<T>
where
    T: MemoryUsage + ?Sized,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.read().unwrap().size_of_val(tracker)
    }
}

#[cfg(test)]
mod test_sync_types {
    use super::*;

    macro_rules! test_memory_usage_for_numeric {
        ($test_name:ident: ($value:expr) == $expected:expr) => {
            #[test]
            fn $test_name() {
                assert_size_of_val_eq!($value, $expected);
            }
        };

        ( $( $test_name:ident: ($value:expr) == $expected:expr );+ $(;)* ) => {
            $( test_memory_usage_for_numeric!( $test_name: ($value) == $expected); )+
        }
    }

    test_memory_usage_for_numeric!(
        test_atomic_bool: (AtomicBool::new(true)) == 1;
        test_atomic_i8: (AtomicI8::new(1i8)) == 1;
        test_atomic_i16: (AtomicI16::new(1i16)) == 2;
        test_atomic_i32: (AtomicI32::new(1i32)) == 4;
        test_atomic_i64: (AtomicI64::new(1i64)) == 8;
        test_atomic_isize: (AtomicIsize::new(1isize)) == POINTER_BYTE_SIZE;
        test_atomic_u8: (AtomicU8::new(1u8)) == 1;
        test_atomic_u16: (AtomicU16::new(1u16)) == 2;
        test_atomic_u32: (AtomicU32::new(1u32)) == 4;
        test_atomic_u64: (AtomicU64::new(1u64)) == 8;
        test_atomic_usize: (AtomicUsize::new(1usize)) == POINTER_BYTE_SIZE;
    );

    #[test]
    fn test_arc() {
        let empty_arc_size = mem::size_of_val(&Arc::new(()));

        let arc: Arc<i32> = Arc::new(1);
        assert_size_of_val_eq!(arc, empty_arc_size + 4);

        let arc: Arc<Option<i32>> = Arc::new(Some(1));
        assert_size_of_val_eq!(arc, empty_arc_size + POINTER_BYTE_SIZE + 4);
    }

    #[test]
    fn test_weak() {
        let empty_weak_size = mem::size_of_val(&Arc::downgrade(&Arc::new(())));

        let arc: Arc<i32> = Arc::new(1);
        let weak: Weak<i32> = Arc::downgrade(&arc);
        assert_size_of_val_eq!(weak, empty_weak_size + 4);

        let arc: Arc<Option<i32>> = Arc::new(Some(1));
        let weak: Weak<Option<i32>> = Arc::downgrade(&arc);
        assert_size_of_val_eq!(weak, empty_weak_size + POINTER_BYTE_SIZE + 4);

        let weak: Weak<i32> = {
            let arc: Arc<i32> = Arc::new(5);
            Arc::downgrade(&arc)
        };
        assert_size_of_val_eq!(weak, empty_weak_size);
    }

    #[test]
    fn test_mutex() {
        let empty_mutex_size = mem::size_of_val(&Mutex::new(()));

        let mutex: Mutex<i32> = Mutex::new(1);
        assert_size_of_val_eq!(mutex, empty_mutex_size + 4);

        let mutex: Mutex<Option<i32>> = Mutex::new(Some(1));
        assert_size_of_val_eq!(mutex, empty_mutex_size + 2 * POINTER_BYTE_SIZE + 4);
    }

    #[test]
    fn test_rwlock() {
        let empty_rwlock_size = mem::size_of_val(&RwLock::new(()));

        let rwlock: RwLock<i32> = RwLock::new(1);
        assert_size_of_val_eq!(rwlock, empty_rwlock_size + 4);

        let rwlock: RwLock<Option<i32>> = RwLock::new(Some(1));
        assert_size_of_val_eq!(rwlock, empty_rwlock_size + 2 * POINTER_BYTE_SIZE + 4);
    }
}
