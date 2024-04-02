use std::fmt::Debug;

use queue_msg_macro::queue_msg;
use serde::{Deserialize, Serialize};

#[queue_msg]
struct Named<
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + std::cmp::PartialEq,
    U: Clone + Debug + Serialize + for<'a> Deserialize<'a> + std::cmp::PartialEq,
    #[cover] V,
    #[cover] W,
> {
    a: String,
    b: u32,
    c: T,
    d: U,
}

fn main() {
    let named: Named<u8, u16, u32, u64> = Named {
        a: String::from("A"),
        b: 42,
        c: 1_u8,
        d: 2_u16,
        __marker: Default::default(),
    };

    let serialized_deserialized: Named<u8, u16, u32, u64> =
        serde_json::from_str(serde_json::to_string(&named).unwrap().as_str()).unwrap();
    assert_eq!(named, serialized_deserialized);

    assert_eq!(
        format!("{:?}", named),
        r#"Named { a: "A", b: 42, c: 1, d: 2 }"#
    );
    assert_eq!(named, named.clone());
}
