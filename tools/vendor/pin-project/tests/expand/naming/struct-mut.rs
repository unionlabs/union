use pin_project::pin_project;

#[pin_project(project = Proj)]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {}
