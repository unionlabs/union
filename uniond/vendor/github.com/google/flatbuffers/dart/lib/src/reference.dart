import 'dart:collection';
import 'dart:convert';
import 'dart:typed_data';
import 'types.dart';

/// Main class to read a value out of a FlexBuffer.
///
/// This class let you access values stored in the buffer in a lazy fashion.
class Reference {
  final ByteData _buffer;
  final int _offset;
  final BitWidth _parentWidth;
  final String _path;
  final int _byteWidth;
  final ValueType _valueType;
  int? _length;

  Reference._(
      this._buffer, this._offset, this._parentWidth, int packedType, this._path,
      [int? byteWidth, ValueType? valueType])
      : _byteWidth = byteWidth ?? 1 << (packedType & 3),
        _valueType = valueType ?? ValueTypeUtils.fromInt(packedType >> 2);

  /// Use this method to access the root value of a FlexBuffer.
  static Reference fromBuffer(ByteBuffer buffer) {
    final len = buffer.lengthInBytes;
    if (len < 3) {
      throw UnsupportedError('Buffer needs to be bigger than 3');
    }
    final byteData = ByteData.view(buffer);
    final byteWidth = byteData.getUint8(len - 1);
    final packedType = byteData.getUint8(len - 2);
    final offset = len - byteWidth - 2;
    return Reference._(ByteData.view(buffer), offset,
        BitWidthUtil.fromByteWidth(byteWidth), packedType, "/");
  }

  /// Returns true if the underlying value is null.
  bool get isNull => _valueType == ValueType.Null;

  /// Returns true if the underlying value can be represented as [num].
  bool get isNum =>
      ValueTypeUtils.isNumber(_valueType) ||
      ValueTypeUtils.isIndirectNumber(_valueType);

  /// Returns true if the underlying value was encoded as a float (direct or indirect).
  bool get isDouble =>
      _valueType == ValueType.Float || _valueType == ValueType.IndirectFloat;

  /// Returns true if the underlying value was encoded as an int or uint (direct or indirect).
  bool get isInt => isNum && !isDouble;

  /// Returns true if the underlying value was encoded as a string or a key.
  bool get isString =>
      _valueType == ValueType.String || _valueType == ValueType.Key;

  /// Returns true if the underlying value was encoded as a bool.
  bool get isBool => _valueType == ValueType.Bool;

  /// Returns true if the underlying value was encoded as a blob.
  bool get isBlob => _valueType == ValueType.Blob;

  /// Returns true if the underlying value points to a vector.
  bool get isVector => ValueTypeUtils.isAVector(_valueType);

  /// Returns true if the underlying value points to a map.
  bool get isMap => _valueType == ValueType.Map;

  /// If this [isBool], returns the bool value. Otherwise, returns null.
  bool? get boolValue {
    if (_valueType == ValueType.Bool) {
      return _readInt(_offset, _parentWidth) != 0;
    }
    return null;
  }

  /// Returns an [int], if the underlying value can be represented as an int.
  ///
  /// Otherwise returns [null].
  int? get intValue {
    if (_valueType == ValueType.Int) {
      return _readInt(_offset, _parentWidth);
    }
    if (_valueType == ValueType.UInt) {
      return _readUInt(_offset, _parentWidth);
    }
    if (_valueType == ValueType.IndirectInt) {
      return _readInt(_indirect, BitWidthUtil.fromByteWidth(_byteWidth));
    }
    if (_valueType == ValueType.IndirectUInt) {
      return _readUInt(_indirect, BitWidthUtil.fromByteWidth(_byteWidth));
    }
    return null;
  }

  /// Returns [double], if the underlying value [isDouble].
  ///
  /// Otherwise returns [null].
  double? get doubleValue {
    if (_valueType == ValueType.Float) {
      return _readFloat(_offset, _parentWidth);
    }
    if (_valueType == ValueType.IndirectFloat) {
      return _readFloat(_indirect, BitWidthUtil.fromByteWidth(_byteWidth));
    }
    return null;
  }

  /// Returns [num], if the underlying value is numeric, be it int uint, or float (direct or indirect).
  ///
  /// Otherwise returns [null].
  num? get numValue => doubleValue ?? intValue;

