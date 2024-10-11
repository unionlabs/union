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

/// Mutable is a protocol that allows us to mutate Scalar values within a ``ByteBuffer``
public protocol Mutable {
  /// makes Flatbuffer accessed within the Protocol
  var bb: ByteBuffer { get }
  /// makes position of the ``Table``/``struct`` accessed within the Protocol
  var postion: Int32 { get }
}

extension Mutable {

  /// Mutates the memory in the buffer, this is only called from the access function of ``Table`` and ``struct``
  /// - Parameters:
  ///   - value: New value to be inserted to the buffer
  ///   - index: index of the Element
  func mutate<T: Scalar>(value: T, o: Int32) -> Bool {
    guard o != 0 else { return false }
    bb.write(value: value, index: Int(o), direct: true)
    return true
  }
}

extension Mutable where Self == Table {

  /// Mutates a value by calling mutate with respect to the position in a ``Table``
  /// - Parameters:
  ///   - value: New value to be inserted to the buffer
  ///   - index: index of the Element
  public func mutate<T: Scalar>(_ value: T, index: Int32) -> Bool {
    guard index != 0 else { return false }
    return mutate(value: value, o: index + postion)
  }

  /// Directly mutates the element by calling mutate
  ///
  /// Mutates the Element at index ignoring the current position by calling mutate
  /// - Parameters:
  ///   - value: New value to be inserted to the buffer
  ///   - index: index of the Element
  public func directMutate<T: Scalar>(_ value: T, index: Int32) -> Bool {
    mutate(value: value, o: index)
  }
}

extension Mutable where Self == Struct {

  /// Mutates a value by calling mutate with respect to the position in the struct
  /// - Parameters:
  ///   - value: New value to be inserted to the buffer
  ///   - index: index of the Element
  public func mutate<T: Scalar>(_ value: T, index: Int32) -> Bool {
    mutate(value: value, o: index + postion)
  }

  /// Directly mutates the element by calling mutate
  ///
  /// Mutates the Element at index ignoring the current position by calling mutate
  /// - Parameters:
  ///   - value: New value to be inserted to the buffer
  ///   - index: index of the Element
  public func directMutate<T: Scalar>(_ value: T, index: Int32) -> Bool {
    mutate(value: value, o: index)
  }
}

extension Struct: Mutable {}
extension Table: Mutable {}
