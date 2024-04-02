use queue_msg_macro::msg_struct;

#[msg_struct]
enum Enum {
    One(String),
    Two(u64),
}

fn main() {
    let message = Enum::One(String::from("string"));

    let serialized_deserialized: Enum =
        serde_json::from_str(serde_json::to_string(&message).unwrap().as_str()).unwrap();
    assert_eq!(message, serialized_deserialized);

    assert_eq!(format!("{:?}", message), r#"Enum::One("string")"#);
    assert_eq!(message, message.clone());
}
