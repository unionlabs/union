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

final class FlatBuffersDoubleTests: XCTestCase {

  let country = "Norway"

  func testCreateFinish() {
    var b = FlatBufferBuilder(initialSize: 16)
    let countryOff = CountryDouble.createCountry(
      builder: &b,
      name: country,
      log: 200,
      lan: 100)
    b.finish(offset: countryOff)
    // swiftformat:disable all
    let v: [UInt8] = [
      16, 0, 0, 0, 0, 0, 10, 0, 28, 0, 4, 0, 8, 0, 16, 0, 10,
      0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 89, 64, 0, 0, 0,
      0, 0, 0, 105, 64, 0, 0, 0, 0, 6, 0, 0, 0, 78, 111, 114, 119,
      97, 121, 0, 0
    ]
    // swiftformat:enable all
    XCTAssertEqual(b.sizedByteArray, v)
  }

  func testCreateFinishWithPrefix() {
    var b = FlatBufferBuilder(initialSize: 16)
    let countryOff = CountryDouble.createCountry(
      builder: &b,
      name: country,
      log: 200,
      lan: 100)
    b.finish(offset: countryOff, addPrefix: true)
    // swiftformat:disable all
    let v: [UInt8] = [
      60, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 28
      , 0, 4, 0, 8, 0, 16, 0, 10, 0, 0, 0, 24, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 89, 64, 0, 0, 0, 0, 0, 0, 105, 64, 0, 0, 0,
      0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0
    ]
    // swiftformat:enable all
    XCTAssertEqual(b.sizedByteArray, v)
  }
}

class CountryDouble {

  static let offsets: (name: VOffset, lan: VOffset, lng: VOffset) = (4, 6, 8)

  private var table: Table

  private init(table t: Table) { table = t }

  static func getRootAsCountry(_ bb: ByteBuffer) -> CountryDouble {
    let pos = bb.read(def: Int32.self, position: Int(bb.size))
    return CountryDouble(table: Table(bb: bb, position: Int32(pos)))
  }

  static func createCountry(
    builder: inout FlatBufferBuilder,
    name: String,
    log: Double,
    lan: Double) -> Offset
  {
    createCountry(
      builder: &builder,
      offset: builder.create(string: name),
      log: log,
      lan: lan)
  }

  static func createCountry(
    builder: inout FlatBufferBuilder,
    offset: Offset,
    log: Double,
    lan: Double) -> Offset
  {
    let _start = builder.startTable(with: 3)
    CountryDouble.add(builder: &builder, lng: log)
    CountryDouble.add(builder: &builder, lan: lan)
    CountryDouble.add(builder: &builder, name: offset)
    return CountryDouble.end(builder: &builder, startOffset: _start)
  }

  static func end(
    builder: inout FlatBufferBuilder,
    startOffset: UOffset) -> Offset
  {
    Offset(offset: builder.endTable(at: startOffset))
  }

  static func add(builder: inout FlatBufferBuilder, name: String) {
    add(builder: &builder, name: builder.create(string: name))
  }

  static func add(builder: inout FlatBufferBuilder, name: Offset) {
    builder.add(offset: name, at: Country.offsets.name)
  }

  static func add(builder: inout FlatBufferBuilder, lan: Double) {
    builder.add(element: lan, def: 0, at: Country.offsets.lan)
  }

  static func add(builder: inout FlatBufferBuilder, lng: Double) {
    builder.add(element: lng, def: 0, at: Country.offsets.lng)
  }
}
