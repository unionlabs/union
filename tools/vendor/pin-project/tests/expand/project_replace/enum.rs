use pin_project::pin_project;

#[pin_project(project_replace = EnumProjOwn)]
enum Enum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}

fn main() {}
