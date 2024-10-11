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

import com.google.flatbuffers.kotlin.FlexBuffersBuilder.Companion.SHARE_KEYS_AND_STRINGS
import kotlin.experimental.and
import kotlin.jvm.JvmInline
import kotlin.math.pow

/**
 * Returns a minified version of this FlexBuffer as a JSON.
 */
public fun Reference.toJson(): String = ArrayReadWriteBuffer(1024).let {
  toJson(it)
  val data = it.data() // it.getString(0, it.writePosition)
  return data.decodeToString(0, it.writePosition)
}

/**
 * Returns a minified version of this FlexBuffer as a JSON.
 * @param out [ReadWriteBuffer] the JSON will be written.
 */
public fun Reference.toJson(out: ReadWriteBuffer) {
  when (type) {
    T_STRING -> {
      val start = buffer.indirect(end, parentWidth)
      val size = buffer.readULong(start - byteWidth, byteWidth).toInt()
      out.jsonEscape(buffer, start, size)
    }
    T_KEY -> {
      val start = buffer.indirect(end, parentWidth)
      val end = buffer.findFirst(0.toByte(), start)
      out.jsonEscape(buffer, start, end - start)
    }
    T_BLOB -> {
      val blob = toBlob()
      out.jsonEscape(out, blob.end, blob.size)
    }
    T_INT -> out.put(toLong().toString())
    T_UINT -> out.put(toULong().toString())
    T_FLOAT -> out.put(toDouble().toString())
    T_NULL -> out.put("null")
    T_BOOL -> out.put(toBoolean().toString())
    T_MAP -> toMap().toJson(out)
    T_VECTOR, T_VECTOR_BOOL, T_VECTOR_FLOAT, T_VECTOR_INT,
    T_VECTOR_UINT, T_VECTOR_KEY, T_VECTOR_STRING_DEPRECATED -> toVector().toJson(out)
    else -> error("Unable to convert type ${type.typeToString()} to JSON")
  }
}

/**
 * Returns a minified version of this FlexBuffer as a JSON.
 */
public fun Map.toJson(): String = ArrayReadWriteBuffer(1024).let { toJson(it); it.toString() }

/**
 * Returns a minified version of this FlexBuffer as a JSON.
 * @param out [ReadWriteBuffer] the JSON will be written.
 */
public fun Map.toJson(out: ReadWriteBuffer) {
  out.put('{'.code.toByte())
  // key values pairs
  for (i in 0 until size) {
    val key = keyAt(i)
    out.jsonEscape(buffer, key.start, key.sizeInBytes)
    out.put(':'.code.toByte())
    get(i).toJson(out)
    if (i != size - 1) {
      out.put(','.code.toByte())
    }
  }
  // close bracket
  out.put('}'.code.toByte())
}

/**
 * Returns a minified version of this FlexBuffer as a JSON.
 */
public fun Vector.toJson(): String = ArrayReadWriteBuffer(1024).let { toJson(it); it.toString() }

/**
 * Returns a minified version of this FlexBuffer as a JSON.
 * @param out that the JSON is being concatenated.
 */
public fun Vector.toJson(out: ReadWriteBuffer) {
  out.put('['.code.toByte())
  for (i in indices) {
    get(i).toJson(out)
    if (i != size - 1) {
      out.put(','.code.toByte())
    }
  }
  out.put(']'.code.toByte())
}

/**
 * JSONParser class is used to parse a JSON as FlexBuffers. Calling [JSONParser.parse] fiils [output]
 * and returns a [Reference] ready to be used.
 */
@ExperimentalUnsignedTypes
public class JSONParser(public var output: FlexBuffersBuilder = FlexBuffersBuilder(1024, SHARE_KEYS_AND_STRINGS)) {
  private var readPos = 0
  private var scopes = ScopeStack()

  /**
   * Parse a json as [String] and returns a [Reference] to a FlexBuffer.
   */
  public fun parse(data: String): Reference = parse(ArrayReadBuffer(data.encodeToByteArray()))

  /**
   * Parse a json as [ByteArray] and returns a [Reference] to a FlexBuffer.
   */
  public fun parse(data: ByteArray): Reference = parse(ArrayReadBuffer(data))

