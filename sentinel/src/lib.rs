use std::collections::HashMap;

pub trait Chain {}

struct Context {
    chains: HashMap<String, Box<dyn Chain>>,
}
