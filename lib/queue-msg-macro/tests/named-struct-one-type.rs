use std::fmt::Debug;

use queue_msg_macro::msg_struct;
use serde::{Deserialize, Serialize};

#[msg_struct]
struct Named<
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + std::cmp::PartialEq,
    #[cover] U,
> {
    a: String,
    b: u32,
    c: T,
}

fn main() {
    let named: Named<u64, u128> = Named {
        a: String::from("A"),
        b: 42,
        c: 1_u64,
        __marker: Default::default(),
    };

    let serialized_deserialized: Named<u64, u128> =
        serde_json::from_str(serde_json::to_string(&named).unwrap().as_str()).unwrap();
    assert_eq!(named, serialized_deserialized);

    assert_eq!(format!("{:?}", named), r#"Named { a: "A", b: 42, c: 1 }"#);
    assert_eq!(named, named.clone());
}
