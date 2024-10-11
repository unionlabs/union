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

final class FlatbuffersVerifierTests: XCTestCase {

  lazy var validStorage: ByteBuffer.Storage = ByteBuffer.Storage(
    count: Int(FlatBufferMaxSize) - 1,
    alignment: 1)
  lazy var errorStorage: ByteBuffer.Storage = ByteBuffer.Storage(
    count: Int(FlatBufferMaxSize) + 1,
    alignment: 1)

  var buffer: ByteBuffer!

  var validFlatbuffersObject: ByteBuffer!
  var invalidFlatbuffersObject: ByteBuffer!
  var invalidFlatbuffersObject2: ByteBuffer!
  var invalidFlatbuffersObject3: ByteBuffer!

  override func setUp() {
    // swiftformat:disable all
    buffer = ByteBuffer(initialSize: 32)
    add(buffer: &buffer, v: 4, p: 16)
    add(buffer: &buffer, v: 4, p: 20)

    validFlatbuffersObject = ByteBuffer(bytes: [48, 0, 0, 0, 77, 79, 78, 83, 0, 0, 0, 0, 36, 0, 72, 0, 40, 0, 0, 0, 38, 0, 32, 0, 0, 0, 28, 0, 0, 0, 27, 0, 20, 0, 16, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 11, 0, 36, 0, 0, 0, 164, 0, 0, 0, 0, 0, 0, 1, 60, 0, 0, 0, 68, 0, 0, 0, 76, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 120, 0, 0, 0, 0, 0, 80, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0, 2, 0, 0, 0, 64, 0, 0, 0, 48, 0, 0, 0, 2, 0, 0, 0, 30, 0, 40, 0, 10, 0, 20, 0, 152, 255, 255, 255, 4, 0, 0, 0, 4, 0, 0, 0, 70, 114, 101, 100, 0, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 50, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 0, 0, 0, 9, 0, 0, 0, 77, 121, 77, 111, 110, 115, 116, 101, 114, 0, 0, 0, 3, 0, 0, 0, 20, 0, 0, 0, 36, 0, 0, 0, 4, 0, 0, 0, 240, 255, 255, 255, 32, 0, 0, 0, 248, 255, 255, 255, 36, 0, 0, 0, 12, 0, 8, 0, 0, 0, 0, 0, 0, 0, 4, 0, 12, 0, 0, 0, 28, 0, 0, 0, 5, 0, 0, 0, 87, 105, 108, 109, 97, 0, 0, 0, 6, 0, 0, 0, 66, 97, 114, 110, 101, 121, 0, 0, 5, 0, 0, 0, 70, 114, 111, 100, 111, 0, 0, 0])

    invalidFlatbuffersObject = ByteBuffer(bytes: [0, 48, 0, 0, 0, 77, 79, 78, 83, 0, 0, 0, 0, 36, 0, 72, 0, 40, 0, 0, 0, 38, 0, 32, 0, 0, 0, 28, 0, 0, 0, 27, 0, 20, 0, 16, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 11, 0, 36, 0, 0, 0, 164, 0, 0, 0, 0, 0, 0, 1, 60, 0, 0, 0, 68, 0, 0, 0, 76, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 120, 0, 0, 0, 0, 0, 80, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0, 2, 0, 0, 0, 64, 0, 0, 0, 48, 0, 0, 0, 2, 0, 0, 0, 30, 0, 40, 0, 10, 0, 20, 0, 152, 255, 255, 255, 4, 0, 0, 0, 4, 0, 0, 0, 70, 114, 101, 100, 0, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 50, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 0, 0, 0, 9, 0, 0, 0, 77, 121, 77, 111, 110, 115, 116, 101, 114, 0, 0, 0, 3, 0, 0, 0, 20, 0, 0, 0, 36, 0, 0, 0, 4, 0, 0, 0, 240, 255, 255, 255, 32, 0, 0, 0, 248, 255, 255, 255, 36, 0, 0, 0, 12, 0, 8, 0, 0, 0, 0, 0, 0, 0, 4, 0, 12, 0, 0, 0, 28, 0, 0, 0, 5, 0, 0, 0, 87, 105, 108, 109, 97, 0, 0, 0, 6, 0, 0, 0, 66, 97, 114, 110, 101, 121, 0, 0, 5, 0, 0, 0, 70, 114, 111, 100, 111, 0, 0, 0])

    // Array failure within a the inventory array
    invalidFlatbuffersObject2 = ByteBuffer(bytes: [48, 0, 0, 0, 77, 79, 78, 83, 0, 0, 0, 0, 36, 0, 72, 0, 40, 0, 0, 0, 38, 0, 32, 0, 0, 0, 28, 0, 0, 0, 27, 0, 20, 0, 16, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 11, 0, 36, 0, 0, 0, 164, 0, 0, 0, 0, 0, 0, 1, 60, 0, 0, 0, 68, 0, 0, 0, 76, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 120, 0, 0, 0, 0, 0, 80, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0, 2, 0, 0, 0, 64, 0, 0, 0, 48, 0, 0, 0, 2, 0, 0, 0, 30, 0, 40, 0, 10, 0, 20, 0, 152, 255, 255, 255, 4, 0, 0, 0, 4, 0, 0, 0, 70, 114, 101, 100, 0, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 0x00, 3, 4, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 50, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 0, 0, 0, 9, 0, 0, 0, 77, 121, 77, 111, 110, 115, 116, 101, 114, 0, 0, 0, 3, 0, 0, 0, 20, 0, 0, 0, 36, 0, 0, 0, 4, 0, 0, 0, 240, 255, 255, 255, 32, 0, 0, 0, 248, 255, 255, 255, 36, 0, 0, 0, 12, 0, 8, 0, 0, 0, 0, 0, 0, 0, 4, 0, 12, 0, 0, 0, 28, 0, 0, 0, 5, 0, 0, 0, 87, 105, 108, 109, 97, 0, 0, 0, 6, 0, 0, 0, 66, 97, 114, 110, 101, 121, 0, 0, 5, 0, 0, 0, 70, 114, 111, 100, 111, 0, 0, 0])

    // Array failure within a the strings array
    invalidFlatbuffersObject3 = ByteBuffer(bytes: [48, 0, 0, 0, 77, 79, 78, 83, 0, 0, 0, 0, 36, 0, 72, 0, 40, 0, 0, 0, 38, 0, 32, 0, 0, 0, 28, 0, 0, 0, 27, 0, 20, 0, 16, 0, 12, 0, 4, 0, 0, 0, 0, 0, 0, 0, 11, 0, 36, 0, 0, 0, 164, 0, 0, 0, 0, 0, 0, 1, 60, 0, 0, 0, 68, 0, 0, 0, 76, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 120, 0, 0, 0, 0, 0, 80, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 2, 0, 5, 0, 6, 0, 0, 0, 2, 0, 0, 0, 64, 0, 0, 0, 48, 0, 0, 0, 2, 0, 0, 0, 30, 0, 40, 0, 10, 0, 20, 0, 152, 255, 255, 255, 4, 0, 0, 0, 4, 0, 0, 0, 70, 114, 101, 100, 0, 0, 0, 0, 5, 0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 50, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 0, 0, 0, 9, 0, 0, 0, 77, 121, 77, 111, 110, 115, 116, 101, 114, 0, 0, 0, 3, 0, 0, 0, 20, 0, 0, 0, 36, 0, 0, 0, 4, 0, 0, 0, 240, 255, 255, 255, 32, 0, 0, 0, 248, 255, 255, 255, 36, 0, 0, 0, 12, 0, 8, 0, 0, 0, 0, 0, 0, 0, 4, 0, 12, 0, 0, 0, 28, 0, 0, 0, 5, 0, 0, 0, 87, 105, 108, 109, 97, 0, 0, 0, 6, 0, 0, 0, 66, 97, 114, 110, 101, 121, 0, 0, 5, 0, 0, 0, 70, 114, 111, 100, 0x00, 111, 0, 0, 0])
    // swiftformat:enable all
  }

