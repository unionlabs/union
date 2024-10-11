extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;

#[allow(dead_code, unused_imports)]
#[path = "../../more_defaults/mod.rs"]
mod more_defaults_generated;
use self::more_defaults_generated::*;

#[test]
fn object_defaults() {
    assert_eq!(
        MoreDefaultsT::default(),
        MoreDefaultsT {
            ints: Vec::new(),
            floats: Vec::new(),
            empty_string: "".to_string(),
            some_string: "some".to_string(),
            abcs: Vec::new(),
            bools: Vec::new(),
        },
    )
}

#[test]
fn nonpresent_values() {
    let m = flatbuffers::root::<MoreDefaults>(&[0; 4]).unwrap();
    assert_eq!(m.ints().len(), 0);
    assert_eq!(m.floats().len(), 0);
    assert_eq!(m.abcs().len(), 0);
    assert_eq!(m.bools().len(), 0);
    assert_eq!(m.empty_string(), "");
    assert_eq!(m.some_string(), "some");
}
