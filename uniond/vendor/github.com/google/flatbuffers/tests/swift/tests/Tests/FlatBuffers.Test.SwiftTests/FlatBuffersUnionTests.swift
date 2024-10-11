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

final class FlatBuffersUnionTests: XCTestCase {

  func testCreateMonstor() {

    var b = FlatBufferBuilder(initialSize: 20)
    let dmg: Int16 = 5
    let str = "Axe"
    let axe = b.create(string: str)
    let weapon = Weapon.createWeapon(builder: &b, offset: axe, dmg: dmg)
    let weapons = b.createVector(ofOffsets: [weapon])
    let root = LocalMonster.createMonster(
      builder: &b,
      offset: weapons,
      equipment: .Weapon,
      equippedOffset: weapon.o)
    b.finish(offset: root)
    let buffer = b.sizedByteArray
    // swiftformat:disable all
    XCTAssertEqual(buffer, [16, 0, 0, 0, 0, 0, 10, 0, 16, 0, 8, 0, 7, 0, 12, 0, 10, 0, 0, 0, 0, 0, 0, 1, 8, 0, 0, 0, 20, 0, 0, 0, 1, 0, 0, 0, 12, 0, 0, 0, 8, 0, 12, 0, 8, 0, 6, 0, 8, 0, 0, 0, 0, 0, 5, 0, 4, 0, 0, 0, 3, 0, 0, 0, 65, 120, 101, 0])
    // swiftformat:enable all
    let monster = LocalMonster.getRootAsMonster(bb: ByteBuffer(bytes: buffer))
    XCTAssertEqual(monster.weapon(at: 0)?.dmg, dmg)
    XCTAssertEqual(monster.weapon(at: 0)?.name, str)
    XCTAssertEqual(monster.weapon(at: 0)?.nameVector, [65, 120, 101])
    let p: Weapon? = monster.equiped()
    XCTAssertEqual(p?.dmg, dmg)
    XCTAssertEqual(p?.name, str)
    XCTAssertEqual(p?.nameVector, [65, 120, 101])
  }

  func testEndTableFinish() {
    var builder = FlatBufferBuilder(initialSize: 20)
    let sword = builder.create(string: "Sword")
    let axe = builder.create(string: "Axe")
    let weaponOne = Weapon.createWeapon(
      builder: &builder,
      offset: sword,
      dmg: 3)
    let weaponTwo = Weapon.createWeapon(builder: &builder, offset: axe, dmg: 5)
    let name = builder.create(string: "Orc")
    let inventory: [UInt8] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    let inv = builder.createVector(inventory, size: 10)
    let weapons = builder.createVector(ofOffsets: [weaponOne, weaponTwo])
    let path = builder.createVector(ofStructs: [
      Vec(x: 4.0, y: 5.0, z: 6.0),
      Vec(x: 1.0, y: 2.0, z: 3.0),
    ])
    let orc = FinalMonster.createMonster(
      builder: &builder,
      position: Vec(x: 1, y: 2, z: 3),
      hp: 300,
      name: name,
      inventory: inv,
      color: .red,
      weapons: weapons,
      equipment: .Weapon,
      equippedOffset: weaponTwo,
      path: path)
    builder.finish(offset: orc)
    // swiftformat:disable all
    XCTAssertEqual(builder.sizedByteArray, [32, 0, 0, 0, 0, 0, 26, 0, 48, 0, 36, 0, 0, 0, 34, 0, 28, 0, 0, 0, 24, 0, 23, 0, 16, 0, 15, 0, 8, 0, 4, 0, 26, 0, 0, 0, 44, 0, 0, 0, 104, 0, 0, 0, 0, 0, 0, 1, 60, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 76, 0, 0, 0, 0, 0, 44, 1, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 2, 0, 0, 0, 0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 2, 0, 0, 0, 52, 0, 0, 0, 28, 0, 0, 0, 10, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 3, 0, 0, 0, 79, 114, 99, 0, 244, 255, 255, 255, 0, 0, 5, 0, 24, 0, 0, 0, 8, 0, 12, 0, 8, 0, 6, 0, 8, 0, 0, 0, 0, 0, 3, 0, 12, 0, 0, 0, 3, 0, 0, 0, 65, 120, 101, 0, 5, 0, 0, 0, 83, 119, 111, 114, 100, 0, 0, 0])
    // swiftformat:enable all
  }

