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

#if !os(WASI)
import Foundation
#else
import SwiftOverlayShims
#endif

/// `TableVerifier` verifies a table object is within a provided memory.
/// It checks if all the objects for a specific generated table, are within
/// the bounds of the buffer, aligned.
public struct TableVerifier {

  /// position of current table in `ByteBuffer`
  fileprivate var _position: Int

  /// Current VTable position
  fileprivate var _vtable: Int

  /// Length of current VTable
  fileprivate var _vtableLength: Int

  /// `Verifier` object created in the base verifable call.
  fileprivate var _verifier: Verifier

  /// Creates a `TableVerifier` verifier that allows the Flatbuffer object
  /// to verify the buffer before accessing any of the data.
  ///
  /// - Parameters:
  ///   - position: Current table Position
  ///   - vtable: Current `VTable` position
  ///   - vtableLength: Current `VTable` length
  ///   - verifier: `Verifier` Object  that caches the data of the verifiable object
  internal init(
    position: Int,
    vtable: Int,
    vtableLength: Int,
    verifier: inout Verifier)
  {
    _position = position
    _vtable = vtable
    _vtableLength = vtableLength
    _verifier = verifier
  }

  /// Dereference the current object position from the `VTable`
  /// - Parameter field: Current VTable refrence to  position.
  /// - Throws: A `FlatbuffersErrors` incase the voffset is not aligned/outOfBounds/apparentSizeTooLarge
  /// - Returns: An optional position for current field
  internal mutating func dereference(_ field: VOffset) throws -> Int? {
    if field >= _vtableLength {
      return nil
    }

    /// Reading the offset for the field needs to be read.
    let offset: VOffset = try _verifier.getValue(
      at: Int(clamping: _vtable &+ Int(field)))

    if offset > 0 {
      return Int(clamping: _position &+ Int(offset))
    }
    return nil
  }

  /// Visits all the fields within the table to validate the integrity
  /// of the data
  /// - Parameters:
  ///   - field: voffset of the current field to be read
  ///   - fieldName: fieldname to report data Errors.
  ///   - required: If the field has to be available in the buffer
  ///   - type: Type of field to be read
  /// - Throws: A `FlatbuffersErrors` where the field is corrupt
  public mutating func visit<T>(
    field: VOffset,
    fieldName: String,
    required: Bool,
    type: T.Type) throws where T: Verifiable
  {
    let derefValue = try dereference(field)

    if let value = derefValue {
      try T.verify(&_verifier, at: value, of: T.self)
      return
    }
    if required {
      throw FlatbuffersErrors.requiredFieldDoesntExist(
        position: field,
        name: fieldName)
    }
  }

  /// Visits all the fields for a union object within the table to
  /// validate the integrity of the data
  /// - Parameters:
  ///   - key: Current Key Voffset
  ///   - field: Current field Voffset
  ///   - unionKeyName: Union key name
  ///   - fieldName: Field key name
  ///   - required: indicates if an object is required to be present
  ///   - completion: Completion is a handler that WILL be called in the generated
  /// - Throws: A `FlatbuffersErrors` where the field is corrupt
  public mutating func visit<T>(
    unionKey key: VOffset,
    unionField field: VOffset,
    unionKeyName: String,
    fieldName: String,
    required: Bool,
    completion: @escaping (inout Verifier, T, Int) throws -> Void) throws
    where T: UnionEnum
  {
    let keyPos = try dereference(key)
    let valPos = try dereference(field)

    if keyPos == nil && valPos == nil {
      if required {
        throw FlatbuffersErrors.requiredFieldDoesntExist(
          position: key,
          name: unionKeyName)
      }
      return
    }

    if let _key = keyPos,
       let _val = valPos
    {
      /// verifiying that the key is within the buffer
      try T.T.verify(&_verifier, at: _key, of: T.T.self)
      guard let _enum = try T.init(value: _verifier._buffer.read(
        def: T.T.self,
        position: _key)) else
      {
        throw FlatbuffersErrors.unknownUnionCase
      }
      /// we are assuming that Unions will always be of type Uint8
      try completion(
        &_verifier,
        _enum,
        _val)
      return
    }
    throw FlatbuffersErrors.valueNotFound(
      key: keyPos,
      keyName: unionKeyName,
      field: valPos,
      fieldName: fieldName)
  }

  /// Visits and validates all the objects within a union vector
  /// - Parameters:
  ///   - key: Current Key Voffset
  ///   - field: Current field Voffset
  ///   - unionKeyName: Union key name
  ///   - fieldName: Field key name
  ///   - required: indicates if an object is required to be present
  ///   - completion: Completion is a handler that WILL be called in the generated
  /// - Throws: A `FlatbuffersErrors` where the field is corrupt
  public mutating func visitUnionVector<T>(
    unionKey key: VOffset,
    unionField field: VOffset,
    unionKeyName: String,
    fieldName: String,
    required: Bool,
    completion: @escaping (inout Verifier, T, Int) throws -> Void) throws
    where T: UnionEnum
  {
    let keyVectorPosition = try dereference(key)
    let offsetVectorPosition = try dereference(field)

    if let keyPos = keyVectorPosition,
       let valPos = offsetVectorPosition
    {
      try UnionVector<T>.verify(
        &_verifier,
        keyPosition: keyPos,
        fieldPosition: valPos,
        unionKeyName: unionKeyName,
        fieldName: fieldName,
        completion: completion)
      return
    }
    if required {
      throw FlatbuffersErrors.requiredFieldDoesntExist(
        position: field,
        name: fieldName)
    }
  }

  /// Finishs the current Table verifier, and subtracts the current
  /// table from the incremented depth.
  public mutating func finish() {
    _verifier.finish()
  }
}
