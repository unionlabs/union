# Lint as: python3
# Copyright 2020 Google Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
"""Unit tests for flexbuffers.py."""

import array
import os.path
import struct
import unittest

from flatbuffers import flexbuffers

Type = flexbuffers.Type

LOG2 = {1: 0, 2: 1, 4: 2, 8: 3}

GOLD_FLEXBUFFER_OBJ = {
    'bar': [1, 2, 3],
    'bar3': [1, 2, 3],
    'bool': True,
    'bools': [True, False, True, False],
    'foo': 100.0,
    'mymap': {'foo': 'Fred'},
    'vec': [-100, 'Fred', 4.0, b'M', False, 4.0]
}

GOLD_FLEXBUFFER_FILE = 'gold_flexbuffer_example.bin'


def read_test_file(name):
  with open(os.path.join(os.path.dirname(__file__), name), 'rb') as f:
    return f.read()


def packed_type(type_, i):
  return (type_ << 2) | LOG2[i]


def uint_size(value):
  """Returns number of bytes (power of two) to represent unsigned value."""
  assert value >= 0

  n = 8
  while not value < (1 << n):
    n *= 2
  return n // 8


def int_size(value):
  """Returns number of bytes (power of two) to represent signed value."""
  n = 8
  while not -(1 << (n - 1)) <= value < (1 << (n - 1)):
    n *= 2
  return n // 8


def uint_sizes(value):
  return tuple(1 << i for i in range(LOG2[uint_size(value)], 4))


def int_sizes(value):
  return tuple(1 << i for i in range(LOG2[int_size(value)], 4))


def int_bytes(value, byte_width):
  return struct.pack('<%s' % {1: 'b', 2: 'h', 4: 'i', 8: 'q'}[byte_width], value)


def uint_bytes(value, byte_width):
  return struct.pack('<%s' % {1: 'B', 2: 'H', 4: 'I', 8: 'Q'}[byte_width], value)


def float_bytes(value, byte_width):
  return struct.pack('<%s' % {4: 'f', 8: 'd'}[byte_width], value)


def min_value(type_, byte_width):
  assert byte_width > 0

  if type_ in (Type.INT, Type.INDIRECT_INT):
    return -(1 << (8 * byte_width - 1))
  elif type_ in (Type.UINT, Type.INDIRECT_UINT):
    return 0
  else:
    raise ValueError('Unsupported type %s' % type_)


def max_value(type_, byte_width):
  assert byte_width > 0

  if type_ in (Type.INT, Type.INDIRECT_INT):
    return (1 << (8 * byte_width - 1)) - 1
  elif type_ in (Type.UINT, Type.INDIRECT_UINT):
    return (1 << 8 * byte_width) - 1
  else:
    raise ValueError('Unsupported type %s' % type_)


def str_bytes(value, byte_width):
  value_bytes = value.encode('utf-8')
  return [*uint_bytes(len(value_bytes), byte_width), *value_bytes, 0]


def key_bytes(value):
  return [*value.encode('ascii'), 0]


def encode_type(type_, value, byte_width=None):
  fbb = flexbuffers.Builder()
  add = fbb.Adder(type_)
  if byte_width:
    add(value, byte_width)
  else:
    add(value)
  return fbb.Finish()


INT_MIN_MAX_VALUES = (min_value(Type.INT, 1), max_value(Type.INT, 1),
                      min_value(Type.INT, 2), max_value(Type.INT, 2),
                      min_value(Type.INT, 4), max_value(Type.INT, 4),
                      min_value(Type.INT, 8), max_value(Type.INT, 8))

UINT_MIN_MAX_VALUES = (0, max_value(Type.UINT, 1), max_value(Type.UINT, 2),
                       max_value(Type.UINT, 4), max_value(Type.UINT, 8))


class UtilTest(unittest.TestCase):
  """Tests to check FlexBuffer utility functions."""

  def _test_type_predicate(self, pred, types):
    for type_ in types:
      with self.subTest(type=type_, pred=pred):
        self.assertTrue(pred(type_))

    for type_ in set(Type).difference(types):
      with self.subTest(type=type_, pred=pred):
        self.assertFalse(pred(type_))

  def test_inline_types(self):
    self._test_type_predicate(
        Type.IsInline, (Type.NULL, Type.INT, Type.UINT, Type.FLOAT, Type.BOOL))

  def test_typed_vector(self):
    self._test_type_predicate(
        Type.IsTypedVector,
        (Type.VECTOR_INT, Type.VECTOR_UINT, Type.VECTOR_FLOAT, Type.VECTOR_KEY,
         Type.VECTOR_STRING_DEPRECATED, Type.VECTOR_BOOL))

    self._test_type_predicate(
        Type.IsTypedVectorElementType,
        (Type.INT, Type.UINT, Type.FLOAT, Type.KEY, Type.STRING, Type.BOOL))

    with self.assertRaises(ValueError):
      Type.ToTypedVectorElementType(Type.VECTOR)
    self.assertIs(Type.ToTypedVectorElementType(Type.VECTOR_INT), Type.INT)
    self.assertIs(Type.ToTypedVectorElementType(Type.VECTOR_UINT), Type.UINT)
    self.assertIs(Type.ToTypedVectorElementType(Type.VECTOR_FLOAT), Type.FLOAT)
    self.assertIs(Type.ToTypedVectorElementType(Type.VECTOR_KEY), Type.KEY)
    self.assertIs(
        Type.ToTypedVectorElementType(Type.VECTOR_STRING_DEPRECATED),
        Type.STRING)
    self.assertIs(Type.ToTypedVectorElementType(Type.VECTOR_BOOL), Type.BOOL)

    with self.assertRaises(ValueError):
      Type.ToTypedVector(Type.VECTOR)
    self.assertIs(Type.ToTypedVector(Type.INT), Type.VECTOR_INT)
    self.assertIs(Type.ToTypedVector(Type.UINT), Type.VECTOR_UINT)
    self.assertIs(Type.ToTypedVector(Type.FLOAT), Type.VECTOR_FLOAT)
    self.assertIs(Type.ToTypedVector(Type.KEY), Type.VECTOR_KEY)
    self.assertIs(
        Type.ToTypedVector(Type.STRING), Type.VECTOR_STRING_DEPRECATED)
    self.assertIs(Type.ToTypedVector(Type.BOOL), Type.VECTOR_BOOL)

  def test_fixed_typed_vector(self):
    self._test_type_predicate(
        Type.IsFixedTypedVector,
        (Type.VECTOR_INT2, Type.VECTOR_UINT2, Type.VECTOR_FLOAT2,
         Type.VECTOR_INT3, Type.VECTOR_UINT3, Type.VECTOR_FLOAT3,
         Type.VECTOR_INT4, Type.VECTOR_UINT4, Type.VECTOR_FLOAT4))

    self._test_type_predicate(Type.IsFixedTypedVectorElementType,
                              (Type.INT, Type.UINT, Type.FLOAT))

    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_INT2), (Type.INT, 2))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_UINT2), (Type.UINT, 2))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_FLOAT2), (Type.FLOAT, 2))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_INT3), (Type.INT, 3))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_UINT3), (Type.UINT, 3))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_FLOAT3), (Type.FLOAT, 3))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_INT4), (Type.INT, 4))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_UINT4), (Type.UINT, 4))
    self.assertEqual(
        Type.ToFixedTypedVectorElementType(Type.VECTOR_FLOAT4), (Type.FLOAT, 4))

    # Invalid size
    for type_ in Type.INT, Type.UINT, Type.FLOAT:
      with self.assertRaises(ValueError):
        Type.ToTypedVector(type_, 1)
      with self.assertRaises(ValueError):
        Type.ToTypedVector(type_, 5)

    # Invalid element type
    for length in 1, 2, 3, 4, 5:
      with self.assertRaises(ValueError):
        Type.ToTypedVector(Type.STRING, length)

    self.assertIs(Type.ToTypedVector(Type.INT, 2), Type.VECTOR_INT2)
    self.assertIs(Type.ToTypedVector(Type.INT, 3), Type.VECTOR_INT3)
    self.assertIs(Type.ToTypedVector(Type.INT, 4), Type.VECTOR_INT4)

    self.assertIs(Type.ToTypedVector(Type.UINT, 2), Type.VECTOR_UINT2)
    self.assertIs(Type.ToTypedVector(Type.UINT, 3), Type.VECTOR_UINT3)
    self.assertIs(Type.ToTypedVector(Type.UINT, 4), Type.VECTOR_UINT4)

    self.assertIs(Type.ToTypedVector(Type.FLOAT, 2), Type.VECTOR_FLOAT2)
    self.assertIs(Type.ToTypedVector(Type.FLOAT, 3), Type.VECTOR_FLOAT3)
    self.assertIs(Type.ToTypedVector(Type.FLOAT, 4), Type.VECTOR_FLOAT4)

  def test_width(self):
    for x in range(1 << 10):
      self.assertEqual(flexbuffers.BitWidth.U(x), LOG2[uint_size(x)])

    for x in range(-(1 << 10), 1 << 10):
      self.assertEqual(flexbuffers.BitWidth.I(x), LOG2[int_size(x)])

  def test_padding(self):
    self.assertEqual(flexbuffers._PaddingBytes(0, 4), 0)
    self.assertEqual(flexbuffers._PaddingBytes(0, 8), 0)
    self.assertEqual(flexbuffers._PaddingBytes(0, 16), 0)

    self.assertEqual(flexbuffers._PaddingBytes(1, 8), 7)
    self.assertEqual(flexbuffers._PaddingBytes(17, 8), 7)

    self.assertEqual(flexbuffers._PaddingBytes(42, 2), 0)


