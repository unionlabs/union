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

extension Int {

  /// Moves the current int into the nearest power of two
  ///
  /// This is used since the UnsafeMutableRawPointer will face issues when writing/reading
  /// if the buffer alignment exceeds that actual size of the buffer
  var convertToPowerofTwo: Int {
    guard self > 0 else { return 1 }
    var n = UOffset(self)

    #if arch(arm) || arch(i386)
    let max = UInt32(Int.max)
    #else
    let max = UInt32.max
    #endif

    n -= 1
    n |= n >> 1
    n |= n >> 2
    n |= n >> 4
    n |= n >> 8
    n |= n >> 16
    if n != max {
      n += 1
    }

    return Int(n)
  }
}
