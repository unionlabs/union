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

use flexbuffers::*;

#[test]
fn read_golden_flexbuffer() {
    let s =
        std::fs::read("../gold_flexbuffer_example.bin").expect("Unable to read golden flexbuffer.");
    let r = Reader::get_root(s.as_ref()).unwrap();
    let m = r.as_map();

    let vec = m.idx("vec").as_vector();
    assert_eq!(vec.idx(0).as_i8(), -100);
    assert_eq!(vec.idx(1).as_str(), "Fred");
    assert_eq!(vec.idx(2).as_f32(), 4.0);
    assert_eq!(vec.idx(3).as_blob(), Blob([77].as_ref()));
    assert_eq!(vec.idx(4).flexbuffer_type(), FlexBufferType::Bool);
    assert_eq!(vec.idx(4).as_bool(), false);
    assert_eq!(vec.idx(5).as_f64(), 4.0);

    let bar = m.idx("bar").as_vector();
    for (i, &x) in [1, 2, 3].iter().enumerate() {
        assert_eq!(bar.idx(i).as_i8(), x);
    }
    let bar3 = m.idx("bar3").as_vector();
    for (i, &x) in [1, 2, 3].iter().enumerate() {
        assert_eq!(bar3.idx(i).as_i8(), x);
    }
    let bools = m.idx("bools").as_vector();
    for (i, &b) in [true, false, true, false].iter().enumerate() {
        assert_eq!(bools.idx(i).as_bool(), b)
    }

    assert_eq!(m.idx("bool").as_bool(), true);
    assert_eq!(m.idx("foo").as_f64(), 100.0);
    let mymap = m.idx("mymap").as_map();
    assert_eq!(mymap.idx("foo").as_str(), "Fred");
}
