# coding=utf-8
# Copyright 2014 Google Inc. All rights reserved.
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

import os.path
import sys
PY_VERSION = sys.version_info[:2]

import ctypes
from collections import defaultdict
import math
import random
import timeit
import unittest

from flatbuffers import compat
from flatbuffers import util
from flatbuffers.compat import range_func as compat_range
from flatbuffers.compat import NumpyRequiredForThisFeature

import flatbuffers
from flatbuffers import number_types as N

import MyGame  # refers to generated code
import MyGame.Example  # refers to generated code
import MyGame.Example.Any  # refers to generated code
import MyGame.Example.Color  # refers to generated code
import MyGame.Example.Monster  # refers to generated code
import MyGame.Example.Test  # refers to generated code
import MyGame.Example.Stat  # refers to generated code
import MyGame.Example.Vec3  # refers to generated code
import MyGame.MonsterExtra  # refers to generated code
import MyGame.InParentNamespace  # refers to generated code
import MyGame.Example.ArrayTable  # refers to generated code
import MyGame.Example.ArrayStruct  # refers to generated code
import MyGame.Example.NestedStruct  # refers to generated code
import MyGame.Example.TestEnum  # refers to generated code
import MyGame.Example.NestedUnion.NestedUnionTest  # refers to generated code
import MyGame.Example.NestedUnion.Vec3  # refers to generated code
import MyGame.Example.NestedUnion.Any  # refers to generated code
import MyGame.Example.NestedUnion.Test  # refers to generated code
import MyGame.Example.NestedUnion.Color  # refers to generated code
import monster_test_generated  # the one-file version
import optional_scalars
import optional_scalars.ScalarStuff


def create_namespace_shortcut(is_onefile):
  # Create shortcut from either the one-file format or the multi-file format
  global _ANY
  global _COLOR
  global _MONSTER
  global _TEST
  global _STAT
  global _VEC3
  global _IN_PARENT_NAMESPACE
  if is_onefile:
    print('Testing with the one-file generated code')
    _ANY = monster_test_generated
    _COLOR = monster_test_generated
    _MONSTER = monster_test_generated
    _TEST = monster_test_generated
    _STAT = monster_test_generated
    _VEC3 = monster_test_generated
    _IN_PARENT_NAMESPACE = monster_test_generated
  else:
    print('Testing with multi-file generated code')
    _ANY = MyGame.Example.Any
    _COLOR = MyGame.Example.Color
    _MONSTER = MyGame.Example.Monster
    _TEST = MyGame.Example.Test
    _STAT = MyGame.Example.Stat
    _VEC3 = MyGame.Example.Vec3
    _IN_PARENT_NAMESPACE = MyGame.InParentNamespace


def assertRaises(test_case, fn, exception_class):
  """ Backwards-compatible assertion for exceptions raised. """

  exc = None
  try:
    fn()
  except Exception as e:
    exc = e
  test_case.assertTrue(exc is not None)
  test_case.assertTrue(isinstance(exc, exception_class))


class TestWireFormat(unittest.TestCase):

  def test_wire_format(self):
    # Verify that using the generated Python code builds a buffer without
    # returning errors, and is interpreted correctly, for size prefixed
    # representation and regular:
    for sizePrefix in [True, False]:
      for file_identifier in [None, b'MONS']:
        gen_buf, gen_off = make_monster_from_generated_code(
            sizePrefix=sizePrefix, file_identifier=file_identifier)
        CheckReadBuffer(
            gen_buf,
            gen_off,
            sizePrefix=sizePrefix,
            file_identifier=file_identifier)

    # Verify that the canonical flatbuffer file is readable by the
    # generated Python code. Note that context managers are not part of
    # Python 2.5, so we use the simpler open/close methods here:
    f = open('monsterdata_test.mon', 'rb')
    canonicalWireData = f.read()
    f.close()
    CheckReadBuffer(bytearray(canonicalWireData), 0, file_identifier=b'MONS')

    # Write the generated buffer out to a file:
    f = open('monsterdata_python_wire.mon', 'wb')
    f.write(gen_buf[gen_off:])
    f.close()


class TestObjectBasedAPI(unittest.TestCase):
  """ Tests the generated object based API."""

  def test_consistency_with_repeated_pack_and_unpack(self):
    """ Checks the serialization and deserialization between a buffer and

        its python object. It tests in the same way as the C++ object API test,
        ObjectFlatBuffersTest in test.cpp.
    """

    buf, off = make_monster_from_generated_code()

    # Turns a buffer into Python object (T class).
    monster1 = _MONSTER.Monster.GetRootAs(buf, off)
    monsterT1 = _MONSTER.MonsterT.InitFromObj(monster1)

    for sizePrefix in [True, False]:
      # Re-serialize the data into a buffer.
      b1 = flatbuffers.Builder(0)
      if sizePrefix:
        b1.FinishSizePrefixed(monsterT1.Pack(b1))
      else:
        b1.Finish(monsterT1.Pack(b1))
      CheckReadBuffer(b1.Bytes, b1.Head(), sizePrefix)

    # Deserializes the buffer into Python object again.
    monster2 = _MONSTER.Monster.GetRootAs(b1.Bytes, b1.Head())
    # Re-serializes the data into a buffer for one more time.
    monsterT2 = _MONSTER.MonsterT.InitFromObj(monster2)
    for sizePrefix in [True, False]:
      # Re-serializes the data into a buffer
      b2 = flatbuffers.Builder(0)
      if sizePrefix:
        b2.FinishSizePrefixed(monsterT2.Pack(b2))
      else:
        b2.Finish(monsterT2.Pack(b2))
      CheckReadBuffer(b2.Bytes, b2.Head(), sizePrefix)

  def test_default_values_with_pack_and_unpack(self):
    """ Serializes and deserializes between a buffer with default values (no

        specific values are filled when the buffer is created) and its python
        object.
    """
    # Creates a flatbuffer with default values.
    b1 = flatbuffers.Builder(0)
    _MONSTER.MonsterStart(b1)
    gen_mon = _MONSTER.MonsterEnd(b1)
    b1.Finish(gen_mon)

    # Converts the flatbuffer into the object class.
    monster1 = _MONSTER.Monster.GetRootAs(b1.Bytes, b1.Head())
    monsterT1 = _MONSTER.MonsterT.InitFromObj(monster1)

    # Packs the object class into another flatbuffer.
    b2 = flatbuffers.Builder(0)
    b2.Finish(monsterT1.Pack(b2))
    monster2 = _MONSTER.Monster.GetRootAs(b2.Bytes, b2.Head())
    # Checks the default values.
    self.assertTrue(monster2.Pos() is None)
    self.assertEqual(monster2.Mana(), 150)
    self.assertEqual(monster2.Hp(), 100)
    self.assertTrue(monster2.Name() is None)
    self.assertEqual(monster2.Inventory(0), 0)
    self.assertEqual(monster2.InventoryAsNumpy(), 0)
    self.assertEqual(monster2.InventoryLength(), 0)
    self.assertTrue(monster2.InventoryIsNone())
    self.assertEqual(monster2.Color(), 8)
    self.assertEqual(monster2.TestType(), 0)
    self.assertTrue(monster2.Test() is None)
    self.assertTrue(monster2.Test4(0) is None)
    self.assertEqual(monster2.Test4Length(), 0)
    self.assertTrue(monster2.Test4IsNone())
    self.assertEqual(monster2.Testarrayofstring(0), '')
    self.assertEqual(monster2.TestarrayofstringLength(), 0)
    self.assertTrue(monster2.TestarrayofstringIsNone())
    self.assertTrue(monster2.Testarrayoftables(0) is None)
    self.assertEqual(monster2.TestarrayoftablesLength(), 0)
    self.assertTrue(monster2.TestarrayoftablesIsNone())
    self.assertTrue(monster2.Enemy() is None)
    self.assertEqual(monster2.Testnestedflatbuffer(0), 0)
    self.assertEqual(monster2.TestnestedflatbufferAsNumpy(), 0)
    self.assertEqual(monster2.TestnestedflatbufferLength(), 0)
    self.assertTrue(monster2.TestnestedflatbufferIsNone())
    self.assertTrue(monster2.Testempty() is None)
    self.assertFalse(monster2.Testbool())
    self.assertEqual(monster2.Testhashs32Fnv1(), 0)
    self.assertEqual(monster2.Testhashu32Fnv1(), 0)
    self.assertEqual(monster2.Testhashs64Fnv1(), 0)
    self.assertEqual(monster2.Testhashu64Fnv1(), 0)
    self.assertEqual(monster2.Testhashs32Fnv1a(), 0)
    self.assertEqual(monster2.Testhashu32Fnv1a(), 0)
    self.assertEqual(monster2.Testhashs64Fnv1a(), 0)
    self.assertEqual(monster2.Testhashu64Fnv1a(), 0)
    self.assertEqual(monster2.Testarrayofbools(0), 0)
    self.assertEqual(monster2.TestarrayofboolsAsNumpy(), 0)
    self.assertEqual(monster2.TestarrayofboolsLength(), 0)
    self.assertTrue(monster2.TestarrayofboolsIsNone())
    self.assertEqual(monster2.Testf(), 3.14159)
    self.assertEqual(monster2.Testf2(), 3.0)
    self.assertEqual(monster2.Testf3(), 0.0)
    self.assertEqual(monster2.Testarrayofstring2(0), '')
    self.assertEqual(monster2.Testarrayofstring2Length(), 0)
    self.assertTrue(monster2.Testarrayofstring2IsNone())
    self.assertTrue(monster2.Testarrayofsortedstruct(0) is None)
    self.assertEqual(monster2.TestarrayofsortedstructLength(), 0)
    self.assertTrue(monster2.TestarrayofsortedstructIsNone())
    self.assertEqual(monster2.Flex(0), 0)
    self.assertEqual(monster2.FlexAsNumpy(), 0)
    self.assertEqual(monster2.FlexLength(), 0)
    self.assertTrue(monster2.FlexIsNone())
    self.assertTrue(monster2.Test5(0) is None)
    self.assertEqual(monster2.Test5Length(), 0)
    self.assertTrue(monster2.Test5IsNone())
    self.assertEqual(monster2.VectorOfLongs(0), 0)
    self.assertEqual(monster2.VectorOfLongsAsNumpy(), 0)
    self.assertEqual(monster2.VectorOfLongsLength(), 0)
    self.assertTrue(monster2.VectorOfLongsIsNone())
    self.assertEqual(monster2.VectorOfDoubles(0), 0)
    self.assertEqual(monster2.VectorOfDoublesAsNumpy(), 0)
    self.assertEqual(monster2.VectorOfDoublesLength(), 0)
    self.assertTrue(monster2.VectorOfDoublesIsNone())
    self.assertTrue(monster2.ParentNamespaceTest() is None)
    self.assertTrue(monster2.VectorOfReferrables(0) is None)
    self.assertEqual(monster2.VectorOfReferrablesLength(), 0)
    self.assertTrue(monster2.VectorOfReferrablesIsNone())
    self.assertEqual(monster2.SingleWeakReference(), 0)
    self.assertEqual(monster2.VectorOfWeakReferences(0), 0)
    self.assertEqual(monster2.VectorOfWeakReferencesAsNumpy(), 0)
    self.assertEqual(monster2.VectorOfWeakReferencesLength(), 0)
    self.assertTrue(monster2.VectorOfWeakReferencesIsNone())
    self.assertTrue(monster2.VectorOfStrongReferrables(0) is None)
    self.assertEqual(monster2.VectorOfStrongReferrablesLength(), 0)
    self.assertTrue(monster2.VectorOfStrongReferrablesIsNone())
    self.assertEqual(monster2.CoOwningReference(), 0)
    self.assertEqual(monster2.VectorOfCoOwningReferences(0), 0)
    self.assertEqual(monster2.VectorOfCoOwningReferencesAsNumpy(), 0)
    self.assertEqual(monster2.VectorOfCoOwningReferencesLength(), 0)
    self.assertTrue(monster2.VectorOfCoOwningReferencesIsNone())
    self.assertEqual(monster2.NonOwningReference(), 0)
    self.assertEqual(monster2.VectorOfNonOwningReferences(0), 0)
    self.assertEqual(monster2.VectorOfNonOwningReferencesAsNumpy(), 0)
    self.assertEqual(monster2.VectorOfNonOwningReferencesLength(), 0)
    self.assertTrue(monster2.VectorOfNonOwningReferencesIsNone())
    self.assertEqual(monster2.AnyUniqueType(), 0)
    self.assertTrue(monster2.AnyUnique() is None)
    self.assertEqual(monster2.AnyAmbiguousType(), 0)
    self.assertTrue(monster2.AnyAmbiguous() is None)
    self.assertEqual(monster2.VectorOfEnums(0), 0)
    self.assertEqual(monster2.VectorOfEnumsAsNumpy(), 0)
    self.assertEqual(monster2.VectorOfEnumsLength(), 0)
    self.assertTrue(monster2.VectorOfEnumsIsNone())

  def test_optional_scalars_with_pack_and_unpack(self):
    """ Serializes and deserializes between a buffer with optional values (no
        specific values are filled when the buffer is created) and its python
        object.
    """
    # Creates a flatbuffer with optional values.
    b1 = flatbuffers.Builder(0)
    optional_scalars.ScalarStuff.ScalarStuffStart(b1)
    gen_opt = optional_scalars.ScalarStuff.ScalarStuffEnd(b1)
    b1.Finish(gen_opt)

    # Converts the flatbuffer into the object class.
    opts1 = optional_scalars.ScalarStuff.ScalarStuff.GetRootAs(b1.Bytes, b1.Head())
    optsT1 = optional_scalars.ScalarStuff.ScalarStuffT.InitFromObj(opts1)

    # Packs the object class into another flatbuffer.
    b2 = flatbuffers.Builder(0)
    b2.Finish(optsT1.Pack(b2))
    opts2 = optional_scalars.ScalarStuff.ScalarStuff.GetRootAs(b2.Bytes, b2.Head())
    optsT2 = optional_scalars.ScalarStuff.ScalarStuffT.InitFromObj(opts2)
    # Checks the default values.
    self.assertTrue(opts2.JustI8() == 0)
    self.assertTrue(opts2.MaybeF32() is None)
    self.assertTrue(opts2.DefaultBool() is True)
    self.assertTrue(optsT2.justU16 == 0)
    self.assertTrue(optsT2.maybeEnum is None)
    self.assertTrue(optsT2.defaultU64 == 42)