  func testEnumVector() {
    let vectorOfEnums: [ColorsNameSpace.RGB] = [.blue, .green]

    var builder = FlatBufferBuilder(initialSize: 1)
    let off = builder.createVector(vectorOfEnums)
    let start = ColorsNameSpace.Monster.startMonster(&builder)
    ColorsNameSpace.Monster.add(colors: off, &builder)
    let end = ColorsNameSpace.Monster.endMonster(&builder, start: start)
    builder.finish(offset: end)
    // swiftformat:disable all
    XCTAssertEqual(builder.sizedByteArray, [12, 0, 0, 0, 0, 0, 6, 0, 8, 0, 4, 0, 6, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0])
    // swiftformat:enable all
    let monster = ColorsNameSpace.Monster.getRootAsMonster(bb: builder.buffer)
    XCTAssertEqual(monster.colorsCount, 2)
    XCTAssertEqual(monster.colors(at: 0), .blue)
    XCTAssertEqual(monster.colors(at: 1), .green)
  }

  func testUnionVector() {
    var fb = FlatBufferBuilder()

    let swordDmg: Int32 = 8
    let attackStart = Attacker.startAttacker(&fb)
    Attacker.add(swordAttackDamage: swordDmg, &fb)
    let attack = Attacker.endAttacker(&fb, start: attackStart)

    let characterType: [Character] = [.belle, .mulan, .bookfan]

    let characters = [
      fb.create(struct: BookReader(booksRead: 7)),
      attack,
      fb.create(struct: BookReader(booksRead: 2)),
    ]
    let types = fb.createVector(characterType)
    let characterVector = fb.createVector(ofOffsets: characters)
    let end = Movie.createMovie(
      &fb,
      charactersTypeVectorOffset: types,
      charactersVectorOffset: characterVector)
    Movie.finish(&fb, end: end)

    var buffer = fb.buffer
    var movie: Movie = getRoot(byteBuffer: &buffer)
    XCTAssertEqual(movie.charactersTypeCount, Int32(characterType.count))
    XCTAssertEqual(movie.charactersCount, Int32(characters.count))

    for i in 0..<movie.charactersTypeCount {
      XCTAssertEqual(movie.charactersType(at: i), characterType[Int(i)])
    }

    XCTAssertEqual(
      movie.characters(at: 0, type: BookReader_Mutable.self)?.booksRead,
      7)
    XCTAssertEqual(
      movie.characters(at: 1, type: Attacker.self)?.swordAttackDamage,
      swordDmg)
    XCTAssertEqual(
      movie.characters(at: 2, type: BookReader_Mutable.self)?.booksRead,
      2)

    var objc: MovieT? = movie.unpack()
    XCTAssertEqual(
      movie.charactersTypeCount,
      Int32(objc?.characters.count ?? 0))
    XCTAssertEqual(
      movie.characters(at: 0, type: BookReader_Mutable.self)?.booksRead,
      (objc?.characters[0]?.value as? BookReader)?.booksRead)
    fb.clear()
    let newMovie = Movie.pack(&fb, obj: &objc)
    fb.finish(offset: newMovie)

    var _buffer = fb.buffer
    let packedMovie: Movie = getRoot(byteBuffer: &_buffer)

    XCTAssertEqual(
      packedMovie.characters(at: 0, type: BookReader_Mutable.self)?.booksRead,
      movie.characters(at: 0, type: BookReader_Mutable.self)?.booksRead)
    XCTAssertEqual(
      packedMovie.characters(at: 1, type: Attacker.self)?.swordAttackDamage,
      movie.characters(at: 1, type: Attacker.self)?.swordAttackDamage)
    XCTAssertEqual(
      packedMovie.characters(at: 2, type: BookReader_Mutable.self)?.booksRead,
      movie.characters(at: 2, type: BookReader_Mutable.self)?.booksRead)
  }

