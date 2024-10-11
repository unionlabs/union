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

/// NativeObject is a protocol that all of the `Object-API` generated code should be
/// conforming to since it allows developers the ease of use to pack and unpack their
/// Flatbuffers objects
public protocol NativeObject {}

extension NativeObject {

  /// Serialize is a helper function that serailizes the data from the Object API to a bytebuffer directly th
  /// - Parameter type: Type of the Flatbuffer object
  /// - Returns: returns the encoded sized ByteBuffer
  public func serialize<T: ObjectAPIPacker>(type: T.Type) -> ByteBuffer
    where T.T == Self
  {
    var builder = FlatBufferBuilder(initialSize: 1024)
    return serialize(builder: &builder, type: type.self)
  }

  /// Serialize is a helper function that serailizes the data from the Object API to a bytebuffer directly.
  ///
  /// - Parameters:
  ///   - builder: A FlatBufferBuilder
  ///   - type: Type of the Flatbuffer object
  /// - Returns: returns the encoded sized ByteBuffer
  /// - Note: The `serialize(builder:type)` can be considered as a function that allows you to create smaller builder instead of the default `1024`.
  ///  It can be considered less expensive in terms of memory allocation
  public func serialize<T: ObjectAPIPacker>(
    builder: inout FlatBufferBuilder,
    type: T.Type) -> ByteBuffer where T.T == Self
  {
    var s = self
    let root = type.pack(&builder, obj: &s)
    builder.finish(offset: root)
    return builder.sizedBuffer
  }
}