  func testVeriferInitPassing() {
    var buffer = ByteBuffer(initialSize: 0)
    buffer._storage = validStorage
    XCTAssertNoThrow(try Verifier(buffer: &buffer))
  }

  func testVeriferInitFailing() {
    var buffer = ByteBuffer(initialSize: 0)
    buffer._storage = errorStorage
    XCTAssertThrowsError(try Verifier(buffer: &buffer))
  }

  func testVerifierCheckAlignment() {
    var verifier = try! Verifier(buffer: &buffer)
    do {
      try verifier.isAligned(position: 20, type: Int.self)
    } catch {
      XCTAssertEqual(
        error as? FlatbuffersErrors,
        .missAlignedPointer(position: 20, type: "Int"))
    }
    XCTAssertNoThrow(try verifier.isAligned(position: 16, type: Int.self))

    var newVerifer = try! Verifier(buffer: &buffer, checkAlignment: false)
    XCTAssertNoThrow(try newVerifer.isAligned(position: 16, type: Int.self))
  }

  func testRangeInBuffer() {
    var verifier = try! Verifier(buffer: &buffer)
    let size = MemoryLayout<Int64>.size
    XCTAssertNoThrow(try verifier.rangeInBuffer(position: 24, size: size))
    XCTAssertThrowsError(try verifier.rangeInBuffer(position: 26, size: size))
    XCTAssertThrowsError(try verifier.rangeInBuffer(position: 26, size: size))
    XCTAssertThrowsError(try verifier.rangeInBuffer(position: 30, size: size))
    XCTAssertThrowsError(try verifier.rangeInBuffer(position: 32, size: size))
    XCTAssertThrowsError(try verifier.rangeInBuffer(position: 34, size: size))

    verifier = try! Verifier(
      buffer: &buffer,
      options: .init(maxDepth: 0, maxTableCount: 0, maxApparentSize: 4))
    do {
      try verifier.rangeInBuffer(position: 24, size: size)
    } catch {
      XCTAssertEqual(
        error as! FlatbuffersErrors,
        .apparentSizeTooLarge)
    }
  }

