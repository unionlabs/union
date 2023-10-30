#[cfg(test)]
use crate::assert_size_of_val_eq;
use crate::{MemoryUsage, MemoryUsageTracker, POINTER_BYTE_SIZE};
use std::cell::{RefCell, UnsafeCell};
use std::mem;

impl<T> MemoryUsage for UnsafeCell<T> {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + if tracker.track(self.get() as *const ()) {
                POINTER_BYTE_SIZE
            } else {
                0
            }
    }
}

impl<T> MemoryUsage for RefCell<T>
where
    T: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + match self.try_borrow() {
                Ok(borrowed) if tracker.track(self.as_ptr() as *const _ as *const ()) => {
                    borrowed.size_of_val(tracker)
                }

                _ => 0,
            }
    }
}

#[cfg(test)]
mod test_cell_types {
    use super::*;
    use crate::size_of_val;

    #[test]
    fn test_unsafecell() {
        let cell = UnsafeCell::<i8>::new(1);
        assert_size_of_val_eq!(cell, mem::size_of_val(&cell) + POINTER_BYTE_SIZE);
    }

    #[test]
    fn test_refcell() {
        let cell = RefCell::<Vec<i8>>::new(vec![]);

        {
            cell.borrow_mut().push(1);
        }

        let cell_size = size_of_val(&cell);

        {
            let mut vec = cell.borrow_mut();
            vec.push(2);
            vec.push(3);

            assert_size_of_val_eq!(cell, mem::size_of_val(&cell));
        }

        assert_size_of_val_eq!(cell, cell_size + 2);
    }
}
