use queue_msg_macro::msg_struct;

#[msg_struct]
union Union {
    a: u32,
    b: f32,
}

fn main() {}