class DecoderTest(unittest.TestCase):
  """Tests to check FlexBuffer decoding functions.

  Common variable names used in the tests for compactness:
    bw: byte_width
    ebw: element_byte_width
    kbw: key_byte_width
    vbw: value_byte_width
    tbw: type_byte_width

  Having '_ignored' suffix means that variable doesn't affect the constructed
  byte buffer size.
  """

  def test_null(self):
    for bw in 1, 2, 4, 8:
      for ebw_ignored in 1, 2, 4, 8:
        with self.subTest(bw=bw, ebw_ignored=ebw_ignored):
          data = bytes([
              *uint_bytes(0, bw),
              packed_type(Type.NULL, ebw_ignored),
              bw,
          ])

          root = flexbuffers.GetRoot(data)
          self.assertTrue(root.IsNull)
          self.assertEqual(root.AsBool, False)
          self.assertEqual(root.AsInt, 0)
          self.assertEqual(root.AsFloat, 0.0)

          for prop in (type(root).AsKey, type(root).AsString, type(root).AsBlob,
                       type(root).AsVector, type(root).AsTypedVector,
                       type(root).AsFixedTypedVector, type(root).AsMap):
            with self.assertRaises(TypeError):
              prop.fget(root)

          self.assertEqual(root.Value, None)

          self.assertIsNone(flexbuffers.Loads(data))

  def test_bool(self):
    for value in False, True:
      for bw in 1, 2, 4, 8:
        for ebw_ignored in 1, 2, 4, 8:
          with self.subTest(bw=bw, ebw_ignored=ebw_ignored):
            data = bytes([
                *uint_bytes(int(value), bw),
                packed_type(Type.BOOL, ebw_ignored),
                bw,
            ])

            root = flexbuffers.GetRoot(data)
            self.assertTrue(root.IsBool)
            self.assertEqual(root.AsBool, value)
            self.assertEqual(root.AsInt, int(value))
            self.assertEqual(root.AsFloat, float(value))

            for prop in (type(root).AsKey, type(root).AsString,
                         type(root).AsBlob,
                         type(root).AsVector, type(root).AsTypedVector,
                         type(root).AsFixedTypedVector, type(root).AsMap):
              with self.assertRaises(TypeError):
                prop.fget(root)

            self.assertEqual(root.Value, value)

            self.assertEqual(flexbuffers.Loads(data), value)

  def test_mutate_bool(self):
    root = flexbuffers.GetRoot(flexbuffers.Dumps(True))
    self.assertTrue(root.IsBool)
    self.assertTrue(root.AsBool)

    self.assertTrue(root.MutateBool(False))
    self.assertTrue(root.IsBool)
    self.assertFalse(root.AsBool)

    self.assertTrue(root.MutateBool(True))
    self.assertTrue(root.IsBool)
    self.assertTrue(root.AsBool)

  def _check_int(self, data, value):
    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsInt)
    self.assertEqual(root.AsInt, value)
    self.assertEqual(root.AsBool, bool(value))
    self.assertEqual(root.AsFloat, float(value))

    for prop in (type(root).AsKey, type(root).AsString, type(root).AsBlob,
                 type(root).AsVector, type(root).AsTypedVector,
                 type(root).AsFixedTypedVector, type(root).AsMap):
      with self.assertRaises(TypeError):
        prop.fget(root)

    self.assertEqual(root.Value, value)

    self.assertEqual(flexbuffers.Loads(data), value)

  def test_int(self):
    for value in (0, 1, -1, 15, -17, *INT_MIN_MAX_VALUES):
      for bw in int_sizes(value):
        for ebw_ignored in 1, 2, 4, 8:
          with self.subTest(value=value, bw=bw, ebw_ignored=ebw_ignored):
            data = bytes([
                *int_bytes(value, bw),
                packed_type(Type.INT, ebw_ignored),
                bw,
            ])

            self._check_int(data, value)

  def test_indirect_int(self):
    for value in (0, 1, -1, 15, -17, *INT_MIN_MAX_VALUES):
      for bw in 1, 2, 4, 8:
        for ebw in int_sizes(value):
          with self.subTest(value=value, bw=bw, ebw=ebw):
            data = bytes([
                # Int
                *int_bytes(value, ebw),
                # Root
                *uint_bytes(ebw, bw),
                packed_type(Type.INDIRECT_INT, ebw),
                bw,
            ])
            self._check_int(data, value)

  def test_uint(self):
    for value in (1, *UINT_MIN_MAX_VALUES):
      for bw in uint_sizes(value):
        for ebw_ignored in 1, 2, 4, 8:
          with self.subTest(value=value, bw=bw, ebw_ignored=ebw_ignored):
            data = bytes([
                *uint_bytes(value, bw),
                packed_type(Type.UINT, ebw_ignored),
                bw,
            ])

            self._check_int(data, value)

  def test_inidirect_uint(self):
    for value in (1, *UINT_MIN_MAX_VALUES):
      for bw in 1, 2, 4, 8:
        for ebw in uint_sizes(value):
          with self.subTest(value=value, bw=bw, ebw=ebw):
            data = bytes([
                # UInt
                *uint_bytes(value, ebw),
                # Root
                *uint_bytes(ebw, bw),
                packed_type(Type.INDIRECT_UINT, ebw),
                bw,
            ])

            self._check_int(data, value)

  def test_mutate_ints(self):
    # Signed
    for type_ in Type.INT, Type.INDIRECT_INT:
      with self.subTest(type=type_):
        root = flexbuffers.GetRoot(encode_type(type_, 56))
        self.assertEqual(root.AsInt, 56)

        for new_value in 0, 1, -1, -128, 127:
          self.assertTrue(root.MutateInt(new_value))
          self.assertEqual(root.AsInt, new_value)

        for new_value in -129, 128:
          self.assertFalse(root.MutateInt(new_value))

    # Unsigned
    for type_ in Type.UINT, Type.INDIRECT_UINT:
      with self.subTest(type=type_):
        root = flexbuffers.GetRoot(encode_type(type_, 1))
        self.assertEqual(root.AsInt, 1)

        for new_value in 0, 1, 255:
          self.assertTrue(root.MutateInt(new_value))
          self.assertEqual(root.AsInt, new_value)

        self.assertFalse(root.MutateInt(256))

    # Inside vector
    fbb = flexbuffers.Builder()
    fbb.VectorFromElements([13, 0, -15])
    data = fbb.Finish()

    self.assertEqual(flexbuffers.Loads(data), [13, 0, -15])
    self.assertTrue(flexbuffers.GetRoot(data).AsVector[0].MutateInt(0))
    self.assertTrue(flexbuffers.GetRoot(data).AsVector[1].MutateInt(-7))
    self.assertTrue(flexbuffers.GetRoot(data).AsVector[2].MutateInt(45))
    self.assertEqual(flexbuffers.Loads(data), [0, -7, 45])

    # Inside map
    fbb = flexbuffers.Builder()
    fbb.MapFromElements({'x': -7, 'y': 46})
    data = fbb.Finish()

    self.assertEqual(flexbuffers.Loads(data), {'x': -7, 'y': 46})
    self.assertTrue(flexbuffers.GetRoot(data).AsMap['x'].MutateInt(14))
    self.assertTrue(flexbuffers.GetRoot(data).AsMap['y'].MutateInt(-1))
    self.assertEqual(flexbuffers.Loads(data), {'x': 14, 'y': -1})

  def _check_float(self, data, value):
    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsFloat)
    self.assertAlmostEqual(root.AsFloat, value)

    for prop in (type(root).AsKey, type(root).AsString, type(root).AsBlob,
                 type(root).AsVector, type(root).AsTypedVector,
                 type(root).AsFixedTypedVector, type(root).AsMap):
      with self.assertRaises(TypeError):
        prop.fget(root)

    self.assertAlmostEqual(root.Value, value)

    self.assertAlmostEqual(flexbuffers.Loads(data), value)

  def test_float(self):
    for value in -1.0, 0.0, 1.0, 3.141592, 1.5e6:
      for bw in 4, 8:
        for ebw_ignored in 1, 2, 4, 8:
          with self.subTest(value=value, bw=bw, ebw_ignored=ebw_ignored):
            data = bytes([
                *float_bytes(value, bw),
                packed_type(Type.FLOAT, ebw_ignored),
                bw,
            ])

            self._check_float(data, value)

  def test_indirect_float(self):
    for value in -1.0, 0.0, 1.0, 3.141592, 1.5e6:
      for bw in 1, 2, 4, 8:
        for ebw in 4, 8:
          with self.subTest(value=value, bw=bw, ebw=ebw):
            data = bytes([
                # Float
                *float_bytes(value, ebw),
                # Root
                *uint_bytes(ebw, bw),
                packed_type(Type.INDIRECT_FLOAT, ebw),
                bw,
            ])

            self._check_float(data, value)

  def test_mutate_float(self):
    for type_ in Type.FLOAT, Type.INDIRECT_FLOAT:
      for bw in 4, 8:
        value = 3.141592
        root = flexbuffers.GetRoot(encode_type(type_, value, bw))
        self.assertAlmostEqual(root.AsFloat, value)

        value = 2.71828
        self.assertTrue(root.MutateFloat(value))
        self.assertAlmostEqual(root.AsFloat, value, places=5)

    # Inside vector
    data = flexbuffers.Dumps([2.4, 1.5, -7.2])

    self.assertTrue(flexbuffers.GetRoot(data).AsVector[0].MutateFloat(0.0))
    self.assertTrue(flexbuffers.GetRoot(data).AsVector[1].MutateFloat(15.2))
    self.assertTrue(flexbuffers.GetRoot(data).AsVector[2].MutateFloat(-5.1))

    for a, b in zip(flexbuffers.Loads(data), [0.0, 15.2, -5.1]):
      self.assertAlmostEqual(a, b)

  def test_string(self):
    for value in 'red', 'green', 'blue', 'flatbuffers + flexbuffers':
      value_bytes = value.encode('utf-8')
      for bw in 1, 2, 4, 8:
        for lbw in 1, 2, 4, 8:
          with self.subTest(bw=bw, lbw=lbw):
            data = bytes([
                # String
                *uint_bytes(len(value_bytes), lbw),
                *value_bytes,
                0,
                # Root
                *uint_bytes(len(value_bytes) + 1, bw),  # offset
                packed_type(Type.STRING, lbw),
                bw,
            ])

            root = flexbuffers.GetRoot(data)
            self.assertTrue(root.IsString)
            self.assertEqual(root.AsString, value)
            self.assertEqual(root.Value, value)
            self.assertEqual(root.AsInt, len(value))

            self.assertEqual(flexbuffers.Loads(data), value)

  def test_mutate_string(self):
    data = encode_type(Type.STRING, '12345')

    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsString)
    self.assertEqual(root.AsString, '12345')

    self.assertFalse(root.MutateString('543210'))

    self.assertTrue(root.MutateString('54321'))
    self.assertTrue(root.IsString)
    self.assertEqual(root.AsString, '54321')

    self.assertTrue(root.MutateString('543'))
    self.assertTrue(root.IsString)
    self.assertEqual(root.AsString, '543')

    self.assertFalse(root.MutateString('54321'))

  def test_empty_blob(self):
    for bw in 1, 2, 4, 8:
      for lbw in 1, 2, 4, 8:
        with self.subTest(bw=bw, lbw=lbw):
          data = bytes([
              # Blob
              *uint_bytes(0, lbw),
              # Root
              *uint_bytes(0, bw),
              packed_type(Type.BLOB, lbw),
              bw,
          ])

          root = flexbuffers.GetRoot(data)
          self.assertTrue(root.IsBlob)
          self.assertEqual(root.AsBlob, bytes())
          self.assertEqual(root.Value, bytes())
          self.assertEqual(flexbuffers.Loads(data), bytes())

  def test_blob(self):
    for blob in [], [215], [23, 75, 124, 0, 45, 15], 255 * [0]:
      for bw in 1, 2, 4, 8:
        for lbw in 1, 2, 4, 8:
          with self.subTest(blob=blob, bw=bw, lbw=lbw):
            data = bytes([
                # Blob
                *uint_bytes(len(blob), lbw),
                *blob,
                # Root
                *uint_bytes(len(blob), bw),
                packed_type(Type.BLOB, lbw),
                bw,
            ])

            root = flexbuffers.GetRoot(data)
            self.assertTrue(root.IsBlob)
            self.assertEqual(root.AsBlob, bytes(blob))
            self.assertEqual(root.Value, bytes(blob))
            self.assertEqual(flexbuffers.Loads(data), bytes(blob))

  def test_key(self):
    for value in '', 'x', 'color':
      for bw in 1, 2, 4, 8:
        with self.subTest(value=value, bw=bw):
          value_bytes = value.encode('ascii')
          data = bytes([
              # Key
              *value_bytes,
              0,
              # Root
              *uint_bytes(len(value_bytes) + 1, bw),
              packed_type(Type.KEY, 1),
              bw,
          ])

          root = flexbuffers.GetRoot(data)
          self.assertTrue(root.IsKey)
          self.assertEqual(root.AsKey, value)
          self.assertEqual(root.Value, value)
          self.assertEqual(flexbuffers.Loads(data), value)

  def _check_fixed_typed_vector(self, data, vector, type_):
    self.assertEqual(flexbuffers.Loads(data), vector)

    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsFixedTypedVector)

    v = root.AsFixedTypedVector
    self.assertEqual(len(v), len(vector))
    self.assertIs(v.ElementType, type_)
    self.assertEqual([e.Value for e in v], vector)
    self.assertSequenceEqual(v.Value, vector)

    self.assertEqual(root.AsInt, len(vector))

  def test_fixed_typed_vector_float(self):
    for type_, vector in ((Type.VECTOR_FLOAT2, [-75.0, 34.89]),
                          (Type.VECTOR_FLOAT3, [-75.0, 34.89, 12.0]),
                          (Type.VECTOR_FLOAT4, [-75.0, 34.89, -1.0, 1.0])):
      for bw in 1, 2, 4, 8:
        for ebw in 4, 8:
          with self.subTest(type=type_, vector=vector, bw=bw, ebw=ebw):
            data = bytes([
                # FixedTypedVector
                *b''.join(float_bytes(e, ebw) for e in vector),
                # Root
                *uint_bytes(len(vector) * ebw, bw),
                packed_type(type_, ebw),
                bw,
            ])

            for a, b in zip(flexbuffers.Loads(data), vector):
              self.assertAlmostEqual(a, b, places=2)

  def test_fixed_typed_vector_int(self):
    for type_, vector in ((Type.VECTOR_INT2, [0, -13]), (Type.VECTOR_INT3,
                                                         [127, 0, -13]),
                          (Type.VECTOR_INT4, [127, 0, -13, 0])):
      for bw in 1, 2, 4, 8:
        for ebw in 1, 2, 4, 8:
          with self.subTest(type=type_, vector=vector, bw=bw, ebw=ebw):
            data = bytes([
                # FixedTypeVector
                *b''.join(int_bytes(e, ebw) for e in vector),
                # Root
                *uint_bytes(ebw * len(vector), bw),
                packed_type(type_, ebw),
                bw,
            ])

            self._check_fixed_typed_vector(data, vector, Type.INT)

  def test_fixed_typed_vector_uint(self):
    for type_, vector in ((Type.VECTOR_UINT2, [0, 13]),
                          (Type.VECTOR_UINT3, [127, 0, 13]), (Type.VECTOR_UINT4,
                                                              [127, 0, 13, 0])):
      for bw in 1, 2, 4, 8:
        for ebw in 1, 2, 4, 8:
          with self.subTest(type=type_, vector=vector, bw=bw, ebw=ebw):
            data = bytes([
                # FixedTypeVector
                *b''.join(uint_bytes(e, ebw) for e in vector),
                # Root
                *uint_bytes(ebw * len(vector), bw),
                packed_type(type_, ebw),
                bw,
            ])

            self._check_fixed_typed_vector(data, vector, Type.UINT)

  def _check_typed_vector(self, data, vector, type_):
    self.assertEqual(flexbuffers.Loads(data), vector)

    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsTypedVector)

    v = root.AsTypedVector
    self.assertIs(v.ElementType, type_)
    self.assertEqual(len(v), len(vector))
    self.assertEqual([e.Value for e in v], vector)
    self.assertSequenceEqual(v.Value, vector)

    self.assertEqual(root.AsInt, len(vector))

  def test_empty_typed_vector(self):
    for type_ in (Type.VECTOR_BOOL, Type.VECTOR_INT, Type.VECTOR_UINT,
                  Type.VECTOR_FLOAT, Type.VECTOR_KEY,
                  Type.VECTOR_STRING_DEPRECATED):
      for bw in 1, 2, 4, 8:
        for ebw in 1, 2, 4, 8:
          with self.subTest(type=type_, bw=bw, ebw=ebw):
            data = bytes([
                # TypedVector[type_]
                *uint_bytes(0, ebw),
                # Root
                *uint_bytes(0, bw),
                packed_type(type_, ebw),
                bw
            ])

            element_type = Type.ToTypedVectorElementType(type_)
            if element_type == Type.STRING:
              element_type = Type.KEY
            self._check_typed_vector(data, [], element_type)

  def test_typed_vector_bool(self):
    vector = [True, False, False, False, True]

    for bw in 1, 2, 4, 8:
      for ebw in 1, 2, 4, 8:
        with self.subTest(bw=bw, ebw=ebw):
          data = bytes([
              # TypedVector[Type.BOOL]
              *uint_bytes(len(vector), ebw),
              *b''.join(uint_bytes(int(e), ebw) for e in vector),
              # Root
              *uint_bytes(len(vector) * ebw, bw),
              packed_type(Type.VECTOR_BOOL, ebw),
              bw,
          ])
          self._check_typed_vector(data, vector, Type.BOOL)

  def test_typed_vector_int(self):
    vector = [-100, 200, -300]

    for bw in 1, 2, 4, 8:
      for ebw in 2, 4, 8:
        with self.subTest(bw=bw, ebw=ebw):
          data = bytes([
              # TypedVector[Type.INT]
              *uint_bytes(len(vector), ebw),
              *b''.join(int_bytes(e, ebw) for e in vector),
              # Root
              *uint_bytes(len(vector) * ebw, bw),
              packed_type(Type.VECTOR_INT, ebw),
              bw,
          ])
          self._check_typed_vector(data, vector, Type.INT)

  def test_typed_vector_uint(self):
    vector = [100, 200, 300, 400, 0]

    for bw in 1, 2, 4, 8:
      for ebw in 2, 4, 8:
        with self.subTest(bw=bw, ebw=ebw):
          data = bytes([
              # TypedVector[Type.UINT]
              *uint_bytes(len(vector), ebw),
              *b''.join(int_bytes(e, ebw) for e in vector),
              # Root
              *uint_bytes(len(vector) * ebw, bw),
              packed_type(Type.VECTOR_UINT, ebw),
              bw,
          ])
          self._check_typed_vector(data, vector, Type.UINT)

  def test_typed_vector_float(self):
    vector = [3.64, -6.36, 3.14, 634.0, -42.0]

    for bw in 1, 2, 4, 8:
      for ebw in 4, 8:
        with self.subTest(bw=bw, ebw=ebw):
          data = bytes([
              # TypedVector[Type.FLOAT]
              *uint_bytes(len(vector), ebw),
              *b''.join(float_bytes(e, ebw) for e in vector),
              # Root
              *uint_bytes(ebw * len(vector), bw),
              packed_type(Type.VECTOR_FLOAT, ebw),
              bw,
          ])

          for a, b in zip(flexbuffers.Loads(data), vector):
            self.assertAlmostEqual(a, b, places=2)

  def test_typed_vector_key(self):
    vector = ['red', 'green', 'blue']

    for bw in 1, 2, 4, 8:
      for ebw in 1, 2, 4, 8:
        with self.subTest(bw=bw, ebw=ebw):
          data = bytes([
              # Keys
              *key_bytes(vector[0]),
              *key_bytes(vector[1]),
              *key_bytes(vector[2]),
              # TypedVector[Type.KEY]
              *uint_bytes(len(vector), ebw),
              *uint_bytes(15 + 1 * ebw, ebw),  # offset to vector[0]
              *uint_bytes(11 + 2 * ebw, ebw),  # offset to vector[1]
              *uint_bytes(5 + 3 * ebw, ebw),  # offset to vector[2]
              # Root
              *uint_bytes(len(vector) * ebw, bw),  # offset to vector
              packed_type(Type.VECTOR_KEY, ebw),
              bw,
          ])
          self._check_typed_vector(data, vector, Type.KEY)

  def test_typed_vector_string(self):
    vector = ['red', 'green', 'blue']

    for bw in 1, 2, 4, 8:
      for ebw in 1, 2, 4, 8:
        with self.subTest(bw=bw, ebw=ebw):
          data = bytes([
              # Strings
              *str_bytes(vector[0], 1),  # 5 bytes
              *str_bytes(vector[1], 1),  # 7 bytes
              *str_bytes(vector[2], 1),  # 6 bytes
              # TypedVector[Type.STRING]
              *uint_bytes(len(vector), ebw),
              *uint_bytes(17 + 1 * ebw, ebw),  # offset to vector[0]
              *uint_bytes(12 + 2 * ebw, ebw),  # offset to vector[1]
              *uint_bytes(5 + 3 * ebw, ebw),  # offset to vector[2]
              # Root
              *uint_bytes(len(vector) * ebw, bw),  # offset to vector
              packed_type(Type.VECTOR_STRING_DEPRECATED, ebw),
              bw,
          ])

          # We have to pass Type.KEY because of Type.VECTOR_STRING_DEPRECATED.
          self._check_typed_vector(data, vector, Type.KEY)

  def test_typed_vector_string_deprecated(self):
    # Check FlexBuffersDeprecatedTest() inside test.cpp for details.
    vector = [300 * 'A', 'test']

    fbb = flexbuffers.Builder()
    with fbb.TypedVector():
      for e in vector:
        fbb.String(e)
    data = fbb.Finish()

    # We have to pass Type.KEY because of Type.VECTOR_STRING_DEPRECATED.
    self._check_typed_vector(data, vector, Type.KEY)

  def test_typed_vector_invalid(self):
    fbb = flexbuffers.Builder()

    with self.assertRaises(RuntimeError):
      fbb.TypedVectorFromElements(['string', 423])

  def test_empty_vector(self):
    for bw in 1, 2, 4, 8:
      for ebw in 1, 2, 4, 8:
        data = bytes([
            *uint_bytes(0, ebw),
            # Root
            *uint_bytes(0, bw),
            packed_type(Type.VECTOR, ebw),
            bw,
        ])

        root = flexbuffers.GetRoot(data)
        self.assertTrue(root.IsVector)
        self.assertEqual(len(root.AsVector), 0)

        self.assertEqual(flexbuffers.Loads(data), [])

  def test_vector1(self):
    vector = [300, 400, 500]

    for bw in 1, 2, 4, 8:
      for ebw in 2, 4, 8:
        for tbw_ignored in 1, 2, 4, 8:
          with self.subTest(bw=bw, ebw=ebw, ignore=tbw_ignored):
            data = bytes([
                # Vector length
                *uint_bytes(len(vector), ebw),
                # Vector elements
                *int_bytes(vector[0], ebw),
                *int_bytes(vector[1], ebw),
                *int_bytes(vector[2], ebw),
                # Vector types
                packed_type(Type.INT, tbw_ignored),
                packed_type(Type.INT, tbw_ignored),
                packed_type(Type.INT, tbw_ignored),
                # Root
                *uint_bytes(ebw * len(vector) + len(vector), bw),
                packed_type(Type.VECTOR, ebw),
                bw,
            ])

            root = flexbuffers.GetRoot(data)
            self.assertTrue(root.IsVector)
            self.assertFalse(root.IsMap)

            v = root.AsVector
            self.assertEqual(len(v), len(vector))

            for i in range(len(v)):
              self.assertTrue(v[i].IsInt)
              self.assertEqual(v[i].AsInt, vector[i])

            for i, e in enumerate(v):
              self.assertTrue(e.IsInt)
              self.assertEqual(e.AsInt, vector[i])

            with self.assertRaises(IndexError):
              v[-1].AsInt  # pylint: disable=pointless-statement

            with self.assertRaises(IndexError):
              v[3].AsInt  # pylint: disable=pointless-statement

            with self.assertRaises(TypeError):
              root.AsMap  # pylint: disable=pointless-statement

            self.assertEqual(root.AsInt, len(vector))
            self.assertEqual(root.AsFloat, float(len(vector)))

            self.assertEqual(flexbuffers.Loads(data), vector)

  def test_vector2(self):
    vector = [1984, 'August', True]

    for bw in 1, 2, 4, 8:
      with self.subTest(bw=bw):
        data = bytes([
            *str_bytes(vector[1], 1),
            # Vector
            *uint_bytes(len(vector), 2),
            *int_bytes(vector[0], 2),
            *uint_bytes(11, 2),  # offset to 'August'
            *uint_bytes(int(vector[2]), 2),
            packed_type(Type.INT, 2),
            packed_type(Type.STRING, 1),
            packed_type(Type.BOOL, 2),
            # Root
            *uint_bytes(2 * len(vector) + len(vector), bw),  # offset to vector
            packed_type(Type.VECTOR, 2),
            bw,
        ])
        self.assertEqual(flexbuffers.Loads(data), vector)

        root = flexbuffers.GetRoot(data)
        self.assertTrue(root.IsVector)

        v = root.AsVector
        self.assertTrue(v[0].IsInt)
        self.assertEqual(v[0].AsInt, 1984)

        self.assertTrue(v[1].IsString)
        self.assertEqual(v[1].AsString, 'August')

        self.assertTrue(v[2].IsBool)
        self.assertTrue(v[2].AsBool)

        self.assertEqual(v.Value, vector)

        self.assertEqual(root.AsInt, len(vector))

  def test_empty_map(self):
    for bw in 1, 2, 4, 8:
      for kbw in 1, 2, 4, 8:
        for vbw in 1, 2, 4, 8:
          data = bytes([
              *uint_bytes(0, kbw),  # Keys length
              *uint_bytes(0, vbw),
              *uint_bytes(kbw, vbw),
              *uint_bytes(0, vbw),  # Values length
              # Root
              *uint_bytes(0, bw),
              packed_type(Type.MAP, vbw),
              bw,
          ])

          root = flexbuffers.GetRoot(data)
          self.assertTrue(root.IsMap)
          self.assertEqual(len(root.AsMap), 0)

          self.assertEqual(flexbuffers.Loads(data), {})

  def test_map(self):
    value = {'foo': 13, 'bar': 14}

    for bw in 1, 2, 4, 8:
      for kbw in 1, 2, 4, 8:
        for vbw in 1, 2, 4, 8:
          with self.subTest(kbw=kbw, vbw=vbw, bw=bw):
            data = bytes([
                *key_bytes('foo'),  # 4 bytes
                *key_bytes('bar'),  # 4 bytes
                # Map
                *uint_bytes(len(value), kbw),
                *uint_bytes(4 + 1 * kbw, kbw),  # offset to 'bar'
                *uint_bytes(8 + 2 * kbw, kbw),  # offset to 'foo'
                *uint_bytes(len(value) * kbw, vbw),  # offset to keys
                *uint_bytes(kbw, vbw),
                *uint_bytes(len(value), vbw),
                *int_bytes(value['bar'], vbw),
                *int_bytes(value['foo'], vbw),
                packed_type(Type.INT, vbw),
                packed_type(Type.INT, vbw),
                # Root
                *uint_bytes(vbw * len(value) + len(value),
                            bw),  # offset to values
                packed_type(Type.MAP, vbw),
                bw,
            ])

            root = flexbuffers.GetRoot(data)
            self.assertTrue(root.IsMap)

            m = root.AsMap
            self.assertEqual(len(m), 2)
            self.assertEqual(m[0].AsInt, 14)
            self.assertEqual(m[1].AsInt, 13)

            self.assertEqual(m['bar'].AsInt, 14)
            self.assertEqual(m['foo'].AsInt, 13)

            for invalid_key in 'a', 'b', 'no':
              with self.assertRaises(KeyError):
                m[invalid_key]  # pylint: disable=pointless-statement

            values = m.Values
            self.assertEqual(len(values), 2)
            self.assertEqual(values[0].AsInt, 14)
            self.assertEqual(values[1].AsInt, 13)

            keys = m.Keys
            self.assertEqual(len(keys), 2)
            self.assertEqual(len(keys[0].AsKey), 3)
            self.assertEqual(keys[0].AsKey, 'bar')
            self.assertEqual(len(keys[1].AsKey), 3)
            self.assertEqual(keys[1].AsKey, 'foo')

            keys = [key.AsKey for key in keys]
            self.assertEqual(sorted(keys), keys)

            self.assertEqual(root.AsInt, len(value))

            self.assertEqual(flexbuffers.Loads(data), value)

  def test_alignment(self):
    value = ['test', 7]

    data = bytes([
        *key_bytes('test'),  # 5 bytes: 'test' and \0
        0,
        0,
        0,  # 3 bytes: alignment
        # Vector
        *uint_bytes(len(value), byte_width=8),
        *uint_bytes(16, byte_width=8),
        *uint_bytes(7, byte_width=8),
        packed_type(Type.KEY, 1),
        packed_type(Type.INT, 8),
        # Root
        *uint_bytes(8 * len(value) + len(value), 1),
        packed_type(Type.VECTOR, 8),
        1,
    ])

    self.assertEqual(flexbuffers.Loads(data), value)