  /**
   * Parse a json as [ReadBuffer] and returns a [Reference] to a FlexBuffer.
   */
  public fun parse(data: ReadBuffer): Reference {
    reset()
    parseValue(data, nextToken(data), null)
    if (readPos < data.limit) {
      val tok = skipWhitespace(data)
      if (tok != CHAR_EOF) {
        makeError(data, "Extraneous charaters after parse has finished", tok)
      }
    }
    output.finish()
    return getRoot(output.buffer)
  }

  private fun parseValue(data: ReadBuffer, token: Token, key: String? = null): FlexBufferType {
    return when (token) {
      TOK_BEGIN_OBJECT -> parseObject(data, key)
      TOK_BEGIN_ARRAY -> parseArray(data, key)
      TOK_TRUE -> T_BOOL.also { output[key] = true }
      TOK_FALSE -> T_BOOL.also { output[key] = false }
      TOK_NULL -> T_NULL.also { output.putNull(key) }
      TOK_BEGIN_QUOTE -> parseString(data, key)
      TOK_NUMBER -> parseNumber(data, data.data(), key)
      else -> makeError(data, "Unexpected Character while parsing", 'x'.code.toByte())
    }
  }

  private fun parseObject(data: ReadBuffer, key: String? = null): FlexBufferType {
    this.scopes.push(SCOPE_OBJ_EMPTY)

    val fPos = output.startMap()
    val limit = data.limit
    while (readPos <= limit) {
      when (val tok = nextToken(data)) {
        TOK_END_OBJECT -> {
          this.scopes.pop()
          output.endMap(fPos, key); return T_MAP
        }
        TOK_BEGIN_QUOTE -> {
          val childKey = readString(data)
          parseValue(data, nextToken(data), childKey)
        }
        else -> makeError(data, "Expecting start of object key", tok)
      }
    }
    makeError(data, "Unable to parse the object", "x".toByte())
  }

  private fun parseArray(data: ReadBuffer, key: String? = null): FlexBufferType {
    this.scopes.push(SCOPE_ARRAY_EMPTY)
    val fPos = output.startVector()
    var elementType = T_INVALID
    var multiType = false
    val limit = data.limit

    while (readPos <= limit) {
      when (val tok = nextToken(data)) {
        TOK_END_ARRAY -> {
          this.scopes.pop()
          return if (!multiType && elementType.isScalar()) {
            output.endTypedVector(fPos, key)
            elementType.toElementTypedVector()
          } else {
            output.endVector(key, fPos)
            T_VECTOR
          }
        }

        else -> {
          val newType = parseValue(data, tok, null)

          if (elementType == T_INVALID) {
            elementType = newType
          } else if (newType != elementType) {
            multiType = true
          }
        }
      }
    }
    makeError(data, "Unable to parse the array")
  }

