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
@file:Suppress("NOTHING_TO_INLINE")

package com.google.flatbuffers.kotlin

public object Utf8 {
  /**
   * Returns the number of bytes in the UTF-8-encoded form of `sequence`. For a string,
   * this method is equivalent to `string.getBytes(UTF_8).length`, but is more efficient in
   * both time and space.
   *
   * @throws IllegalArgumentException if `sequence` contains ill-formed UTF-16 (unpaired
   * surrogates)
   */
  private fun computeEncodedLength(sequence: CharSequence): Int {
    // Warning to maintainers: this implementation is highly optimized.
    val utf16Length = sequence.length
    var utf8Length = utf16Length
    var i = 0

    // This loop optimizes for pure ASCII.
    while (i < utf16Length && sequence[i].code < 0x80) {
      i++
    }

    // This loop optimizes for chars less than 0x800.
    while (i < utf16Length) {
      val c = sequence[i]
      if (c.code < 0x800) {
        utf8Length += 0x7f - c.code ushr 31 // branch free!
      } else {
        utf8Length += encodedLengthGeneral(sequence, i)
        break
      }
      i++
    }
    if (utf8Length < utf16Length) {
      // Necessary and sufficient condition for overflow because of maximum 3x expansion
      error("UTF-8 length does not fit in int: ${(utf8Length + (1L shl 32))}")
    }
    return utf8Length
  }

  private fun encodedLengthGeneral(sequence: CharSequence, start: Int): Int {
    val utf16Length = sequence.length
    var utf8Length = 0
    var i = start
    while (i < utf16Length) {
      val c = sequence[i]
      if (c.code < 0x800) {
        utf8Length += 0x7f - c.code ushr 31 // branch free!
      } else {
        utf8Length += 2
        if (c.isSurrogate()) {
          // Check that we have a well-formed surrogate pair.
          val cp: Int = codePointAt(sequence, i)
          if (cp < MIN_SUPPLEMENTARY_CODE_POINT) {
            errorSurrogate(i, utf16Length)
          }
          i++
        }
      }
      i++
    }
    return utf8Length
  }

  /**
   * Returns the number of bytes in the UTF-8-encoded form of `sequence`. For a string,
   * this method is equivalent to `string.getBytes(UTF_8).length`, but is more efficient in
   * both time and space.
   *
   * @throws IllegalArgumentException if `sequence` contains ill-formed UTF-16 (unpaired
   * surrogates)
   */
  public fun encodedLength(sequence: CharSequence): Int = computeEncodedLength(sequence)

  /**
   * Returns whether this is a single-byte codepoint (i.e., ASCII) with the form '0XXXXXXX'.
   */
  public inline fun isOneByte(b: Byte): Boolean = b >= 0

  /**
   * Returns whether this is a two-byte codepoint with the form 110xxxxx  0xC0..0xDF.
   */
  public inline fun isTwoBytes(b: Byte): Boolean = b < 0xE0.toByte()

  /**
   * Returns whether this is a three-byte codepoint with the form 1110xxxx  0xE0..0xEF.
   */
  public inline fun isThreeBytes(b: Byte): Boolean = b < 0xF0.toByte()

  /**
   * Returns whether this is a four-byte codepoint with the form 11110xxx  0xF0..0xF4.
   */
  public inline fun isFourByte(b: Byte): Boolean = b < 0xF8.toByte()

  public fun handleOneByte(byte1: Byte, resultArr: CharArray, resultPos: Int) {
    resultArr[resultPos] = byte1.toInt().toChar()
  }

  public fun handleTwoBytes(
    byte1: Byte,
    byte2: Byte,
    resultArr: CharArray,
    resultPos: Int
  ) {
    // Simultaneously checks for illegal trailing-byte in leading position (<= '11000000') and
    // overlong 2-byte, '11000001'.
    if (byte1 < 0xC2.toByte()) {
      error("Invalid UTF-8: Illegal leading byte in 2 bytes utf")
    }
    if (isNotTrailingByte(byte2)) {
      error("Invalid UTF-8: Illegal trailing byte in 2 bytes utf")
    }
    resultArr[resultPos] = (byte1.toInt() and 0x1F shl 6 or trailingByteValue(byte2)).toChar()
  }

