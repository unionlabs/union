import 'dart:typed_data';

/// Represents the number of bits a value occupies.
enum BitWidth { width8, width16, width32, width64 }

class BitWidthUtil {
  static int toByteWidth(BitWidth self) {
    return 1 << self.index;
  }

  static BitWidth width(num value) {
    if (value is int) {
      var v = value.toInt().abs();
      if (v >> 7 == 0) return BitWidth.width8;
      if (v >> 15 == 0) return BitWidth.width16;
      if (v >> 31 == 0) return BitWidth.width32;
      return BitWidth.width64;
    }
    return value == _toF32(value as double)
        ? BitWidth.width32
        : BitWidth.width64;
  }

  static BitWidth uwidth(num value) {
    if (value.toInt() == value) {
      var v = value.toInt().abs();
      if (v >> 8 == 0) return BitWidth.width8;
      if (v >> 16 == 0) return BitWidth.width16;
      if (v >> 32 == 0) return BitWidth.width32;
      return BitWidth.width64;
    }
    return value == _toF32(value as double)
        ? BitWidth.width32
        : BitWidth.width64;
  }

  static BitWidth fromByteWidth(int value) {
    if (value == 1) {
      return BitWidth.width8;
    }
    if (value == 2) {
      return BitWidth.width16;
    }
    if (value == 4) {
      return BitWidth.width32;
    }
    if (value == 8) {
      return BitWidth.width64;
    }
    throw Exception('Unexpected value $value');
  }

  static int paddingSize(int bufSize, int scalarSize) {
    return (~bufSize + 1) & (scalarSize - 1);
  }

  static double _toF32(double value) {
    var bdata = ByteData(4);
    bdata.setFloat32(0, value);
    return bdata.getFloat32(0);
  }

  static BitWidth max(BitWidth self, BitWidth other) {
    if (self.index < other.index) {
      return other;
    }
    return self;
  }
}

/// Represents all internal FlexBuffer types.
enum ValueType {
  Null,
  Int,
  UInt,
  Float,
  Key,
  String,
  IndirectInt,
  IndirectUInt,
  IndirectFloat,
  Map,
  Vector,
  VectorInt,
  VectorUInt,
  VectorFloat,
  VectorKey,
  @Deprecated(
      'VectorString is deprecated due to a flaw in the binary format (https://github.com/google/flatbuffers/issues/5627)')
  VectorString,
  VectorInt2,
  VectorUInt2,
  VectorFloat2,
  VectorInt3,
  VectorUInt3,
  VectorFloat3,
  VectorInt4,
  VectorUInt4,
  VectorFloat4,
  Blob,
  Bool,
  VectorBool
}

class ValueTypeUtils {
  static int toInt(ValueType self) {
    if (self == ValueType.VectorBool) return 36;
    return self.index;
  }

  static ValueType fromInt(int value) {
    if (value == 36) return ValueType.VectorBool;
    return ValueType.values[value];
  }

  static bool isInline(ValueType self) {
    return self == ValueType.Bool || toInt(self) <= toInt(ValueType.Float);
  }

  static bool isNumber(ValueType self) {
    return toInt(self) >= toInt(ValueType.Int) &&
        toInt(self) <= toInt(ValueType.Float);
  }

  static bool isIndirectNumber(ValueType self) {
    return toInt(self) >= toInt(ValueType.IndirectInt) &&
        toInt(self) <= toInt(ValueType.IndirectFloat);
  }

  static bool isTypedVectorElement(ValueType self) {
    return self == ValueType.Bool ||
        (toInt(self) >= toInt(ValueType.Int) &&
            toInt(self) <= toInt(ValueType.String));
  }

  static bool isTypedVector(ValueType self) {
    return self == ValueType.VectorBool ||
        (toInt(self) >= toInt(ValueType.VectorInt) &&
            toInt(self) <= toInt(ValueType.VectorString));
  }

  static bool isFixedTypedVector(ValueType self) {
    return (toInt(self) >= toInt(ValueType.VectorInt2) &&
        toInt(self) <= toInt(ValueType.VectorFloat4));
  }

  static bool isAVector(ValueType self) {
    return (isTypedVector(self) ||
        isFixedTypedVector(self) ||
        self == ValueType.Vector);
  }

  static ValueType toTypedVector(ValueType self, int length) {
    if (length == 0) {
      return ValueTypeUtils.fromInt(
          toInt(self) - toInt(ValueType.Int) + toInt(ValueType.VectorInt));
    }
    if (length == 2) {
      return ValueTypeUtils.fromInt(
          toInt(self) - toInt(ValueType.Int) + toInt(ValueType.VectorInt2));
    }
    if (length == 3) {
      return ValueTypeUtils.fromInt(
          toInt(self) - toInt(ValueType.Int) + toInt(ValueType.VectorInt3));
    }
    if (length == 4) {
      return ValueTypeUtils.fromInt(
          toInt(self) - toInt(ValueType.Int) + toInt(ValueType.VectorInt4));
    }
    throw Exception('unexpected length ' + length.toString());
  }

  static ValueType typedVectorElementType(ValueType self) {
    return ValueTypeUtils.fromInt(
        toInt(self) - toInt(ValueType.VectorInt) + toInt(ValueType.Int));
  }

  static ValueType fixedTypedVectorElementType(ValueType self) {
    return ValueTypeUtils.fromInt(
        (toInt(self) - toInt(ValueType.VectorInt2)) % 3 + toInt(ValueType.Int));
  }

  static int fixedTypedVectorElementSize(ValueType self) {
    return (toInt(self) - toInt(ValueType.VectorInt2)) ~/ 3 + 2;
  }

  static int packedType(ValueType self, BitWidth bitWidth) {
    return bitWidth.index | (toInt(self) << 2);
  }
}
