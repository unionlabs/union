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

/// Verifier that check if the buffer passed into it is a valid,
/// safe, aligned Flatbuffers object since swift read from `unsafeMemory`
public struct Verifier {

  /// Flag to check for alignment if true
  fileprivate let _checkAlignment: Bool
  /// Capacity of the current buffer
  fileprivate var _capacity: Int
  /// Current ApparentSize
  fileprivate var _apparentSize: UOffset = 0
  /// Amount of tables present within a buffer
  fileprivate var _tableCount = 0

  /// Capacity of the buffer
  internal var capacity: Int { _capacity }
  /// Current reached depth within the buffer
  internal var _depth = 0
  /// Current verifiable ByteBuffer
  internal var _buffer: ByteBuffer
  /// Options for verification
  internal let _options: VerifierOptions

  /// Initializer for the verifier
  /// - Parameters:
  ///   - buffer: Bytebuffer that is required to be verified
  ///   - options: `VerifierOptions` that set the rule for some of the verification done
  ///   - checkAlignment: If alignment check is required to be preformed
  /// - Throws: `exceedsMaxSizeAllowed` if capacity of the buffer is more than 2GiB
  public init(
    buffer: inout ByteBuffer,
    options: VerifierOptions = .init(),
    checkAlignment: Bool = true) throws
  {
    guard buffer.capacity < FlatBufferMaxSize else {
      throw FlatbuffersErrors.exceedsMaxSizeAllowed
    }

    _buffer = buffer
    _capacity = buffer.capacity
    _checkAlignment = checkAlignment
    _options = options
  }

  /// Resets the verifier to initial state
  public mutating func reset() {
    _depth = 0
    _tableCount = 0
  }

  /// Checks if the value of type `T` is aligned properly in the buffer
  /// - Parameters:
  ///   - position: Current position
  ///   - type: Type of value to check
  /// - Throws: `missAlignedPointer` if the pointer is not aligned properly
  public mutating func isAligned<T>(position: Int, type: T.Type) throws {

    /// If check alignment is false this mutating function doesnt continue
    if !_checkAlignment { return }

    /// advance pointer to position X
    let ptr = _buffer._storage.memory.advanced(by: position)
    /// Check if the pointer is aligned
    if Int(bitPattern: ptr) & (MemoryLayout<T>.alignment &- 1) == 0 {
      return
    }

    throw FlatbuffersErrors.missAlignedPointer(
      position: position,
      type: String(describing: T.self))
  }

  /// Checks if the value of Size "X" is within the range of the buffer
  /// - Parameters:
  ///   - position: Current postion to be read
  ///   - size: `Byte` Size of readable object within the buffer
  /// - Throws: `outOfBounds` if the value is out of the bounds of the buffer
  /// and `apparentSizeTooLarge` if the apparent size is bigger than the one specified
  /// in `VerifierOptions`
  public mutating func rangeInBuffer(position: Int, size: Int) throws {
    let end = UInt(clamping: (position &+ size).magnitude)
    if end > _buffer.capacity {
      throw FlatbuffersErrors.outOfBounds(position: end, end: capacity)
    }
    _apparentSize = _apparentSize &+ UInt32(size)
    if _apparentSize > _options._maxApparentSize {
      throw FlatbuffersErrors.apparentSizeTooLarge
    }
  }

  /// Validates if a value of type `T` is aligned and within the bounds of
  /// the buffer
  /// - Parameters:
  ///   - position: Current readable position
  ///   - type: Type of value to check
  /// - Throws: FlatbuffersErrors
  public mutating func inBuffer<T>(position: Int, of type: T.Type) throws {
    try isAligned(position: position, type: type)
    try rangeInBuffer(position: position, size: MemoryLayout<T>.size)
  }

  /// Visits a table at the current position and validates if the table meets
  /// the rules specified in the `VerifierOptions`
  /// - Parameter position: Current position to be read
  /// - Throws: FlatbuffersErrors
  /// - Returns: A `TableVerifier` at the current readable table
  public mutating func visitTable(at position: Int) throws -> TableVerifier {
    let vtablePosition = try derefOffset(position: position)
    let vtableLength: VOffset = try getValue(at: vtablePosition)

    let length = Int(vtableLength)
    try isAligned(
      position: Int(clamping: (vtablePosition + length).magnitude),
      type: VOffset.self)
    try rangeInBuffer(position: vtablePosition, size: length)

    _tableCount += 1

    if _tableCount > _options._maxTableCount {
      throw FlatbuffersErrors.maximumTables
    }

    _depth += 1

    if _depth > _options._maxDepth {
      throw FlatbuffersErrors.maximumDepth
    }

    return TableVerifier(
      position: position,
      vtable: vtablePosition,
      vtableLength: length,
      verifier: &self)
  }

  /// Validates if a value of type `T` is within the buffer and returns it
  /// - Parameter position: Current position to be read
  /// - Throws: `inBuffer` errors
  /// - Returns: a value of type `T` usually a `VTable` or a table offset
  internal mutating func getValue<T>(at position: Int) throws -> T {
    try inBuffer(position: position, of: T.self)
    return _buffer.read(def: T.self, position: position)
  }

  /// derefrences an offset within a vtable to get the position of the field
  /// in the bytebuffer
  /// - Parameter position: Current readable position
  /// - Throws: `inBuffer` errors & `signedOffsetOutOfBounds`
  /// - Returns: Current readable position for a field
  @inline(__always)
  internal mutating func derefOffset(position: Int) throws -> Int {
    try inBuffer(position: position, of: Int32.self)

    let offset = _buffer.read(def: Int32.self, position: position)
    // switching to int32 since swift's default Int is int64
    // this should be safe since we already checked if its within
    // the buffer
    let _int32Position = UInt32(position)

    let reportedOverflow: (partialValue: UInt32, overflow: Bool)
    if offset > 0 {
      reportedOverflow = _int32Position
        .subtractingReportingOverflow(offset.magnitude)
    } else {
      reportedOverflow = _int32Position
        .addingReportingOverflow(offset.magnitude)
    }

    /// since `subtractingReportingOverflow` & `addingReportingOverflow` returns true,
    /// if there is overflow we return failure
    if reportedOverflow.overflow || reportedOverflow.partialValue > _buffer
      .capacity
    {
      throw FlatbuffersErrors.signedOffsetOutOfBounds(
        offset: Int(offset),
        position: position)
    }

    return Int(reportedOverflow.partialValue)
  }

  /// finishes the current iteration of verification on an object
  internal mutating func finish() {
    _depth -= 1
  }

  mutating func verify(id: String) throws {
    let size = MemoryLayout<Int32>.size
    let str = _buffer.readString(at: size, count: size)
    if id == str {
      return
    }
    throw FlatbuffersErrors.bufferIdDidntMatchPassedId
  }

}
