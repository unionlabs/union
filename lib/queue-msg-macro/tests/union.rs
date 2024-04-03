use queue_msg_macro::queue_msg;

#[queue_msg]
union Union {
    a: u32,
    b: f32,
}

fn main() {}
