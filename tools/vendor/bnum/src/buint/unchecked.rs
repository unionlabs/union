macro_rules! unchecked {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        crate::int::unchecked::impls!($BUint, I);
    };
}

use crate::doc;

crate::macro_impl!(unchecked);
