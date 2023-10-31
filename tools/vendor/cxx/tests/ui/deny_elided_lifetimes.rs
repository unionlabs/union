#![deny(elided_lifetimes_in_paths)]

#[cxx::bridge]
mod ffi {
    #[derive(PartialEq, PartialOrd, Hash)]
    struct Struct<'a> {
        reference: &'a i32,
    }

    extern "Rust" {
        type Rust<'a>;
    }

    unsafe extern "C++" {
        type Cpp<'a>;

        fn lifetime_named<'a>(s: &'a i32) -> UniquePtr<Cpp<'a>>;

        fn lifetime_underscore(s: &i32) -> UniquePtr<Cpp<'_>>;

        fn lifetime_elided(s: &i32) -> UniquePtr<Cpp>;
    }
}

pub struct Rust<'a>(&'a i32);

fn main() {}
