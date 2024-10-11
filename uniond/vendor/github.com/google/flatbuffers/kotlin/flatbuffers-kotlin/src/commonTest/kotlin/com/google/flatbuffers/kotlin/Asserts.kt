package com.google.flatbuffers.kotlin

import kotlin.test.assertTrue

fun <T> assertArrayEquals(expected: Array<out T>, actual: Array<out T>) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun assertArrayEquals(expected: IntArray, actual: IntArray) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun assertArrayEquals(expected: ShortArray, actual: ShortArray) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun assertArrayEquals(expected: LongArray, actual: LongArray) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun assertArrayEquals(expected: ByteArray, actual: ByteArray) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun assertArrayEquals(expected: DoubleArray, actual: DoubleArray) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun assertArrayEquals(expected: FloatArray, actual: FloatArray) =
  assertTrue(expected contentEquals actual, arrayFailMessage(expected, actual))

fun <T> arrayFailMessage(expected: Array<out T>, actual: Array<out T>): String =
  failMessage(expected.contentToString(), actual.contentToString())

fun arrayFailMessage(expected: IntArray, actual: IntArray): String =
  failMessage(expected.contentToString(), actual.contentToString())

fun arrayFailMessage(expected: ShortArray, actual: ShortArray): String =
  failMessage(expected.contentToString(), actual.contentToString())

fun arrayFailMessage(expected: LongArray, actual: LongArray): String =
  failMessage(expected.contentToString(), actual.contentToString())

fun failMessage(expected: String, actual: String): String =
  "Expected: $expected\nActual: $actual"

fun arrayFailMessage(expected: FloatArray, actual: FloatArray): String {
  return "Expected: ${expected.contentToString()}\nActual: ${actual.contentToString()}"
}

fun arrayFailMessage(expected: DoubleArray, actual: DoubleArray): String {
  return "Expected: ${expected.contentToString()}\nActual: ${actual.contentToString()}"
}

fun arrayFailMessage(expected: BooleanArray, actual: BooleanArray): String {
  return "Expected: ${expected.contentToString()}\nActual: ${actual.contentToString()}"
}

fun arrayFailMessage(expected: ByteArray, actual: ByteArray): String {
  return "Expected: ${expected.contentToString()}\nActual: ${actual.contentToString()}"
}
