//! Checks that types like `::std::option::Option` are not special

use structopt::StructOpt;

#[rustversion::since(1.37)]
#[test]
fn special_types_bool() {
    mod inner {
        #[allow(non_camel_case_types)]
        #[derive(PartialEq, Debug)]
        pub struct bool(pub String);

        impl std::str::FromStr for self::bool {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(self::bool(s.into()))
            }
        }
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        arg: inner::bool,
    }

    assert_eq!(
        Opt {
            arg: inner::bool("success".into())
        },
        Opt::from_iter(&["test", "success"])
    );
}

#[test]
fn special_types_option() {
    fn parser(s: &str) -> Option<String> {
        Some(s.to_string())
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(parse(from_str = parser))]
        arg: ::std::option::Option<String>,
    }

    assert_eq!(
        Opt {
            arg: Some("success".into())
        },
        Opt::from_iter(&["test", "success"])
    );
}

#[test]
fn special_types_vec() {
    fn parser(s: &str) -> Vec<String> {
        vec![s.to_string()]
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(parse(from_str = parser))]
        arg: ::std::vec::Vec<String>,
    }

    assert_eq!(
        Opt {
            arg: vec!["success".into()]
        },
        Opt::from_iter(&["test", "success"])
    );
}
