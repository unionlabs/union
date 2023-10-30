use structopt::StructOpt;

#[test]
fn test_single_word_enum_variant_is_default_renamed_into_kebab_case() {
    #[derive(StructOpt, Debug, PartialEq)]
    enum Opt {
        Command { foo: u32 },
    }

    assert_eq!(
        Opt::Command { foo: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "command", "0"]))
    );
}

#[test]
fn test_multi_word_enum_variant_is_renamed() {
    #[derive(StructOpt, Debug, PartialEq)]
    enum Opt {
        FirstCommand { foo: u32 },
    }

    assert_eq!(
        Opt::FirstCommand { foo: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "first-command", "0"]))
    );
}

#[test]
fn test_standalone_long_generates_kebab_case() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct Opt {
        #[structopt(long)]
        FOO_OPTION: bool,
    }

    assert_eq!(
        Opt { FOO_OPTION: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo-option"]))
    );
}

#[test]
fn test_custom_long_overwrites_default_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long = "foo")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo"]))
    );
}

#[test]
fn test_standalone_long_uses_previous_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(name = "foo", long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo"]))
    );
}

#[test]
fn test_standalone_long_ignores_afterwards_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long, name = "foo")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo-option"]))
    );
}

#[test]
fn test_standalone_short_generates_kebab_case() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct Opt {
        #[structopt(short)]
        FOO_OPTION: bool,
    }

    assert_eq!(
        Opt { FOO_OPTION: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-f"]))
    );
}

#[test]
fn test_custom_short_overwrites_default_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short = "o")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-o"]))
    );
}

#[test]
fn test_standalone_short_uses_previous_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(name = "option", short)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-o"]))
    );
}

#[test]
fn test_standalone_short_ignores_afterwards_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short, name = "option")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-f"]))
    );
}

#[test]
fn test_standalone_long_uses_previous_defined_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "screaming_snake", long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--FOO_OPTION"]))
    );
}

#[test]
fn test_standalone_short_uses_previous_defined_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "screaming_snake", short)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-F"]))
    );
}

#[test]
fn test_standalone_long_works_with_verbatim_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct Opt {
        #[structopt(rename_all = "verbatim", long)]
        _fOO_oPtiON: bool,
    }

    assert_eq!(
        Opt { _fOO_oPtiON: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--_fOO_oPtiON"]))
    );
}

#[test]
fn test_standalone_short_works_with_verbatim_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "verbatim", short)]
        _foo: bool,
    }

    assert_eq!(
        Opt { _foo: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-_"]))
    );
}

#[test]
fn test_rename_all_is_propagated_from_struct_to_fields() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[structopt(rename_all = "screaming_snake")]
    struct Opt {
        #[structopt(long)]
        foo: bool,
    }

    assert_eq!(
        Opt { foo: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--FOO"]))
    );
}

#[test]
fn test_rename_all_is_not_propagated_from_struct_into_flattened() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[structopt(rename_all = "screaming_snake")]
    struct Opt {
        #[structopt(flatten)]
        foo: Foo,
    }

    #[derive(StructOpt, Debug, PartialEq)]
    struct Foo {
        #[structopt(long)]
        foo: bool,
    }

    assert_eq!(
        Opt {
            foo: Foo { foo: true }
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo"]))
    );
}

#[test]
fn test_rename_all_is_not_propagated_from_struct_into_subcommand() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[structopt(rename_all = "screaming_snake")]
    struct Opt {
        #[structopt(subcommand)]
        foo: Foo,
    }

    #[derive(StructOpt, Debug, PartialEq)]
    enum Foo {
        Command {
            #[structopt(long)]
            foo: bool,
        },
    }

    assert_eq!(
        Opt {
            foo: Foo::Command { foo: true }
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "command", "--foo"]))
    );
}

#[test]
fn test_rename_all_is_propagated_from_enum_to_variants_and_their_fields() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[structopt(rename_all = "screaming_snake")]
    enum Opt {
        FirstVariant,
        SecondVariant {
            #[structopt(long)]
            foo: bool,
        },
    }

    assert_eq!(
        Opt::FirstVariant,
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "FIRST_VARIANT"]))
    );

    assert_eq!(
        Opt::SecondVariant { foo: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "SECOND_VARIANT", "--FOO"]))
    );
}

#[test]
fn test_rename_all_is_propagation_can_be_overridden() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[structopt(rename_all = "screaming_snake")]
    enum Opt {
        #[structopt(rename_all = "kebab_case")]
        FirstVariant {
            #[structopt(long)]
            foo_option: bool,
        },
        SecondVariant {
            #[structopt(rename_all = "kebab_case", long)]
            foo_option: bool,
        },
    }

    assert_eq!(
        Opt::FirstVariant { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "first-variant", "--foo-option"]))
    );

    assert_eq!(
        Opt::SecondVariant { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "SECOND_VARIANT", "--foo-option"]))
    );
}

#[test]
fn test_lower_is_renamed() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "lower", long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foooption"]))
    );
}

#[test]
fn test_upper_is_renamed() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "upper", long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--FOOOPTION"]))
    );
}
