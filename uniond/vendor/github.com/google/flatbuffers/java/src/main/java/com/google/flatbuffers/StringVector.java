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
 * Helper type for accessing vector of String.
 */
public final class StringVector extends BaseVector {
  private Utf8 utf8 = Utf8.getDefault();

  /**
   * Assigns vector access object to vector data.
   *
   * @param _vector Start data of a vector.
   * @param _element_size Size of a vector element.
   * @param _bb Table's ByteBuffer.
   * @return Returns current vector access object assigned to vector data whose offset is stored at
   *         `vector`.
   */
  public StringVector __assign(int _vector, int _element_size, ByteBuffer _bb) {
    __reset(_vector, _element_size, _bb); return this;
  }

  /**
   * Reads the String at the given index.
   *
   * @param j The index from which the String value will be read.
   * @return the String at the given index.
   */
  public String get(int j) {
    return Table.__string(__element(j), bb, utf8);
  }
}