class TestAllMutableCodePathsOfExampleSchema(unittest.TestCase):
  """ Tests the object API generated for monster_test.fbs for mutation

        purposes. In each test, the default values will be changed through the
        object API. We'll then pack the object class into the buf class and read
        the updated values out from it to validate if the values are mutated as
        expected.
  """

  def setUp(self, *args, **kwargs):
    super(TestAllMutableCodePathsOfExampleSchema, self).setUp(*args, **kwargs)
    # Creates an empty monster flatbuffer, and loads it into the object
    # class for future tests.
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStart(b)
    self.monsterT = self._create_and_load_object_class(b)

  def _pack_and_load_buf_class(self, monsterT):
    """ Packs the object class into a flatbuffer and loads it into a buf

        class.
    """
    b = flatbuffers.Builder(0)
    b.Finish(monsterT.Pack(b))
    monster = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    return monster

  def _create_and_load_object_class(self, b):
    """ Finishs the creation of a monster flatbuffer and loads it into an

        object class.
    """
    gen_mon = _MONSTER.MonsterEnd(b)
    b.Finish(gen_mon)
    monster = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    monsterT = _MONSTER.MonsterT()
    monsterT.InitFromObj(monster)
    return monsterT

  def test_mutate_pos(self):
    posT = _VEC3.Vec3T()
    posT.x = 4.0
    posT.y = 5.0
    posT.z = 6.0
    posT.test1 = 6.0
    posT.test2 = 7
    test3T = _TEST.TestT()
    test3T.a = 8
    test3T.b = 9
    posT.test3 = test3T
    self.monsterT.pos = posT

    # Packs the updated values.
    monster = self._pack_and_load_buf_class(self.monsterT)

    # Checks if values are loaded correctly into the object class.
    pos = monster.Pos()

    # Verifies the properties of the Vec3.
    self.assertEqual(pos.X(), 4.0)
    self.assertEqual(pos.Y(), 5.0)
    self.assertEqual(pos.Z(), 6.0)
    self.assertEqual(pos.Test1(), 6.0)
    self.assertEqual(pos.Test2(), 7)
    t3 = _TEST.Test()
    t3 = pos.Test3(t3)
    self.assertEqual(t3.A(), 8)
    self.assertEqual(t3.B(), 9)

  def test_mutate_mana(self):
    self.monsterT.mana = 200
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Mana(), 200)

  def test_mutate_hp(self):
    self.monsterT.hp = 200
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Hp(), 200)

  def test_mutate_name(self):
    self.monsterT.name = 'MyMonster'
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Name(), b'MyMonster')

  def test_mutate_inventory(self):
    self.monsterT.inventory = [1, 7, 8]
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Inventory(0), 1)
    self.assertEqual(monster.Inventory(1), 7)
    self.assertEqual(monster.Inventory(2), 8)

  def test_empty_inventory(self):
    self.monsterT.inventory = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.InventoryIsNone())

  def test_mutate_color(self):
    self.monsterT.color = _COLOR.Color.Red
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Color(), _COLOR.Color.Red)

  def test_mutate_testtype(self):
    self.monsterT.testType = _ANY.Any.Monster
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.TestType(), _ANY.Any.Monster)

  def test_mutate_test(self):
    testT = _MONSTER.MonsterT()
    testT.hp = 200
    self.monsterT.test = testT
    monster = self._pack_and_load_buf_class(self.monsterT)
    # Initializes a Table from a union field Test(...).
    table = monster.Test()

    # Initializes a Monster from the Table from the union.
    test_monster = _MONSTER.Monster()
    test_monster.Init(table.Bytes, table.Pos)
    self.assertEqual(test_monster.Hp(), 200)

  def test_mutate_test4(self):
    test0T = _TEST.TestT()
    test0T.a = 10
    test0T.b = 20
    test1T = _TEST.TestT()
    test1T.a = 30
    test1T.b = 40
    self.monsterT.test4 = [test0T, test1T]

    monster = self._pack_and_load_buf_class(self.monsterT)
    test0 = monster.Test4(0)
    self.assertEqual(test0.A(), 10)
    self.assertEqual(test0.B(), 20)
    test1 = monster.Test4(1)
    self.assertEqual(test1.A(), 30)
    self.assertEqual(test1.B(), 40)

  def test_empty_test4(self):
    self.monsterT.test4 = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.Test4IsNone())

  def test_mutate_testarrayofstring(self):
    self.monsterT.testarrayofstring = []
    self.monsterT.testarrayofstring.append('test1')
    self.monsterT.testarrayofstring.append('test2')
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Testarrayofstring(0), b'test1')
    self.assertEqual(monster.Testarrayofstring(1), b'test2')

  def test_empty_testarrayofstring(self):
    self.monsterT.testarrayofstring = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.TestarrayofstringIsNone())

  def test_mutate_testarrayoftables(self):
    monsterT0 = _MONSTER.MonsterT()
    monsterT0.hp = 200
    monsterT1 = _MONSTER.MonsterT()
    monsterT1.hp = 400
    self.monsterT.testarrayoftables = []
    self.monsterT.testarrayoftables.append(monsterT0)
    self.monsterT.testarrayoftables.append(monsterT1)
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Testarrayoftables(0).Hp(), 200)
    self.assertEqual(monster.Testarrayoftables(1).Hp(), 400)

  def test_empty_testarrayoftables(self):
    self.monsterT.testarrayoftables = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.TestarrayoftablesIsNone())

  def test_mutate_enemy(self):
    monsterT = _MONSTER.MonsterT()
    monsterT.hp = 200
    self.monsterT.enemy = monsterT
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Enemy().Hp(), 200)

  def test_mutate_testnestedflatbuffer(self):
    self.monsterT.testnestedflatbuffer = [8, 2, 4]
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Testnestedflatbuffer(0), 8)
    self.assertEqual(monster.Testnestedflatbuffer(1), 2)
    self.assertEqual(monster.Testnestedflatbuffer(2), 4)

  def test_empty_testnestedflatbuffer(self):
    self.monsterT.testnestedflatbuffer = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.TestnestedflatbufferIsNone())

  def test_mutate_testbool(self):
    self.monsterT.testbool = True
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertTrue(monster.Testbool())

  def test_mutate_testhashes(self):
    self.monsterT.testhashs32Fnv1 = 1
    self.monsterT.testhashu32Fnv1 = 2
    self.monsterT.testhashs64Fnv1 = 3
    self.monsterT.testhashu64Fnv1 = 4
    self.monsterT.testhashs32Fnv1a = 5
    self.monsterT.testhashu32Fnv1a = 6
    self.monsterT.testhashs64Fnv1a = 7
    self.monsterT.testhashu64Fnv1a = 8
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Testhashs32Fnv1(), 1)
    self.assertEqual(monster.Testhashu32Fnv1(), 2)
    self.assertEqual(monster.Testhashs64Fnv1(), 3)
    self.assertEqual(monster.Testhashu64Fnv1(), 4)
    self.assertEqual(monster.Testhashs32Fnv1a(), 5)
    self.assertEqual(monster.Testhashu32Fnv1a(), 6)
    self.assertEqual(monster.Testhashs64Fnv1a(), 7)
    self.assertEqual(monster.Testhashu64Fnv1a(), 8)

  def test_mutate_testarrayofbools(self):
    self.monsterT.testarrayofbools = []
    self.monsterT.testarrayofbools.append(True)
    self.monsterT.testarrayofbools.append(True)
    self.monsterT.testarrayofbools.append(False)
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Testarrayofbools(0), True)
    self.assertEqual(monster.Testarrayofbools(1), True)
    self.assertEqual(monster.Testarrayofbools(2), False)

  def test_empty_testarrayofbools(self):
    self.monsterT.testarrayofbools = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.TestarrayofboolsIsNone())

  def test_mutate_testf(self):
    self.monsterT.testf = 2.0
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.Testf(), 2.0)

  def test_mutate_vectoroflongs(self):
    self.monsterT.vectorOfLongs = []
    self.monsterT.vectorOfLongs.append(1)
    self.monsterT.vectorOfLongs.append(100)
    self.monsterT.vectorOfLongs.append(10000)
    self.monsterT.vectorOfLongs.append(1000000)
    self.monsterT.vectorOfLongs.append(100000000)
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.VectorOfLongs(0), 1)
    self.assertEqual(monster.VectorOfLongs(1), 100)
    self.assertEqual(monster.VectorOfLongs(2), 10000)
    self.assertEqual(monster.VectorOfLongs(3), 1000000)
    self.assertEqual(monster.VectorOfLongs(4), 100000000)

  def test_empty_vectoroflongs(self):
    self.monsterT.vectorOfLongs = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.VectorOfLongsIsNone())

  def test_mutate_vectorofdoubles(self):
    self.monsterT.vectorOfDoubles = []
    self.monsterT.vectorOfDoubles.append(-1.7976931348623157e+308)
    self.monsterT.vectorOfDoubles.append(0)
    self.monsterT.vectorOfDoubles.append(1.7976931348623157e+308)
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.VectorOfDoubles(0), -1.7976931348623157e+308)
    self.assertEqual(monster.VectorOfDoubles(1), 0)
    self.assertEqual(monster.VectorOfDoubles(2), 1.7976931348623157e+308)

  def test_empty_vectorofdoubles(self):
    self.monsterT.vectorOfDoubles = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.VectorOfDoublesIsNone())

  def test_mutate_parentnamespacetest(self):
    self.monsterT.parentNamespaceTest = _IN_PARENT_NAMESPACE.InParentNamespaceT(
    )
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertTrue(
        isinstance(monster.ParentNamespaceTest(),
                   _IN_PARENT_NAMESPACE.InParentNamespace))

  def test_mutate_vectorofEnums(self):
    self.monsterT.vectorOfEnums = []
    self.monsterT.vectorOfEnums.append(_COLOR.Color.Red)
    self.monsterT.vectorOfEnums.append(_COLOR.Color.Blue)
    self.monsterT.vectorOfEnums.append(_COLOR.Color.Red)
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertEqual(monster.VectorOfEnums(0), _COLOR.Color.Red)
    self.assertEqual(monster.VectorOfEnums(1), _COLOR.Color.Blue)
    self.assertEqual(monster.VectorOfEnums(2), _COLOR.Color.Red)

  def test_empty_vectorofEnums(self):
    self.monsterT.vectorOfEnums = []
    monster = self._pack_and_load_buf_class(self.monsterT)
    self.assertFalse(monster.VectorOfEnumsIsNone())


