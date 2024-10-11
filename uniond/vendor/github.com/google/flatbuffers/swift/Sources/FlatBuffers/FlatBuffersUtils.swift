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

/// FlatBuffersUtils hosts some utility functions that might be useful
public enum FlatBuffersUtils {

  /// Gets the size of the prefix
  /// - Parameter bb: Flatbuffer object
  public static func getSizePrefix(bb: ByteBuffer) -> Int32 {
    bb.read(def: Int32.self, position: bb.reader)
  }

  /// Removes the prefix by duplicating the Flatbuffer this call is expensive since its
  /// creates a new buffer use `readPrefixedSizeCheckedRoot` instead
  /// unless a completely new buffer is required
  /// - Parameter bb: Flatbuffer object
  ///
  ///
  public static func removeSizePrefix(bb: ByteBuffer) -> ByteBuffer {
    bb.duplicate(removing: MemoryLayout<Int32>.size)
  }
}
