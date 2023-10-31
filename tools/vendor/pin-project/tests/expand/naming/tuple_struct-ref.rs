use pin_project::pin_project;

#[pin_project(project_ref = ProjRef)]
struct TupleStruct<T, U>(#[pin] T, U);

fn main() {}
