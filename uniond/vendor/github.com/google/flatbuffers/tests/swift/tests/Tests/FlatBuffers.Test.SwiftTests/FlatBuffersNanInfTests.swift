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

final class FlatBuffersNanInfTests: XCTestCase {

  func createTestTable() -> FlatBufferBuilder {
    var fbb = FlatBufferBuilder()
    let msg = Swift_Tests_NanInfTable.createNanInfTable(
      &fbb,
      valueNan: .nan,
      valueInf: .infinity,
      valueNinf: -.infinity,
      value: 100.0)
    fbb.finish(offset: msg)
    return fbb
  }

  func testInfNanBinary() {
    let fbb = createTestTable()
    let data = fbb.sizedByteArray

    var buffer = ByteBuffer(bytes: data)
    let table: Swift_Tests_NanInfTable = getRoot(byteBuffer: &buffer)
    XCTAssert(table.defaultNan.isNaN)
    XCTAssertEqual(table.defaultInf, .infinity)
    XCTAssertEqual(table.defaultNinf, -.infinity)
    XCTAssert(table.valueNan.isNaN)
    XCTAssertEqual(table.valueInf, .infinity)
    XCTAssertEqual(table.valueNinf, -.infinity)
    XCTAssertEqual(table.value, 100.0)
  }

  func testInfNanJSON() {
    let fbb = createTestTable()
    var bb = fbb.sizedBuffer
    do {
      struct Test: Decodable {
        let valueInf: Double
        let value: Int
        let valueNan: Double
        let valueNinf: Double
      }
      let reader: Swift_Tests_NanInfTable = try getCheckedRoot(byteBuffer: &bb)
      let encoder = JSONEncoder()
      encoder.keyEncodingStrategy = .convertToSnakeCase
      encoder.nonConformingFloatEncodingStrategy =
        .convertToString(
          positiveInfinity: "inf",
          negativeInfinity: "-inf",
          nan: "nan")
      let data = try encoder.encode(reader)
      let decoder = JSONDecoder()
      decoder.nonConformingFloatDecodingStrategy = .convertFromString(
        positiveInfinity: "inf",
        negativeInfinity: "-inf",
        nan: "nan")
      decoder.keyDecodingStrategy = .convertFromSnakeCase
      let value = try decoder.decode(Test.self, from: data)
      XCTAssertEqual(value.value, 100)
      XCTAssertEqual(value.valueInf, .infinity)
      XCTAssertEqual(value.valueNinf, -.infinity)
    } catch {
      XCTFail(error.localizedDescription)
    }
  }

}