  private fun parseNumber(data: ReadBuffer, array: ByteArray, key: String?): FlexBufferType {
    val ary = array
    var cursor = readPos
    var c = data[readPos++]
    var useDouble = false
    val limit = ary.size
    var sign = 1
    var double: Double
    var long = 0L
    var digits = 0

    if (c == CHAR_MINUS) {
      cursor++
      checkEOF(data, cursor)
      c = ary[cursor]
      sign = -1
    }

    // peek first byte
    when (c) {
      CHAR_0 -> {
        cursor++
        if (cursor != limit) {
          c = ary[cursor]
        }
      }
      !in CHAR_0..CHAR_9 -> makeError(data, "Invalid Number", c)
      else -> {
        do {
          val digit = c - CHAR_0
          // double = 10.0 * double + digit
          long = 10 * long + digit
          digits++
          cursor++
          if (cursor == limit) break
          c = ary[cursor]
        } while (c in CHAR_0..CHAR_9)
      }
    }

    var exponent = 0
    // If we find '.' we need to convert to double
    if (c == CHAR_DOT) {
      useDouble = true
      checkEOF(data, cursor)
      c = ary[++cursor]
      if (c < CHAR_0 || c > CHAR_9) {
        makeError(data, "Invalid Number", c)
      }
      do {
        // double = double * 10 + (tok - CHAR_0)
        long = 10 * long + (c - CHAR_0)
        digits++
        --exponent
        cursor++
        if (cursor == limit) break
        c = ary[cursor]
      } while (c in CHAR_0..CHAR_9)
    }

    // If we find 'e' we need to convert to double
    if (c == CHAR_e || c == CHAR_E) {
      useDouble = true
      ++cursor
      checkEOF(data, cursor)
      c = ary[cursor]
      var negativeExponent = false
      if (c == CHAR_MINUS) {
        ++cursor
        checkEOF(data, cursor)
        negativeExponent = true
        c = ary[cursor]
      } else if (c == CHAR_PLUS) {
        ++cursor
        checkEOF(data, cursor)
        c = ary[cursor]
      }
      if (c < CHAR_0 || c > CHAR_9) {
        makeError(data, "Missing exponent", c)
      }
      var exp = 0
      do {
        val digit = c - CHAR_0
        exp = 10 * exp + digit
        ++cursor
        if (cursor == limit) break
        c = ary[cursor]
      } while (c in CHAR_0..CHAR_9)

      exponent += if (negativeExponent) -exp else exp
    }

    if (digits > 17 || exponent < -19 || exponent > 19) {
      // if the float number is not simple enough
      // we use language's Double parsing, which is slower but
      // produce more expected results for extreme numbers.
      val firstPos = readPos - 1
      val str = data.getString(firstPos, cursor - firstPos)
      if (useDouble) {
        double = str.toDouble()
        output[key] = double
      } else {
        long = str.toLong()
        output[key] = long
      }
    } else {
      // this happens on single numbers outside any object
      // or array
      if (useDouble || exponent != 0) {
        double = if (long == 0L) 0.0 else long.toDouble() * 10.0.pow(exponent)
        double *= sign
        output[key] = double
      } else {
        long *= sign
        output[key] = long
      }
    }
    readPos = cursor
    return if (useDouble) T_FLOAT else T_INT
  }

  private fun parseString(data: ReadBuffer, key: String?): FlexBufferType {
    output[key] = readString(data)
    return T_STRING
  }

  private fun readString(data: ReadBuffer): String {
    val limit = data.limit
    if (data is ArrayReadBuffer) {
      val ary = data.data()
      // enables range check elimination
      return readString(data, limit) { ary[it] }
    }
    return readString(data, limit) { data[it] }
  }

  private inline fun readString(data: ReadBuffer, limit: Int, crossinline fetch: (Int) -> Byte): String {
    var cursorPos = readPos
    var foundEscape = false
    var currentChar: Byte = 0
    // we loop over every 4 bytes until find any non-plain char
    while (limit - cursorPos >= 4) {
      currentChar = fetch(cursorPos)
      if (!isPlainStringChar(currentChar)) {
        foundEscape = true
        break
      }
      currentChar = fetch(cursorPos + 1)
      if (!isPlainStringChar(currentChar)) {
        cursorPos += 1
        foundEscape = true
        break
      }
      currentChar = fetch(cursorPos + 2)
      if (!isPlainStringChar(currentChar)) {
        cursorPos += 2
        foundEscape = true
        break
      }
      currentChar = fetch(cursorPos + 3)
      if (!isPlainStringChar(currentChar)) {
        cursorPos += 3
        foundEscape = true
        break
      }
      cursorPos += 4
    }
    if (!foundEscape) {
      // if non-plain string char is not found we loop over
      // the remaining bytes
      while (true) {
        if (cursorPos >= limit) {
          error("Unexpected end of string")
        }
        currentChar = fetch(cursorPos)
        if (!isPlainStringChar(currentChar)) {
          break
        }
        ++cursorPos
      }
    }
    if (currentChar == CHAR_DOUBLE_QUOTE) {
      val str = data.getString(readPos, cursorPos - readPos)
      readPos = cursorPos + 1
      return str
    }
    if (currentChar in 0..0x1f) {
      error("Illegal Codepoint")
    } else {
      // backslash or >0x7f
      return readStringSlow(data, currentChar, cursorPos)
    }
  }