  func testStringUnion() {
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

    var buffer = fb.sizedBuffer
    var movie: Movie = getRoot(byteBuffer: &buffer)
    XCTAssertEqual(movie.mainCharacter(type: String.self), string)
    XCTAssertEqual(
      movie.characters(at: 0, type: BookReader_Mutable.self)?.booksRead,
      7)
    XCTAssertEqual(movie.characters(at: 1, type: String.self), string)

    var objc: MovieT? = movie.unpack()
    XCTAssertEqual(objc?.mainCharacter?.value as? String, string)
    XCTAssertEqual((objc?.characters[0]?.value as? BookReader)?.booksRead, 7)
    XCTAssertEqual(objc?.characters[1]?.value as? String, string)
    fb.clear()
    let newMovie = Movie.pack(&fb, obj: &objc)
    fb.finish(offset: newMovie)

    var _buffer = fb.buffer
    let packedMovie: Movie = getRoot(byteBuffer: &_buffer)
    XCTAssertEqual(packedMovie.mainCharacter(type: String.self), string)
    XCTAssertEqual(
      packedMovie.characters(at: 0, type: BookReader_Mutable.self)?.booksRead,
      7)
    XCTAssertEqual(packedMovie.characters(at: 1, type: String.self), string)
  }

  func testEncoding() {
    let string = "Awesome \\\\t\t\nstring!"
    var fb = FlatBufferBuilder()

    let stringOffset = fb.create(string: string)

    let swordDmg: Int32 = 8
    let attackStart = Attacker.startAttacker(&fb)
    Attacker.add(swordAttackDamage: swordDmg, &fb)
    let attack = Attacker.endAttacker(&fb, start: attackStart)

    let characterType: [Character] = [.belle, .mulan, .bookfan, .other]

    let characters = [
      fb.create(struct: BookReader(booksRead: 7)),
      attack,
      fb.create(struct: BookReader(booksRead: 2)),
      stringOffset,
    ]
    let types = fb.createVector(characterType)
    let characterVector = fb.createVector(ofOffsets: characters)
    let end = Movie.createMovie(
      &fb,
      charactersTypeVectorOffset: types,
      charactersVectorOffset: characterVector)
    Movie.finish(&fb, end: end)

    var sizedBuffer = fb.sizedBuffer
    do {
      let reader: Movie = try getCheckedRoot(byteBuffer: &sizedBuffer)
      let encoder = JSONEncoder()
      encoder.keyEncodingStrategy = .convertToSnakeCase
      _ = try encoder.encode(reader)
    } catch {
      XCTFail(error.localizedDescription)
    }
  }

  var jsonData: String {
    "{\"characters_type\":[\"Belle\",\"MuLan\",\"BookFan\",\"Other\"],\"characters\":[{\"books_read\":7},{\"sword_attack_damage\":8},{\"books_read\":2},\"Awesome \\\\\\\\t\\t\\nstring!\"]}"
  }
}

public enum ColorsNameSpace {

  enum RGB: Int32, Enum {
    typealias T = Int32
    static var byteSize: Int { MemoryLayout<Int32>.size }
    var value: Int32 { rawValue }
    case red = 0, green = 1, blue = 2
  }

  struct Monster: FlatBufferObject {
    var __buffer: ByteBuffer! { _accessor.bb }

    private var _accessor: Table
    static func getRootAsMonster(bb: ByteBuffer) -> Monster { Monster(Table(
      bb: bb,
      position: Int32(bb.read(def: UOffset.self, position: bb.reader)) +
        Int32(bb.reader))) }

    init(_ t: Table) { _accessor = t }
    init(_ bb: ByteBuffer, o: Int32) { _accessor = Table(bb: bb, position: o) }

    public var colorsCount: Int32 {
      let o = _accessor.offset(4); return o == 0 ? 0 : _accessor
        .vector(count: o) }
    public func colors(at index: Int32) -> ColorsNameSpace
      .RGB?
    { let o = _accessor.offset(4); return o == 0 ? ColorsNameSpace
      .RGB(rawValue: 0)! : ColorsNameSpace.RGB(rawValue: _accessor.directRead(
        of: Int32.self,
        offset: _accessor.vector(at: o) + index * 4)) }
    static func startMonster(_ fbb: inout FlatBufferBuilder) -> UOffset { fbb
      .startTable(with: 1) }
    static func add(colors: Offset, _ fbb: inout FlatBufferBuilder) { fbb.add(
      offset: colors,
      at: 4)  }
    static func endMonster(
      _ fbb: inout FlatBufferBuilder,
      start: UOffset)
      -> Offset
    { let end = Offset(offset: fbb.endTable(at: start)); return end
    }
  }
}