  /// Returns [String] value or null otherwise.
  ///
  /// This method performers a utf8 decoding, as FlexBuffers format stores strings in utf8 encoding.
  String? get stringValue {
    if (_valueType == ValueType.String || _valueType == ValueType.Key) {
      return utf8.decode(_buffer.buffer.asUint8List(_indirect, length));
    }
    return null;
  }

  /// Returns [Uint8List] value or null otherwise.
  Uint8List? get blobValue {
    if (_valueType == ValueType.Blob) {
      return _buffer.buffer.asUint8List(_indirect, length);
    }
    return null;
  }

  /// Can be used with an [int] or a [String] value for key.
  /// If the underlying value in FlexBuffer is a vector, then use [int] for access.
  /// If the underlying value in FlexBuffer is a map, then use [String] for access.
  /// Returns [Reference] value. Throws an exception when [key] is not applicable.
  Reference operator [](Object key) {
    if (key is int && ValueTypeUtils.isAVector(_valueType)) {
      final index = key;
      if (index >= length || index < 0) {
        throw ArgumentError(
            'Key: [$key] is not applicable on: $_path of: $_valueType length: $length');
      }
      final elementOffset = _indirect + index * _byteWidth;
      int packedType = 0;
      int? byteWidth;
      ValueType? valueType;
      if (ValueTypeUtils.isTypedVector(_valueType)) {
        byteWidth = 1;
        valueType = ValueTypeUtils.typedVectorElementType(_valueType);
      } else if (ValueTypeUtils.isFixedTypedVector(_valueType)) {
        byteWidth = 1;
        valueType = ValueTypeUtils.fixedTypedVectorElementType(_valueType);
      } else {
        packedType = _buffer.getUint8(_indirect + length * _byteWidth + index);
      }
      return Reference._(
          _buffer,
          elementOffset,
          BitWidthUtil.fromByteWidth(_byteWidth),
          packedType,
          "$_path[$index]",
          byteWidth,
          valueType);
    }
    if (key is String && _valueType == ValueType.Map) {
      final index = _keyIndex(key);
      if (index != null) {
        return _valueForIndexWithKey(index, key);
      }
    }
    throw ArgumentError(
        'Key: [$key] is not applicable on: $_path of: $_valueType');
  }

  /// Get an iterable if the underlying flexBuffer value is a vector.
  /// Otherwise throws an exception.
  Iterable<Reference> get vectorIterable {
    if (isVector == false) {
      throw UnsupportedError('Value is not a vector. It is: $_valueType');
    }
    return _VectorIterator(this);
  }

  /// Get an iterable for keys if the underlying flexBuffer value is a map.
  /// Otherwise throws an exception.
  Iterable<String> get mapKeyIterable {
    if (isMap == false) {
      throw UnsupportedError('Value is not a map. It is: $_valueType');
    }
    return _MapKeyIterator(this);
  }

  /// Get an iterable for values if the underlying flexBuffer value is a map.
  /// Otherwise throws an exception.
  Iterable<Reference> get mapValueIterable {
    if (isMap == false) {
      throw UnsupportedError('Value is not a map. It is: $_valueType');
    }
    return _MapValueIterator(this);
  }

  /// Returns the length of the underlying FlexBuffer value.
  /// If the underlying value is [null] the length is 0.
  /// If the underlying value is a number, or a bool, the length is 1.
  /// If the underlying value is a vector, or map, the length reflects number of elements / element pairs.
  /// If the values is a string or a blob, the length reflects a number of bytes the value occupies (strings are encoded in utf8 format).
  int get length {
    if (_length == null) {
      // needs to be checked before more generic isAVector
      if (ValueTypeUtils.isFixedTypedVector(_valueType)) {
        _length = ValueTypeUtils.fixedTypedVectorElementSize(_valueType);
      } else if (_valueType == ValueType.Blob ||
          ValueTypeUtils.isAVector(_valueType) ||
          _valueType == ValueType.Map) {
        _length = _readUInt(
            _indirect - _byteWidth, BitWidthUtil.fromByteWidth(_byteWidth));
      } else if (_valueType == ValueType.Null) {
        _length = 0;
      } else if (_valueType == ValueType.String) {
        final indirect = _indirect;
        var sizeByteWidth = _byteWidth;
        var size = _readUInt(indirect - sizeByteWidth,
            BitWidthUtil.fromByteWidth(sizeByteWidth));
        while (_buffer.getInt8(indirect + size) != 0) {
          sizeByteWidth <<= 1;
          size = _readUInt(indirect - sizeByteWidth,
              BitWidthUtil.fromByteWidth(sizeByteWidth));
        }
        _length = size;
      } else if (_valueType == ValueType.Key) {
        final indirect = _indirect;
        var size = 1;
        while (_buffer.getInt8(indirect + size) != 0) {
          size += 1;
        }
        _length = size;
      } else {
        _length = 1;
      }
    }
    return _length!;
  }

