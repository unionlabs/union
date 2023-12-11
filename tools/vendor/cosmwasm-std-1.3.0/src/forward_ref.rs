/// Given an implementation of `T == U`, implements:
/// - `&T == U`
/// - `T == &U`
///
/// We don't need to add `&T == &U` here because this is implemented automatically.
#[macro_export]
macro_rules! forward_ref_partial_eq {
    ($t:ty, $u:ty) => {
        // `&T == U`
        impl<'a> PartialEq<$u> for &'a $t {
            #[inline]
            fn eq(&self, rhs: &$u) -> bool {
                **self == *rhs // Implement via T == U
            }
        }

        // `T == &U`
        impl PartialEq<&$u> for $t {
            #[inline]
            fn eq(&self, rhs: &&$u) -> bool {
                *self == **rhs // Implement via T == U
            }
        }
    };
}
