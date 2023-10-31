use pin_project::pin_project;

#[pin_project]
struct TupleStruct<T, U>(#[pin] T, U);

fn main() {}
