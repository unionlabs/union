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

extern crate flexbuffers;

use flexbuffers::*;
use std::alloc::{GlobalAlloc, Layout, System};

/// We take over the Rust allocator to count allocations. This is super not thread safe.
static mut NUM_ALLOCS: usize = 0;
fn current_allocs() -> usize {
    unsafe { NUM_ALLOCS }
}
struct TrackingAllocator;
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        NUM_ALLOCS += 1;
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}
#[global_allocator]
static T: TrackingAllocator = TrackingAllocator;

/// Make some example data
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
    // TODO(cneo): Directly pushing string slices has alloc.
}

// Read back the data from make_monster.
fn validate_monster(flexbuffer: &[u8]) {
    let r = Reader::get_root(flexbuffer).unwrap().as_map();

    assert!(!r.is_empty());
    assert!(r.index_key("not_a_field").is_none());

    assert_eq!(r.len(), 7);
    assert_eq!(r.idx("type").as_str(), "great orc");
    assert_eq!(r.idx("age").as_u8(), 100);
    assert_eq!(r.idx("name").as_str(), "Mr. Orc");

    let coins = r.idx("coins").as_vector();
    for (i, &c) in [1, 25, 50, 100, 250].iter().enumerate() {
        assert_eq!(coins.idx(i).as_u16(), c);
    }
    let color = r.idx("color").as_vector();
    for (i, &c) in [255, 0, 0, 0].iter().enumerate() {
        assert_eq!(color.idx(i).as_i32(), c);
    }
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

    let sounds = r.idx("sounds").as_vector();
    for (i, &s) in ["grr", "rawr", "muahaha"].iter().enumerate() {
        assert_eq!(sounds.idx(i).as_str(), s);
    }
}

// This is in a separate binary than tests because taking over the global allocator is not
// hermetic and not thread safe.
#[cfg(not(miri))]  // slow.
fn main() {
    let start_up = current_allocs();

    // Let's build a flexbuffer from a new (cold) flexbuffer builder.
    let mut builder = Builder::default();
    make_monster(builder.start_map());
    let after_warmup = current_allocs();

    // The builder makes some allocations while warming up.
    assert!(after_warmup > start_up);
    assert!(after_warmup < start_up + 20);

    // A warm builder should make no allocations.
    make_monster(builder.start_map());
    assert_eq!(after_warmup, current_allocs());

    // Nor should a reader.
    validate_monster(builder.view());
    assert_eq!(after_warmup, current_allocs());

    // Do it again just for kicks.
    make_monster(builder.start_map());
    validate_monster(builder.view());
    assert_eq!(after_warmup, current_allocs());

    let final_allocs = current_allocs(); // dbg! does allocate.
    dbg!(start_up, after_warmup, final_allocs);
}

#[test]
#[cfg(not(miri))]  // slow.
fn no_extra_allocations() {
    main()
}
