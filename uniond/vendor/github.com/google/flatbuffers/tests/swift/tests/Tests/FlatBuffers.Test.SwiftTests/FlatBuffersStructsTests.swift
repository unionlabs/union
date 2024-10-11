/*
 * Copyright 2023 Google Inc. All rights reserved.
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

import XCTest
@testable import FlatBuffers

final class FlatBuffersStructsTests: XCTestCase {

  func testWritingAndMutatingBools() {
    var fbb = FlatBufferBuilder()
    let start = TestMutatingBool.startTestMutatingBool(&fbb)
    TestMutatingBool.add(b: Property(property: false), &fbb)
    let root = TestMutatingBool.endTestMutatingBool(&fbb, start: start)
    fbb.finish(offset: root)

    var buffer = fbb.sizedBuffer
    let testMutatingBool: TestMutatingBool = getRoot(byteBuffer: &buffer)
    let property = testMutatingBool.mutableB
    XCTAssertEqual(property?.property, false)
    property?.mutate(property: false)
    XCTAssertEqual(property?.property, false)
    property?.mutate(property: true)
    XCTAssertEqual(property?.property, true)
  }

}

struct Vec: NativeStruct {
  var x: Float32
  var y: Float32
  var z: Float32
}
