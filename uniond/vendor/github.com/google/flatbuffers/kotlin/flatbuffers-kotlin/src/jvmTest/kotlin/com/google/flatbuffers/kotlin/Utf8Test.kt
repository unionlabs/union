/*
 * Copyright 2021 Google Inc. All rights reserved.
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
package com.google.flatbuffers.kotlin

import org.junit.Test
import kotlin.test.assertEquals

class Utf8Test {

  @Test
  fun testUtf8EncodingDecoding() {
    val classLoader = this.javaClass.classLoader
    val utf8Lines = String(classLoader.getResourceAsStream("utf8_sample.txt")!!.readBytes())
      .split("\n")
      .filter { it.trim().isNotEmpty() }

    val utf8Bytes = utf8Lines.map {
        s -> ByteArray(Utf8.encodedLength(s)).also {
          Utf8.encodeUtf8Array(s, it)
        }
    }
    utf8Bytes.indices.forEach {
      assertArrayEquals(utf8Lines[it].encodeToByteArray(), utf8Bytes[it])
      assertEquals(utf8Lines[it], Utf8.decodeUtf8Array(utf8Bytes[it]))
    }
  }
}
