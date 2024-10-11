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

final class FlatBuffersVectors: XCTestCase {

  func testCreatingTwoCountries() {
    let norway = "Norway"
    let denmark = "Denmark"
    var b = FlatBufferBuilder(initialSize: 20)
    let noStr = b.create(string: norway)
    let deStr = b.create(string: denmark)
    let n = Country.createCountry(
      builder: &b,
      offset: noStr,
      log: 888,
      lan: 700)
    let d = Country.createCountry(
      builder: &b,
      offset: deStr,
      log: 200,
      lan: 100)
    let vector = [n, d]
    let vectorOffset = b.createVector(ofOffsets: vector)
    b.finish(offset: vectorOffset)
    // swiftformat:disable all
    XCTAssertEqual(b.sizedByteArray, [4, 0, 0, 0, 2, 0, 0, 0, 48, 0, 0, 0, 16, 0, 0, 0, 0, 0, 10, 0, 18, 0, 4, 0, 8, 0, 12, 0, 10, 0, 0, 0, 40, 0, 0, 0, 100, 0, 0, 0, 200, 0, 0, 0, 0, 0, 10, 0, 16, 0, 4, 0, 8, 0, 12, 0, 10, 0, 0, 0, 24, 0, 0, 0, 188, 2, 0, 0, 120, 3, 0, 0, 7, 0, 0, 0, 68, 101, 110, 109, 97, 114, 107, 0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0])
    // swiftformat:enable all
  }

  func testCreateIntArray() {
    let numbers: [Int32] = [1, 2, 3, 4, 5]
    var b = FlatBufferBuilder(initialSize: 20)
    let o = b.createVector(numbers, size: numbers.count)
    b.finish(offset: o)
    // swiftformat:disable all
    XCTAssertEqual(b.sizedByteArray, [4, 0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0])
    // swiftformat:enable all
  }

  func testCreateStructArray() {
    struct Vec: NativeStruct {
      let x, y, z: Float32
    }
    let vector: [Vec] = [
      Vec(x: 1, y: 2, z: 3),
      Vec(x: 4, y: 5, z: 6),
      Vec(x: 7, y: 8, z: 9),
    ]
    var b = FlatBufferBuilder(initialSize: 100)
    let o = b.createVector(ofStructs: vector)
    b.finish(offset: o)
    vector.withUnsafeBytes { pointer in
      print(Array(pointer))
    }
    // swiftformat:disable all
    XCTAssertEqual(b.sizedByteArray, [4, 0, 0, 0, 3, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 0, 0, 16, 65])
    // swiftformat:enable all
  }

  func testCreateEmptyIntArray() {
    let numbers: [Int32] = []
    var b = FlatBufferBuilder(initialSize: 20)
    let o = b.createVector(numbers, size: numbers.count)
    b.finish(offset: o)
    XCTAssertEqual(b.sizedByteArray, [4, 0, 0, 0, 0, 0, 0, 0])
  }

  func testCreateVectorOfStrings() {
    let strs = ["Denmark", "Norway"]
    var b = FlatBufferBuilder(initialSize: 20)
    let o = b.createVector(ofStrings: strs)
    b.finish(offset: o)
    // swiftformat:disable all
    XCTAssertEqual(b.sizedByteArray, [4, 0, 0, 0, 2, 0, 0, 0, 20, 0, 0, 0, 4, 0, 0, 0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0, 7, 0, 0, 0, 68, 101, 110, 109, 97, 114, 107, 0])
    // swiftformat:enable all
  }
  func testCreateSharedStringVector() {
    let norway = "Norway"
    let denmark = "Denmark"
    var b = FlatBufferBuilder(initialSize: 20)
    let noStr = b.createShared(string: norway)
    let deStr = b.createShared(string: denmark)
    let _noStr = b.createShared(string: norway)
    let _deStr = b.createShared(string: denmark)
    let v = [noStr, deStr, _noStr, _deStr]
    let end = b.createVector(ofOffsets: v)
    b.finish(offset: end)
    // swiftformat:disable all
    XCTAssertEqual(b.sizedByteArray, [4, 0, 0, 0, 4, 0, 0, 0, 28, 0, 0, 0, 12, 0, 0, 0, 20, 0, 0, 0, 4, 0, 0, 0, 7, 0, 0, 0, 68, 101, 110, 109, 97, 114, 107, 0, 6, 0, 0, 0, 78, 111, 114, 119, 97, 121, 0, 0])
    // swiftformat:enable all
  }

