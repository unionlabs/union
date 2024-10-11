/*
 * Copyright 2019 Google Inc. All rights reserved.
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

package com.google.flatbuffers;

import static com.google.flatbuffers.Constants.*;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.Charset;

/**
 * Helper type for accessing vector of signed or unsigned 8-bit values.
 */
public final class ByteVector extends BaseVector {
  /**
   * Assigns vector access object to vector data.
   *
   * @param vector Start data of a vector.
   * @param bb Table's ByteBuffer.
   * @return Returns current vector access object assigned to vector data whose offset is stored at
   *         `vector`.
   */
  public ByteVector __assign(int vector, ByteBuffer bb) { 
    __reset(vector, Constants.SIZEOF_BYTE, bb); return this;
  }

  /**
   * Reads the byte at the given index.
   *
   * @param j The index from which the byte will be read.
   * @return the 8-bit value at the given index.
   */
  public byte get(int j) {
     return bb.get(__element(j));
  }

  /**
   * Reads the byte at the given index, zero-extends it to type int, and returns the result,
   * which is therefore in the range 0 through 255.
   *
   * @param j The index from which the byte will be read.
   * @return the unsigned 8-bit at the given index.
   */
  public int getAsUnsigned(int j) {
    return (int) get(j) & 0xFF;
  }
}