  func testPositionInBuffer() {
    var verifier = try! Verifier(buffer: &buffer)
    XCTAssertNoThrow(try verifier.inBuffer(position: 0, of: Int64.self))
    XCTAssertNoThrow(try verifier.inBuffer(position: 24, of: Int64.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: -9, of: Int64.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: 25, of: Int64.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: 26, of: Int32.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: 26, of: Int64.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: 30, of: Int64.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: 32, of: Int64.self))
    XCTAssertThrowsError(try verifier.inBuffer(position: 34, of: Int64.self))
  }

  func testVisitTable() {
    var verifier = try! Verifier(buffer: &validFlatbuffersObject)
    XCTAssertNoThrow(try verifier.visitTable(at: 48))
    verifier.reset()
  }

  func testTableVerifier() {
    var verifier = try! Verifier(buffer: &validFlatbuffersObject)

    var tableVerifer = try! verifier.visitTable(at: 48)
    XCTAssertNoThrow(try tableVerifer.visit(
      field: 4,
      fieldName: "Vec",
      required: false,
      type: Vec3.self))
    XCTAssertNoThrow(try tableVerifer.visit(
      field: 8,
      fieldName: "hp",
      required: false,
      type: Int16.self))

    XCTAssertNoThrow(try tableVerifer.visit(
      field: 10,
      fieldName: "name",
      required: true,
      type: ForwardOffset<String>.self))

    XCTAssertNoThrow(try tableVerifer.visit(
      field: 14,
      fieldName: "inventory",
      required: false,
      type: ForwardOffset<Vector<UInt8, UInt8>>.self))

    XCTAssertNoThrow(try tableVerifer.visit(
      field: 22,
      fieldName: "test4",
      required: false,
      type: ForwardOffset<Vector<MyGame_Example_Test, MyGame_Example_Test>>
        .self))

    XCTAssertNoThrow(try tableVerifer.visit(
      field: 24,
      fieldName: "Vector of strings",
      required: false,
      type: ForwardOffset<Vector<ForwardOffset<String>, String>>.self))

    do {
      try tableVerifer.visit(
        field: 13,
        fieldName: "notvalid",
        required: false,
        type: Int16.self)
    } catch {
      XCTAssertEqual(
        error as! FlatbuffersErrors,
        .missAlignedPointer(position: 25, type: "UInt16"))
    }

    do {
      try tableVerifer.visit(
        unionKey: 18,
        unionField: 20,
        unionKeyName: "testType",
        fieldName: "test",
        required: false,
        completion: { (verifier, key: MyGame_Example_Any_, pos) in
          switch key {
          case .none_:
            break
          case .monster:
            try ForwardOffset<MyGame_Example_Monster>.verify(
              &verifier,
              at: pos,
              of: MyGame_Example_Monster.self)

          case .testsimpletablewithenum:
            break
          case .mygameExample2Monster:
            break
          }
        })
    } catch {
      XCTAssertEqual(
        error as! FlatbuffersErrors,
        .missAlignedPointer(position: 25, type: "UInt16"))
    }
  }

