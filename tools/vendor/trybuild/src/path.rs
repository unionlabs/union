macro_rules! path {
    ($($tt:tt)+) => {
        tokenize_path!([] [] $($tt)+)
    };
}

// Private implementation detail.
macro_rules! tokenize_path {
    ([$(($($component:tt)+))*] [$($cur:tt)+] /) => {
        crate::directory::Directory::new(tokenize_path!([$(($($component)+))*] [$($cur)+]))
    };

    ([$(($($component:tt)+))*] [$($cur:tt)+] / $($rest:tt)+) => {
        tokenize_path!([$(($($component)+))* ($($cur)+)] [] $($rest)+)
    };

    ([$(($($component:tt)+))*] [$($cur:tt)*] $first:tt $($rest:tt)*) => {
        tokenize_path!([$(($($component)+))*] [$($cur)* $first] $($rest)*)
    };

    ([$(($($component:tt)+))*] [$($cur:tt)+]) => {
        tokenize_path!([$(($($component)+))* ($($cur)+)])
    };

    ([$(($($component:tt)+))*]) => {{
        let mut path = std::path::PathBuf::new();
        $(
            path.push(&($($component)+));
        )*
        path
    }};
}

#[test]
fn test_path_macro() {
    use std::path::{Path, PathBuf};

    struct Project {
        dir: PathBuf,
    }

    let project = Project {
        dir: PathBuf::from("../target/tests"),
    };

    let cargo_dir = path!(project.dir / ".cargo" / "config");
    assert_eq!(cargo_dir, Path::new("../target/tests/.cargo/config"));
}