  private fun readStringSlow(data: ReadBuffer, first: Byte, lastPos: Int): String {
    var cursorPos = lastPos

    var endOfString = lastPos
    while (true) {
      val pos = data.findFirst(CHAR_DOUBLE_QUOTE, endOfString)
      when {
        pos == -1 -> makeError(data, "Unexpected EOF, missing end of string '\"'", first)
        data[pos - 1] == CHAR_BACKSLASH && data[pos - 2] != CHAR_BACKSLASH -> {
          // here we are checking for double quotes preceded by backslash. eg \"
          // we have to look past pos -2 to make sure that the backlash is not
          // part of a previous escape, eg "\\"
          endOfString = pos + 1
        }
        else -> {
          endOfString = pos; break
        }
      }
    }
    // copy everything before the escape
    val builder = StringBuilder(data.getString(readPos, lastPos - readPos))
    while (true) {
      when (val pos = data.findFirst(CHAR_BACKSLASH, cursorPos, endOfString)) {
        -1 -> {
          val doubleQuotePos = data.findFirst(CHAR_DOUBLE_QUOTE, cursorPos)
          if (doubleQuotePos == -1) makeError(data, "Reached EOF before enclosing string", first)
          val rest = data.getString(cursorPos, doubleQuotePos - cursorPos)
          builder.append(rest)
          readPos = doubleQuotePos + 1
          return builder.toString()
        }

        else -> {
          // we write everything up to \
          builder.append(data.getString(cursorPos, pos - cursorPos))
          val c = data[pos + 1]
          builder.append(readEscapedChar(data, c, pos))
          cursorPos = pos + if (c == CHAR_u) 6 else 2
        }
      }
    }
  }

  private inline fun isPlainStringChar(c: Byte): Boolean {
    val flags = parseFlags
    // return c in 0x20..0x7f && c != 0x22.toByte() && c != 0x5c.toByte()
    return (flags[c.toInt() and 0xFF] and 1) != 0.toByte()
  }

  private inline fun isWhitespace(c: Byte): Boolean {
    val flags = parseFlags
    // return c == '\r'.toByte() || c == '\n'.toByte() || c == '\t'.toByte() || c == ' '.toByte()
    return (flags[c.toInt() and 0xFF] and 2) != 0.toByte()
  }

  private fun reset() {
    readPos = 0
    output.clear()
    scopes.reset()
  }

  private fun nextToken(data: ReadBuffer): Token {
    val scope = this.scopes.last

    when (scope) {
      SCOPE_ARRAY_EMPTY -> this.scopes.last = SCOPE_ARRAY_FILLED
      SCOPE_ARRAY_FILLED -> {
        when (val c = skipWhitespace(data)) {
          CHAR_CLOSE_ARRAY -> return TOK_END_ARRAY
          CHAR_COMMA -> Unit
          else -> makeError(data, "Unfinished Array", c)
        }
      }
      SCOPE_OBJ_EMPTY, SCOPE_OBJ_FILLED -> {
        this.scopes.last = SCOPE_OBJ_KEY
        // Look for a comma before the next element.
        if (scope == SCOPE_OBJ_FILLED) {
          when (val c = skipWhitespace(data)) {
            CHAR_CLOSE_OBJECT -> return TOK_END_OBJECT
            CHAR_COMMA -> Unit
            else -> makeError(data, "Unfinished Object", c)
          }
        }
        return when (val c = skipWhitespace(data)) {
          CHAR_DOUBLE_QUOTE -> TOK_BEGIN_QUOTE
          CHAR_CLOSE_OBJECT -> if (scope != SCOPE_OBJ_FILLED) {
            TOK_END_OBJECT
          } else {
            makeError(data, "Expected Key", c)
          }
          else -> {
            makeError(data, "Expected Key/Value", c)
          }
        }
      }
      SCOPE_OBJ_KEY -> {
        this.scopes.last = SCOPE_OBJ_FILLED
        when (val c = skipWhitespace(data)) {
          CHAR_COLON -> Unit
          else -> makeError(data, "Expect ${CHAR_COLON.print()}", c)
        }
      }
      SCOPE_DOC_EMPTY -> this.scopes.last = SCOPE_DOC_FILLED
      SCOPE_DOC_FILLED -> {
        val c = skipWhitespace(data)
        if (c != CHAR_EOF)
          makeError(data, "Root object already finished", c)
        return TOK_EOF
      }
    }

    val c = skipWhitespace(data)
    when (c) {
      CHAR_CLOSE_ARRAY -> if (scope == SCOPE_ARRAY_EMPTY) return TOK_END_ARRAY
      CHAR_COLON -> makeError(data, "Unexpected character", c)
      CHAR_DOUBLE_QUOTE -> return TOK_BEGIN_QUOTE
      CHAR_OPEN_ARRAY -> return TOK_BEGIN_ARRAY
      CHAR_OPEN_OBJECT -> return TOK_BEGIN_OBJECT
      CHAR_t -> {
        checkEOF(data, readPos + 2)
        // 0x65757274 is equivalent to ['t', 'r', 'u', 'e' ] as a 4 byte Int
        if (data.getInt(readPos - 1) != 0x65757274) {
          makeError(data, "Expecting keyword \"true\"", c)
        }
        readPos += 3
        return TOK_TRUE
      }
      CHAR_n -> {
        checkEOF(data, readPos + 2)
        // 0x6c6c756e  is equivalent to ['n', 'u', 'l', 'l' ] as a 4 byte Int
        if (data.getInt(readPos - 1) != 0x6c6c756e) {
          makeError(data, "Expecting keyword \"null\"", c)
        }
        readPos += 3
        return TOK_NULL
      }
      CHAR_f -> {
        checkEOF(data, readPos + 3)
        // 0x65736c61 is equivalent to ['a', 'l', 's', 'e' ] as a 4 byte Int
        if (data.getInt(readPos) != 0x65736c61) {
          makeError(data, "Expecting keyword \"false\"", c)
        }
        readPos += 4
        return TOK_FALSE
      }
      CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4, CHAR_5,
      CHAR_6, CHAR_7, CHAR_8, CHAR_9, CHAR_MINUS -> return TOK_NUMBER.also {
        readPos-- // rewind one position so we don't lose first digit
      }
    }
    makeError(data, "Expecting element", c)
  }

