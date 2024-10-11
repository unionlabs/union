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

/// `VerifierOptions` is a set of options to verify a flatbuffer
public struct VerifierOptions {

  /// Maximum `Apparent` size if the buffer can be expanded into a DAG tree
  internal var _maxApparentSize: UOffset

  /// Maximum table count allowed in a buffer
  internal var _maxTableCount: UOffset

  /// Maximum depth allowed in a buffer
  internal var _maxDepth: UOffset

  /// Ignoring missing null terminals in strings
  internal var _ignoreMissingNullTerminators: Bool

  /// initializes the set of options for the verifier
  /// - Parameters:
  ///   - maxDepth: Maximum depth allowed in a buffer
  ///   - maxTableCount: Maximum table count allowed in a buffer
  ///   - maxApparentSize: Maximum `Apparent` size if the buffer can be expanded into a DAG tree
  ///   - ignoreMissingNullTerminators: Ignoring missing null terminals in strings *Currently not supported in swift*
  public init(
    maxDepth: UOffset = 64,
    maxTableCount: UOffset = 1000000,
    maxApparentSize: UOffset = 1 << 31,
    ignoreMissingNullTerminators: Bool = false)
  {
    _maxDepth = maxDepth
    _maxTableCount = maxTableCount
    _maxApparentSize = maxApparentSize
    _ignoreMissingNullTerminators = ignoreMissingNullTerminators
  }

}
