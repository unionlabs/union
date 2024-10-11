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

final class FlatBuffersTests: XCTestCase {

  let country = "Norway"

  func testEndian() { XCTAssertEqual(isLitteEndian, true) }

  func testOffset() {
    let o = Offset()
    let b = Offset(offset: 1)
    XCTAssertEqual(o.isEmpty, true)
    XCTAssertEqual(b.isEmpty, false)
  }

  func testCreateString() {
    let helloWorld = "Hello, world!"
    var b = FlatBufferBuilder(initialSize: 16)
    XCTAssertEqual(b.create(string: country).o, 12)
    XCTAssertEqual(b.create(string: helloWorld).o, 32)
    b.clear()
    XCTAssertEqual(b.create(string: helloWorld).o, 20)
    XCTAssertEqual(b.create(string: country).o, 32)
    b.clear()
    XCTAssertEqual(b.create(string: String(repeating: "a", count: 257)).o, 264)
  }

  func testStartTable() {
    var b = FlatBufferBuilder(initialSize: 16)
    XCTAssertNoThrow(b.startTable(with: 0))
    b.clear()
    XCTAssertEqual(b.create(string: country).o, 12)
    XCTAssertEqual(b.startTable(with: 0), 12)
  }

  func testCreateFinish() {
    var b = FlatBufferBuilder(initialSize: 16)
    let countryOff = Country.createCountry(
      builder: &b,
      name: country,
      log: 200,
      lan: 100)
    b.finish(offset: countryOff)
    // swiftformat:disable all
    let v: [UInt8] = [16, 0, 0, 0, 0, 0, 10, 0, 16, 0, 4, 0, 8, 0, 12, 0, 10, 0, 0, 0, 12, 0, 0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0]
    // swiftformat:enable all
    XCTAssertEqual(b.sizedByteArray, v)
  }

  func testCreateFinishWithPrefix() {
    var b = FlatBufferBuilder(initialSize: 16)
    let countryOff = Country.createCountry(
      builder: &b,
      name: country,
      log: 200,
      lan: 100)
    b.finish(offset: countryOff, addPrefix: true)
    // swiftformat:disable all
    let v: [UInt8] = [44, 0, 0, 0, 16, 0, 0, 0, 0, 0, 10, 0, 16, 0, 4, 0, 8, 0, 12, 0, 10, 0, 0, 0, 12, 0, 0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0]
    // swiftformat:enable all
    XCTAssertEqual(b.sizedByteArray, v)
  }

  func testReadCountry() {
    // swiftformat:disable all
    let v: [UInt8] = [16, 0, 0, 0, 0, 0, 10, 0, 16, 0, 4, 0, 8, 0, 12, 0, 10, 0, 0, 0, 12, 0, 0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0]
    // swiftformat:enable all
    let buffer = ByteBuffer(bytes: v)
    let c = Country.getRootAsCountry(buffer)
    XCTAssertEqual(c.lan, 100)
    XCTAssertEqual(c.log, 200)
    XCTAssertEqual(c.nameVector, [78, 111, 114, 119, 97, 121])
    XCTAssertEqual(c.name, country)
  }

  func testWriteNullableStrings() {
    var b = FlatBufferBuilder()
    XCTAssertTrue(b.create(string: nil).isEmpty)
    XCTAssertTrue(b.createShared(string: nil).isEmpty)
  }

  func testWriteOptionalValues() {
    var b = FlatBufferBuilder()
    let root = optional_scalars_ScalarStuff.createScalarStuff(
      &b,
      justI8: 80,
      maybeI8: nil,
      justU8: 100,
      maybeU8: 10,
      maybeBool: true,
      justEnum: .one,
      maybeEnum: nil)
    b.finish(offset: root)
    var buffer = b.sizedBuffer
    let scalarTable: optional_scalars_ScalarStuff = getRoot(byteBuffer: &buffer)
    XCTAssertEqual(scalarTable.justI8, 80)
    XCTAssertNil(scalarTable.maybeI8)
    XCTAssertEqual(scalarTable.maybeBool, true)
    XCTAssertEqual(scalarTable.defaultI8, 42)
    XCTAssertEqual(scalarTable.justU8, 100)
    XCTAssertEqual(scalarTable.maybeU8, 10)
    XCTAssertEqual(scalarTable.justEnum, .one)
    XCTAssertNil(scalarTable.maybeEnum)
  }
}

class Country {

  static let offsets: (name: VOffset, lan: VOffset, lng: VOffset) = (4, 6, 8)
  private var __t: Table

  private init(_ t: Table) {
    __t = t
  }

  var lan: Int32 { let o = __t.offset(6); return o == 0 ? 0 : __t.readBuffer(
    of: Int32.self,
    at: o) }
  var log: Int32 { let o = __t.offset(8); return o == 0 ? 0 : __t.readBuffer(
    of: Int32.self,
    at: o) }
  var nameVector: [UInt8]? { __t.getVector(at: 4) }
  var name: String? {
    let o = __t.offset(4); return o == 0 ? nil : __t.string(at: o) }

  @inlinable
  static func getRootAsCountry(_ bb: ByteBuffer) -> Country {
    Country(Table(
      bb: bb,
      position: Int32(bb.read(def: UOffset.self, position: 0))))
  }

  @inlinable
  static func createCountry(
    builder: inout FlatBufferBuilder,
    name: String,
    log: Int32,
    lan: Int32) -> Offset
  {
    createCountry(
      builder: &builder,
      offset: builder.create(string: name),
      log: log,
      lan: lan)
  }

  @inlinable
  static func createCountry(
    builder: inout FlatBufferBuilder,
    offset: Offset,
    log: Int32,
    lan: Int32) -> Offset
  {
    let _start = builder.startTable(with: 3)
    Country.add(builder: &builder, lng: log)
    Country.add(builder: &builder, lan: lan)
    Country.add(builder: &builder, name: offset)
    return Country.end(builder: &builder, startOffset: _start)
  }

  @inlinable
  static func end(
    builder: inout FlatBufferBuilder,
    startOffset: UOffset) -> Offset
  {
    Offset(offset: builder.endTable(at: startOffset))
  }

  @inlinable
  static func add(builder: inout FlatBufferBuilder, name: String) {
    add(builder: &builder, name: builder.create(string: name))
  }

  @inlinable
  static func add(builder: inout FlatBufferBuilder, name: Offset) {
    builder.add(offset: name, at: Country.offsets.name)
  }

  @inlinable
  static func add(builder: inout FlatBufferBuilder, lan: Int32) {
    builder.add(element: lan, def: 0, at: Country.offsets.lan)
  }

  @inlinable
  static func add(builder: inout FlatBufferBuilder, lng: Int32) {
    builder.add(element: lng, def: 0, at: Country.offsets.lng)
  }
}
