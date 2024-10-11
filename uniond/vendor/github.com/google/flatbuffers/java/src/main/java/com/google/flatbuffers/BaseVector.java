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

import java.nio.ByteBuffer;

/// @cond FLATBUFFERS_INTERNAL

/**
 * All vector access objects derive from this class, and add their own accessors.
 */
public class BaseVector {
  /** Used to hold the vector data position. */
  private int vector;
  /** Used to hold the vector size. */
  private int length;
  /** Used to hold the vector element size in table. */
  private int element_size;
  /** The underlying ByteBuffer to hold the data of the vector. */
  protected ByteBuffer bb;

  /**
   * Get the start data of a vector.
   *
   * @return Returns the start of the vector data.
   */
  protected int __vector() {
    return vector;
  }

  /**
   * Gets the element position in vector's ByteBuffer.
   *
   * @param j An `int` index of element into a vector.
   * @return Returns the position of the vector element in a ByteBuffer.
   */
  protected int __element(int j) {
    return vector + j * element_size;
  }

  /**
   * Re-init the internal state with an external buffer {@code ByteBuffer}, an offset within and
   * element size.
   *
   * This method exists primarily to allow recycling vector instances without risking memory leaks
   * due to {@code ByteBuffer} references.
   */
  protected void __reset(int _vector, int _element_size, ByteBuffer _bb) { 
    bb = _bb;
    if (bb != null) {
      vector = _vector;
      length = bb.getInt(_vector - Constants.SIZEOF_INT);
      element_size = _element_size;
    } else {
      vector = 0;
      length = 0;
      element_size = 0;
    }
  }

  /**
   * Resets the internal state with a null {@code ByteBuffer} and a zero position.
   *
   * This method exists primarily to allow recycling vector instances without risking memory leaks
   * due to {@code ByteBuffer} references. The instance will be unusable until it is assigned
   * again to a {@code ByteBuffer}.
   */
  public void reset() {
    __reset(0, 0, null);
  }

  /**
   * Get the length of a vector.
   *
   * @return Returns the length of the vector.
   */
  public int length() {
    return length;
  }
}

/// @endcond
