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

/// Enum is a protocol that all flatbuffers enums should conform to
/// Since it allows us to get the actual `ByteSize` and `Value` from
/// a swift enum.
public protocol Enum {
  /// associatedtype that the type of the enum should conform to
  associatedtype T: Scalar & Verifiable
  /// Size of the current associatedtype in the enum
  static var byteSize: Int { get }
  /// The current value the enum hosts
  var value: T { get }
}

extension Enum where Self: Verifiable {

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

/// UnionEnum is a Protocol that allows us to create Union type of enums
/// and their value initializers. Since an `init` was required by
/// the verifier
public protocol UnionEnum: Enum {
  init?(value: T) throws
}