def CheckReadBuffer(buf, offset, sizePrefix=False, file_identifier=None):
  """ CheckReadBuffer checks that the given buffer is evaluated correctly

        as the example Monster.
  """

  def asserter(stmt):
    """ An assertion helper that is separated from TestCase classes. """
    if not stmt:
      raise AssertionError('CheckReadBuffer case failed')

  if file_identifier:
    # test prior to removal of size_prefix
    asserter(
        util.GetBufferIdentifier(buf, offset, size_prefixed=sizePrefix) ==
        file_identifier)
    asserter(
        util.BufferHasIdentifier(
            buf,
            offset,
            file_identifier=file_identifier,
            size_prefixed=sizePrefix))
    asserter(
        _MONSTER.Monster.MonsterBufferHasIdentifier(
            buf, offset, size_prefixed=sizePrefix))
  if sizePrefix:
    size = util.GetSizePrefix(buf, offset)
    asserter(size == len(buf[offset:]) - 4)
    buf, offset = util.RemoveSizePrefix(buf, offset)
  if file_identifier:
    asserter(_MONSTER.Monster.MonsterBufferHasIdentifier(buf, offset))
  else:
    asserter(not _MONSTER.Monster.MonsterBufferHasIdentifier(buf, offset))
  monster = _MONSTER.Monster.GetRootAs(buf, offset)

  asserter(monster.Hp() == 80)
  asserter(monster.Mana() == 150)
  asserter(monster.Name() == b'MyMonster')

  # initialize a Vec3 from Pos()
  vec = monster.Pos()
  asserter(vec is not None)

  # verify the properties of the Vec3
  asserter(vec.X() == 1.0)
  asserter(vec.Y() == 2.0)
  asserter(vec.Z() == 3.0)
  asserter(vec.Test1() == 3.0)
  asserter(vec.Test2() == 2)

  # initialize a Test from Test3(...)
  t = _TEST.Test()
  t = vec.Test3(t)
  asserter(t is not None)

  # verify the properties of the Test
  asserter(t.A() == 5)
  asserter(t.B() == 6)

  # verify that the enum code matches the enum declaration:
  union_type = _ANY.Any
  asserter(monster.TestType() == union_type.Monster)

  # initialize a Table from a union field Test(...)
  table2 = monster.Test()
  asserter(type(table2) is flatbuffers.table.Table)

  # initialize a Monster from the Table from the union
  monster2 = _MONSTER.Monster()
  monster2.Init(table2.Bytes, table2.Pos)

  asserter(monster2.Name() == b'Fred')

  # iterate through the first monster's inventory:
  asserter(monster.InventoryLength() == 5)
  asserter(not monster.InventoryIsNone())

  invsum = 0
  for i in compat_range(monster.InventoryLength()):
    v = monster.Inventory(i)
    invsum += int(v)
  asserter(invsum == 10)

  for i in range(5):
    asserter(monster.VectorOfLongs(i) == 10**(i * 2))

  asserter(not monster.VectorOfDoublesIsNone())
  asserter(([-1.7976931348623157e+308, 0, 1.7976931348623157e+308] == [
      monster.VectorOfDoubles(i) for i in range(monster.VectorOfDoublesLength())
  ]))

  try:
    # if numpy exists, then we should be able to get the
    # vector as a numpy array
    import numpy as np

    asserter(monster.InventoryAsNumpy().sum() == 10)
    asserter(monster.InventoryAsNumpy().dtype == np.dtype('<u1'))

    VectorOfLongs = monster.VectorOfLongsAsNumpy()
    asserter(VectorOfLongs.dtype == np.dtype('<i8'))
    for i in range(5):
      asserter(VectorOfLongs[i] == 10**(i * 2))

    VectorOfDoubles = monster.VectorOfDoublesAsNumpy()
    asserter(VectorOfDoubles.dtype == np.dtype('<f8'))
    asserter(VectorOfDoubles[0] == np.finfo('<f8').min)
    asserter(VectorOfDoubles[1] == 0.0)
    asserter(VectorOfDoubles[2] == np.finfo('<f8').max)

  except ImportError:
    # If numpy does not exist, trying to get vector as numpy
    # array should raise NumpyRequiredForThisFeature. The way
    # assertRaises has been implemented prevents us from
    # asserting this error is raised outside of a test case.
    pass

  asserter(monster.Test4Length() == 2)
  asserter(not monster.Test4IsNone())

  # create a 'Test' object and populate it:
  test0 = monster.Test4(0)
  asserter(type(test0) is _TEST.Test)

  test1 = monster.Test4(1)
  asserter(type(test1) is _TEST.Test)

  # the position of test0 and test1 are swapped in monsterdata_java_wire
  # and monsterdata_test_wire, so ignore ordering
  v0 = test0.A()
  v1 = test0.B()
  v2 = test1.A()
  v3 = test1.B()
  sumtest12 = int(v0) + int(v1) + int(v2) + int(v3)

  asserter(sumtest12 == 100)

  asserter(not monster.TestarrayofstringIsNone())
  asserter(monster.TestarrayofstringLength() == 2)
  asserter(monster.Testarrayofstring(0) == b'test1')
  asserter(monster.Testarrayofstring(1) == b'test2')

  asserter(monster.TestarrayoftablesIsNone())
  asserter(monster.TestarrayoftablesLength() == 0)
  asserter(monster.TestnestedflatbufferIsNone())
  asserter(monster.TestnestedflatbufferLength() == 0)
  asserter(monster.Testempty() is None)


class TestFuzz(unittest.TestCase):
  """ Low level stress/fuzz test: serialize/deserialize a variety of

        different kinds of data in different combinations
  """

  binary_type = compat.binary_types[0]  # this will always exist
  ofInt32Bytes = binary_type([0x83, 0x33, 0x33, 0x33])
  ofInt64Bytes = binary_type([0x84, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44])
  overflowingInt32Val = flatbuffers.encode.Get(flatbuffers.packer.int32,
                                               ofInt32Bytes, 0)
  overflowingInt64Val = flatbuffers.encode.Get(flatbuffers.packer.int64,
                                               ofInt64Bytes, 0)

  # Values we're testing against: chosen to ensure no bits get chopped
  # off anywhere, and also be different from eachother.
  boolVal = True
  int8Val = N.Int8Flags.py_type(-127)  # 0x81
  uint8Val = N.Uint8Flags.py_type(0xFF)
  int16Val = N.Int16Flags.py_type(-32222)  # 0x8222
  uint16Val = N.Uint16Flags.py_type(0xFEEE)
  int32Val = N.Int32Flags.py_type(overflowingInt32Val)
  uint32Val = N.Uint32Flags.py_type(0xFDDDDDDD)
  int64Val = N.Int64Flags.py_type(overflowingInt64Val)
  uint64Val = N.Uint64Flags.py_type(0xFCCCCCCCCCCCCCCC)
  # Python uses doubles, so force it here
  float32Val = N.Float32Flags.py_type(ctypes.c_float(3.14159).value)
  float64Val = N.Float64Flags.py_type(3.14159265359)

  def test_fuzz(self):
    return self.check_once(11, 100)

  def check_once(self, fuzzFields, fuzzObjects):
    testValuesMax = 11  # hardcoded to the number of scalar types

    builder = flatbuffers.Builder(0)
    l = LCG()

    objects = [0 for _ in compat_range(fuzzObjects)]

    # Generate fuzzObjects random objects each consisting of
    # fuzzFields fields, each of a random type.
    for i in compat_range(fuzzObjects):
      builder.StartObject(fuzzFields)

      for j in compat_range(fuzzFields):
        choice = int(l.Next()) % testValuesMax
        if choice == 0:
          builder.PrependBoolSlot(int(j), self.boolVal, False)
        elif choice == 1:
          builder.PrependInt8Slot(int(j), self.int8Val, 0)
        elif choice == 2:
          builder.PrependUint8Slot(int(j), self.uint8Val, 0)
        elif choice == 3:
          builder.PrependInt16Slot(int(j), self.int16Val, 0)
        elif choice == 4:
          builder.PrependUint16Slot(int(j), self.uint16Val, 0)
        elif choice == 5:
          builder.PrependInt32Slot(int(j), self.int32Val, 0)
        elif choice == 6:
          builder.PrependUint32Slot(int(j), self.uint32Val, 0)
        elif choice == 7:
          builder.PrependInt64Slot(int(j), self.int64Val, 0)
        elif choice == 8:
          builder.PrependUint64Slot(int(j), self.uint64Val, 0)
        elif choice == 9:
          builder.PrependFloat32Slot(int(j), self.float32Val, 0)
        elif choice == 10:
          builder.PrependFloat64Slot(int(j), self.float64Val, 0)
        else:
          raise RuntimeError('unreachable')

      off = builder.EndObject()

      # store the offset from the end of the builder buffer,
      # since it will keep growing:
      objects[i] = off

    # Do some bookkeeping to generate stats on fuzzes:
    stats = defaultdict(int)

    def check(table, desc, want, got):
      stats[desc] += 1
      self.assertEqual(want, got, '%s != %s, %s' % (want, got, desc))

    l = LCG()  # Reset.

    # Test that all objects we generated are readable and return the
    # expected values. We generate random objects in the same order
    # so this is deterministic.
    for i in compat_range(fuzzObjects):

      table = flatbuffers.table.Table(builder.Bytes,
                                      len(builder.Bytes) - objects[i])

      for j in compat_range(fuzzFields):
        field_count = flatbuffers.builder.VtableMetadataFields + j
        f = N.VOffsetTFlags.py_type(field_count * N.VOffsetTFlags.bytewidth)
        choice = int(l.Next()) % testValuesMax

        if choice == 0:
          check(table, 'bool', self.boolVal,
                table.GetSlot(f, False, N.BoolFlags))
        elif choice == 1:
          check(table, '<i1', self.int8Val, table.GetSlot(f, 0, N.Int8Flags))
        elif choice == 2:
          check(table, '<u1', self.uint8Val,
                table.GetSlot(f, 0, N.Uint8Flags))
        elif choice == 3:
          check(table, '<i2', self.int16Val,
                table.GetSlot(f, 0, N.Int16Flags))
        elif choice == 4:
          check(table, '<u2', self.uint16Val,
                table.GetSlot(f, 0, N.Uint16Flags))
        elif choice == 5:
          check(table, '<i4', self.int32Val,
                table.GetSlot(f, 0, N.Int32Flags))
        elif choice == 6:
          check(table, '<u4', self.uint32Val,
                table.GetSlot(f, 0, N.Uint32Flags))
        elif choice == 7:
          check(table, '<i8', self.int64Val,
                table.GetSlot(f, 0, N.Int64Flags))
        elif choice == 8:
          check(table, '<u8', self.uint64Val,
                table.GetSlot(f, 0, N.Uint64Flags))
        elif choice == 9:
          check(table, '<f4', self.float32Val,
                table.GetSlot(f, 0, N.Float32Flags))
        elif choice == 10:
          check(table, '<f8', self.float64Val,
                table.GetSlot(f, 0, N.Float64Flags))
        else:
          raise RuntimeError('unreachable')

    # If enough checks were made, verify that all scalar types were used:
    self.assertEqual(testValuesMax, len(stats),
                     'fuzzing failed to test all scalar types: %s' % stats)


