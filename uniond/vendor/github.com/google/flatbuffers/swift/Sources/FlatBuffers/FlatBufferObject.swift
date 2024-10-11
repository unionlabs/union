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

/// NativeStruct is a protocol that indicates if the struct is a native `swift` struct
/// since now we will be serializing native structs into the buffer.
public protocol NativeStruct {}

/// FlatbuffersInitializable is a protocol that allows any object to be
/// Initialized from a ByteBuffer
public protocol FlatbuffersInitializable {
  /// Any flatbuffers object that confirms to this protocol is going to be
  /// initializable through this initializer
  init(_ bb: ByteBuffer, o: Int32)
}

/// FlatbufferObject structures all the Flatbuffers objects
public protocol FlatBufferObject: FlatbuffersInitializable {
  var __buffer: ByteBuffer! { get }
}

/// ``ObjectAPIPacker`` is a protocol that allows object to pack and unpack from a
/// ``NativeObject`` to a flatbuffers Object and vice versa.
public protocol ObjectAPIPacker {
  /// associatedtype to the object that should be unpacked.
  associatedtype T

  /// ``pack(_:obj:)-3ptws`` tries to pacs the variables of a native Object into the `ByteBuffer` by using
  /// a FlatBufferBuilder
  /// - Parameters:
  ///   - builder: FlatBufferBuilder that will host incoming data
  ///   - obj: Object of associatedtype to the current implementer
  ///
  /// ``pack(_:obj:)-3ptws`` can be called by passing through an already initialized ``FlatBufferBuilder``
  /// or it can be called by using the public API that will create a new ``FlatBufferBuilder``
  static func pack(_ builder: inout FlatBufferBuilder, obj: inout T?) -> Offset

  /// ``pack(_:obj:)-20ipk`` packs the variables of a native Object into the `ByteBuffer` by using
  /// the FlatBufferBuilder
  /// - Parameters:
  ///   - builder: FlatBufferBuilder that will host incoming data
  ///   - obj: Object of associatedtype to the current implementer
  ///
  /// ``pack(_:obj:)-20ipk`` can be called by passing through an already initialized ``FlatBufferBuilder``
  /// or it can be called by using the public API that will create a new ``FlatBufferBuilder``
  static func pack(_ builder: inout FlatBufferBuilder, obj: inout T) -> Offset

  /// ``unpack()`` unpacks a ``FlatBuffers`` object into a Native swift object.
  mutating func unpack() -> T
}
