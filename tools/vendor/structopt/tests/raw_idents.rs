use structopt::StructOpt;

#[test]
fn raw_idents() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short, long)]
        r#type: Vec<String>,
    }

    assert_eq!(
        Opt {
            r#type: vec!["long".into(), "short".into()]
        },
        Opt::from_iter(&["test", "--type", "long", "-t", "short"])
    );
}