  /// Returns a minified JSON representation of the underlying FlexBuffer value.
  ///
  /// This method involves materializing the entire object tree, which may be
  /// expensive. It is more efficient to work with [Reference] and access only the needed data.
  /// Blob values are represented as base64 encoded string.
  String get json {
    if (_valueType == ValueType.Bool) {
      return boolValue! ? 'true' : 'false';
    }
    if (_valueType == ValueType.Null) {
      return 'null';
    }
    if (ValueTypeUtils.isNumber(_valueType)) {
      return jsonEncode(numValue);
    }
    if (_valueType == ValueType.String) {
      return jsonEncode(stringValue);
    }
    if (_valueType == ValueType.Blob) {
      return jsonEncode(base64Encode(blobValue!));
    }
    if (ValueTypeUtils.isAVector(_valueType)) {
      final result = StringBuffer();
      result.write('[');
      for (var i = 0; i < length; i++) {
        result.write(this[i].json);
        if (i < length - 1) {
          result.write(',');
        }
      }
      result.write(']');
      return result.toString();
    }
    if (_valueType == ValueType.Map) {
      final result = StringBuffer();
      result.write('{');
      for (var i = 0; i < length; i++) {
        result.write(jsonEncode(_keyForIndex(i)));
        result.write(':');
        result.write(_valueForIndex(i).json);
        if (i < length - 1) {
          result.write(',');
        }
      }
      result.write('}');
      return result.toString();
    }
    throw UnsupportedError(
        'Type: $_valueType is not supported for JSON conversion');
  }

  /// Computes the indirect offset of the value.
  ///
  /// To optimize for the more common case of being called only once, this
  /// value is not cached. Callers that need to use it more than once should
  /// cache the return value in a local variable.
  int get _indirect {
    final step = _readUInt(_offset, _parentWidth);
    return _offset - step;
  }

  int _readInt(int offset, BitWidth width) {
    _validateOffset(offset, width);
    if (width == BitWidth.width8) {
      return _buffer.getInt8(offset);
    }
    if (width == BitWidth.width16) {
      return _buffer.getInt16(offset, Endian.little);
    }
    if (width == BitWidth.width32) {
      return _buffer.getInt32(offset, Endian.little);
    }
    return _buffer.getInt64(offset, Endian.little);
  }

  int _readUInt(int offset, BitWidth width) {
    _validateOffset(offset, width);
    if (width == BitWidth.width8) {
      return _buffer.getUint8(offset);
    }
    if (width == BitWidth.width16) {
      return _buffer.getUint16(offset, Endian.little);
    }
    if (width == BitWidth.width32) {
      return _buffer.getUint32(offset, Endian.little);
    }
    return _buffer.getUint64(offset, Endian.little);
  }

  double _readFloat(int offset, BitWidth width) {
    _validateOffset(offset, width);
    if (width.index < BitWidth.width32.index) {
      throw StateError('Bad width: $width');
    }

    if (width == BitWidth.width32) {
      return _buffer.getFloat32(offset, Endian.little);
    }

    return _buffer.getFloat64(offset, Endian.little);
  }

  void _validateOffset(int offset, BitWidth width) {
    if (_offset < 0 ||
        _buffer.lengthInBytes <= offset + width.index ||
        offset & (BitWidthUtil.toByteWidth(width) - 1) != 0) {
      throw StateError('Bad offset: $offset, width: $width');
    }
  }

