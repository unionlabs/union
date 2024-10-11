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

/// Struct is a representation of a mutable `Flatbuffers` struct
/// since native structs are value types and cant be mutated
@frozen
public struct Struct {

  /// Hosting Bytebuffer
  public private(set) var bb: ByteBuffer
  /// Current position of the struct
  public private(set) var postion: Int32

  /// Initializer for a mutable flatbuffers struct
  /// - Parameters:
  ///   - bb: Current hosting Bytebuffer
  ///   - position: Current position for the struct in the ByteBuffer
  public init(bb: ByteBuffer, position: Int32 = 0) {
    self.bb = bb
    postion = position
  }

  /// Reads data from the buffer directly at offset O
  /// - Parameters:
  ///   - type: Type of data to be read
  ///   - o: Current offset of the data
  /// - Returns: Data of Type T that conforms to type Scalar
  public func readBuffer<T: Scalar>(of type: T.Type, at o: Int32) -> T {
    let r = bb.read(def: T.self, position: Int(o + postion))
    return r
  }
}
