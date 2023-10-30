use pin_project::pin_project;

#[pin_project(project = Proj)]
struct TupleStruct<T, U>(#[pin] T, U);

fn main() {}