class EncoderTest(unittest.TestCase):
  """Tests to check FlexBuffer encoding functions."""

  def test_null(self):
    def encode_null():
      fbb = flexbuffers.Builder()
      fbb.Null()
      return fbb.Finish()

    self.assertIsNone(flexbuffers.Loads(encode_null()))

  def test_bool(self):
    for value in False, True:
      data = encode_type(Type.BOOL, value)
      self.assertEqual(flexbuffers.Loads(data), value)

  def test_int(self):
    for byte_width in 1, 2, 4, 8:
      for type_ in Type.INT, Type.INDIRECT_INT, Type.UINT, Type.INDIRECT_UINT:
        with self.subTest(byte_width=byte_width, type=type_):
          value = min_value(type_, byte_width)
          data = encode_type(type_, value)
          self.assertEqual(flexbuffers.Loads(data), value)

          value = max_value(type_, byte_width)
          data = encode_type(type_, value)
          self.assertEqual(flexbuffers.Loads(data), value)

  def test_float(self):
    for value in 3.141592, 7.62, 999.99:
      for type_ in Type.FLOAT, Type.INDIRECT_FLOAT:
        with self.subTest(value=value, type=type_):
          data = encode_type(type_, value)
          self.assertEqual(flexbuffers.Loads(data), value)

          data = encode_type(type_, value, 4)
          self.assertAlmostEqual(flexbuffers.Loads(data), value, places=4)

          data = encode_type(type_, value, 8)
          self.assertEqual(flexbuffers.Loads(data), value)

  def test_string(self):
    for value in '', 'x', 'color', 'hello world':
      with self.subTest(value=value):
        data = encode_type(Type.STRING, value)
        self.assertEqual(flexbuffers.Loads(data), value)

  def test_blob(self):
    for value in bytes(), bytes([240, 12, 143, 7]), bytes(1000 * [17]):
      with self.subTest(value=value):
        data = encode_type(Type.BLOB, value)
        self.assertEqual(flexbuffers.Loads(data), value)

  def test_key(self):
    for value in '', 'color', 'hello world':
      with self.subTest(value=value):
        data = encode_type(Type.KEY, value)
        self.assertEqual(flexbuffers.Loads(data), value)

    with self.assertRaises(ValueError):
      encode_type(Type.KEY, (b'\x00' * 10).decode('ascii'))

  def test_vector(self):

    def encode_vector(elements, element_type):
      fbb = flexbuffers.Builder()
      with fbb.Vector():
        add = fbb.Adder(element_type)
        for e in elements:
          add(e)
      return fbb.Finish()

    def encode_vector_from_elements(elements):
      fbb = flexbuffers.Builder()
      fbb.VectorFromElements(elements)
      return fbb.Finish()

    for elements in [], [1435], [56, 23, 0, 6783]:
      data = encode_vector(elements, Type.INT)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_vector_from_elements(elements)
      self.assertEqual(flexbuffers.Loads(data), elements)

    # Elements of different type: one by one
    elements = [56.0, 'flexbuffers', 0, False, 75123]

    fbb = flexbuffers.Builder()
    with fbb.Vector():
      fbb.Float(elements[0])
      fbb.String(elements[1])
      fbb.UInt(elements[2], 8)
      fbb.Bool(elements[3])
      fbb.Int(elements[4])
    data = fbb.Finish()
    self.assertEqual(flexbuffers.Loads(data), elements)

    # Elements of different type: all at once
    fbb = flexbuffers.Builder()
    fbb.VectorFromElements(elements)
    data = fbb.Finish()
    self.assertEqual(flexbuffers.Loads(data), elements)

  def test_nested_vectors(self):
    fbb = flexbuffers.Builder()
    with fbb.Vector():
      fbb.String('begin')
      fbb.IndirectInt(42)
      with fbb.Vector():
        for i in range(5):
          fbb.Int(i)
      fbb.String('end')
    data = fbb.Finish()

    self.assertEqual(
        flexbuffers.Loads(data), ['begin', 42, [0, 1, 2, 3, 4], 'end'])

  def test_big_vector(self):
    n = 10 * 1000
    fbb = flexbuffers.Builder()
    with fbb.Vector():
      for i in range(n):
        fbb.Int(i)
    self.assertEqual(flexbuffers.Loads(fbb.Finish()), list(range(n)))

  def test_typed_vector(self):

    def encode_typed_vector_from_elements(elements, element_type=None):
      fbb = flexbuffers.Builder()
      fbb.TypedVectorFromElements(elements, element_type)
      return fbb.Finish()

    for elements in [], [False], [True], [False, True, True, False, False]:
      data = encode_typed_vector_from_elements(elements, Type.BOOL)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_typed_vector_from_elements(elements)
      self.assertEqual(flexbuffers.Loads(data), elements)

    for elements in [], [23455], [351, -2, 0, 6783, 0, -10]:
      data = encode_typed_vector_from_elements(elements, Type.INT)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_typed_vector_from_elements(elements)
      self.assertEqual(flexbuffers.Loads(data), elements)

    for elements in [], [23455], [351, 2, 0, 6783, 0, 10]:
      data = encode_typed_vector_from_elements(elements)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_typed_vector_from_elements(elements, Type.INT)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_typed_vector_from_elements(elements, Type.UINT)
      self.assertEqual(flexbuffers.Loads(data), elements)

    for elements in [], [7.0], [52.0, 51.2, 70.0, -4.0]:
      data = encode_typed_vector_from_elements(elements, Type.FLOAT)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_typed_vector_from_elements(elements)
      self.assertEqual(flexbuffers.Loads(data), elements)

    for elements in [], ['color'], ['x', 'y']:
      data = encode_typed_vector_from_elements(elements, Type.KEY)
      self.assertEqual(flexbuffers.Loads(data), elements)

      data = encode_typed_vector_from_elements(elements)
      self.assertEqual(flexbuffers.Loads(data), elements)

  def test_typed_vector_from_array(self):

    def encode_array(typecode, values):
      fbb = flexbuffers.Builder()
      fbb.VectorFromElements(array.array(typecode, values))
      return fbb.Finish()

    values = [1.0, 3.14, -2.54, 0.0]
    data = encode_array('f', values)
    for a, b in zip(flexbuffers.Loads(data), values):
      self.assertAlmostEqual(a, b, places=2)

    values = [1.0, 3.14, -2.54, 0.0]
    data = encode_array('d', values)
    self.assertEqual(flexbuffers.Loads(data), values)

    values = [1, -7, 9, 26, 12]
    data = encode_array('i', values)
    self.assertEqual(flexbuffers.Loads(data), values)

    values = [0, 1, 2, 3, 4, 5, 6]
    data = encode_array('I', values)
    self.assertEqual(flexbuffers.Loads(data), values)

  def test_fixed_typed_vector(self):

    def encode_fixed_typed_vector(elements, element_type=None):
      fbb = flexbuffers.Builder()
      fbb.FixedTypedVectorFromElements(elements, element_type)
      return fbb.Finish()

    for elements in ((-2, 2), (1, 2, 3), (100, -100, 200, -200), (4.0, 7.0),
                     (0.0, 1.0, 8.0), (9.0, 7.0, 1.0, 5.5)):
      with self.subTest(elements=elements):
        data = encode_fixed_typed_vector(elements)
        self.assertSequenceEqual(flexbuffers.Loads(data), elements)

    elements = [-170, 432, 0, -7]
    data = encode_fixed_typed_vector(elements, Type.INT)
    self.assertSequenceEqual(flexbuffers.Loads(data), elements)

    with self.assertRaises(ValueError):
      encode_fixed_typed_vector([])  # Invalid input length

    with self.assertRaises(ValueError):
      encode_fixed_typed_vector([1])  # Invalid input length

    with self.assertRaises(ValueError):
      encode_fixed_typed_vector([1, 2, 3, 4, 5])  # Invalid input length

    with self.assertRaises(TypeError):
      encode_fixed_typed_vector([1, 1.0])  # Invalid input types

    with self.assertRaises(TypeError):
      encode_fixed_typed_vector(['', ''])  # Invalid input types

  def test_map_builder(self):

    def get_keys(data):
      return [key.AsKey for key in flexbuffers.GetRoot(data).AsMap.Keys]

    # Empty map
    fbb = flexbuffers.Builder()
    with fbb.Map():
      pass
    data = fbb.Finish()

    self.assertEqual(flexbuffers.Loads(data), {})

    # Two-element map of Int
    fbb = flexbuffers.Builder()
    with fbb.Map():
      fbb.Int('y', -2)
      fbb.Int('x', 10)
    data = fbb.Finish()

    self.assertEqual(flexbuffers.Loads(data), {'x': 10, 'y': -2})

    # Multiple-element map of vectors
    fbb = flexbuffers.Builder()
    with fbb.Map():
      with fbb.Vector('v'):
        fbb.Int(45)
      with fbb.TypedVector('tv'):
        fbb.Int(-7)
      fbb.FixedTypedVectorFromElements('ftv', [-2.0, 1.0])
    data = fbb.Finish()

    self.assertEqual(
        flexbuffers.Loads(data), {
            'v': [45],
            'tv': [-7],
            'ftv': [-2.0, 1.0]
        })

    keys = get_keys(data)
    self.assertEqual(sorted(keys), keys)

    # Multiple-element map of different types
    fbb = flexbuffers.Builder()
    with fbb.Map():
      fbb.Null('n')
      fbb.Bool('b', False)
      fbb.Int('i', -27)
      fbb.UInt('u', 27)
      fbb.Float('f', -0.85)
      fbb.String('s', 'String')
      fbb.Blob('bb', b'data')
      fbb.IndirectInt('ii', -9500)
      fbb.IndirectUInt('iu', 540)
      fbb.IndirectFloat('if', 0.0)
      fbb.VectorFromElements('v', [2, 1, 0.0])
      fbb.TypedVectorFromElements('tv', [2, 1, 0])
      fbb.FixedTypedVectorFromElements('ftv', [2.0, -6.0])
    data = fbb.Finish()

    self.assertEqual(
        flexbuffers.Loads(data), {
            'n': None,
            'b': False,
            'i': -27,
            'u': 27,
            'f': -0.85,
            's': 'String',
            'bb': b'data',
            'ii': -9500,
            'iu': 540,
            'if': 0.0,
            'v': [2, 1, 0.0],
            'tv': [2, 1, 0],
            'ftv': [2.0, -6.0]
        })

    keys = get_keys(data)
    self.assertEqual(sorted(keys), keys)

  def test_map_python(self):
    maps = [
        {},
        {
            'key': 'value'
        },
        {
            'x': None,
            'y': 3400,
            'z': -7040
        },
        {
            'zzz': 100,
            'aaa': 5.0,
            'ccc': ['Test', 32, False, None, True]
        },
        {
            'name': ['John', 'Smith'],
            'valid': True,
            'note': None,
            'address': {
                'lines': [175, 'Alhambra'],
                'city': 'San Francisco',
                'zip': 94123,
            },
        },
    ]

    for m in maps:
      self.assertEqual(flexbuffers.Loads(flexbuffers.Dumps(m)), m)

  def test_gold_from_file(self):
    data = read_test_file(GOLD_FLEXBUFFER_FILE)
    self.assertEqual(flexbuffers.Loads(data), GOLD_FLEXBUFFER_OBJ)

  def test_gold_from_builder(self):
    fbb = flexbuffers.Builder()
    with fbb.Map():
      with fbb.Vector('vec'):
        fbb.Int(-100)
        fbb.String('Fred')
        fbb.IndirectFloat(4.0)
        i_f = fbb.LastValue
        fbb.Blob(bytes([77]))
        fbb.Bool(False)
        fbb.ReuseValue(i_f)

      vec = [1, 2, 3]
      fbb.VectorFromElements('bar', vec)
      fbb.FixedTypedVectorFromElements('bar3', [1, 2, 3])
      fbb.VectorFromElements('bools', [True, False, True, False])
      fbb.Bool('bool', True)
      fbb.Float('foo', 100)
      with fbb.Map('mymap'):
        fbb.String('foo', 'Fred')
    data = fbb.Finish()

    self.assertEqual(flexbuffers.Loads(data), GOLD_FLEXBUFFER_OBJ)

  def test_min_bit_width(self):
    fbb = flexbuffers.Builder(force_min_bit_width=flexbuffers.BitWidth.W8)
    fbb.TypedVectorFromElements([0, 1, 0, 1, 0])
    data = fbb.Finish()

    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsTypedVector)
    self.assertEqual(root.AsTypedVector.ByteWidth, 1)

    fbb = flexbuffers.Builder(force_min_bit_width=flexbuffers.BitWidth.W32)
    fbb.TypedVectorFromElements([0, 1, 0, 1, 0])
    data = fbb.Finish()

    root = flexbuffers.GetRoot(data)
    self.assertTrue(root.IsTypedVector)
    self.assertEqual(root.AsTypedVector.ByteWidth, 4)

  def test_share_keys(self):

    def encode_key_vector(value, count, share_keys):
      fbb = flexbuffers.Builder(share_keys=share_keys)
      with fbb.Vector():
        for _ in range(count):
          fbb.Key(value)
      return fbb.Finish(), fbb.KeyPool.Elements

    data, pool = encode_key_vector('test', 10, share_keys=False)
    self.assertEqual(len(pool), 0)
    self.assertEqual(len(data), 74)
    self.assertEqual(flexbuffers.Loads(data), 10 * ['test'])

    data, pool = encode_key_vector('test', 10, share_keys=True)
    self.assertEqual(len(pool), 1)
    self.assertEqual(pool[0], 'test'.encode('ascii'))
    self.assertEqual(len(data), 29)
    self.assertEqual(flexbuffers.Loads(data), 10 * ['test'])

  def test_share_strings(self):

    def encode_string_vector(value, count, share_strings):
      fbb = flexbuffers.Builder(share_strings=share_strings)
      with fbb.Vector():
        for _ in range(count):
          fbb.String(value)
      return fbb.Finish(), fbb.StringPool.Elements

    data, pool = encode_string_vector('test', 10, share_strings=False)
    self.assertEqual(len(pool), 0)
    self.assertEqual(len(data), 84)
    self.assertEqual(flexbuffers.Loads(data), 10 * ['test'])

    data, pool = encode_string_vector('test', 10, share_strings=True)
    self.assertEqual(len(pool), 1)
    self.assertEqual(pool[0], 'test'.encode('utf-8'))
    self.assertEqual(len(data), 30)
    self.assertEqual(flexbuffers.Loads(data), 10 * ['test'])

  def test_invalid_stack_size(self):
    fbb = flexbuffers.Builder()

    with self.assertRaises(RuntimeError):
      fbb.Finish()

    fbb.Int(100)
    fbb.Int(200)
    with self.assertRaises(RuntimeError):
      fbb.Finish()

    fbb.Clear()
    fbb.Int(420)
    fbb.Finish()


if __name__ == '__main__':
  unittest.main()
