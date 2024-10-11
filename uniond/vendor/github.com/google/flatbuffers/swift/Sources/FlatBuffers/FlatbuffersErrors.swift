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

/// Collection of thrown from the Flatbuffer verifier
public enum FlatbuffersErrors: Error, Equatable {

  /// Thrown when verifying a file id that doesnt match buffer id
  case bufferIdDidntMatchPassedId
  /// Prefixed size doesnt match the current (readable) buffer size
  case prefixedSizeNotEqualToBufferSize
  /// Thrown when buffer is bigger than the allowed 2GiB
  case exceedsMaxSizeAllowed
  /// Thrown when there is an missaligned pointer at position
  /// of type
  case missAlignedPointer(position: Int, type: String)
  /// Thrown when trying to read a value that goes out of the
  /// current buffer bounds
  case outOfBounds(position: UInt, end: Int)
  /// Thrown when the signed offset is out of the bounds of the
  /// current buffer
  case signedOffsetOutOfBounds(offset: Int, position: Int)
  /// Thrown when a required field doesnt exist within the buffer
  case requiredFieldDoesntExist(position: VOffset, name: String)
  /// Thrown when a string is missing its NULL Terminator `\0`,
  /// this can be disabled in the `VerifierOptions`
  case missingNullTerminator(position: Int, str: String?)
  /// Thrown when the verifier has reached the maximum tables allowed,
  /// this can be disabled in the `VerifierOptions`
  case maximumTables
  /// Thrown when the verifier has reached the maximum depth allowed,
  /// this can be disabled in the `VerifierOptions`
  case maximumDepth
  /// Thrown when the verifier is presented with an unknown union case
  case unknownUnionCase
  /// thrown when a value for a union is not found within the buffer
  case valueNotFound(key: Int?, keyName: String, field: Int?, fieldName: String)
  /// thrown when the size of the keys vector doesnt match fields vector
  case unionVectorSize(
    keyVectorSize: Int,
    fieldVectorSize: Int,
    unionKeyName: String,
    fieldName: String)
  case apparentSizeTooLarge

}

#if !os(WASI)

extension FlatbuffersErrors {
  public static func == (
    lhs: FlatbuffersErrors,
    rhs: FlatbuffersErrors) -> Bool
  {
    lhs.localizedDescription == rhs.localizedDescription
  }
}

#endif
