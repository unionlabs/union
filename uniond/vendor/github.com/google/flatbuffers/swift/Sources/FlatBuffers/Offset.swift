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

/// Offset object for all the Objects that are written into the buffer
public struct Offset {
  /// Offset of the object in the buffer
  public var o: UOffset
  /// Returns false if the offset is equal to zero
  public var isEmpty: Bool { o == 0 }

  public init(offset: UOffset) { o = offset }
  public init() { o = 0 }
}