  public fun handleThreeBytes(
    byte1: Byte,
    byte2: Byte,
    byte3: Byte,
    resultArr: CharArray,
    resultPos: Int
  ) {
    if (isNotTrailingByte(byte2) || // overlong? 5 most significant bits must not all be zero
      byte1 == 0xE0.toByte() && byte2 < 0xA0.toByte() || // check for illegal surrogate codepoints
      byte1 == 0xED.toByte() && byte2 >= 0xA0.toByte() ||
      isNotTrailingByte(byte3)
    ) {
      error("Invalid UTF-8")
    }
    resultArr[resultPos] =
      (byte1.toInt() and 0x0F shl 12 or (trailingByteValue(byte2) shl 6) or trailingByteValue(byte3)).toChar()
  }

  public fun handleFourBytes(
    byte1: Byte,
    byte2: Byte,
    byte3: Byte,
    byte4: Byte,
    resultArr: CharArray,
    resultPos: Int
  ) {
    if (isNotTrailingByte(byte2) || // Check that 1 <= plane <= 16.  Tricky optimized form of:
      //   valid 4-byte leading byte?
      // if (byte1 > (byte) 0xF4 ||
      //   overlong? 4 most significant bits must not all be zero
      //     byte1 == (byte) 0xF0 && byte2 < (byte) 0x90 ||
      //   codepoint larger than the highest code point (U+10FFFF)?
      //     byte1 == (byte) 0xF4 && byte2 > (byte) 0x8F)
      (byte1.toInt() shl 28) + (byte2 - 0x90.toByte()) shr 30 != 0 || isNotTrailingByte(byte3) ||
      isNotTrailingByte(byte4)
    ) {
      error("Invalid UTF-8")
    }
    val codepoint: Int = (
      byte1.toInt() and 0x07 shl 18
        or (trailingByteValue(byte2) shl 12)
        or (trailingByteValue(byte3) shl 6)
        or trailingByteValue(byte4)
      )
    resultArr[resultPos] = highSurrogate(codepoint)
    resultArr[resultPos + 1] = lowSurrogate(codepoint)
  }

  /**
   * Returns whether the byte is not a valid continuation of the form '10XXXXXX'.
   */
  private fun isNotTrailingByte(b: Byte): Boolean = b > 0xBF.toByte()

  /**
   * Returns the actual value of the trailing byte (removes the prefix '10') for composition.
   */
  private fun trailingByteValue(b: Byte): Int = b.toInt() and 0x3F

  private fun highSurrogate(codePoint: Int): Char =
    (
      Char.MIN_HIGH_SURROGATE - (MIN_SUPPLEMENTARY_CODE_POINT ushr 10) +
        (codePoint ushr 10)
      )

  private fun lowSurrogate(codePoint: Int): Char = (Char.MIN_LOW_SURROGATE + (codePoint and 0x3ff))

