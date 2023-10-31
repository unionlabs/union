pub(crate) fn usize_to_u32(u: usize) -> Option<u32> {
    #[cfg(not(no_try_from))]
    {
        use core::convert::TryFrom;

        u32::try_from(u).ok()
    }

    #[cfg(no_try_from)]
    {
        use core::mem;

        if mem::size_of::<usize>() <= mem::size_of::<u32>() || u <= u32::max_value() as usize {
            Some(u as u32)
        } else {
            None
        }
    }
}
