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

extension String: Verifiable {

  /// Verifies that the current value is which the bounds of the buffer, and if
  /// the current `Value` is aligned properly
  /// - Parameters:
  ///   - verifier: Verifier that hosts the buffer
  ///   - position: Current position within the buffer
  ///   - type: The type of the object to be verified
  /// - Throws: Errors coming from `inBuffer`, `missingNullTerminator` and `outOfBounds`
  public static func verify<T>(
    _ verifier: inout Verifier,
    at position: Int,
    of type: T.Type) throws where T: Verifiable
  {

    let range = try String.verifyRange(&verifier, at: position, of: UInt8.self)
    /// Safe &+ since we already check for overflow in verify range
    let stringLen = range.start &+ range.count

    if stringLen >= verifier.capacity {
      throw FlatbuffersErrors.outOfBounds(
        position: UInt(clamping: stringLen.magnitude),
        end: verifier.capacity)
    }

    let isNullTerminated = verifier._buffer.read(
      def: UInt8.self,
      position: stringLen) == 0

    if !verifier._options._ignoreMissingNullTerminators && !isNullTerminated {
      let str = verifier._buffer.readString(at: range.start, count: range.count)
      throw FlatbuffersErrors.missingNullTerminator(
        position: position,
        str: str)
    }
  }
}

extension String: FlatbuffersInitializable {

  /// Initailizes a string from a Flatbuffers ByteBuffer
  /// - Parameters:
  ///   - bb: ByteBuffer containing the readable string
  ///   - o: Current position
  public init(_ bb: ByteBuffer, o: Int32) {
    let v = Int(o)
    let count = bb.read(def: Int32.self, position: v)
    self = bb.readString(
      at: MemoryLayout<Int32>.size + v,
      count: Int(count)) ?? ""
  }
}

extension String: ObjectAPIPacker {

  public static func pack(
    _ builder: inout FlatBufferBuilder,
    obj: inout String?) -> Offset
  {
    guard var obj = obj else { return Offset() }
    return pack(&builder, obj: &obj)
  }

  public static func pack(
    _ builder: inout FlatBufferBuilder,
    obj: inout String) -> Offset
  {
    builder.create(string: obj)
  }

  public mutating func unpack() -> String {
    self
  }

}

extension String: NativeObject {

  public func serialize<T: ObjectAPIPacker>(type: T.Type) -> ByteBuffer
    where T.T == Self
  {
    fatalError("serialize should never be called from string directly")
  }

  public func serialize<T: ObjectAPIPacker>(
    builder: inout FlatBufferBuilder,
    type: T.Type) -> ByteBuffer where T.T == Self
  {
    fatalError("serialize should never be called from string directly")
  }
}
