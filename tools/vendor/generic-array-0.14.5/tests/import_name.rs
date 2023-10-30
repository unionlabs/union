#[macro_use]
extern crate generic_array as gen_arr;

use gen_arr::typenum;

#[test]
fn test_different_crate_name() {
    let _: gen_arr::GenericArray<u32, typenum::U4> = arr![u32; 0, 1, 2, 3];
    let _: gen_arr::GenericArray<u32, typenum::U0> = arr![u32;];
}
