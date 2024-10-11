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

/// Takes in a prefixed sized buffer, where the prefixed size would be skipped.
/// And would verify that the buffer passed is a valid `Flatbuffers` Object.
/// - Parameters:
///   - byteBuffer: Buffer that needs to be checked and read
///   - options: Verifier options
/// - Throws: FlatbuffersErrors
/// - Returns: Returns a valid, checked Flatbuffers object
///
/// ``getPrefixedSizeCheckedRoot(byteBuffer:options:)`` would skip the first Bytes in
/// the ``ByteBuffer`` and verifies the buffer by calling ``getCheckedRoot(byteBuffer:options:)``
public func getPrefixedSizeCheckedRoot<T: FlatBufferObject & Verifiable>(
  byteBuffer: inout ByteBuffer,
  fileId: String? = nil,
  options: VerifierOptions = .init()) throws -> T
{
  byteBuffer.skipPrefix()
  return try getCheckedRoot(
    byteBuffer: &byteBuffer,
    fileId: fileId,
    options: options)
}

/// Takes in a prefixed sized buffer, where we check if the sized buffer is equal to prefix size.
/// And would verify that the buffer passed is a valid `Flatbuffers` Object.
/// - Parameters:
///   - byteBuffer: Buffer that needs to be checked and read
///   - options: Verifier options
/// - Throws: FlatbuffersErrors
/// - Returns: Returns a valid, checked Flatbuffers object
///
/// ``getPrefixedSizeCheckedRoot(byteBuffer:options:)`` would skip the first Bytes in
/// the ``ByteBuffer`` and verifies the buffer by calling ``getCheckedRoot(byteBuffer:options:)``
public func getCheckedPrefixedSizeRoot<T: FlatBufferObject & Verifiable>(
  byteBuffer: inout ByteBuffer,
  fileId: String? = nil,
  options: VerifierOptions = .init()) throws -> T
{
  let prefix = byteBuffer.skipPrefix()
  if prefix != byteBuffer.size {
    throw FlatbuffersErrors.prefixedSizeNotEqualToBufferSize
  }
  return try getCheckedRoot(
    byteBuffer: &byteBuffer,
    fileId: fileId,
    options: options)
}

/// Takes in a prefixed sized buffer, where the prefixed size would be skipped.
/// Returns a `NON-Checked` flatbuffers object
/// - Parameter byteBuffer: Buffer that contains data
/// - Returns: Returns a Flatbuffers object
///
/// ``getPrefixedSizeCheckedRoot(byteBuffer:options:)`` would skip the first Bytes in
/// the ``ByteBuffer`` and then calls ``getRoot(byteBuffer:)``
public func getPrefixedSizeRoot<T: FlatBufferObject>(byteBuffer: inout ByteBuffer)
  -> T
{
  byteBuffer.skipPrefix()
  return getRoot(byteBuffer: &byteBuffer)

}

/// Verifies that the buffer passed is a valid `Flatbuffers` Object.
/// - Parameters:
///   - byteBuffer: Buffer that needs to be checked and read
///   - options: Verifier options
/// - Throws: FlatbuffersErrors
/// - Returns: Returns a valid, checked Flatbuffers object
///
/// ``getCheckedRoot(byteBuffer:options:)`` Takes in a ``ByteBuffer`` and verifies
/// that by creating a ``Verifier`` and checkes if all the `Bytes` and correctly aligned
/// and within the ``ByteBuffer`` range.
public func getCheckedRoot<T: FlatBufferObject & Verifiable>(
  byteBuffer: inout ByteBuffer,
  fileId: String? = nil,
  options: VerifierOptions = .init()) throws -> T
{
  var verifier = try Verifier(buffer: &byteBuffer, options: options)
  if let fileId = fileId {
    try verifier.verify(id: fileId)
  }
  try ForwardOffset<T>.verify(&verifier, at: 0, of: T.self)
  return T.init(
    byteBuffer,
    o: Int32(byteBuffer.read(def: UOffset.self, position: byteBuffer.reader)) +
      Int32(byteBuffer.reader))
}

/// Returns a `NON-Checked` flatbuffers object
/// - Parameter byteBuffer: Buffer that contains data
/// - Returns: Returns a Flatbuffers object
public func getRoot<T: FlatBufferObject>(byteBuffer: inout ByteBuffer) -> T {
  T.init(
    byteBuffer,
    o: Int32(byteBuffer.read(def: UOffset.self, position: byteBuffer.reader)) +
      Int32(byteBuffer.reader))
}
