use pin_project::pin_project;

#[pin_project(project_ref = ProjRef)]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {}
