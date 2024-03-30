use queue_msg_macro::msg_struct;

#[msg_struct]
enum NonStruct {
    One,
    Two,
}

fn main() {}