class TestByteLayout(unittest.TestCase):
  """ TestByteLayout checks the bytes of a Builder in various scenarios. """

  def assertBuilderEquals(self, builder, want_chars_or_ints):

    def integerize(x):
      if isinstance(x, compat.string_types):
        return ord(x)
      return x

    want_ints = list(map(integerize, want_chars_or_ints))
    want = bytearray(want_ints)
    got = builder.Bytes[builder.Head():]  # use the buffer directly
    self.assertEqual(want, got)

  def test_numbers(self):
    b = flatbuffers.Builder(0)
    self.assertBuilderEquals(b, [])
    b.PrependBool(True)
    self.assertBuilderEquals(b, [1])
    b.PrependInt8(-127)
    self.assertBuilderEquals(b, [129, 1])
    b.PrependUint8(255)
    self.assertBuilderEquals(b, [255, 129, 1])
    b.PrependInt16(-32222)
    self.assertBuilderEquals(b, [0x22, 0x82, 0, 255, 129, 1])  # first pad
    b.PrependUint16(0xFEEE)
    # no pad this time:
    self.assertBuilderEquals(b, [0xEE, 0xFE, 0x22, 0x82, 0, 255, 129, 1])
    b.PrependInt32(-53687092)
    self.assertBuilderEquals(
        b, [204, 204, 204, 252, 0xEE, 0xFE, 0x22, 0x82, 0, 255, 129, 1])
    b.PrependUint32(0x98765432)
    self.assertBuilderEquals(b, [
        0x32, 0x54, 0x76, 0x98, 204, 204, 204, 252, 0xEE, 0xFE, 0x22, 0x82, 0,
        255, 129, 1
    ])

  def test_numbers64(self):
    b = flatbuffers.Builder(0)
    b.PrependUint64(0x1122334455667788)
    self.assertBuilderEquals(b,
                             [0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11])

    b = flatbuffers.Builder(0)
    b.PrependInt64(0x1122334455667788)
    self.assertBuilderEquals(b,
                             [0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11])

  def test_1xbyte_vector(self):
    b = flatbuffers.Builder(0)
    self.assertBuilderEquals(b, [])
    b.StartVector(flatbuffers.number_types.Uint8Flags.bytewidth, 1, 1)
    self.assertBuilderEquals(b, [0, 0, 0])  # align to 4bytes
    b.PrependByte(1)
    self.assertBuilderEquals(b, [1, 0, 0, 0])
    b.EndVector()
    self.assertBuilderEquals(b, [1, 0, 0, 0, 1, 0, 0, 0])  # padding

  def test_2xbyte_vector(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Uint8Flags.bytewidth, 2, 1)
    self.assertBuilderEquals(b, [0, 0])  # align to 4bytes
    b.PrependByte(1)
    self.assertBuilderEquals(b, [1, 0, 0])
    b.PrependByte(2)
    self.assertBuilderEquals(b, [2, 1, 0, 0])
    b.EndVector()
    self.assertBuilderEquals(b, [2, 0, 0, 0, 2, 1, 0, 0])  # padding

  def test_1xuint16_vector(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Uint16Flags.bytewidth, 1, 1)
    self.assertBuilderEquals(b, [0, 0])  # align to 4bytes
    b.PrependUint16(1)
    self.assertBuilderEquals(b, [1, 0, 0, 0])
    b.EndVector()
    self.assertBuilderEquals(b, [1, 0, 0, 0, 1, 0, 0, 0])  # padding

  def test_2xuint16_vector(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Uint16Flags.bytewidth, 2, 1)
    self.assertBuilderEquals(b, [])  # align to 4bytes
    b.PrependUint16(0xABCD)
    self.assertBuilderEquals(b, [0xCD, 0xAB])
    b.PrependUint16(0xDCBA)
    self.assertBuilderEquals(b, [0xBA, 0xDC, 0xCD, 0xAB])
    b.EndVector()
    self.assertBuilderEquals(b, [2, 0, 0, 0, 0xBA, 0xDC, 0xCD, 0xAB])

  def test_create_ascii_shared_string(self):
    b = flatbuffers.Builder(0)
    b.CreateSharedString(u'foo', encoding='ascii')
    b.CreateSharedString(u'foo', encoding='ascii')

    # 0-terminated, no pad:
    self.assertBuilderEquals(b, [3, 0, 0, 0, 'f', 'o', 'o', 0])
    b.CreateSharedString(u'moop', encoding='ascii')
    b.CreateSharedString(u'moop', encoding='ascii')
    # 0-terminated, 3-byte pad:
    self.assertBuilderEquals(b, [
        4, 0, 0, 0, 'm', 'o', 'o', 'p', 0, 0, 0, 0, 3, 0, 0, 0, 'f', 'o', 'o', 0
    ])

  def test_create_utf8_shared_string(self):
    b = flatbuffers.Builder(0)
    b.CreateSharedString(u'Цлїςσδε')
    b.CreateSharedString(u'Цлїςσδε')
    self.assertBuilderEquals(b, '\x0e\x00\x00\x00\xd0\xa6\xd0\xbb\xd1\x97' \
        '\xcf\x82\xcf\x83\xce\xb4\xce\xb5\x00\x00')

    b.CreateSharedString(u'ﾌﾑｱﾑｶﾓｹﾓ')
    b.CreateSharedString(u'ﾌﾑｱﾑｶﾓｹﾓ')
    self.assertBuilderEquals(b, '\x18\x00\x00\x00\xef\xbe\x8c\xef\xbe\x91' \
        '\xef\xbd\xb1\xef\xbe\x91\xef\xbd\xb6\xef\xbe\x93\xef\xbd\xb9\xef' \
        '\xbe\x93\x00\x00\x00\x00\x0e\x00\x00\x00\xd0\xa6\xd0\xbb\xd1\x97' \
        '\xcf\x82\xcf\x83\xce\xb4\xce\xb5\x00\x00')

  def test_create_arbitrary_shared_string(self):
    b = flatbuffers.Builder(0)
    s = '\x01\x02\x03'
    b.CreateSharedString(s)  # Default encoding is utf-8.
    b.CreateSharedString(s)
    # 0-terminated, no pad:
    self.assertBuilderEquals(b, [3, 0, 0, 0, 1, 2, 3, 0])
    s2 = '\x04\x05\x06\x07'
    b.CreateSharedString(s2)  # Default encoding is utf-8.
    b.CreateSharedString(s2)
    # 0-terminated, 3-byte pad:
    self.assertBuilderEquals(
        b, [4, 0, 0, 0, 4, 5, 6, 7, 0, 0, 0, 0, 3, 0, 0, 0, 1, 2, 3, 0])

  def test_create_ascii_string(self):
    b = flatbuffers.Builder(0)
    b.CreateString(u'foo', encoding='ascii')

    # 0-terminated, no pad:
    self.assertBuilderEquals(b, [3, 0, 0, 0, 'f', 'o', 'o', 0])
    b.CreateString(u'moop', encoding='ascii')
    # 0-terminated, 3-byte pad:
    self.assertBuilderEquals(b, [
        4, 0, 0, 0, 'm', 'o', 'o', 'p', 0, 0, 0, 0, 3, 0, 0, 0, 'f', 'o', 'o', 0
    ])

  def test_create_utf8_string(self):
    b = flatbuffers.Builder(0)
    b.CreateString(u'Цлїςσδε')
    self.assertBuilderEquals(b, '\x0e\x00\x00\x00\xd0\xa6\xd0\xbb\xd1\x97' \
        '\xcf\x82\xcf\x83\xce\xb4\xce\xb5\x00\x00')

    b.CreateString(u'ﾌﾑｱﾑｶﾓｹﾓ')
    self.assertBuilderEquals(b, '\x18\x00\x00\x00\xef\xbe\x8c\xef\xbe\x91' \
        '\xef\xbd\xb1\xef\xbe\x91\xef\xbd\xb6\xef\xbe\x93\xef\xbd\xb9\xef' \
        '\xbe\x93\x00\x00\x00\x00\x0e\x00\x00\x00\xd0\xa6\xd0\xbb\xd1\x97' \
        '\xcf\x82\xcf\x83\xce\xb4\xce\xb5\x00\x00')

  def test_create_arbitrary_string(self):
    b = flatbuffers.Builder(0)
    s = '\x01\x02\x03'
    b.CreateString(s)  # Default encoding is utf-8.
    # 0-terminated, no pad:
    self.assertBuilderEquals(b, [3, 0, 0, 0, 1, 2, 3, 0])
    s2 = '\x04\x05\x06\x07'
    b.CreateString(s2)  # Default encoding is utf-8.
    # 0-terminated, 3-byte pad:
    self.assertBuilderEquals(
        b, [4, 0, 0, 0, 4, 5, 6, 7, 0, 0, 0, 0, 3, 0, 0, 0, 1, 2, 3, 0])

  def test_create_byte_vector(self):
    b = flatbuffers.Builder(0)
    b.CreateByteVector(b'')
    # 0-byte pad:
    self.assertBuilderEquals(b, [0, 0, 0, 0])

    b = flatbuffers.Builder(0)
    b.CreateByteVector(b'\x01\x02\x03')
    # 1-byte pad:
    self.assertBuilderEquals(b, [3, 0, 0, 0, 1, 2, 3, 0])

  def test_create_numpy_vector_int8(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Systems endian:
      b = flatbuffers.Builder(0)
      x = np.array([1, 2, -3], dtype=np.int8)
      b.CreateNumpyVector(x)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              2,
              256 - 3,
              0  # vector value + padding
          ])

      # Reverse endian:
      b = flatbuffers.Builder(0)
      x_other_endian = x.byteswap().newbyteorder()
      b.CreateNumpyVector(x_other_endian)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              2,
              256 - 3,
              0  # vector value + padding
          ])
    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_uint16(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Systems endian:
      b = flatbuffers.Builder(0)
      x = np.array([1, 2, 312], dtype=np.uint16)
      b.CreateNumpyVector(x)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              0,  # 1
              2,
              0,  # 2
              312 - 256,
              1,  # 312
              0,
              0  # padding
          ])

      # Reverse endian:
      b = flatbuffers.Builder(0)
      x_other_endian = x.byteswap().newbyteorder()
      b.CreateNumpyVector(x_other_endian)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              0,  # 1
              2,
              0,  # 2
              312 - 256,
              1,  # 312
              0,
              0  # padding
          ])
    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_int64(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Systems endian:
      b = flatbuffers.Builder(0)
      x = np.array([1, 2, -12], dtype=np.int64)
      b.CreateNumpyVector(x)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              0,
              0,
              0,
              0,
              0,
              0,
              0,  # 1
              2,
              0,
              0,
              0,
              0,
              0,
              0,
              0,  # 2
              256 - 12,
              255,
              255,
              255,
              255,
              255,
              255,
              255  # -12
          ])

      # Reverse endian:
      b = flatbuffers.Builder(0)
      x_other_endian = x.byteswap().newbyteorder()
      b.CreateNumpyVector(x_other_endian)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              0,
              0,
              0,
              0,
              0,
              0,
              0,  # 1
              2,
              0,
              0,
              0,
              0,
              0,
              0,
              0,  # 2
              256 - 12,
              255,
              255,
              255,
              255,
              255,
              255,
              255  # -12
          ])

    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_float32(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Systems endian:
      b = flatbuffers.Builder(0)
      x = np.array([1, 2, -12], dtype=np.float32)
      b.CreateNumpyVector(x)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              0,
              0,
              128,
              63,  # 1
              0,
              0,
              0,
              64,  # 2
              0,
              0,
              64,
              193  # -12
          ])

      # Reverse endian:
      b = flatbuffers.Builder(0)
      x_other_endian = x.byteswap().newbyteorder()
      b.CreateNumpyVector(x_other_endian)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              0,
              0,
              128,
              63,  # 1
              0,
              0,
              0,
              64,  # 2
              0,
              0,
              64,
              193  # -12
          ])

    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_float64(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Systems endian:
      b = flatbuffers.Builder(0)
      x = np.array([1, 2, -12], dtype=np.float64)
      b.CreateNumpyVector(x)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              0,
              0,
              0,
              0,
              0,
              0,
              240,
              63,  # 1
              0,
              0,
              0,
              0,
              0,
              0,
              0,
              64,  # 2
              0,
              0,
              0,
              0,
              0,
              0,
              40,
              192  # -12
          ])

      # Reverse endian:
      b = flatbuffers.Builder(0)
      x_other_endian = x.byteswap().newbyteorder()
      b.CreateNumpyVector(x_other_endian)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              0,
              0,
              0,
              0,
              0,
              0,
              240,
              63,  # 1
              0,
              0,
              0,
              0,
              0,
              0,
              0,
              64,  # 2
              0,
              0,
              0,
              0,
              0,
              0,
              40,
              192  # -12
          ])

    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_bool(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Systems endian:
      b = flatbuffers.Builder(0)
      x = np.array([True, False, True], dtype=bool)
      b.CreateNumpyVector(x)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              0,
              1,
              0  # vector values + padding
          ])

      # Reverse endian:
      b = flatbuffers.Builder(0)
      x_other_endian = x.byteswap().newbyteorder()
      b.CreateNumpyVector(x_other_endian)
      self.assertBuilderEquals(
          b,
          [
              3,
              0,
              0,
              0,  # vector length
              1,
              0,
              1,
              0  # vector values + padding
          ])

    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_reject_strings(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Create String array
      b = flatbuffers.Builder(0)
      x = np.array(['hello', 'fb', 'testing'])
      assertRaises(self, lambda: b.CreateNumpyVector(x), TypeError)

    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_create_numpy_vector_reject_object(self):
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      # Create String array
      b = flatbuffers.Builder(0)
      x = np.array([{'m': 0}, {'as': -2.1, 'c': 'c'}])
      assertRaises(self, lambda: b.CreateNumpyVector(x), TypeError)

    except ImportError:
      b = flatbuffers.Builder(0)
      x = 0
      assertRaises(self, lambda: b.CreateNumpyVector(x),
                   NumpyRequiredForThisFeature)

  def test_empty_vtable(self):
    b = flatbuffers.Builder(0)
    b.StartObject(0)
    self.assertBuilderEquals(b, [])
    b.EndObject()
    self.assertBuilderEquals(b, [4, 0, 4, 0, 4, 0, 0, 0])

  def test_vtable_with_one_true_bool(self):
    b = flatbuffers.Builder(0)
    self.assertBuilderEquals(b, [])
    b.StartObject(1)
    self.assertBuilderEquals(b, [])
    b.PrependBoolSlot(0, True, False)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            6,
            0,  # vtable bytes
            8,
            0,  # length of object including vtable offset
            7,
            0,  # start of bool value
            6,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0,
            0,
            0,  # padded to 4 bytes
            1,  # bool value
        ])

  def test_vtable_with_one_default_bool(self):
    b = flatbuffers.Builder(0)
    self.assertBuilderEquals(b, [])
    b.StartObject(1)
    self.assertBuilderEquals(b, [])
    b.PrependBoolSlot(0, False, False)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            4,
            0,  # vtable bytes
            4,
            0,  # end of object from here
            # entry 1 is zero and not stored
            4,
            0,
            0,
            0,  # offset for start of vtable (int32)
        ])

  def test_vtable_with_one_int16(self):
    b = flatbuffers.Builder(0)
    b.StartObject(1)
    b.PrependInt16Slot(0, 0x789A, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            6,
            0,  # vtable bytes
            8,
            0,  # end of object from here
            6,
            0,  # offset to value
            6,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0,
            0,  # padding to 4 bytes
            0x9A,
            0x78,
        ])

  def test_vtable_with_two_int16(self):
    b = flatbuffers.Builder(0)
    b.StartObject(2)
    b.PrependInt16Slot(0, 0x3456, 0)
    b.PrependInt16Slot(1, 0x789A, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            8,
            0,  # vtable bytes
            8,
            0,  # end of object from here
            6,
            0,  # offset to value 0
            4,
            0,  # offset to value 1
            8,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0x9A,
            0x78,  # value 1
            0x56,
            0x34,  # value 0
        ])

  def test_vtable_with_int16_and_bool(self):
    b = flatbuffers.Builder(0)
    b.StartObject(2)
    b.PrependInt16Slot(0, 0x3456, 0)
    b.PrependBoolSlot(1, True, False)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            8,
            0,  # vtable bytes
            8,
            0,  # end of object from here
            6,
            0,  # offset to value 0
            5,
            0,  # offset to value 1
            8,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0,  # padding
            1,  # value 1
            0x56,
            0x34,  # value 0
        ])

  def test_vtable_with_empty_vector(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Uint8Flags.bytewidth, 0, 1)
    vecend = b.EndVector()
    b.StartObject(1)
    b.PrependUOffsetTRelativeSlot(0, vecend, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            6,
            0,  # vtable bytes
            8,
            0,
            4,
            0,  # offset to vector offset
            6,
            0,
            0,
            0,  # offset for start of vtable (int32)
            4,
            0,
            0,
            0,
            0,
            0,
            0,
            0,  # length of vector (not in struct)
        ])

  def test_vtable_with_empty_vector_of_byte_and_some_scalars(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Uint8Flags.bytewidth, 0, 1)
    vecend = b.EndVector()
    b.StartObject(2)
    b.PrependInt16Slot(0, 55, 0)
    b.PrependUOffsetTRelativeSlot(1, vecend, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            8,
            0,  # vtable bytes
            12,
            0,
            10,
            0,  # offset to value 0
            4,
            0,  # offset to vector offset
            8,
            0,
            0,
            0,  # vtable loc
            8,
            0,
            0,
            0,  # value 1
            0,
            0,
            55,
            0,  # value 0
            0,
            0,
            0,
            0,  # length of vector (not in struct)
        ])

  def test_vtable_with_1_int16_and_2vector_of_int16(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Int16Flags.bytewidth, 2, 1)
    b.PrependInt16(0x1234)
    b.PrependInt16(0x5678)
    vecend = b.EndVector()
    b.StartObject(2)
    b.PrependUOffsetTRelativeSlot(1, vecend, 0)
    b.PrependInt16Slot(0, 55, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            8,
            0,  # vtable bytes
            12,
            0,  # length of object
            6,
            0,  # start of value 0 from end of vtable
            8,
            0,  # start of value 1 from end of buffer
            8,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0,
            0,  # padding
            55,
            0,  # value 0
            4,
            0,
            0,
            0,  # vector position from here
            2,
            0,
            0,
            0,  # length of vector (uint32)
            0x78,
            0x56,  # vector value 1
            0x34,
            0x12,  # vector value 0
        ])

  def test_vtable_with_1_struct_of_1_int8__1_int16__1_int32(self):
    b = flatbuffers.Builder(0)
    b.StartObject(1)
    b.Prep(4 + 4 + 4, 0)
    b.PrependInt8(55)
    b.Pad(3)
    b.PrependInt16(0x1234)
    b.Pad(2)
    b.PrependInt32(0x12345678)
    structStart = b.Offset()
    b.PrependStructSlot(0, structStart, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            6,
            0,  # vtable bytes
            16,
            0,  # end of object from here
            4,
            0,  # start of struct from here
            6,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0x78,
            0x56,
            0x34,
            0x12,  # value 2
            0,
            0,  # padding
            0x34,
            0x12,  # value 1
            0,
            0,
            0,  # padding
            55,  # value 0
        ])

  def test_vtable_with_1_vector_of_2_struct_of_2_int8(self):
    b = flatbuffers.Builder(0)
    b.StartVector(flatbuffers.number_types.Int8Flags.bytewidth * 2, 2, 1)
    b.PrependInt8(33)
    b.PrependInt8(44)
    b.PrependInt8(55)
    b.PrependInt8(66)
    vecend = b.EndVector()
    b.StartObject(1)
    b.PrependUOffsetTRelativeSlot(0, vecend, 0)
    b.EndObject()
    self.assertBuilderEquals(
        b,
        [
            6,
            0,  # vtable bytes
            8,
            0,
            4,
            0,  # offset of vector offset
            6,
            0,
            0,
            0,  # offset for start of vtable (int32)
            4,
            0,
            0,
            0,  # vector start offset
            2,
            0,
            0,
            0,  # vector length
            66,  # vector value 1,1
            55,  # vector value 1,0
            44,  # vector value 0,1
            33,  # vector value 0,0
        ])

  def test_table_with_some_elements(self):
    b = flatbuffers.Builder(0)
    b.StartObject(2)
    b.PrependInt8Slot(0, 33, 0)
    b.PrependInt16Slot(1, 66, 0)
    off = b.EndObject()
    b.Finish(off)

    self.assertBuilderEquals(
        b,
        [
            12,
            0,
            0,
            0,  # root of table: points to vtable offset
            8,
            0,  # vtable bytes
            8,
            0,  # end of object from here
            7,
            0,  # start of value 0
            4,
            0,  # start of value 1
            8,
            0,
            0,
            0,  # offset for start of vtable (int32)
            66,
            0,  # value 1
            0,  # padding
            33,  # value 0
        ])

  def test__one_unfinished_table_and_one_finished_table(self):
    b = flatbuffers.Builder(0)
    b.StartObject(2)
    b.PrependInt8Slot(0, 33, 0)
    b.PrependInt8Slot(1, 44, 0)
    off = b.EndObject()
    b.Finish(off)

    b.StartObject(3)
    b.PrependInt8Slot(0, 55, 0)
    b.PrependInt8Slot(1, 66, 0)
    b.PrependInt8Slot(2, 77, 0)
    off = b.EndObject()
    b.Finish(off)

    self.assertBuilderEquals(
        b,
        [
            16,
            0,
            0,
            0,  # root of table: points to object
            0,
            0,  # padding
            10,
            0,  # vtable bytes
            8,
            0,  # size of object
            7,
            0,  # start of value 0
            6,
            0,  # start of value 1
            5,
            0,  # start of value 2
            10,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0,  # padding
            77,  # value 2
            66,  # value 1
            55,  # value 0
            12,
            0,
            0,
            0,  # root of table: points to object
            8,
            0,  # vtable bytes
            8,
            0,  # size of object
            7,
            0,  # start of value 0
            6,
            0,  # start of value 1
            8,
            0,
            0,
            0,  # offset for start of vtable (int32)
            0,
            0,  # padding
            44,  # value 1
            33,  # value 0
        ])

  def test_a_bunch_of_bools(self):
    b = flatbuffers.Builder(0)
    b.StartObject(8)
    b.PrependBoolSlot(0, True, False)
    b.PrependBoolSlot(1, True, False)
    b.PrependBoolSlot(2, True, False)
    b.PrependBoolSlot(3, True, False)
    b.PrependBoolSlot(4, True, False)
    b.PrependBoolSlot(5, True, False)
    b.PrependBoolSlot(6, True, False)
    b.PrependBoolSlot(7, True, False)
    off = b.EndObject()
    b.Finish(off)

    self.assertBuilderEquals(
        b,
        [
            24,
            0,
            0,
            0,  # root of table: points to vtable offset
            20,
            0,  # vtable bytes
            12,
            0,  # size of object
            11,
            0,  # start of value 0
            10,
            0,  # start of value 1
            9,
            0,  # start of value 2
            8,
            0,  # start of value 3
            7,
            0,  # start of value 4
            6,
            0,  # start of value 5
            5,
            0,  # start of value 6
            4,
            0,  # start of value 7
            20,
            0,
            0,
            0,  # vtable offset
            1,  # value 7
            1,  # value 6
            1,  # value 5
            1,  # value 4
            1,  # value 3
            1,  # value 2
            1,  # value 1
            1,  # value 0
        ])

  def test_three_bools(self):
    b = flatbuffers.Builder(0)
    b.StartObject(3)
    b.PrependBoolSlot(0, True, False)
    b.PrependBoolSlot(1, True, False)
    b.PrependBoolSlot(2, True, False)
    off = b.EndObject()
    b.Finish(off)

    self.assertBuilderEquals(
        b,
        [
            16,
            0,
            0,
            0,  # root of table: points to vtable offset
            0,
            0,  # padding
            10,
            0,  # vtable bytes
            8,
            0,  # size of object
            7,
            0,  # start of value 0
            6,
            0,  # start of value 1
            5,
            0,  # start of value 2
            10,
            0,
            0,
            0,  # vtable offset from here
            0,  # padding
            1,  # value 2
            1,  # value 1
            1,  # value 0
        ])

  def test_some_floats(self):
    b = flatbuffers.Builder(0)
    b.StartObject(1)
    b.PrependFloat32Slot(0, 1.0, 0.0)
    off = b.EndObject()

    self.assertBuilderEquals(
        b,
        [
            6,
            0,  # vtable bytes
            8,
            0,  # size of object
            4,
            0,  # start of value 0
            6,
            0,
            0,
            0,  # vtable offset
            0,
            0,
            128,
            63,  # value 0
        ])