  /**
   * Encode a [CharSequence] UTF8 codepoint into a byte array.
   * @param `in` CharSequence to be encoded
   * @param start start position of the first char in the codepoint
   * @param out byte array of 4 bytes to be filled
   * @return return the amount of bytes occupied by the codepoint
   */
  public fun encodeUtf8CodePoint(input: CharSequence, start: Int, out: ByteArray): Int {
    // utf8 codepoint needs at least 4 bytes
    val inLength = input.length
    if (start >= inLength) {
      return 0
    }
    val c = input[start]
    return if (c.code < 0x80) {
      // One byte (0xxx xxxx)
      out[0] = c.code.toByte()
      1
    } else if (c.code < 0x800) {
      // Two bytes (110x xxxx 10xx xxxx)
      out[0] = (0xC0 or (c.code ushr 6)).toByte()
      out[1] = (0x80 or (0x3F and c.code)).toByte()
      2
    } else if (c < Char.MIN_SURROGATE || Char.MAX_SURROGATE < c) {
      // Three bytes (1110 xxxx 10xx xxxx 10xx xxxx)
      // Maximum single-char code point is 0xFFFF, 16 bits.
      out[0] = (0xE0 or (c.code ushr 12)).toByte()
      out[1] = (0x80 or (0x3F and (c.code ushr 6))).toByte()
      out[2] = (0x80 or (0x3F and c.code)).toByte()
      3
    } else {
      // Four bytes (1111 xxxx 10xx xxxx 10xx xxxx 10xx xxxx)
      // Minimum code point represented by a surrogate pair is 0x10000, 17 bits, four UTF-8
      // bytes
      val low: Char = input[start + 1]
      if (start + 1 == inLength || !(c.isHighSurrogate() and low.isLowSurrogate())) {
        errorSurrogate(start, inLength)
      }
      val codePoint: Int = toCodePoint(c, low)
      out[0] = (0xF shl 4 or (codePoint ushr 18)).toByte()
      out[1] = (0x80 or (0x3F and (codePoint ushr 12))).toByte()
      out[2] = (0x80 or (0x3F and (codePoint ushr 6))).toByte()
      out[3] = (0x80 or (0x3F and codePoint)).toByte()
      4
    }
  }

  // Decodes a code point starting at index into out. Out parameter
  // should have at least 2 chars.
  public fun decodeUtf8CodePoint(bytes: ReadBuffer, index: Int, out: CharArray) {
    // Bitwise OR combines the sign bits so any negative value fails the check.
    val b1 = bytes[index]
    when {
      isOneByte(b1) -> handleOneByte(b1, out, 0)
      isTwoBytes(b1) -> handleTwoBytes(b1, bytes[index + 1], out, 0)
      isThreeBytes(b1) -> handleThreeBytes(b1, bytes[index + 1], bytes[index + 2], out, 0)
      else -> handleFourBytes(b1, bytes[index + 1], bytes[index + 2], bytes[index + 3], out, 0)
    }
  }

  public fun decodeUtf8Array(bytes: ByteArray, index: Int = 0, size: Int = bytes.size): String {
    // Bitwise OR combines the sign bits so any negative value fails the check.
    if (index or size or bytes.size - index - size < 0) {
      error("buffer length=${bytes.size}, index=$index, size=$size")
    }
    var offset = index
    val limit = offset + size

    // The longest possible resulting String is the same as the number of input bytes, when it is
    // all ASCII. For other cases, this over-allocates and we will truncate in the end.
    val resultArr = CharArray(size)
    var resultPos = 0

    // Optimize for 100% ASCII (Hotspot loves small simple top-level loops like this).
    // This simple loop stops when we encounter a byte >= 0x80 (i.e. non-ASCII).
    while (offset < limit) {
      val b = bytes[offset]
      if (!isOneByte(b)) {
        break
      }
      offset++
      handleOneByte(b, resultArr, resultPos++)
    }
    while (offset < limit) {
      val byte1 = bytes[offset++]
      if (isOneByte(byte1)) {
        handleOneByte(byte1, resultArr, resultPos++)
        // It's common for there to be multiple ASCII characters in a run mixed in, so add an
        // extra optimized loop to take care of these runs.
        while (offset < limit) {
          val b = bytes[offset]
          if (!isOneByte(b)) {
            break
          }
          offset++
          handleOneByte(b, resultArr, resultPos++)
        }
      } else if (isTwoBytes(byte1)) {
        if (offset >= limit) {
          error("Invalid UTF-8")
        }
        handleTwoBytes(
          byte1, /* byte2 */
          bytes[offset++], resultArr, resultPos++
        )
      } else if (isThreeBytes(byte1)) {
        if (offset >= limit - 1) {
          error("Invalid UTF-8")
        }
        handleThreeBytes(
          byte1, /* byte2 */
          bytes[offset++], /* byte3 */
          bytes[offset++],
          resultArr,
          resultPos++
        )
      } else {
        if (offset >= limit - 2) {
          error("Invalid UTF-8")
        }
        handleFourBytes(
          byte1, /* byte2 */
          bytes[offset++], /* byte3 */
          bytes[offset++], /* byte4 */
          bytes[offset++],
          resultArr,
          resultPos++
        )
        // 4-byte case requires two chars.
        resultPos++
      }
    }
    return resultArr.concatToString(0, resultPos)
  }

