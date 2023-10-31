use pin_project::pin_project;

#[pin_project(project_replace)]
struct TupleStruct<T, U>(#[pin] T, U);

fn main() {}