def make_monster_from_generated_code(b=None, sizePrefix=False, file_identifier=None):
  """ Use generated code to build the example Monster. """
  if b is None:
    b = flatbuffers.Builder(0)
  string = b.CreateString('MyMonster')
  test1 = b.CreateString('test1')
  test2 = b.CreateString('test2')
  fred = b.CreateString('Fred')

  _MONSTER.MonsterStartInventoryVector(b, 5)
  b.PrependByte(4)
  b.PrependByte(3)
  b.PrependByte(2)
  b.PrependByte(1)
  b.PrependByte(0)
  inv = b.EndVector()

  _MONSTER.MonsterStart(b)
  _MONSTER.MonsterAddName(b, fred)
  mon2 = _MONSTER.MonsterEnd(b)

  _MONSTER.MonsterStartTest4Vector(b, 2)
  _TEST.CreateTest(b, 10, 20)
  _TEST.CreateTest(b, 30, 40)
  test4 = b.EndVector()

  _MONSTER.MonsterStartTestarrayofstringVector(b, 2)
  b.PrependUOffsetTRelative(test2)
  b.PrependUOffsetTRelative(test1)
  testArrayOfString = b.EndVector()

  _MONSTER.MonsterStartVectorOfLongsVector(b, 5)
  b.PrependInt64(100000000)
  b.PrependInt64(1000000)
  b.PrependInt64(10000)
  b.PrependInt64(100)
  b.PrependInt64(1)
  VectorOfLongs = b.EndVector()

  _MONSTER.MonsterStartVectorOfDoublesVector(b, 3)
  b.PrependFloat64(1.7976931348623157e+308)
  b.PrependFloat64(0)
  b.PrependFloat64(-1.7976931348623157e+308)
  VectorOfDoubles = b.EndVector()

  _MONSTER.MonsterStart(b)

  pos = _VEC3.CreateVec3(b, 1.0, 2.0, 3.0, 3.0, 2, 5, 6)
  _MONSTER.MonsterAddPos(b, pos)

  _MONSTER.MonsterAddHp(b, 80)
  _MONSTER.MonsterAddName(b, string)
  _MONSTER.MonsterAddInventory(b, inv)
  _MONSTER.MonsterAddTestType(b, 1)
  _MONSTER.MonsterAddTest(b, mon2)
  _MONSTER.MonsterAddTest4(b, test4)
  _MONSTER.MonsterAddTestarrayofstring(b, testArrayOfString)
  _MONSTER.MonsterAddVectorOfLongs(b, VectorOfLongs)
  _MONSTER.MonsterAddVectorOfDoubles(b, VectorOfDoubles)
  mon = _MONSTER.MonsterEnd(b)

  if sizePrefix:
    b.FinishSizePrefixed(mon, file_identifier)
  else:
    b.Finish(mon, file_identifier)

  return b.Bytes, b.Head()


