import { fromByteWidth } from './bit-width-util.js'
import { ValueType } from './value-type.js'
import { isNumber, isIndirectNumber, isAVector, fixedTypedVectorElementSize, isFixedTypedVector, isTypedVector, typedVectorElementType, packedType, fixedTypedVectorElementType } from './value-type-util.js'
import { indirect, keyForIndex, keyIndex, readFloat, readInt, readUInt } from './reference-util.js'
import { fromUTF8Array } from './flexbuffers-util.js';
import { BitWidth } from './bit-width.js';

export function toReference(buffer: ArrayBuffer): Reference {
  const len = buffer.byteLength;

  if (len < 3) {
    throw "Buffer needs to be bigger than 3";
  }

  const dataView = new DataView(buffer);
  const byteWidth = dataView.getUint8(len - 1);
  const packedType = dataView.getUint8(len - 2);
  const parentWidth = fromByteWidth(byteWidth);
  const offset = len - byteWidth - 2;

  return new Reference(dataView, offset, parentWidth, packedType, "/")
}

function valueForIndexWithKey(index: number, key: string, dataView: DataView, offset: number, parentWidth: number, byteWidth: number, length: number, path: string): Reference {
  const _indirect = indirect(dataView, offset, parentWidth);
  const elementOffset = _indirect + index * byteWidth;
  const packedType = dataView.getUint8(_indirect + length * byteWidth + index);
  return new Reference(dataView, elementOffset, fromByteWidth(byteWidth), packedType, `${path}/${key}`)
}

export class Reference {
  private readonly byteWidth: number
  private readonly valueType: ValueType
  private _length = -1
  constructor(private dataView: DataView, private offset: number, private parentWidth: number, private packedType: ValueType, private path: string) {
    this.byteWidth = 1 << (packedType & 3)
    this.valueType = packedType >> 2
  }

  isNull(): boolean { return this.valueType === ValueType.NULL; }
  isNumber(): boolean { return isNumber(this.valueType) || isIndirectNumber(this.valueType); }
  isFloat(): boolean { return ValueType.FLOAT === this.valueType || ValueType.INDIRECT_FLOAT === this.valueType; }
  isInt(): boolean { return this.isNumber() && !this.isFloat(); }
  isString(): boolean { return ValueType.STRING === this.valueType || ValueType.KEY === this.valueType; }
  isBool(): boolean { return ValueType.BOOL === this.valueType; }
  isBlob(): boolean { return ValueType.BLOB === this.valueType; }
  isVector(): boolean { return isAVector(this.valueType); }
  isMap(): boolean { return ValueType.MAP === this.valueType; }

  boolValue(): boolean | null {
    if (this.isBool()) {
      return readInt(this.dataView, this.offset, this.parentWidth) > 0;
    }
    return null;
  }

  intValue(): number | bigint | null {
    if (this.valueType === ValueType.INT) {
      return readInt(this.dataView, this.offset, this.parentWidth);
    }
    if (this.valueType === ValueType.UINT) {
      return readUInt(this.dataView, this.offset, this.parentWidth);
    }
    if (this.valueType === ValueType.INDIRECT_INT) {
      return readInt(this.dataView, indirect(this.dataView, this.offset, this.parentWidth), fromByteWidth(this.byteWidth));
    }
    if (this.valueType === ValueType.INDIRECT_UINT) {
      return readUInt(this.dataView, indirect(this.dataView, this.offset, this.parentWidth), fromByteWidth(this.byteWidth));
    }
    return null;
  }

  floatValue(): number | null {
    if (this.valueType === ValueType.FLOAT) {
      return readFloat(this.dataView, this.offset, this.parentWidth);
    }
    if (this.valueType === ValueType.INDIRECT_FLOAT) {
      return readFloat(this.dataView, indirect(this.dataView, this.offset, this.parentWidth), fromByteWidth(this.byteWidth));
    }
    return null;
  }

  numericValue(): number | bigint | null { return this.floatValue() || this.intValue()}

  stringValue(): string | null {
    if (this.valueType === ValueType.STRING || this.valueType === ValueType.KEY) {
      const begin = indirect(this.dataView, this.offset, this.parentWidth);
      return fromUTF8Array(new Uint8Array(this.dataView.buffer, begin, this.length()));
    }
    return null;
  }

