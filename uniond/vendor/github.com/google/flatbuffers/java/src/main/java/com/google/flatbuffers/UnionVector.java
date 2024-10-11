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
 * Helper type for accessing vector of unions.
 */
public final class UnionVector extends BaseVector {
  /**
   * Assigns vector access object to vector data.
   *
   * @param _vector Start data of a vector.
   * @param _element_size Size of a vector element.
   * @param _bb Table's ByteBuffer.
   * @return Returns current vector access object assigned to vector data whose offset is stored at
   *         `vector`.
   */
  public UnionVector __assign(int _vector, int _element_size, ByteBuffer _bb) {
    __reset(_vector, _element_size, _bb); return this;
  }


  /**
   * Initialize any Table-derived type to point to the union at the given `index`.
   *
   * @param obj A `Table`-derived type that should point to the union at `index`.
   * @param j An `int` index into the union vector.
   * @return Returns the Table that points to the union at `index`.
   */
  public Table get(Table obj, int j) {
    return Table.__union(obj, __element(j), bb);
  }
}