  public fun encodeUtf8Array(input: CharSequence,
                             out: ByteArray,
                             offset: Int = 0,
                             length: Int = out.size - offset): Int {
    val utf16Length = input.length
    var j = offset
    var i = 0
    val limit = offset + length
    // Designed to take advantage of
    // https://wikis.oracle.com/display/HotSpotInternals/RangeCheckElimination

    if (utf16Length == 0)
      return 0
    var cc: Char = input[i]
    while (i < utf16Length && i + j < limit && input[i].also { cc = it }.code < 0x80) {
      out[j + i] = cc.code.toByte()
      i++
    }
    if (i == utf16Length) {
      return j + utf16Length
    }
    j += i
    var c: Char
    while (i < utf16Length) {
      c = input[i]
      if (c.code < 0x80 && j < limit) {
        out[j++] = c.code.toByte()
      } else if (c.code < 0x800 && j <= limit - 2) { // 11 bits, two UTF-8 bytes
        out[j++] = (0xF shl 6 or (c.code ushr 6)).toByte()
        out[j++] = (0x80 or (0x3F and c.code)).toByte()
      } else if ((c < Char.MIN_SURROGATE || Char.MAX_SURROGATE < c) && j <= limit - 3) {
        // Maximum single-char code point is 0xFFFF, 16 bits, three UTF-8 bytes
        out[j++] = (0xF shl 5 or (c.code ushr 12)).toByte()
        out[j++] = (0x80 or (0x3F and (c.code ushr 6))).toByte()
        out[j++] = (0x80 or (0x3F and c.code)).toByte()
      } else if (j <= limit - 4) {
        // Minimum code point represented by a surrogate pair is 0x10000, 17 bits,
        // four UTF-8 bytes
        var low: Char = Char.MIN_VALUE
        if (i + 1 == input.length ||
          !isSurrogatePair(c, input[++i].also { low = it })
        ) {
          errorSurrogate(i - 1, utf16Length)
        }
        val codePoint: Int = toCodePoint(c, low)
        out[j++] = (0xF shl 4 or (codePoint ushr 18)).toByte()
        out[j++] = (0x80 or (0x3F and (codePoint ushr 12))).toByte()
        out[j++] = (0x80 or (0x3F and (codePoint ushr 6))).toByte()
        out[j++] = (0x80 or (0x3F and codePoint)).toByte()
      } else {
        // If we are surrogates and we're not a surrogate pair, always throw an
        // UnpairedSurrogateException instead of an ArrayOutOfBoundsException.
        if (Char.MIN_SURROGATE <= c && c <= Char.MAX_SURROGATE &&
          (i + 1 == input.length || !isSurrogatePair(c, input[i + 1]))
        ) {
          errorSurrogate(i, utf16Length)
        }
        error("Failed writing character ${c.code.toShort().toString(radix = 16)} at index $j")
      }
      i++
    }
    return j
  }

  public fun codePointAt(seq: CharSequence, position: Int): Int {
    var index = position
    val c1 = seq[index]
    if (c1.isHighSurrogate() && ++index < seq.length) {
      val c2 = seq[index]
      if (c2.isLowSurrogate()) {
        return toCodePoint(c1, c2)
      }
    }
    return c1.code
  }

  private fun isSurrogatePair(high: Char, low: Char) = high.isHighSurrogate() and low.isLowSurrogate()

  private fun toCodePoint(high: Char, low: Char): Int = (high.code shl 10) + low.code +
    (MIN_SUPPLEMENTARY_CODE_POINT - (Char.MIN_HIGH_SURROGATE.code shl 10) - Char.MIN_LOW_SURROGATE.code)

  private fun errorSurrogate(i: Int, utf16Length: Int): Unit =
    error("Unpaired surrogate at index $i of $utf16Length length")

  // The minimum value of Unicode supplementary code point, constant `U+10000`.
  private const val MIN_SUPPLEMENTARY_CODE_POINT = 0x010000
}