  // keeps increasing [readPos] until finds a non-whitespace byte
  private inline fun skipWhitespace(data: ReadBuffer): Byte {
    val limit = data.limit
    if (data is ArrayReadBuffer) {
      // enables range check elimination
      val ary = data.data()
      return skipWhitespace(limit) { ary[it] }
    }
    return skipWhitespace(limit) { data[it] }
  }

  private inline fun skipWhitespace(limit: Int, crossinline fetch: (Int) -> Byte): Byte {
    var pos = readPos
    while (pos < limit) {
      val d = fetch(pos++)
      if (!isWhitespace(d)) {
        readPos = pos
        return d
      }
    }
    readPos = limit
    return CHAR_EOF
  }

  // byte1 is expected to be first char before `\`
  private fun readEscapedChar(data: ReadBuffer, byte1: Byte, cursorPos: Int): Char {
    return when (byte1) {
      CHAR_u -> {
        checkEOF(data, cursorPos + 1 + 4)
        var result: Char = 0.toChar()
        var i = cursorPos + 2 // cursorPos is on '\\', cursorPos + 1 is 'u'
        val end = i + 4
        while (i < end) {
          val part: Byte = data[i]
          result = (result.code shl 4).toChar()
          result += when (part) {
            in CHAR_0..CHAR_9 -> part - CHAR_0
            in CHAR_a..CHAR_f -> part - CHAR_a + 10
            in CHAR_A..CHAR_F -> part - CHAR_A + 10
            else -> makeError(data, "Invalid utf8 escaped character", -1)
          }
          i++
        }
        result
      }
      CHAR_b -> '\b'
      CHAR_t -> '\t'
      CHAR_r -> '\r'
      CHAR_n -> '\n'
      CHAR_f -> 12.toChar() // '\f'
      CHAR_DOUBLE_QUOTE, CHAR_BACKSLASH, CHAR_FORWARDSLASH -> byte1.toInt().toChar()
      else -> makeError(data, "Invalid escape sequence.", byte1)
    }
  }

  private fun Byte.print(): String = when (this) {
    in 0x21..0x7E -> "'${this.toInt().toChar()}'" // visible ascii chars
    CHAR_EOF -> "EOF"
    else -> "'0x${this.toString(16)}'"
  }

