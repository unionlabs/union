/// A placeholder byte-sized type which is just used to provide some amount of type
/// safety when dealing with pointers to JIT-compiled function bodies. Note that it's
/// deliberately not Copy, as we shouldn't be carelessly copying function body bytes
/// around.
#[repr(C)]
pub struct VMFunctionBody(u8);

#[cfg(test)]
mod test_vmfunction_body {
    use super::VMFunctionBody;
    use std::mem::size_of;

    #[test]
    fn check_vmfunction_body_offsets() {
        assert_eq!(size_of::<VMFunctionBody>(), 1);
    }
}