  func testVerifyUnionVectors() {
    // swiftformat:disable all
    var byteBuffer = ByteBuffer(bytes: [20, 0, 0, 0, 77, 79, 86, 73, 12, 0, 12, 0, 0, 0, 0, 0, 8, 0, 4, 0, 12, 0, 0, 0, 8, 0, 0, 0, 20, 0, 0, 0, 3, 0, 0, 0, 24, 0, 0, 0, 32, 0, 0, 0, 12, 0, 0, 0, 3, 0, 0, 0, 3, 1, 4, 0, 2, 0, 0, 0, 7, 0, 0, 0, 0, 0, 6, 0, 8, 0, 4, 0, 6, 0, 0, 0, 8, 0, 0, 0])
    // swiftformat:enable all
    XCTAssertNoThrow(try getCheckedRoot(byteBuffer: &byteBuffer) as Movie)
  }

  func testErrorWrongFileId() {
    // swiftformat:disable all
    var byteBuffer = ByteBuffer(bytes: [20, 0, 0, 0, 77, 79, 86, 73, 12, 0, 12, 0, 0, 0, 0, 0, 8, 0, 4, 0, 12, 0, 0, 0, 8, 0, 0, 0, 20, 0, 0, 0, 3, 0, 0, 0, 24, 0, 0, 0, 32, 0, 0, 0, 12, 0, 0, 0, 3, 0, 0, 0, 3, 1, 4, 0, 2, 0, 0, 0, 7, 0, 0, 0, 0, 0, 6, 0, 8, 0, 4, 0, 6, 0, 0, 0, 8, 0, 0, 0])
    // swiftformat:enable all
    XCTAssertThrowsError(try getCheckedRoot(
      byteBuffer: &byteBuffer,
      fileId: "FLEX") as Movie)
  }

  func testVerifyPrefixedBuffer() {
    // swiftformat:disable all
    var byteBuffer = ByteBuffer(bytes: [0, 0, 0, 1, 20, 0, 0, 0, 77, 79, 86, 73, 12, 0, 12, 0, 0, 0, 0, 0, 8, 0, 4, 0, 12, 0, 0, 0, 8, 0, 0, 0, 20, 0, 0, 0, 3, 0, 0, 0, 24, 0, 0, 0, 32, 0, 0, 0, 12, 0, 0, 0, 3, 0, 0, 0, 3, 1, 4, 0, 2, 0, 0, 0, 7, 0, 0, 0, 0, 0, 6, 0, 8, 0, 4, 0, 6, 0, 0, 0, 8, 0, 0, 0])
    // swiftformat:enable all
    XCTAssertThrowsError(
      try getCheckedPrefixedSizeRoot(byteBuffer: &byteBuffer) as Movie)
  }

  func testFullVerifier() {
    XCTAssertNoThrow(
      try getCheckedRoot(
        byteBuffer: &validFlatbuffersObject) as MyGame_Example_Monster)
  }

  func testFullVerifierWithFileId() {
    XCTAssertNoThrow(
      try getCheckedRoot(
        byteBuffer: &validFlatbuffersObject,
        fileId: MyGame_Example_Monster.id) as MyGame_Example_Monster)
  }

  func testInvalidBuffer() {
    XCTAssertThrowsError(
      try getCheckedRoot(
        byteBuffer: &invalidFlatbuffersObject) as MyGame_Example_Monster)
  }

  func testInvalidBuffer2() {
    XCTAssertThrowsError(
      try getCheckedRoot(
        byteBuffer: &invalidFlatbuffersObject2) as MyGame_Example_Monster)
  }

  func testInvalidBuffer3() {
    XCTAssertThrowsError(
      try getCheckedRoot(
        byteBuffer: &invalidFlatbuffersObject3) as MyGame_Example_Monster)
  }

  func testValidUnionBuffer() {
    let string = "Awesome \\\\t\t\nstring!"
    var fb = FlatBufferBuilder()
    let stringOffset = fb.create(string: string)
    let characterType: [Character] = [.bookfan, .other]

    let characters = [
      fb.create(struct: BookReader(booksRead: 7)),
      stringOffset,
    ]
    let types = fb.createVector(characterType)
    let characterVector = fb.createVector(ofOffsets: characters)

    let end = Movie.createMovie(
      &fb,
      mainCharacterType: .other,
      mainCharacterOffset: Offset(offset: stringOffset.o),
      charactersTypeVectorOffset: types,
      charactersVectorOffset: characterVector)
    Movie.finish(&fb, end: end)
    var buf = fb.sizedBuffer
    XCTAssertNoThrow(try getCheckedRoot(byteBuffer: &buf) as Movie)
  }

  func add(buffer: inout ByteBuffer, v: Int32, p: Int) {
    buffer.write(value: v, index: p)
  }
}