  blobValue(): Uint8Array | null {
    if (this.isBlob()) {
      const begin = indirect(this.dataView, this.offset, this.parentWidth);
      return new Uint8Array(this.dataView.buffer, begin, this.length());
    }
    return null;
  }

  get(key: number): Reference {
    const length = this.length();
    if (Number.isInteger(key) && isAVector(this.valueType)) {
      if (key >= length || key < 0) {
        throw `Key: [${key}] is not applicable on ${this.path} of ${this.valueType} length: ${length}`;
      }
      const _indirect = indirect(this.dataView, this.offset, this.parentWidth);
      const elementOffset = _indirect + key * this.byteWidth;
      let _packedType = this.dataView.getUint8(_indirect + length * this.byteWidth + key);
      if (isTypedVector(this.valueType)) {
        const _valueType = typedVectorElementType(this.valueType);
        _packedType = packedType(_valueType, BitWidth.WIDTH8);
      } else if (isFixedTypedVector(this.valueType)) {
        const _valueType = fixedTypedVectorElementType(this.valueType);
        _packedType = packedType(_valueType, BitWidth.WIDTH8);
      }
      return new Reference(this.dataView, elementOffset, fromByteWidth(this.byteWidth), _packedType, `${this.path}[${key}]`);
    }
    if (typeof key === 'string') {
      const index = keyIndex(key, this.dataView, this.offset, this.parentWidth, this.byteWidth, length);
      if (index !== null) {
        return valueForIndexWithKey(index, key, this.dataView, this.offset, this.parentWidth, this.byteWidth, length, this.path)
      }
    }
    throw `Key [${key}] is not applicable on ${this.path} of ${this.valueType}`;
  }

  length(): number {
    let size;
    if (this._length > -1) {
      return this._length;
    }
    if (isFixedTypedVector(this.valueType)) {
      this._length = fixedTypedVectorElementSize(this.valueType);
    } else if (this.valueType === ValueType.BLOB
      || this.valueType === ValueType.MAP
      || isAVector(this.valueType)) {
      this._length = readUInt(this.dataView, indirect(this.dataView, this.offset, this.parentWidth) - this.byteWidth, fromByteWidth(this.byteWidth)) as number
    } else if (this.valueType === ValueType.NULL) {
      this._length = 0;
    } else if (this.valueType === ValueType.STRING) {
      const _indirect = indirect(this.dataView, this.offset, this.parentWidth);
      let sizeByteWidth = this.byteWidth;
      size = readUInt(this.dataView, _indirect - sizeByteWidth, fromByteWidth(this.byteWidth));
      while (this.dataView.getInt8(_indirect + (size as number)) !== 0) {
        sizeByteWidth <<= 1;
        size = readUInt(this.dataView, _indirect - sizeByteWidth, fromByteWidth(this.byteWidth));
      }
      this._length = size as number;
    } else if (this.valueType === ValueType.KEY) {
      const _indirect = indirect(this.dataView, this.offset, this.parentWidth);
      size = 1;
      while (this.dataView.getInt8(_indirect + size) !== 0) {
        size++;
      }
      this._length = size;
    } else {
      this._length = 1;
    }
    return Number(this._length);
  }

  toObject(): unknown {
    const length = this.length();
    if (this.isVector()) {
      const result = [];
      for (let i = 0; i < length; i++) {
        result.push(this.get(i).toObject());
      }
      return result;
    }
    if (this.isMap()) {
      const result: Record<string, unknown> = {};
      for (let i = 0; i < length; i++) {
        const key = keyForIndex(i, this.dataView, this.offset, this.parentWidth, this.byteWidth);
        result[key] = valueForIndexWithKey(i, key, this.dataView, this.offset, this.parentWidth, this.byteWidth, length, this.path).toObject();
      }
      return result;
    }
    if (this.isNull()) {
      return null;
    }
    if (this.isBool()) {
      return this.boolValue();
    }
    if (this.isNumber()) {
      return this.numericValue();
    }
    return this.blobValue() || this.stringValue();
  }
}
