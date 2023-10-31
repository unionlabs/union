use pin_project::pin_project;

#[pin_project]
pub struct Struct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

fn main() {}
