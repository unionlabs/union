// In this example, a build.rs file generates the code and then copies it into generated/
extern crate flatbuffers;
#[allow(unused_imports, dead_code)]
mod generated;
use generated::my_game::sample::{Monster, MonsterArgs};

fn main() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let name = Some(fbb.create_string("bob"));
    let m = Monster::create(
        &mut fbb,
        &MonsterArgs {
            hp: 1,
            mana: 2,
            name,
            ..Default::default()
        },
    );
    fbb.finish(m, None);
    let mon = flatbuffers::root::<Monster>(fbb.finished_data()).unwrap();
    assert_eq!(mon.hp(), 1);
    assert_eq!(mon.mana(), 2);
    assert_eq!(mon.name().unwrap(), "bob");
}

#[test]
fn test_main() {
    main()
}
