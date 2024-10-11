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

/// Verifiable is a protocol all swift flatbuffers object should conform to,
/// since swift is similar to `cpp` and `rust` where the data is read directly
/// from `unsafeMemory` thus the need to verify if the buffer received is a valid one
public protocol Verifiable {

  /// Verifies that the current value is which the bounds of the buffer, and if
  /// the current `Value` is aligned properly
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - position: Current position within the buffer
  ///   - type: The type of the object to be verified
  /// - Throws: Errors coming from `inBuffer` function
  static func verify<T>(
    _ verifier: inout Verifier,
    at position: Int,
    of type: T.Type) throws where T: Verifiable
}

extension Verifiable {

  /// Verifies if the current range to be read is within the bounds of the buffer,
  /// and if the range is properly aligned
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - position: Current position within the buffer
  ///   - type: The type of the object to be verified
  /// - Throws: Erros thrown from `isAligned` & `rangeInBuffer`
  /// - Returns: a tuple of the start position and the count of objects within the range
  @discardableResult
  public static func verifyRange<T>(
    _ verifier: inout Verifier,
    at position: Int, of type: T.Type) throws -> (start: Int, count: Int)
  {
    let len: UOffset = try verifier.getValue(at: position)
    let intLen = Int(len)
    let start = Int(clamping: (position &+ MemoryLayout<Int32>.size).magnitude)
    try verifier.isAligned(position: start, type: type.self)
    try verifier.rangeInBuffer(position: start, size: intLen)
    return (start, intLen)
  }
}

extension Verifiable where Self: Scalar {

  /// Verifies that the current value is which the bounds of the buffer, and if
  /// the current `Value` is aligned properly
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - position: Current position within the buffer
  ///   - type: The type of the object to be verified
  /// - Throws: Errors coming from `inBuffer` function
  public static func verify<T>(
    _ verifier: inout Verifier,
    at position: Int,
    of type: T.Type) throws where T: Verifiable
  {
    try verifier.inBuffer(position: position, of: type.self)
  }
}

// MARK: - ForwardOffset

/// ForwardOffset is a container to wrap around the Generic type to be verified
/// from the flatbuffers object.
public enum ForwardOffset<U>: Verifiable where U: Verifiable {

  /// Verifies that the current value is which the bounds of the buffer, and if
  /// the current `Value` is aligned properly
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - position: Current position within the buffer
  ///   - type: The type of the object to be verified
  /// - Throws: Errors coming from `inBuffer` function
  public static func verify<T>(
    _ verifier: inout Verifier,
    at position: Int,
    of type: T.Type) throws where T: Verifiable
  {
    let offset: UOffset = try verifier.getValue(at: position)
    let nextOffset = Int(clamping: (Int(offset) &+ position).magnitude)
    try U.verify(&verifier, at: nextOffset, of: U.self)
  }
}

// MARK: - Vector

/// Vector is a container to wrap around the Generic type to be verified
/// from the flatbuffers object.
public enum Vector<U, S>: Verifiable where U: Verifiable, S: Verifiable {

  /// Verifies that the current value is which the bounds of the buffer, and if
  /// the current `Value` is aligned properly
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - position: Current position within the buffer
  ///   - type: The type of the object to be verified
  /// - Throws: Errors coming from `inBuffer` function
  public static func verify<T>(
    _ verifier: inout Verifier,
    at position: Int,
    of type: T.Type) throws where T: Verifiable
  {
    /// checks if the next verification type S is equal to U of type forwardOffset
    /// This had to be done since I couldnt find a solution for duplicate call functions
    /// A fix will be appreciated
    if U.self is ForwardOffset<S>.Type {
      let range = try verifyRange(&verifier, at: position, of: UOffset.self)
      for index in stride(
        from: range.start,
        to: Int(
          clamping: range
            .start &+ (range.count &* MemoryLayout<Int32>.size)),
        by: MemoryLayout<UOffset>.size)
      {
        try U.verify(&verifier, at: index, of: U.self)
      }
    } else {
      try S.verifyRange(&verifier, at: position, of: S.self)
    }
  }
}

// MARK: - UnionVector

/// UnionVector is a container to wrap around the Generic type to be verified
/// from the flatbuffers object.
public enum UnionVector<S> where S: UnionEnum {

  /// Completion handler for the function Verify, that passes the verifier
  /// enum type and position of union field
  public typealias Completion = (inout Verifier, S, Int) throws -> Void

  /// Verifies if the current range to be read is within the bounds of the buffer,
  /// and if the range is properly aligned. It also verifies if the union type is a
  /// *valid/supported* union type.
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - keyPosition: Current union key position within the buffer
  ///   - fieldPosition: Current union field position within the buffer
  ///   - unionKeyName: Name of key to written if error is presented
  ///   - fieldName: Name of field to written if error is presented
  ///   - completion: Completion is a handler that WILL be called in the generated
  ///   code to verify the actual objects
  /// - Throws: FlatbuffersErrors
  public static func verify(
    _ verifier: inout Verifier,
    keyPosition: Int,
    fieldPosition: Int,
    unionKeyName: String,
    fieldName: String,
    completion: @escaping Completion) throws
  {
    /// Get offset for union key vectors and offset vectors
    let keyOffset: UOffset = try verifier.getValue(at: keyPosition)
    let fieldOffset: UOffset = try verifier.getValue(at: fieldPosition)

    /// Check if values are within the buffer, returns the start position of vectors, and vector counts
    /// Using &+ is safe since we already verified that the value is within the buffer, where the max is
    /// going to be 2Gib and swift supports Int64 by default
    let keysRange = try S.T.verifyRange(
      &verifier,
      at: Int(keyOffset) &+ keyPosition,
      of: S.T.self)
    let offsetsRange = try UOffset.verifyRange(
      &verifier,
      at: Int(fieldOffset) &+ fieldPosition,
      of: UOffset.self)

    guard keysRange.count == offsetsRange.count else {
      throw FlatbuffersErrors.unionVectorSize(
        keyVectorSize: keysRange.count,
        fieldVectorSize: offsetsRange.count,
        unionKeyName: unionKeyName,
        fieldName: fieldName)
    }

    var count = 0
    /// Iterate over the vector of keys and offsets.
    while count < keysRange.count {

      /// index of readable enum value in array
      let keysIndex = MemoryLayout<S.T>.size * count
      guard let _enum = try S.init(value: verifier._buffer.read(
        def: S.T.self,
        position: keysRange.start + keysIndex)) else
      {
        throw FlatbuffersErrors.unknownUnionCase
      }
      /// index of readable offset value in array
      let fieldIndex = MemoryLayout<UOffset>.size * count
      try completion(&verifier, _enum, offsetsRange.start + fieldIndex)
      count += 1
    }
  }
}
