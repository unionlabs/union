#[cfg(target_pointer_width = "32")]
macro_rules! limbs {
    ( $($limb:expr),+ ) => {
        [ $($limb),+ ]
    };
}

#[cfg(target_pointer_width = "64")]
macro_rules! limbs {
    ( $($limb_lo:expr, $limb_hi:expr),+) => {
        [ $((($limb_hi | 0u64) << 32) | $limb_lo),+ ]
    };
}
