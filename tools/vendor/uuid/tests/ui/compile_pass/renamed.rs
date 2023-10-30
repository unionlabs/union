use ::uuid::{uuid as id, Uuid as Id};

mod uuid {
    struct MyType;
}

struct Uuid;

const _: Id = id!("67e55044-10b1-426f-9247-bb680e5fe0c8");

fn main() {}
