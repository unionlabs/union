use queue_msg_macro::msg_struct;

#[msg_struct]
struct Named {
    a: String,
    b: u32,
}

fn main() {
    let named = Named {
        a: String::from("A"),
        b: 42,
    };

    let serialized_deserialized: Named =
        serde_json::from_str(serde_json::to_string(&named).unwrap().as_str()).unwrap();
    assert_eq!(named, serialized_deserialized);

    assert_eq!(format!("{:?}", named), r#"Named { a: "A", b: 42 }"#);
    assert_eq!(named, named.clone());
}
