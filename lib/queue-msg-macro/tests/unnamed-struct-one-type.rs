use queue_msg_macro::msg_struct;

#[msg_struct]
struct Unnamed(String);

fn main() {
    let unnamed = Unnamed(String::from("A"));

    let serialized_deserialized: Unnamed =
        serde_json::from_str(serde_json::to_string(&unnamed).unwrap().as_str()).unwrap();
    assert_eq!(unnamed, serialized_deserialized);

    assert_eq!(format!("{:?}", unnamed), "Unnamed(\"A\")");
    assert_eq!(unnamed, unnamed.clone());
}
