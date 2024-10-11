import { ValueType } from './value-type.js'

export function isInline(value: ValueType): boolean {
  return value === ValueType.BOOL
    || value <= ValueType.FLOAT;
}

export function isNumber(value: ValueType): boolean {
  return value >= ValueType.INT
    && value <= ValueType.FLOAT;
}

export function isIndirectNumber(value: ValueType): boolean {
  return value >= ValueType.INDIRECT_INT
    && value <= ValueType.INDIRECT_FLOAT;
}

export function isTypedVectorElement(value: ValueType): boolean {
  return value === ValueType.BOOL
    || (value >= ValueType.INT
      && value <= ValueType.STRING);
}

export function isTypedVector(value: ValueType): boolean {
  return value === ValueType.VECTOR_BOOL
    || (value >= ValueType.VECTOR_INT
      && value <= ValueType.VECTOR_STRING_DEPRECATED);
}

export function isFixedTypedVector(value: ValueType): boolean {
  return value >= ValueType.VECTOR_INT2
    && value <= ValueType.VECTOR_FLOAT4;
}

export function isAVector(value: ValueType): boolean {
  return isTypedVector(value)
    || isFixedTypedVector(value)
    || value === ValueType.VECTOR;
}

export function toTypedVector(valueType: ValueType, length: number): ValueType {
  if (length === 0) return valueType - ValueType.INT + ValueType.VECTOR_INT;
  if (length === 2) return valueType - ValueType.INT + ValueType.VECTOR_INT2;
  if (length === 3) return valueType - ValueType.INT + ValueType.VECTOR_INT3;
  if (length === 4) return valueType - ValueType.INT + ValueType.VECTOR_INT4;
  throw "Unexpected length " + length;
}

export function typedVectorElementType(valueType: ValueType): ValueType {
  return valueType - ValueType.VECTOR_INT + ValueType.INT;
}

export function fixedTypedVectorElementType(valueType: ValueType): ValueType {
  return ((valueType - ValueType.VECTOR_INT2) % 3) + ValueType.INT;
}

export function fixedTypedVectorElementSize(valueType: ValueType): ValueType {
  // The x / y >> 0 trick is to have an int division. Suppose to be faster than Math.floor()
  return (((valueType - ValueType.VECTOR_INT2) / 3) >> 0) + 2;
}

export function packedType(valueType: ValueType, bitWidth: number): ValueType {
  return bitWidth | (valueType << 2);
}