enum Equipment: Byte { case none, Weapon }

enum Color3: Int8 { case red = 0, green, blue }

struct FinalMonster {

  @inlinable
  static func createMonster(
    builder: inout FlatBufferBuilder,
    position: Vec,
    hp: Int16,
    name: Offset,
    inventory: Offset,
    color: Color3,
    weapons: Offset,
    equipment: Equipment = .none,
    equippedOffset: Offset,
    path: Offset) -> Offset
  {
    let start = builder.startTable(with: 11)
    builder.create(struct: position, position: 4)
    builder.add(element: hp, def: 100, at: 8)
    builder.add(offset: name, at: 10)
    builder.add(offset: inventory, at: 14)
    builder.add(element: color.rawValue, def: Color3.green.rawValue, at: 16)
    builder.add(offset: weapons, at: 18)
    builder.add(
      element: equipment.rawValue,
      def: Equipment.none.rawValue,
      at: 20)
    builder.add(offset: equippedOffset, at: 22)
    builder.add(offset: path, at: 24)
    return Offset(offset: builder.endTable(at: start))
  }
}

struct LocalMonster {

  private var __t: Table

  init(_ fb: ByteBuffer, o: Int32) { __t = Table(bb: fb, position: o) }
  init(_ t: Table) { __t = t }

  func weapon(at index: Int32) -> Weapon? { let o = __t
    .offset(4); return o == 0 ? nil : Weapon.assign(
      __t.indirect(__t.vector(at: o) + (index * 4)),
      __t.bb) }

  func equiped<T: FlatBufferObject>() -> T? {
    let o = __t.offset(8); return o == 0 ? nil : __t.union(o)
  }

  static func getRootAsMonster(bb: ByteBuffer) -> LocalMonster {
    LocalMonster(Table(
      bb: bb,
      position: Int32(bb.read(def: UOffset.self, position: 0))))
  }

  @inlinable
  static func createMonster(
    builder: inout FlatBufferBuilder,
    offset: Offset,
    equipment: Equipment = .none,
    equippedOffset: UOffset) -> Offset
  {
    let start = builder.startTable(with: 3)
    builder.add(element: equippedOffset, def: 0, at: 8)
    builder.add(offset: offset, at: 4)
    builder.add(
      element: equipment.rawValue,
      def: Equipment.none.rawValue,
      at: 6)
    return Offset(offset: builder.endTable(at: start))
  }
}

struct Weapon: FlatBufferObject {

  var __buffer: ByteBuffer! { __t.bb }

  static let offsets: (name: VOffset, dmg: VOffset) = (4, 6)
  private var __t: Table

  init(_ t: Table) { __t = t }
  init(_ fb: ByteBuffer, o: Int32) { __t = Table(bb: fb, position: o)}

  var dmg: Int16 { let o = __t.offset(6); return o == 0 ? 0 : __t.readBuffer(
    of: Int16.self,
    at: o) }
  var nameVector: [UInt8]? { __t.getVector(at: 4) }
  var name: String? {
    let o = __t.offset(4); return o == 0 ? nil : __t.string(at: o) }

  static func assign(_ i: Int32, _ bb: ByteBuffer) -> Weapon { Weapon(Table(
    bb: bb,
    position: i)) }

  @inlinable
  static func createWeapon(
    builder: inout FlatBufferBuilder,
    offset: Offset,
    dmg: Int16) -> Offset
  {
    let _start = builder.startTable(with: 2)
    Weapon.add(builder: &builder, name: offset)
    Weapon.add(builder: &builder, dmg: dmg)
    return Weapon.end(builder: &builder, startOffset: _start)
  }

  @inlinable
  static func end(
    builder: inout FlatBufferBuilder,
    startOffset: UOffset) -> Offset
  {
    Offset(offset: builder.endTable(at: startOffset))
  }

  @inlinable
  static func add(builder: inout FlatBufferBuilder, name: Offset) {
    builder.add(offset: name, at: Weapon.offsets.name)
  }

  @inlinable
  static func add(builder: inout FlatBufferBuilder, dmg: Int16) {
    builder.add(element: dmg, def: 0, at: Weapon.offsets.dmg)
  }
}
