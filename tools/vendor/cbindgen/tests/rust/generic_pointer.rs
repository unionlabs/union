#[repr(C)]
pub struct Foo<T> {
    a: T,
}

pub type Boo = Foo<*mut u8>;

#[no_mangle]
pub extern "C" fn root(
    x: Boo,
) { }
