use pin_project::pin_project;

#[pin_project(project_replace = ProjOwn)]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {}