class TestBuilderForceDefaults(unittest.TestCase):
  """Verify that the builder adds default values when forced."""

  test_flags = [N.BoolFlags(), N.Uint8Flags(), N.Uint16Flags(), \
                N.Uint32Flags(), N.Uint64Flags(), N.Int8Flags(), \
                N.Int16Flags(), N.Int32Flags(), N.Int64Flags(), \
                N.Float32Flags(), N.Float64Flags(), N.UOffsetTFlags()]

  def test_default_force_defaults(self):
    for flag in self.test_flags:
      b = flatbuffers.Builder(0)
      b.StartObject(1)
      stored_offset = b.Offset()
      if flag != N.UOffsetTFlags():
        b.PrependSlot(flag, 0, 0, 0)
      else:
        b.PrependUOffsetTRelativeSlot(0, 0, 0)
      end_offset = b.Offset()
      b.EndObject()
      self.assertEqual(0, end_offset - stored_offset)

  def test_force_defaults_true(self):
    for flag in self.test_flags:
      b = flatbuffers.Builder(0)
      b.ForceDefaults(True)
      b.StartObject(1)
      stored_offset = b.Offset()
      if flag != N.UOffsetTFlags():
        b.PrependSlot(flag, 0, 0, 0)
      else:
        b.PrependUOffsetTRelativeSlot(0, 0, 0)
      end_offset = b.Offset()
      b.EndObject()
      self.assertEqual(flag.bytewidth, end_offset - stored_offset)


