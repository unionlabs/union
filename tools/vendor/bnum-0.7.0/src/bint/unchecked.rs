macro_rules! unchecked {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        crate::int::unchecked::impls!($BInt, I);
    };
}

use crate::doc;

crate::macro_impl!(unchecked);
