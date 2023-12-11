/// Converts an input of type usize to u32.
///
/// On 32 bit platforms such as wasm32 this is just a safe cast.
/// On other platforms the conversion panics for values larger than
/// `u32::MAX`.
#[inline]
pub fn force_to_u32(input: usize) -> u32 {
    #[cfg(target_pointer_width = "32")]
    {
        // usize = u32 on this architecture
        input as u32
    }
    #[cfg(not(target_pointer_width = "32"))]
    {
        input.try_into().expect("Input exceeds u32 range")
    }
}