class TestAllCodePathsOfExampleSchema(unittest.TestCase):

  def setUp(self, *args, **kwargs):
    super(TestAllCodePathsOfExampleSchema, self).setUp(*args, **kwargs)

    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStart(b)
    gen_mon = _MONSTER.MonsterEnd(b)
    b.Finish(gen_mon)

    self.mon = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())

  def test_default_monster_pos(self):
    self.assertTrue(self.mon.Pos() is None)

  def test_nondefault_monster_mana(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddMana(b, 50)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    got_mon = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertEqual(50, got_mon.Mana())

  def test_default_monster_hp(self):
    self.assertEqual(100, self.mon.Hp())

  def test_default_monster_name(self):
    self.assertEqual(None, self.mon.Name())

  def test_default_monster_inventory_item(self):
    self.assertEqual(0, self.mon.Inventory(0))

  def test_default_monster_inventory_length(self):
    self.assertEqual(0, self.mon.InventoryLength())
    self.assertTrue(self.mon.InventoryIsNone())

  def test_empty_monster_inventory_vector(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStartInventoryVector(b, 0)
    inv = b.EndVector()
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddInventory(b, inv)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertFalse(mon2.InventoryIsNone())

  def test_default_monster_color(self):
    self.assertEqual(_COLOR.Color.Blue, self.mon.Color())

  def test_nondefault_monster_color(self):
    b = flatbuffers.Builder(0)
    color = _COLOR.Color.Red
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddColor(b, color)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertEqual(_COLOR.Color.Red, mon2.Color())

  def test_default_monster_testtype(self):
    self.assertEqual(0, self.mon.TestType())

  def test_default_monster_test_field(self):
    self.assertEqual(None, self.mon.Test())

  def test_default_monster_test4_item(self):
    self.assertEqual(None, self.mon.Test4(0))

  def test_default_monster_test4_length(self):
    self.assertEqual(0, self.mon.Test4Length())
    self.assertTrue(self.mon.Test4IsNone())

  def test_empty_monster_test4_vector(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStartTest4Vector(b, 0)
    test4 = b.EndVector()
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTest4(b, test4)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertFalse(mon2.Test4IsNone())

  def test_default_monster_testarrayofstring(self):
    self.assertEqual('', self.mon.Testarrayofstring(0))

  def test_default_monster_testarrayofstring_length(self):
    self.assertEqual(0, self.mon.TestarrayofstringLength())
    self.assertTrue(self.mon.TestarrayofstringIsNone())

  def test_empty_monster_testarrayofstring_vector(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStartTestarrayofstringVector(b, 0)
    testarrayofstring = b.EndVector()
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestarrayofstring(b, testarrayofstring)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertFalse(mon2.TestarrayofstringIsNone())

  def test_default_monster_testarrayoftables(self):
    self.assertEqual(None, self.mon.Testarrayoftables(0))

  def test_nondefault_monster_testarrayoftables(self):
    b = flatbuffers.Builder(0)

    # make a child Monster within a vector of Monsters:
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddHp(b, 99)
    sub_monster = _MONSTER.MonsterEnd(b)

    # build the vector:
    _MONSTER.MonsterStartTestarrayoftablesVector(b, 1)
    b.PrependUOffsetTRelative(sub_monster)
    vec = b.EndVector()

    # make the parent monster and include the vector of Monster:
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestarrayoftables(b, vec)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Output(), 0)
    self.assertEqual(99, mon2.Testarrayoftables(0).Hp())
    self.assertEqual(1, mon2.TestarrayoftablesLength())
    self.assertFalse(mon2.TestarrayoftablesIsNone())

  def test_default_monster_testarrayoftables_length(self):
    self.assertEqual(0, self.mon.TestarrayoftablesLength())
    self.assertTrue(self.mon.TestarrayoftablesIsNone())

  def test_empty_monster_testarrayoftables_vector(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStartTestarrayoftablesVector(b, 0)
    testarrayoftables = b.EndVector()
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestarrayoftables(b, testarrayoftables)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertFalse(mon2.TestarrayoftablesIsNone())

  def test_default_monster_testarrayoftables_length(self):
    self.assertEqual(0, self.mon.TestarrayoftablesLength())

  def test_nondefault_monster_enemy(self):
    b = flatbuffers.Builder(0)

    # make an Enemy object:
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddHp(b, 88)
    enemy = _MONSTER.MonsterEnd(b)
    b.Finish(enemy)

    # make the parent monster and include the vector of Monster:
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddEnemy(b, enemy)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertEqual(88, mon2.Enemy().Hp())

  def test_default_monster_testnestedflatbuffer(self):
    self.assertEqual(0, self.mon.Testnestedflatbuffer(0))

  def test_default_monster_testnestedflatbuffer_length(self):
    self.assertEqual(0, self.mon.TestnestedflatbufferLength())
    self.assertTrue(self.mon.TestnestedflatbufferIsNone())

  def test_empty_monster_testnestedflatbuffer_vector(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStartTestnestedflatbufferVector(b, 0)
    testnestedflatbuffer = b.EndVector()
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestnestedflatbuffer(b, testnestedflatbuffer)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertFalse(mon2.TestnestedflatbufferIsNone())

  def test_nondefault_monster_testnestedflatbuffer(self):
    b = flatbuffers.Builder(0)

    _MONSTER.MonsterStartTestnestedflatbufferVector(b, 3)
    b.PrependByte(4)
    b.PrependByte(2)
    b.PrependByte(0)
    sub_buf = b.EndVector()

    # make the parent monster and include the vector of Monster:
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestnestedflatbuffer(b, sub_buf)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertEqual(3, mon2.TestnestedflatbufferLength())
    self.assertFalse(mon2.TestnestedflatbufferIsNone())
    self.assertEqual(0, mon2.Testnestedflatbuffer(0))
    self.assertEqual(2, mon2.Testnestedflatbuffer(1))
    self.assertEqual(4, mon2.Testnestedflatbuffer(2))
    try:
      # if numpy exists, then we should be able to get the
      # vector as a numpy array
      import numpy as np

      self.assertEqual([0, 2, 4], mon2.TestnestedflatbufferAsNumpy().tolist())
    except ImportError:
      assertRaises(self, lambda: mon2.TestnestedflatbufferAsNumpy(),
                   NumpyRequiredForThisFeature)

  def test_nested_monster_testnestedflatbuffer(self):
    b = flatbuffers.Builder(0)

    # build another monster to nest inside testnestedflatbuffer
    nestedB = flatbuffers.Builder(0)
    nameStr = nestedB.CreateString('Nested Monster')
    _MONSTER.MonsterStart(nestedB)
    _MONSTER.MonsterAddHp(nestedB, 30)
    _MONSTER.MonsterAddName(nestedB, nameStr)
    nestedMon = _MONSTER.MonsterEnd(nestedB)
    nestedB.Finish(nestedMon)

    # write the nested FB bytes
    sub_buf = _MONSTER.MonsterMakeTestnestedflatbufferVectorFromBytes(
        b, nestedB.Output())

    # make the parent monster and include the bytes of the nested monster
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestnestedflatbuffer(b, sub_buf)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    nestedMon2 = mon2.TestnestedflatbufferNestedRoot()
    self.assertEqual(b'Nested Monster', nestedMon2.Name())
    self.assertEqual(30, nestedMon2.Hp())

  def test_nondefault_monster_testempty(self):
    b = flatbuffers.Builder(0)

    # make a Stat object:
    _STAT.StatStart(b)
    _STAT.StatAddVal(b, 123)
    my_stat = _STAT.StatEnd(b)
    b.Finish(my_stat)

    # include the stat object in a monster:
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestempty(b, my_stat)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertEqual(123, mon2.Testempty().Val())

  def test_default_monster_testbool(self):
    self.assertFalse(self.mon.Testbool())

  def test_nondefault_monster_testbool(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTestbool(b, True)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertTrue(mon2.Testbool())

  def test_default_monster_testhashes(self):
    self.assertEqual(0, self.mon.Testhashs32Fnv1())
    self.assertEqual(0, self.mon.Testhashu32Fnv1())
    self.assertEqual(0, self.mon.Testhashs64Fnv1())
    self.assertEqual(0, self.mon.Testhashu64Fnv1())
    self.assertEqual(0, self.mon.Testhashs32Fnv1a())
    self.assertEqual(0, self.mon.Testhashu32Fnv1a())
    self.assertEqual(0, self.mon.Testhashs64Fnv1a())
    self.assertEqual(0, self.mon.Testhashu64Fnv1a())

  def test_nondefault_monster_testhashes(self):
    b = flatbuffers.Builder(0)
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddTesthashs32Fnv1(b, 1)
    _MONSTER.MonsterAddTesthashu32Fnv1(b, 2)
    _MONSTER.MonsterAddTesthashs64Fnv1(b, 3)
    _MONSTER.MonsterAddTesthashu64Fnv1(b, 4)
    _MONSTER.MonsterAddTesthashs32Fnv1a(b, 5)
    _MONSTER.MonsterAddTesthashu32Fnv1a(b, 6)
    _MONSTER.MonsterAddTesthashs64Fnv1a(b, 7)
    _MONSTER.MonsterAddTesthashu64Fnv1a(b, 8)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # inspect the resulting data:
    mon2 = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertEqual(1, mon2.Testhashs32Fnv1())
    self.assertEqual(2, mon2.Testhashu32Fnv1())
    self.assertEqual(3, mon2.Testhashs64Fnv1())
    self.assertEqual(4, mon2.Testhashu64Fnv1())
    self.assertEqual(5, mon2.Testhashs32Fnv1a())
    self.assertEqual(6, mon2.Testhashu32Fnv1a())
    self.assertEqual(7, mon2.Testhashs64Fnv1a())
    self.assertEqual(8, mon2.Testhashu64Fnv1a())

  def test_default_monster_parent_namespace_test(self):
    self.assertEqual(None, self.mon.ParentNamespaceTest())

  def test_nondefault_monster_parent_namespace_test(self):
    b = flatbuffers.Builder(0)
    _IN_PARENT_NAMESPACE.InParentNamespaceStart(b)
    parent = _IN_PARENT_NAMESPACE.InParentNamespaceEnd(b)
    _MONSTER.MonsterStart(b)
    _MONSTER.MonsterAddParentNamespaceTest(b, parent)
    mon = _MONSTER.MonsterEnd(b)
    b.Finish(mon)

    # Inspect the resulting data.
    monster = _MONSTER.Monster.GetRootAs(b.Bytes, b.Head())
    self.assertTrue(
        isinstance(monster.ParentNamespaceTest(),
                   _IN_PARENT_NAMESPACE.InParentNamespace))

  def test_getrootas_for_nonroot_table(self):
    b = flatbuffers.Builder(0)
    string = b.CreateString('MyStat')

    _STAT.StatStart(b)
    _STAT.StatAddId(b, string)
    _STAT.StatAddVal(b, 12345678)
    _STAT.StatAddCount(b, 12345)
    stat = _STAT.StatEnd(b)
    b.Finish(stat)

    stat2 = _STAT.Stat.GetRootAs(b.Bytes, b.Head())

    self.assertEqual(b'MyStat', stat2.Id())
    self.assertEqual(12345678, stat2.Val())
    self.assertEqual(12345, stat2.Count())


class TestAllCodePathsOfMonsterExtraSchema(unittest.TestCase):

  def setUp(self, *args, **kwargs):
    super(TestAllCodePathsOfMonsterExtraSchema, self).setUp(*args, **kwargs)

    b = flatbuffers.Builder(0)
    MyGame.MonsterExtra.Start(b)
    gen_mon = MyGame.MonsterExtra.End(b)
    b.Finish(gen_mon)

    self.mon = MyGame.MonsterExtra.MonsterExtra.GetRootAs(b.Bytes, b.Head())

  def test_default_nan_inf(self):
    self.assertTrue(math.isnan(self.mon.F1()))
    self.assertEqual(self.mon.F2(), float('inf'))
    self.assertEqual(self.mon.F3(), float('-inf'))

    self.assertTrue(math.isnan(self.mon.D1()))
    self.assertEqual(self.mon.D2(), float('inf'))
    self.assertEqual(self.mon.D3(), float('-inf'))


class TestVtableDeduplication(unittest.TestCase):
  """ TestVtableDeduplication verifies that vtables are deduplicated. """

  def test_vtable_deduplication(self):
    b = flatbuffers.Builder(0)

    b.StartObject(4)
    b.PrependByteSlot(0, 0, 0)
    b.PrependByteSlot(1, 11, 0)
    b.PrependByteSlot(2, 22, 0)
    b.PrependInt16Slot(3, 33, 0)
    obj0 = b.EndObject()

    b.StartObject(4)
    b.PrependByteSlot(0, 0, 0)
    b.PrependByteSlot(1, 44, 0)
    b.PrependByteSlot(2, 55, 0)
    b.PrependInt16Slot(3, 66, 0)
    obj1 = b.EndObject()

    b.StartObject(4)
    b.PrependByteSlot(0, 0, 0)
    b.PrependByteSlot(1, 77, 0)
    b.PrependByteSlot(2, 88, 0)
    b.PrependInt16Slot(3, 99, 0)
    obj2 = b.EndObject()

    got = b.Bytes[b.Head():]

    want = bytearray([
        240,
        255,
        255,
        255,  # == -12. offset to dedupped vtable.
        99,
        0,
        88,
        77,
        248,
        255,
        255,
        255,  # == -8. offset to dedupped vtable.
        66,
        0,
        55,
        44,
        12,
        0,
        8,
        0,
        0,
        0,
        7,
        0,
        6,
        0,
        4,
        0,
        12,
        0,
        0,
        0,
        33,
        0,
        22,
        11,
    ])

    self.assertEqual((len(want), want), (len(got), got))

    table0 = flatbuffers.table.Table(b.Bytes, len(b.Bytes) - obj0)
    table1 = flatbuffers.table.Table(b.Bytes, len(b.Bytes) - obj1)
    table2 = flatbuffers.table.Table(b.Bytes, len(b.Bytes) - obj2)

    def _checkTable(tab, voffsett_value, b, c, d):
      # vtable size
      got = tab.GetVOffsetTSlot(0, 0)
      self.assertEqual(12, got, 'case 0, 0')

      # object size
      got = tab.GetVOffsetTSlot(2, 0)
      self.assertEqual(8, got, 'case 2, 0')

      # default value
      got = tab.GetVOffsetTSlot(4, 0)
      self.assertEqual(voffsett_value, got, 'case 4, 0')

      got = tab.GetSlot(6, 0, N.Uint8Flags)
      self.assertEqual(b, got, 'case 6, 0')

      val = tab.GetSlot(8, 0, N.Uint8Flags)
      self.assertEqual(c, val, 'failed 8, 0')

      got = tab.GetSlot(10, 0, N.Uint8Flags)
      self.assertEqual(d, got, 'failed 10, 0')

    _checkTable(table0, 0, 11, 22, 33)
    _checkTable(table1, 0, 44, 55, 66)
    _checkTable(table2, 0, 77, 88, 99)


class TestExceptions(unittest.TestCase):

  def test_object_is_nested_error(self):
    b = flatbuffers.Builder(0)
    b.StartObject(0)
    assertRaises(self, lambda: b.StartObject(0),
                 flatbuffers.builder.IsNestedError)

  def test_object_is_not_nested_error(self):
    b = flatbuffers.Builder(0)
    assertRaises(self, lambda: b.EndObject(),
                 flatbuffers.builder.IsNotNestedError)

  def test_struct_is_not_inline_error(self):
    b = flatbuffers.Builder(0)
    b.StartObject(0)
    assertRaises(self, lambda: b.PrependStructSlot(0, 1, 0),
                 flatbuffers.builder.StructIsNotInlineError)

  def test_unreachable_error(self):
    b = flatbuffers.Builder(0)
    assertRaises(self, lambda: b.PrependUOffsetTRelative(1),
                 flatbuffers.builder.OffsetArithmeticError)

  def test_create_shared_string_is_nested_error(self):
    b = flatbuffers.Builder(0)
    b.StartObject(0)
    s = 'test1'
    assertRaises(self, lambda: b.CreateSharedString(s),
                 flatbuffers.builder.IsNestedError)

  def test_create_string_is_nested_error(self):
    b = flatbuffers.Builder(0)
    b.StartObject(0)
    s = 'test1'
    assertRaises(self, lambda: b.CreateString(s),
                 flatbuffers.builder.IsNestedError)

  def test_create_byte_vector_is_nested_error(self):
    b = flatbuffers.Builder(0)
    b.StartObject(0)
    s = b'test1'
    assertRaises(self, lambda: b.CreateByteVector(s),
                 flatbuffers.builder.IsNestedError)

  def test_finished_bytes_error(self):
    b = flatbuffers.Builder(0)
    assertRaises(self, lambda: b.Output(),
                 flatbuffers.builder.BuilderNotFinishedError)


class TestFixedLengthArrays(unittest.TestCase):

  def test_fixed_length_array(self):
    builder = flatbuffers.Builder(0)

    a = 0.5
    b = range(0, 15)
    c = 1
    d_a = [[1, 2], [3, 4]]
    d_b = [MyGame.Example.TestEnum.TestEnum.B, \
            MyGame.Example.TestEnum.TestEnum.C]
    d_c = [[MyGame.Example.TestEnum.TestEnum.A, \
            MyGame.Example.TestEnum.TestEnum.B], \
            [MyGame.Example.TestEnum.TestEnum.C, \
             MyGame.Example.TestEnum.TestEnum.B]]
    d_d = [[-1, 1], [-2, 2]]
    e = 2
    f = [-1, 1]

    arrayOffset = MyGame.Example.ArrayStruct.CreateArrayStruct(builder, \
        a, b, c, d_a, d_b, d_c, d_d, e, f)

    # Create a table with the ArrayStruct.
    MyGame.Example.ArrayTable.Start(builder)
    MyGame.Example.ArrayTable.AddA(builder, arrayOffset)
    tableOffset = MyGame.Example.ArrayTable.End(builder)

    builder.Finish(tableOffset)

    buf = builder.Output()

    table = MyGame.Example.ArrayTable.ArrayTable.GetRootAs(buf)

    # Verify structure.
    nested = MyGame.Example.NestedStruct.NestedStruct()
    self.assertEqual(table.A().A(), 0.5)
    self.assertEqual(table.A().B(), \
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14])
    self.assertEqual(table.A().C(), 1)
    self.assertEqual(table.A().D(0).A(), [1, 2])
    self.assertEqual(table.A().D(1).A(), [3, 4])
    self.assertEqual(table.A().D(0).B(), \
        MyGame.Example.TestEnum.TestEnum.B)
    self.assertEqual(table.A().D(1).B(), \
        MyGame.Example.TestEnum.TestEnum.C)
    self.assertEqual(table.A().D(0).C(), \
        [MyGame.Example.TestEnum.TestEnum.A, \
         MyGame.Example.TestEnum.TestEnum.B])
    self.assertEqual(table.A().D(1).C(), \
        [MyGame.Example.TestEnum.TestEnum.C, \
         MyGame.Example.TestEnum.TestEnum.B])
    self.assertEqual(table.A().D(0).D(), [-1, 1])
    self.assertEqual(table.A().D(1).D(), [-2, 2])
    self.assertEqual(table.A().E(), 2)
    self.assertEqual(table.A().F(), [-1, 1])
    self.assertEqual(table.A().D(0).D(0), -1)
    self.assertEqual(table.A().D(0).D(1), 1)
    self.assertEqual(table.A().D(1).D(0), -2)
    self.assertEqual(table.A().D(1).D(1), 2)

class TestNestedUnionTables(unittest.TestCase):

  def test_nested_union_tables(self):
    nestUnion = MyGame.Example.NestedUnion.NestedUnionTest.NestedUnionTestT()
    nestUnion.name = b"testUnion1"
    nestUnion.id = 1
    nestUnion.data = MyGame.Example.NestedUnion.Vec3.Vec3T()
    nestUnion.dataType = MyGame.Example.NestedUnion.Any.Any.Vec3
    nestUnion.data.x = 4.278975356
    nestUnion.data.y = 5.32
    nestUnion.data.z = -6.464
    nestUnion.data.test1 = 0.9
    nestUnion.data.test2 = MyGame.Example.NestedUnion.Color.Color.Red
    nestUnion.data.test3 = MyGame.Example.NestedUnion.Test.TestT()
    nestUnion.data.test3.a = 5
    nestUnion.data.test3.b = 2

    b = flatbuffers.Builder(0)
    b.Finish(nestUnion.Pack(b))

    nestUnionDecode = MyGame.Example.NestedUnion.NestedUnionTest.NestedUnionTest.GetRootAs(b.Bytes, b.Head())
    nestUnionDecodeT = MyGame.Example.NestedUnion.NestedUnionTest.NestedUnionTestT.InitFromObj(nestUnionDecode)
    self.assertEqual(nestUnionDecodeT.name, nestUnion.name)
    self.assertEqual(nestUnionDecodeT.id, nestUnion.id)
    self.assertEqual(nestUnionDecodeT.dataType, nestUnion.dataType)
    self.assertEqual(nestUnionDecodeT.data.x, nestUnion.data.x)
    self.assertEqual(nestUnionDecodeT.data.y, nestUnion.data.y)
    self.assertEqual(nestUnionDecodeT.data.z, nestUnion.data.z)
    self.assertEqual(nestUnionDecodeT.data.test1, nestUnion.data.test1)
    self.assertEqual(nestUnionDecodeT.data.test2, nestUnion.data.test2)
    self.assertEqual(nestUnionDecodeT.data.test3.a, nestUnion.data.test3.a)
    self.assertEqual(nestUnionDecodeT.data.test3.b, nestUnion.data.test3.b)

    nestUnionDecodeTFromBuf = MyGame.Example.NestedUnion.NestedUnionTest.NestedUnionTestT.InitFromPackedBuf(b.Bytes, b.Head())
    self.assertEqual(nestUnionDecodeTFromBuf.name, nestUnion.name)
    self.assertEqual(nestUnionDecodeTFromBuf.id, nestUnion.id)
    self.assertEqual(nestUnionDecodeTFromBuf.dataType, nestUnion.dataType)
    self.assertEqual(nestUnionDecodeTFromBuf.data.x, nestUnion.data.x)
    self.assertEqual(nestUnionDecodeTFromBuf.data.y, nestUnion.data.y)
    self.assertEqual(nestUnionDecodeTFromBuf.data.z, nestUnion.data.z)
    self.assertEqual(nestUnionDecodeTFromBuf.data.test1, nestUnion.data.test1)
    self.assertEqual(nestUnionDecodeTFromBuf.data.test2, nestUnion.data.test2)
    self.assertEqual(nestUnionDecodeTFromBuf.data.test3.a, nestUnion.data.test3.a)
    self.assertEqual(nestUnionDecodeTFromBuf.data.test3.b, nestUnion.data.test3.b)


    nestUnionDecodeTFromBuf2 = MyGame.Example.NestedUnion.NestedUnionTest.NestedUnionTestT.InitFromPackedBuf(b.Output())
    self.assertEqual(nestUnionDecodeTFromBuf2.name, nestUnion.name)
    self.assertEqual(nestUnionDecodeTFromBuf2.id, nestUnion.id)
    self.assertEqual(nestUnionDecodeTFromBuf2.dataType, nestUnion.dataType)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.x, nestUnion.data.x)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.y, nestUnion.data.y)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.z, nestUnion.data.z)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.test1, nestUnion.data.test1)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.test2, nestUnion.data.test2)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.test3.a, nestUnion.data.test3.a)
    self.assertEqual(nestUnionDecodeTFromBuf2.data.test3.b, nestUnion.data.test3.b)


