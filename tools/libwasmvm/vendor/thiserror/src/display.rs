use std::fmt::Display;
use std::path::{self, Path, PathBuf};

pub trait DisplayAsDisplay {
    fn as_display(&self) -> Self;
}

impl<T: Display> DisplayAsDisplay for &T {
    fn as_display(&self) -> Self {
        self
    }
}

pub trait PathAsDisplay {
    fn as_display(&self) -> path::Display<'_>;
}

impl PathAsDisplay for Path {
    fn as_display(&self) -> path::Display<'_> {
        self.display()
    }
}

impl PathAsDisplay for PathBuf {
    fn as_display(&self) -> path::Display<'_> {
        self.display()
    }
}