  int? _keyIndex(String key) {
    final input = utf8.encode(key);
    final keysVectorOffset = _indirect - _byteWidth * 3;
    final indirectOffset = keysVectorOffset -
        _readUInt(keysVectorOffset, BitWidthUtil.fromByteWidth(_byteWidth));
    final byteWidth = _readUInt(
        keysVectorOffset + _byteWidth, BitWidthUtil.fromByteWidth(_byteWidth));
    var low = 0;
    var high = length - 1;
    while (low <= high) {
      final mid = (high + low) >> 1;
      final dif = _diffKeys(input, mid, indirectOffset, byteWidth);
      if (dif == 0) return mid;
      if (dif < 0) {
        high = mid - 1;
      } else {
        low = mid + 1;
      }
    }
    return null;
  }

  int _diffKeys(List<int> input, int index, int indirectOffset, int byteWidth) {
    final keyOffset = indirectOffset + index * byteWidth;
    final keyIndirectOffset =
        keyOffset - _readUInt(keyOffset, BitWidthUtil.fromByteWidth(byteWidth));
    for (var i = 0; i < input.length; i++) {
      final dif = input[i] - _buffer.getUint8(keyIndirectOffset + i);
      if (dif != 0) {
        return dif;
      }
    }
    return (_buffer.getUint8(keyIndirectOffset + input.length) == 0) ? 0 : -1;
  }

  Reference _valueForIndexWithKey(int index, String key) {
    final indirect = _indirect;
    final elementOffset = indirect + index * _byteWidth;
    final packedType = _buffer.getUint8(indirect + length * _byteWidth + index);
    return Reference._(_buffer, elementOffset,
        BitWidthUtil.fromByteWidth(_byteWidth), packedType, "$_path/$key");
  }

  Reference _valueForIndex(int index) {
    final indirect = _indirect;
    final elementOffset = indirect + index * _byteWidth;
    final packedType = _buffer.getUint8(indirect + length * _byteWidth + index);
    return Reference._(_buffer, elementOffset,
        BitWidthUtil.fromByteWidth(_byteWidth), packedType, "$_path/[$index]");
  }

  String _keyForIndex(int index) {
    final keysVectorOffset = _indirect - _byteWidth * 3;
    final indirectOffset = keysVectorOffset -
        _readUInt(keysVectorOffset, BitWidthUtil.fromByteWidth(_byteWidth));
    final byteWidth = _readUInt(
        keysVectorOffset + _byteWidth, BitWidthUtil.fromByteWidth(_byteWidth));
    final keyOffset = indirectOffset + index * byteWidth;
    final keyIndirectOffset =
        keyOffset - _readUInt(keyOffset, BitWidthUtil.fromByteWidth(byteWidth));
    var length = 0;
    while (_buffer.getUint8(keyIndirectOffset + length) != 0) {
      length += 1;
    }
    return utf8.decode(_buffer.buffer.asUint8List(keyIndirectOffset, length));
  }
}

class _VectorIterator
    with IterableMixin<Reference>
    implements Iterator<Reference> {
  final Reference _vector;
  int index = -1;

  _VectorIterator(this._vector);

  @override
  Reference get current => _vector[index];

  @override
  bool moveNext() {
    index++;
    return index < _vector.length;
  }

  @override
  Iterator<Reference> get iterator => this;
}

class _MapKeyIterator with IterableMixin<String> implements Iterator<String> {
  final Reference _map;
  int index = -1;

  _MapKeyIterator(this._map);

  @override
  String get current => _map._keyForIndex(index);

  @override
  bool moveNext() {
    index++;
    return index < _map.length;
  }

  @override
  Iterator<String> get iterator => this;
}

class _MapValueIterator
    with IterableMixin<Reference>
    implements Iterator<Reference> {
  final Reference _map;
  int index = -1;

  _MapValueIterator(this._map);

  @override
  Reference get current => _map._valueForIndex(index);

  @override
  bool moveNext() {
    index++;
    return index < _map.length;
  }

  @override
  Iterator<Reference> get iterator => this;
}