  private inline fun makeError(data: ReadBuffer, msg: String, tok: Byte? = null): Nothing {
    val (line, column) = calculateErrorPosition(data, readPos)
    if (tok != null) {
      error("Error At ($line, $column): $msg, got ${tok.print()}")
    } else {
      error("Error At ($line, $column): $msg")
    }
  }

  private inline fun makeError(data: ReadBuffer, msg: String, tok: Token): Nothing {
    val (line, column) = calculateErrorPosition(data, readPos)
    error("Error At ($line, $column): $msg, got ${tok.print()}")
  }

  private inline fun checkEOF(data: ReadBuffer, pos: Int) {
    if (pos >= data.limit)
      makeError(data, "Unexpected end of file", -1)
  }

  private fun calculateErrorPosition(data: ReadBuffer, endPos: Int): Pair<Int, Int> {
    var line = 1
    var column = 1
    var current = 0
    while (current < endPos - 1) {
      if (data[current++] == CHAR_NEWLINE) {
        ++line
        column = 1
      } else {
        ++column
      }
    }
    return Pair(line, column)
  }
}

internal inline fun Int.toPaddedHex(): String = "\\u${this.toString(16).padStart(4, '0')}"

private inline fun ReadWriteBuffer.jsonEscape(data: ReadBuffer, start: Int, size: Int) {
  val replacements = JSON_ESCAPE_CHARS
  put(CHAR_DOUBLE_QUOTE)
  var last = start
  val length: Int = size
  val ary = data.data()
  for (i in start until start + length) {
    val c = ary[i].toUByte()
    var replacement: ByteArray?
    if (c.toInt() < 128) {
      replacement = replacements[c.toInt()]
      if (replacement == null) {
        continue
      }
    } else {
      continue
    }
    if (last < i) {
      put(ary, last, i - last)
    }
    put(replacement, 0, replacement.size)
    last = i + 1
  }
  if (last < (last + length)) {
    put(ary, last, (start + length) - last)
  }
  put(CHAR_DOUBLE_QUOTE)
}

// Following escape strategy defined in RFC7159.
private val JSON_ESCAPE_CHARS: Array<ByteArray?> = arrayOfNulls<ByteArray>(128).apply {
  this['\n'.code] = "\\n".encodeToByteArray()
  this['\t'.code] = "\\t".encodeToByteArray()
  this['\r'.code] = "\\r".encodeToByteArray()
  this['\b'.code] = "\\b".encodeToByteArray()
  this[0x0c] = "\\f".encodeToByteArray()
  this['"'.code] = "\\\"".encodeToByteArray()
  this['\\'.code] = "\\\\".encodeToByteArray()
  for (i in 0..0x1f) {
    this[i] = "\\u${i.toPaddedHex()}".encodeToByteArray()
  }
}

// Scope is used to the define current space that the scanner is operating.
@JvmInline
private value class Scope(val id: Int)
private val SCOPE_DOC_EMPTY = Scope(0)
private val SCOPE_DOC_FILLED = Scope(1)
private val SCOPE_OBJ_EMPTY = Scope(2)
private val SCOPE_OBJ_KEY = Scope(3)
private val SCOPE_OBJ_FILLED = Scope(4)
private val SCOPE_ARRAY_EMPTY = Scope(5)
private val SCOPE_ARRAY_FILLED = Scope(6)

// Keeps the stack state of the scopes being scanned. Currently defined to have a
// max stack size of 22, as per tests cases defined in http://json.org/JSON_checker/
private class ScopeStack(
  private val ary: IntArray = IntArray(22) { SCOPE_DOC_EMPTY.id },
  var lastPos: Int = 0
) {
  var last: Scope
    get() = Scope(ary[lastPos])
    set(x) {
      ary[lastPos] = x.id
    }

  fun reset() {
    lastPos = 0
    ary[0] = SCOPE_DOC_EMPTY.id
  }

  fun pop(): Scope {
    // println("Popping: ${last.print()}")
    return Scope(ary[lastPos--])
  }

  fun push(scope: Scope): Scope {
    if (lastPos == ary.size - 1)
      error("Too much nesting reached. Max nesting is ${ary.size} levels")
    // println("PUSHING : ${scope.print()}")
    ary[++lastPos] = scope.id
    return scope
  }
}

