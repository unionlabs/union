use queue_msg_macro::queue_msg;

#[queue_msg]
struct Unnamed(String, u16);

fn main() {}
