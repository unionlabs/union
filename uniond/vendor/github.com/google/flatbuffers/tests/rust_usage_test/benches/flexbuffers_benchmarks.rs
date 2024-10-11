// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bencher::{benchmark_group, benchmark_main, Bencher};
use flexbuffers::*;

fn push_vec_u64_to_map(b: &mut Bencher) {
    let va = vec![u64::max_value() - 10; 512];
    let vb = vec![u64::max_value() - 20; 512];
    let vc = vec![u64::max_value() - 30; 512];
    let mut n = 0;

    b.iter(|| {
        let mut fxb = Builder::default();
        let mut m = fxb.start_map();
        let mut ma = m.start_vector("a");
        for &a in va.iter() {
            ma.push(a);
        }
        ma.end_vector();
        let mut mb = m.start_vector("b");
        for &b in vb.iter() {
            mb.push(b);
        }
        mb.end_vector();
        let mut mc = m.start_vector("c");
        for &c in vc.iter() {
            mc.push(c);
        }
        mc.end_vector();
        m.end_map();
        n = fxb.view().len();
    });
    b.bytes = n as u64;
}
fn push_vec_u64_to_map_reused(b: &mut Bencher) {
    let va = vec![u64::max_value() - 10; 512];
    let vb = vec![u64::max_value() - 20; 512];
    let vc = vec![u64::max_value() - 30; 512];
    let mut fxb = Builder::default();
    let mut n = 0;
    let mut go = || {
        let mut m = fxb.start_map();
        let mut ma = m.start_vector("a");
        for &a in va.iter() {
            ma.push(a);
        }
        ma.end_vector();
        let mut mb = m.start_vector("b");
        for &b in vb.iter() {
            mb.push(b);
        }
        mb.end_vector();
        let mut mc = m.start_vector("c");
        for &c in vc.iter() {
            mc.push(c);
        }
        mc.end_vector();
        m.end_map();
        n = fxb.view().len();
    };
    go(); // warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}
fn push_vec_u64_to_map_direct(b: &mut Bencher) {
    let va = vec![u64::max_value() - 10; 512];
    let vb = vec![u64::max_value() - 20; 512];
    let vc = vec![u64::max_value() - 30; 512];
    let mut n = 0;

    b.iter(|| {
        let mut fxb = Builder::default();
        let mut m = fxb.start_map();
        m.push("a", &va);
        m.push("b", &vb);
        m.push("c", &vc);
        m.end_map();
        n = fxb.view().len();
    });
    b.bytes = n as u64;
}
fn push_vec_u64_to_map_direct_reused(b: &mut Bencher) {
    let va = vec![u64::max_value() - 10; 512];
    let vb = vec![u64::max_value() - 20; 512];
    let vc = vec![u64::max_value() - 30; 512];
    let mut n = 0;
    let mut fxb = Builder::default();
    let mut go = || {
        let mut m = fxb.start_map();
        m.push("a", &va);
        m.push("b", &vb);
        m.push("c", &vc);
        m.end_map();
        n = fxb.view().len();
    };
    go(); // warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}

fn push_vec_without_indirect(b: &mut Bencher) {
    let mut builder = Builder::default();
    let mut n = 0;
    let mut go = || {
        let mut b = builder.start_vector();
        for i in 0..1024u16 {
            b.push(i);
        }
        b.push(i64::max_value());
        b.end_vector();
        n = builder.view().len();
    };
    go(); // warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}
// This isn't actually faster than the alternative but it is a lot smaller.
// Based on the above benchmarks a lot of time is stuck in the `values` stack.
fn push_vec_with_indirect(b: &mut Bencher) {
    let mut builder = Builder::default();
    let mut n = 0;
    let mut go = || {
        let mut b = builder.start_vector();
        for i in 0..1024u16 {
            b.push(i);
        }
        b.push(IndirectInt(i64::max_value()));
        b.end_vector();
        n = builder.view().len();
    };
    go(); // warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}

fn example_map<'a>(m: &mut MapBuilder<'a>) {
    m.push("some_ints", &[256; 5]);
    m.push("some_uints", &[256u16; 5]);
    m.push("some_floats", &[256f32; 5]);
    m.push("some_strings", "muahahahahaha");
}
fn hundred_maps(b: &mut Bencher) {
    let mut builder = Builder::default();
    let mut n = 0;
    let mut go = || {
        let mut v = builder.start_vector();
        for _ in 0..100 {
            example_map(&mut v.start_map());
        }
        v.end_vector();
        n = builder.view().len();
    };
    go(); // Warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}
fn hundred_maps_pooled(b: &mut Bencher) {
    let mut builder = Builder::default();
    let mut n = 0;
    let mut go = || {
        let mut v = builder.start_vector();
        for _ in 0..100 {
            example_map(&mut v.start_map());
        }
        v.end_vector();
        n = builder.view().len();
    };
    go(); // Warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}
fn make_monster(mut monster: MapBuilder) {
    monster.push("type", "great orc");
    monster.push("age", 100u8);
    monster.push("name", "Mr. Orc");
    monster.push("coins", &[1, 25, 50, 100, 250]);
    monster.push("color", &[255u8, 0, 0, 0]);
    {
        let mut weapons = monster.start_vector("weapons");
        {
            let mut hammer = weapons.start_map();
            hammer.push("name", "hammer");
            hammer.push("damage type", "crush");
            hammer.push("damage", 20);
        }
        {
            let mut axe = weapons.start_map();
            axe.push("name", "Great Axe");
            axe.push("damage type", "slash");
            axe.push("damage", 30);
        }
    }
    {
        let mut sounds = monster.start_vector("sounds");
        sounds.push("grr");
        sounds.push("rawr");
        sounds.push("muahaha");
    }
}
fn serialize_monsters(b: &mut Bencher) {
    let mut builder = Builder::default();
    let mut n = 0;
    let mut go = || {
        let mut monsters = builder.start_vector();
        for _ in 0..100 {
            make_monster(monsters.start_map())
        }
        monsters.end_vector();
        n = builder.view().len();
    };
    go(); // Warm up allocations.
    b.iter(go);
    b.bytes = n as u64;
}
fn validate_monster(r: MapReader<&[u8]>) {
    assert_eq!(r.idx("type").as_str(), "great orc");
    assert_eq!(r.idx("age").as_u8(), 100);
    assert_eq!(r.idx("name").as_str(), "Mr. Orc");
    assert!(r
        .idx("coins")
        .as_vector()
        .iter()
        .map(|c| c.as_i16())
        .eq([1, 25, 50, 100, 250].iter().cloned()));
    assert!(r
        .idx("color")
        .as_vector()
        .iter()
        .map(|c| c.as_u8())
        .eq([255, 0, 0, 0].iter().cloned()));

    let weapons = r.idx("weapons").as_vector();
    assert_eq!(weapons.len(), 2);

    let hammer = weapons.idx(0).as_map();
    assert_eq!(hammer.idx("name").as_str(), "hammer");
    assert_eq!(hammer.idx("damage type").as_str(), "crush");
    assert_eq!(hammer.idx("damage").as_u64(), 20);

    let axe = weapons.idx(1).as_map();
    assert_eq!(axe.idx("name").as_str(), "Great Axe");
    assert_eq!(axe.idx("damage type").as_str(), "slash");
    assert_eq!(axe.idx("damage").as_u64(), 30);

    assert!(r
        .idx("sounds")
        .as_vector()
        .iter()
        .map(|s| s.as_str())
        .eq(["grr", "rawr", "muahaha"].iter().cloned()));
}
fn read_monsters(b: &mut Bencher) {
    let mut builder = Builder::default();
    let mut monsters = builder.start_vector();
    for _ in 0..100 {
        make_monster(monsters.start_map());
    }
    monsters.end_vector();
    b.bytes = builder.view().len() as u64;
    let go = || {
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        assert_eq!(r.len(), 100);
        for i in 0..100 {
            validate_monster(r.idx(i).as_map());
        }
    };
    b.iter(go);
}

benchmark_group!(
    benches,
    push_vec_u64_to_map,
    push_vec_u64_to_map_reused,
    push_vec_u64_to_map_direct,
    push_vec_u64_to_map_direct_reused,
    push_vec_without_indirect,
    push_vec_with_indirect,
    hundred_maps,
    hundred_maps_pooled,
    serialize_monsters,
    read_monsters,
);
benchmark_main!(benches);