@JvmInline
private value class Token(val id: Int) {
  fun print(): String = when (this) {
    TOK_EOF -> "TOK_EOF"
    TOK_NONE -> "TOK_NONE"
    TOK_BEGIN_OBJECT -> "TOK_BEGIN_OBJECT"
    TOK_END_OBJECT -> "TOK_END_OBJECT"
    TOK_BEGIN_ARRAY -> "TOK_BEGIN_ARRAY"
    TOK_END_ARRAY -> "TOK_END_ARRAY"
    TOK_NUMBER -> "TOK_NUMBER"
    TOK_TRUE -> "TOK_TRUE"
    TOK_FALSE -> "TOK_FALSE"
    TOK_NULL -> "TOK_NULL"
    TOK_BEGIN_QUOTE -> "TOK_BEGIN_QUOTE"
    else -> this.toString()
  }
}

private val TOK_EOF = Token(-1)
private val TOK_NONE = Token(0)
private val TOK_BEGIN_OBJECT = Token(1)
private val TOK_END_OBJECT = Token(2)
private val TOK_BEGIN_ARRAY = Token(3)
private val TOK_END_ARRAY = Token(4)
private val TOK_NUMBER = Token(5)
private val TOK_TRUE = Token(6)
private val TOK_FALSE = Token(7)
private val TOK_NULL = Token(8)
private val TOK_BEGIN_QUOTE = Token(9)

private const val CHAR_NEWLINE = '\n'.code.toByte()
private const val CHAR_OPEN_OBJECT = '{'.code.toByte()
private const val CHAR_COLON = ':'.code.toByte()
private const val CHAR_CLOSE_OBJECT = '}'.code.toByte()
private const val CHAR_OPEN_ARRAY = '['.code.toByte()
private const val CHAR_CLOSE_ARRAY = ']'.code.toByte()
private const val CHAR_DOUBLE_QUOTE = '"'.code.toByte()
private const val CHAR_BACKSLASH = '\\'.code.toByte()
private const val CHAR_FORWARDSLASH = '/'.code.toByte()
private const val CHAR_f = 'f'.code.toByte()
private const val CHAR_a = 'a'.code.toByte()
private const val CHAR_r = 'r'.code.toByte()
private const val CHAR_t = 't'.code.toByte()
private const val CHAR_n = 'n'.code.toByte()
private const val CHAR_b = 'b'.code.toByte()
private const val CHAR_e = 'e'.code.toByte()
private const val CHAR_E = 'E'.code.toByte()
private const val CHAR_u = 'u'.code.toByte()
private const val CHAR_A = 'A'.code.toByte()
private const val CHAR_F = 'F'.code.toByte()
private const val CHAR_EOF = (-1).toByte()
private const val CHAR_COMMA = ','.code.toByte()
private const val CHAR_0 = '0'.code.toByte()
private const val CHAR_1 = '1'.code.toByte()
private const val CHAR_2 = '2'.code.toByte()
private const val CHAR_3 = '3'.code.toByte()
private const val CHAR_4 = '4'.code.toByte()
private const val CHAR_5 = '5'.code.toByte()
private const val CHAR_6 = '6'.code.toByte()
private const val CHAR_7 = '7'.code.toByte()
private const val CHAR_8 = '8'.code.toByte()
private const val CHAR_9 = '9'.code.toByte()
private const val CHAR_MINUS = '-'.code.toByte()
private const val CHAR_PLUS = '+'.code.toByte()
private const val CHAR_DOT = '.'.code.toByte()

// This template utilizes the One Definition Rule to create global arrays in a
// header. As seen in:
// https://github.com/chadaustin/sajson/blob/master/include/sajson.h
// bit 0 (1) - set if: plain ASCII string character
// bit 1 (2) - set if: whitespace
// bit 4 (0x10) - set if: 0-9 e E .
private val parseFlags = byteArrayOf(
// 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 2, 0, 0, // 0
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 1
  3, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x11, 1, // 2
  0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 1, 1, 1, 1, 1, 1, // 3
  1, 1, 1, 1, 1, 0x11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, // 5
  1, 1, 1, 1, 1, 0x11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7

  // 128-255
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
)
