macro_rules! if_loom {
    ($($t:tt)*) => {{
        #[cfg(loom)]
        const LOOM: bool = true;
        #[cfg(not(loom))]
        const LOOM: bool = false;

        if LOOM {
            $($t)*
        }
    }}
}