  func testReadInt32Array() {
    let data: [Int32] = [1, 2, 3, 4, 5]
    var b = FlatBufferBuilder(initialSize: 20)
    let v = Numbers.createNumbersVector(b: &b, array: data)
    let end = Numbers.createNumbers(b: &b, o: v)
    b.finish(offset: end)
    let number = Numbers.getRootAsNumbers(ByteBuffer(bytes: b.sizedByteArray))
    XCTAssertEqual(number.vArrayInt32, [1, 2, 3, 4, 5])
  }

  func testReadDoubleArray() {
    let data: [Double] = [1, 2, 3, 4, 5]
    var b = FlatBufferBuilder(initialSize: 20)
    let v = Numbers.createNumbersVector(b: &b, array: data)
    let end = Numbers.createNumbers(b: &b, o: v)
    b.finish(offset: end)
    let number = Numbers.getRootAsNumbers(ByteBuffer(bytes: b.sizedByteArray))
    XCTAssertEqual(number.vArrayDouble, [1, 2, 3, 4, 5])
  }

  func testHasForArray() {
    var builder = FlatBufferBuilder(initialSize: 20)
    let emptyVector = [UInt8]()
    let emptyOffset = builder.createVector(emptyVector)
    let nonEmptyVector = [1, 2, 3]
    let nonEmptyVectorOffest = builder.createVector(nonEmptyVector)
    let start = Swift_Tests_Vectors.startVectors(&builder)
    Swift_Tests_Vectors.addVectorOf(empty: emptyOffset, &builder)
    Swift_Tests_Vectors.addVectorOf(array: nonEmptyVectorOffest, &builder)
    let finish = Swift_Tests_Vectors.endVectors(&builder, start: start)
    builder.finish(offset: finish)

    var byteBuffer = ByteBuffer(bytes: builder.sizedByteArray)
    let msg: Swift_Tests_Vectors = getRoot(byteBuffer: &byteBuffer)
    XCTAssertEqual(msg.hasNone, false)
    XCTAssertEqual(msg.hasEmpty, true)
    XCTAssertEqual(msg.emptyCount, 0)
    XCTAssertEqual(msg.hasArray, true)
    XCTAssertEqual(msg.arrayCount, 3)
    XCTAssertEqual(msg.array, [1, 2, 3])
  }
}

struct Numbers {

  private var __t: Table

  private init(_ t: Table) {
    __t = t
  }

  @inlinable
  static func getRootAsNumbers(_ bb: ByteBuffer) -> Numbers {
    Numbers(Table(
      bb: bb,
      position: Int32(bb.read(def: UOffset.self, position: 0))))
  }

  var vArrayInt: [Int]? { __t.getVector(at: 4) }
  var vArrayInt32: [Int32]? { __t.getVector(at: 4) }
  var vArrayDouble: [Double]? { __t.getVector(at: 4) }
  var vArrayFloat: [Float32]? { __t.getVector(at: 4) }

  static func createNumbersVector(
    b: inout FlatBufferBuilder,
    array: [Int]) -> Offset
  {
    b.createVector(array, size: array.count)
  }

  static func createNumbersVector(
    b: inout FlatBufferBuilder,
    array: [Int32]) -> Offset
  {
    b.createVector(array, size: array.count)
  }

  static func createNumbersVector(
    b: inout FlatBufferBuilder,
    array: [Double]) -> Offset
  {
    b.createVector(array, size: array.count)
  }

  static func createNumbersVector(
    b: inout FlatBufferBuilder,
    array: [Float32]) -> Offset
  {
    b.createVector(array, size: array.count)
  }

  static func createNumbers(b: inout FlatBufferBuilder, o: Offset) -> Offset {
    let start = b.startTable(with: 1)
    b.add(offset: o, at: 4)
    return Offset(offset: b.endTable(at: start))
  }
}
