use pin_project::pin_project;

#[pin_project(project_replace)]
struct TupleStruct<T, U>(#[pin] T, #[pin] T, U, U);

fn main() {}