class TestBuilderClear(unittest.TestCase):

  def test_consistency(self):
    """ Checks if clear resets the state of the builder. """
    b = flatbuffers.Builder(0)

    # Add some data to the buffer
    off1 = b.CreateString('a' * 1024)
    want = b.Bytes[b.Head():]

    # Reset the builder
    b.Clear()

    # Readd the same data into the buffer
    off2 = b.CreateString('a' * 1024)
    got = b.Bytes[b.Head():]

    # Expect to get the same data into the buffer at the same offset
    self.assertEqual(off1, off2)
    self.assertEqual(want, got)

  def test_repeated_clear_after_builder_reuse(self):
    init_buf = None
    init_off = None
    b = flatbuffers.Builder(0)

    for i in range(5):
      buf, off = make_monster_from_generated_code(b)
      b.Clear()

      if i > 0:
        self.assertEqual(init_buf, buf)
        self.assertEqual(init_off, off)
      else:
        init_buf = buf
        init_off = off

def CheckAgainstGoldDataGo():
  try:
    gen_buf, gen_off = make_monster_from_generated_code()
    fn = 'monsterdata_go_wire.mon'
    if not os.path.exists(fn):
      print('Go-generated data does not exist, failed.')
      return False

    # would like to use a context manager here, but it's less
    # backwards-compatible:
    f = open(fn, 'rb')
    go_wire_data = f.read()
    f.close()

    CheckReadBuffer(bytearray(go_wire_data), 0)
    if not bytearray(gen_buf[gen_off:]) == bytearray(go_wire_data):
      raise AssertionError('CheckAgainstGoldDataGo failed')
  except:
    print('Failed to test against Go-generated test data.')
    return False

  print(
      'Can read Go-generated test data, and Python generates bytewise identical data.'
  )
  return True


def CheckAgainstGoldDataJava():
  try:
    gen_buf, gen_off = make_monster_from_generated_code()
    fn = 'monsterdata_java_wire.mon'
    if not os.path.exists(fn):
      print('Java-generated data does not exist, failed.')
      return False
    f = open(fn, 'rb')
    java_wire_data = f.read()
    f.close()

    CheckReadBuffer(bytearray(java_wire_data), 0)
  except:
    print('Failed to read Java-generated test data.')
    return False

  print('Can read Java-generated test data.')
  return True


class LCG(object):
  """ Include simple random number generator to ensure results will be the

        same cross platform.
        http://en.wikipedia.org/wiki/Park%E2%80%93Miller_random_number_generator
        """

  __slots__ = ['n']

  InitialLCGSeed = 48271

  def __init__(self):
    self.n = self.InitialLCGSeed

  def Reset(self):
    self.n = self.InitialLCGSeed

  def Next(self):
    self.n = ((self.n * 279470273) % 4294967291) & 0xFFFFFFFF
    return self.n


def BenchmarkVtableDeduplication(count):
  """
    BenchmarkVtableDeduplication measures the speed of vtable deduplication
    by creating `prePop` vtables, then populating `count` objects with a
    different single vtable.

    When count is large (as in long benchmarks), memory usage may be high.
    """

  for prePop in (1, 10, 100, 1000):
    builder = flatbuffers.Builder(0)
    n = 1 + int(math.log(prePop, 1.5))

    # generate some layouts:
    layouts = set()
    r = list(compat_range(n))
    while len(layouts) < prePop:
      layouts.add(tuple(sorted(random.sample(r, int(max(1, n / 2))))))

    layouts = list(layouts)

    # pre-populate vtables:
    for layout in layouts:
      builder.StartObject(n)
      for j in layout:
        builder.PrependInt16Slot(j, j, 0)
      builder.EndObject()

    # benchmark deduplication of a new vtable:
    def f():
      layout = random.choice(layouts)
      builder.StartObject(n)
      for j in layout:
        builder.PrependInt16Slot(j, j, 0)
      builder.EndObject()

    duration = timeit.timeit(stmt=f, number=count)
    rate = float(count) / duration
    print(('vtable deduplication rate (n=%d, vtables=%d): %.2f sec' %
           (prePop, len(builder.vtables), rate)))


def BenchmarkCheckReadBuffer(count, buf, off):
  """
    BenchmarkCheckReadBuffer measures the speed of flatbuffer reading
    by re-using the CheckReadBuffer function with the gold data.
    """

  def f():
    CheckReadBuffer(buf, off)

  duration = timeit.timeit(stmt=f, number=count)
  rate = float(count) / duration
  data = float(len(buf) * count) / float(1024 * 1024)
  data_rate = data / float(duration)

  print(('traversed %d %d-byte flatbuffers in %.2fsec: %.2f/sec, %.2fMB/sec') %
        (count, len(buf), duration, rate, data_rate))


def BenchmarkMakeMonsterFromGeneratedCode(count, length):
  """
    BenchmarkMakeMonsterFromGeneratedCode measures the speed of flatbuffer
    creation by re-using the make_monster_from_generated_code function for
    generating gold data examples.
    """

  duration = timeit.timeit(stmt=make_monster_from_generated_code, number=count)
  rate = float(count) / duration
  data = float(length * count) / float(1024 * 1024)
  data_rate = data / float(duration)

  print(('built %d %d-byte flatbuffers in %.2fsec: %.2f/sec, %.2fMB/sec' % \
         (count, length, duration, rate, data_rate)))


def BenchmarkBuilderClear(count, length):
  b = flatbuffers.Builder(length)
  duration = timeit.timeit(stmt=lambda: make_monster_from_generated_code(b),
                           number=count)
  rate = float(count) / duration
  data = float(length * count) / float(1024 * 1024)
  data_rate = data / float(duration)

  print(('built %d %d-byte flatbuffers (reused buffer) in %.2fsec:'
         ' %.2f/sec, %.2fMB/sec' % (count, length, duration, rate, data_rate)))


def backward_compatible_run_tests(**kwargs):
  if PY_VERSION < (2, 6):
    sys.stderr.write('Python version less than 2.6 are not supported')
    sys.stderr.flush()
    return False

  # python2.6 has a reduced-functionality unittest.main function:
  if PY_VERSION == (2, 6):
    try:
      unittest.main(**kwargs)
    except SystemExit as e:
      if not e.code == 0:
        return False
    return True

  # python2.7 and above let us not exit once unittest.main is run:
  kwargs['exit'] = False
  kwargs['verbosity'] = 0
  ret = unittest.main(**kwargs)
  if ret.result.errors or ret.result.failures:
    return False

  return True


def main():
  import os
  import sys
  if not len(sys.argv) == 6:
    sys.stderr.write('Usage: %s <benchmark vtable count> '
                     '<benchmark read count> <benchmark build count> '
                     '<benchmark clear builder> <is_onefile>\n' % sys.argv[0])
    sys.stderr.write('       Provide COMPARE_GENERATED_TO_GO=1   to check'
                     'for bytewise comparison to Go data.\n')
    sys.stderr.write('       Provide COMPARE_GENERATED_TO_JAVA=1 to check'
                     'for bytewise comparison to Java data.\n')
    sys.stderr.flush()
    sys.exit(1)

  kwargs = dict(argv=sys.argv[:-5])

  create_namespace_shortcut(sys.argv[5].lower() == 'true')

  # show whether numpy is present, as it changes the test logic:
  try:
    import numpy
    print('numpy available')
  except ImportError:
    print('numpy not available')

  # run tests, and run some language comparison checks if needed:
  success = backward_compatible_run_tests(**kwargs)
  if success and os.environ.get('COMPARE_GENERATED_TO_GO', 0) == '1':
    success = success and CheckAgainstGoldDataGo()
  if success and os.environ.get('COMPARE_GENERATED_TO_JAVA', 0) == '1':
    success = success and CheckAgainstGoldDataJava()

  if not success:
    sys.stderr.write('Tests failed, skipping benchmarks.\n')
    sys.stderr.flush()
    sys.exit(1)

  # run benchmarks (if 0, they will be a noop):
  bench_vtable = int(sys.argv[1])
  bench_traverse = int(sys.argv[2])
  bench_build = int(sys.argv[3])
  bench_clear = int(sys.argv[4])
  if bench_vtable:
    BenchmarkVtableDeduplication(bench_vtable)
  if bench_traverse:
    buf, off = make_monster_from_generated_code()
    BenchmarkCheckReadBuffer(bench_traverse, buf, off)
  if bench_build:
    buf, off = make_monster_from_generated_code()
    BenchmarkMakeMonsterFromGeneratedCode(bench_build, len(buf))
  if bench_clear:
    buf, off = make_monster_from_generated_code()
    BenchmarkBuilderClear(bench_build, len(buf))

if __name__ == '__main__':
  main()
