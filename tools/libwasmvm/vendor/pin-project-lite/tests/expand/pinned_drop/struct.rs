use pin_project_lite::pin_project;
use std::pin::Pin;

pin_project! {
    struct Struct<T, U> {
        #[pin]
        pinned: T,
        unpinned: U,
    }
    impl<T, U> PinnedDrop for Struct<T, U> {
        fn drop(this: Pin<&mut Self>) {
            let _ = this;
        }
    }
}

fn main() {}
