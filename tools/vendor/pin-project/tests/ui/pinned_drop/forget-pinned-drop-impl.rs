use pin_project::pin_project;

#[pin_project(PinnedDrop)] //~ ERROR E0277
struct Struct {
    #[pin]
    f: u8,
}

fn main() {}
