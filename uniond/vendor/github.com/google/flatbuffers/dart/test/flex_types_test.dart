import 'package:flat_buffers/src/types.dart';
import 'package:test/test.dart';

void main() {
  test('is inline', () {
    expect(ValueTypeUtils.isInline(ValueType.Bool), isTrue);
    expect(ValueTypeUtils.isInline(ValueType.Int), isTrue);
    expect(ValueTypeUtils.isInline(ValueType.UInt), isTrue);
    expect(ValueTypeUtils.isInline(ValueType.Float), isTrue);
    expect(ValueTypeUtils.isInline(ValueType.Null), isTrue);
    expect(ValueTypeUtils.isInline(ValueType.String), isFalse);
  });
  test('is type vector element', () {
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.Bool), isTrue);
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.Int), isTrue);
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.UInt), isTrue);
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.Float), isTrue);
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.Key), isTrue);
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.String), isTrue);

    expect(ValueTypeUtils.isTypedVectorElement(ValueType.Null), isFalse);
    expect(ValueTypeUtils.isTypedVectorElement(ValueType.Blob), isFalse);
  });
  test('is typed vector', () {
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorInt), isTrue);
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorUInt), isTrue);
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorFloat), isTrue);
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorBool), isTrue);
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorKey), isTrue);
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorString), isTrue);

    expect(ValueTypeUtils.isTypedVector(ValueType.Vector), isFalse);
    expect(ValueTypeUtils.isTypedVector(ValueType.Map), isFalse);
    expect(ValueTypeUtils.isTypedVector(ValueType.Bool), isFalse);
    expect(ValueTypeUtils.isTypedVector(ValueType.VectorInt2), isFalse);
  });
  test('is fixed typed vector', () {
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorInt2), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorInt3), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorInt4), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorUInt2), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorUInt3), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorUInt4), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorFloat2), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorFloat3), isTrue);
    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorFloat4), isTrue);

    expect(ValueTypeUtils.isFixedTypedVector(ValueType.VectorInt), isFalse);
  });
  test('to typed vector', () {
    expect(ValueTypeUtils.toTypedVector(ValueType.Int, 0),
        equals(ValueType.VectorInt));
    expect(ValueTypeUtils.toTypedVector(ValueType.UInt, 0),
        equals(ValueType.VectorUInt));
    expect(ValueTypeUtils.toTypedVector(ValueType.Bool, 0),
        equals(ValueType.VectorBool));
    expect(ValueTypeUtils.toTypedVector(ValueType.Float, 0),
        equals(ValueType.VectorFloat));
    expect(ValueTypeUtils.toTypedVector(ValueType.Key, 0),
        equals(ValueType.VectorKey));
    expect(ValueTypeUtils.toTypedVector(ValueType.String, 0),
        equals(ValueType.VectorString));

    expect(ValueTypeUtils.toTypedVector(ValueType.Int, 2),
        equals(ValueType.VectorInt2));
    expect(ValueTypeUtils.toTypedVector(ValueType.UInt, 2),
        equals(ValueType.VectorUInt2));
    expect(ValueTypeUtils.toTypedVector(ValueType.Float, 2),
        equals(ValueType.VectorFloat2));

    expect(ValueTypeUtils.toTypedVector(ValueType.Int, 3),
        equals(ValueType.VectorInt3));
    expect(ValueTypeUtils.toTypedVector(ValueType.UInt, 3),
        equals(ValueType.VectorUInt3));
    expect(ValueTypeUtils.toTypedVector(ValueType.Float, 3),
        equals(ValueType.VectorFloat3));

    expect(ValueTypeUtils.toTypedVector(ValueType.Int, 4),
        equals(ValueType.VectorInt4));
    expect(ValueTypeUtils.toTypedVector(ValueType.UInt, 4),
        equals(ValueType.VectorUInt4));
    expect(ValueTypeUtils.toTypedVector(ValueType.Float, 4),
        equals(ValueType.VectorFloat4));
  });
  test('typed vector element type', () {
    expect(ValueTypeUtils.typedVectorElementType(ValueType.VectorInt),
        equals(ValueType.Int));
    expect(ValueTypeUtils.typedVectorElementType(ValueType.VectorUInt),
        equals(ValueType.UInt));
    expect(ValueTypeUtils.typedVectorElementType(ValueType.VectorFloat),
        equals(ValueType.Float));
    expect(ValueTypeUtils.typedVectorElementType(ValueType.VectorString),
        equals(ValueType.String));
    expect(ValueTypeUtils.typedVectorElementType(ValueType.VectorKey),
        equals(ValueType.Key));
    expect(ValueTypeUtils.typedVectorElementType(ValueType.VectorBool),
        equals(ValueType.Bool));
  });
  test('fixed typed vector element type', () {
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorInt2),
        equals(ValueType.Int));
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorInt3),
        equals(ValueType.Int));
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorInt4),
        equals(ValueType.Int));

    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorUInt2),
        equals(ValueType.UInt));
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorUInt3),
        equals(ValueType.UInt));
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorUInt4),
        equals(ValueType.UInt));

    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorFloat2),
        equals(ValueType.Float));
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorFloat3),
        equals(ValueType.Float));
    expect(ValueTypeUtils.fixedTypedVectorElementType(ValueType.VectorFloat4),
        equals(ValueType.Float));
  });
  test('fixed typed vector element size', () {
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorInt2),
        equals(2));
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorInt3),
        equals(3));
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorInt4),
        equals(4));

    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorUInt2),
        equals(2));
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorUInt3),
        equals(3));
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorUInt4),
        equals(4));

    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorFloat2),
        equals(2));
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorFloat3),
        equals(3));
    expect(ValueTypeUtils.fixedTypedVectorElementSize(ValueType.VectorFloat4),
        equals(4));
  });
  test('packed type', () {
    expect(
        ValueTypeUtils.packedType(ValueType.Null, BitWidth.width8), equals(0));
    expect(
        ValueTypeUtils.packedType(ValueType.Null, BitWidth.width16), equals(1));
    expect(
        ValueTypeUtils.packedType(ValueType.Null, BitWidth.width32), equals(2));
    expect(
        ValueTypeUtils.packedType(ValueType.Null, BitWidth.width64), equals(3));

    expect(
        ValueTypeUtils.packedType(ValueType.Int, BitWidth.width8), equals(4));
    expect(
        ValueTypeUtils.packedType(ValueType.Int, BitWidth.width16), equals(5));
    expect(
        ValueTypeUtils.packedType(ValueType.Int, BitWidth.width32), equals(6));
    expect(
        ValueTypeUtils.packedType(ValueType.Int, BitWidth.width64), equals(7));
  });
  test('bit width', () {
    expect(BitWidthUtil.width(0), BitWidth.width8);
    expect(BitWidthUtil.width(-20), BitWidth.width8);
    expect(BitWidthUtil.width(127), BitWidth.width8);
    expect(BitWidthUtil.width(128), BitWidth.width16);
    expect(BitWidthUtil.width(128123), BitWidth.width32);
    expect(BitWidthUtil.width(12812324534), BitWidth.width64);
    expect(BitWidthUtil.width(-127), BitWidth.width8);
    expect(BitWidthUtil.width(-128), BitWidth.width16);
    expect(BitWidthUtil.width(-12812324534), BitWidth.width64);
    expect(BitWidthUtil.width(-0.1), BitWidth.width64);
    expect(BitWidthUtil.width(0.25), BitWidth.width32);
  });
  test('padding size', () {
    expect(BitWidthUtil.paddingSize(10, 8), 6);
    expect(BitWidthUtil.paddingSize(10, 4), 2);
    expect(BitWidthUtil.paddingSize(15, 4), 1);
    expect(BitWidthUtil.paddingSize(15, 2), 1);
    expect(BitWidthUtil.paddingSize(15, 1), 0);
    expect(BitWidthUtil.paddingSize(16, 8), 0);
    expect(BitWidthUtil.paddingSize(17, 8), 7);
  });
}
