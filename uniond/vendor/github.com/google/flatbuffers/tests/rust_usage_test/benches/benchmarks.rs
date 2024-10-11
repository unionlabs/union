/*
 * Copyright 2020 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[macro_use]
extern crate bencher;
extern crate flatbuffers;
extern crate flexbuffers;

mod flatbuffers_benchmarks;
mod flexbuffers_benchmarks;

#[allow(dead_code, unused_imports)]
#[path = "../../monster_test/mod.rs"]
mod monster_test_generated;
pub use monster_test_generated::my_game;

benchmark_main!(
    flatbuffers_benchmarks::benches,
    flexbuffers_benchmarks::benches
);
