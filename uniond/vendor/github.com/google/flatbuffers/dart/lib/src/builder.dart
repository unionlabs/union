import 'dart:convert';
import 'dart:typed_data';

import 'types.dart';

/// The main builder class for creation of a FlexBuffer.
class Builder {
  final ByteData _buffer;
  List<_StackValue> _stack = [];
  List<_StackPointer> _stackPointers = [];
  int _offset = 0;
  bool _finished = false;
  final Map<String, _StackValue> _stringCache = {};
  final Map<String, _StackValue> _keyCache = {};
  final Map<_KeysHash, _StackValue> _keyVectorCache = {};
  final Map<int, _StackValue> _indirectIntCache = {};
  final Map<double, _StackValue> _indirectDoubleCache = {};

  /// Instantiate the builder if you intent to gradually build up the buffer by calling
  /// add... methods and calling [finish] to receive the resulting byte array.
  ///
  /// The default size of internal buffer is set to 2048. Provide a different value in order to avoid buffer copies.
  Builder({int size = 2048}) : _buffer = ByteData(size);

  /// Use this method in order to turn an object into a FlexBuffer directly.
  ///
  /// Use the manual instantiation of the [Builder] and gradual addition of values, if performance is more important than convenience.
  static ByteBuffer buildFromObject(Object? value) {
    final builder = Builder();
    builder._add(value);
    final buffer = builder.finish();
    final byteData = ByteData(buffer.lengthInBytes);
    byteData.buffer.asUint8List().setAll(0, buffer);
    return byteData.buffer;
  }

  void _add(Object? value) {
    if (value == null) {
      addNull();
    } else if (value is bool) {
      addBool(value);
    } else if (value is int) {
      addInt(value);
    } else if (value is double) {
      addDouble(value);
    } else if (value is ByteBuffer) {
      addBlob(value);
    } else if (value is String) {
      addString(value);
    } else if (value is List<dynamic>) {
      startVector();
      for (var i = 0; i < value.length; i++) {
        _add(value[i]);
      }
      end();
    } else if (value is Map<String, dynamic>) {
      startMap();
      value.forEach((key, value) {
        addKey(key);
        _add(value);
      });
      end();
    } else {
      throw UnsupportedError('Value of unexpected type: $value');
    }
  }

  /// Use this method if you want to store a null value.
  ///
  /// Specifically useful when building up a vector where values can be null.
  void addNull() {
    _integrityCheckOnValueAddition();
    _stack.add(_StackValue.withNull());
  }

  /// Adds a string value.
  void addInt(int value) {
    _integrityCheckOnValueAddition();
    _stack.add(_StackValue.withInt(value));
  }

  /// Adds a bool value.
  void addBool(bool value) {
    _integrityCheckOnValueAddition();
    _stack.add(_StackValue.withBool(value));
  }

  /// Adds a double value.
  void addDouble(double value) {
    _integrityCheckOnValueAddition();
    _stack.add(_StackValue.withDouble(value));
  }

  /// Adds a string value.
  void addString(String value) {
    _integrityCheckOnValueAddition();
    if (_stringCache.containsKey(value)) {
      _stack.add(_stringCache[value]!);
      return;
    }
    final utf8String = utf8.encode(value);
    final length = utf8String.length;
    final bitWidth = BitWidthUtil.uwidth(length);
    final byteWidth = _align(bitWidth);
    _writeUInt(length, byteWidth);
    final stringOffset = _offset;
    final newOffset = _newOffset(length + 1);
    _pushBuffer(utf8String);
    _offset = newOffset;
    final stackValue =
        _StackValue.withOffset(stringOffset, ValueType.String, bitWidth);
    _stack.add(stackValue);
    _stringCache[value] = stackValue;
  }

  /// This methods adds a key to a map and should be followed by an add... value call.
  ///
  /// It also implies that you call this method only after you called [startMap].
  void addKey(String value) {
    _integrityCheckOnKeyAddition();
    if (_keyCache.containsKey(value)) {
      _stack.add(_keyCache[value]!);
      return;
    }
    final utf8String = utf8.encode(value);
    final length = utf8String.length;
    final keyOffset = _offset;
    final newOffset = _newOffset(length + 1);
    _pushBuffer(utf8String);
    _offset = newOffset;
    final stackValue =
        _StackValue.withOffset(keyOffset, ValueType.Key, BitWidth.width8);
    _stack.add(stackValue);
    _keyCache[value] = stackValue;
  }

  /// Adds a byte array.
  ///
  /// This method can be used to store any generic BLOB.
  void addBlob(ByteBuffer value) {
    _integrityCheckOnValueAddition();
    final length = value.lengthInBytes;
    final bitWidth = BitWidthUtil.uwidth(length);
    final byteWidth = _align(bitWidth);
    _writeUInt(length, byteWidth);
    final blobOffset = _offset;
    final newOffset = _newOffset(length);
    _pushBuffer(value.asUint8List());
    _offset = newOffset;
    final stackValue =
        _StackValue.withOffset(blobOffset, ValueType.Blob, bitWidth);
    _stack.add(stackValue);
  }

  /// Stores int value indirectly in the buffer.
  ///
  /// Adding large integer values indirectly might be beneficial if those values suppose to be store in a vector together with small integer values.
  /// This is due to the fact that FlexBuffers will add padding to small integer values, if they are stored together with large integer values.
  /// When we add integer indirectly the vector of ints will contain not the value itself, but only the relative offset to the value.
  /// By setting the [cache] parameter to true, you make sure that the builder tracks added int value and performs deduplication.
  void addIntIndirectly(int value, {bool cache = false}) {
    _integrityCheckOnValueAddition();
    if (_indirectIntCache.containsKey(value)) {
      _stack.add(_indirectIntCache[value]!);
      return;
    }
    final stackValue = _StackValue.withInt(value);
    final byteWidth = _align(stackValue.width);
    final newOffset = _newOffset(byteWidth);
    final valueOffset = _offset;
    _pushBuffer(stackValue.asU8List(stackValue.width));
    final stackOffset = _StackValue.withOffset(
        valueOffset, ValueType.IndirectInt, stackValue.width);
    _stack.add(stackOffset);
    _offset = newOffset;
    if (cache) {
      _indirectIntCache[value] = stackOffset;
    }
  }

  /// Stores double value indirectly in the buffer.
  ///
  /// Double are stored as 8 or 4 byte values in FlexBuffers. If they are stored in a mixed vector, values which are smaller than 4 / 8 bytes will be padded.
  /// When we add double indirectly, the vector will contain not the value itself, but only the relative offset to the value. Which could occupy only 1 or 2 bytes, reducing the odds for unnecessary padding.
  /// By setting the [cache] parameter to true, you make sure that the builder tracks already added double value and performs deduplication.
  void addDoubleIndirectly(double value, {bool cache = false}) {
    _integrityCheckOnValueAddition();
    if (cache && _indirectDoubleCache.containsKey(value)) {
      _stack.add(_indirectDoubleCache[value]!);
      return;
    }
    final stackValue = _StackValue.withDouble(value);
    final byteWidth = _align(stackValue.width);
    final newOffset = _newOffset(byteWidth);
    final valueOffset = _offset;
    _pushBuffer(stackValue.asU8List(stackValue.width));
    final stackOffset = _StackValue.withOffset(
        valueOffset, ValueType.IndirectFloat, stackValue.width);
    _stack.add(stackOffset);
    _offset = newOffset;
    if (cache) {
      _indirectDoubleCache[value] = stackOffset;
    }
  }

  /// This method starts a vector definition and needs to be followed by 0 to n add... value calls.
  ///
  /// The vector definition needs to be finished with an [end] call.
  /// It is also possible to add nested vector or map by calling [startVector] / [startMap].
  void startVector() {
    _integrityCheckOnValueAddition();
    _stackPointers.add(_StackPointer(_stack.length, true));
  }

  /// This method starts a map definition.
  ///
  /// This method call needs to be followed by 0 to n [addKey] +  add... value calls.
  /// The map definition needs to be finished with an [end] call.
  /// It is also possible to add nested vector or map by calling [startVector] / [startMap] after calling [addKey].
  void startMap() {
    _integrityCheckOnValueAddition();
    _stackPointers.add(_StackPointer(_stack.length, false));
  }

  /// Marks that the addition of values to the last vector, or map have ended.
  void end() {
    final pointer = _stackPointers.removeLast();
    if (pointer.isVector) {
      _endVector(pointer);
    } else {
      _sortKeysAndEndMap(pointer);
    }
  }

  /// Finish building the FlatBuffer and return array of bytes.
  ///
  /// Can be called multiple times, to get the array of bytes.
  /// After the first call, adding values, or starting vectors / maps will result in an exception.
  Uint8List finish() {
    if (_finished == false) {
      _finish();
    }
    return _buffer.buffer.asUint8List(0, _offset);
  }

  /// Builds a FlatBuffer with current state without finishing the builder.
  ///
  /// Creates an internal temporary copy of current builder and finishes the copy.
  /// Use this method, when the state of a long lasting builder need to be persisted periodically.
  ByteBuffer snapshot() {
    final tmp = Builder(size: _offset + 200);
    tmp._offset = _offset;
    tmp._stack = List.from(_stack);
    tmp._stackPointers = List.from(_stackPointers);
    tmp._buffer.buffer
        .asUint8List()
        .setAll(0, _buffer.buffer.asUint8List(0, _offset));
    for (var i = 0; i < tmp._stackPointers.length; i++) {
      tmp.end();
    }
    final buffer = tmp.finish();
    final bd = ByteData(buffer.lengthInBytes);
    bd.buffer.asUint8List().setAll(0, buffer);
    return bd.buffer;
  }

  void _integrityCheckOnValueAddition() {
    if (_finished) {
      throw StateError('Adding values after finish is prohibited');
    }
    if (_stackPointers.isNotEmpty && _stackPointers.last.isVector == false) {
      if (_stack.last.type != ValueType.Key) {
        throw StateError(
            'Adding value to a map before adding a key is prohibited');
      }
    }
  }

  void _integrityCheckOnKeyAddition() {
    if (_finished) {
      throw StateError('Adding values after finish is prohibited');
    }
    if (_stackPointers.isEmpty || _stackPointers.last.isVector) {
      throw StateError('Adding key before staring a map is prohibited');
    }
  }

  void _finish() {
    if (_stack.length != 1) {
      throw StateError(
          'Stack has to be exactly 1, but is ${_stack.length}. You have to end all started vectors and maps, before calling [finish]');
    }
    final value = _stack[0];
    final byteWidth = _align(value.elementWidth(_offset, 0));
    _writeStackValue(value, byteWidth);
    _writeUInt(value.storedPackedType(), 1);
    _writeUInt(byteWidth, 1);
    _finished = true;
  }

  _StackValue _createVector(int start, int vecLength, int step,
      [_StackValue? keys]) {
    var bitWidth = BitWidthUtil.uwidth(vecLength);
    var prefixElements = 1;
    if (keys != null) {
      var elemWidth = keys.elementWidth(_offset, 0);
      if (elemWidth.index > bitWidth.index) {
        bitWidth = elemWidth;
      }
      prefixElements += 2;
    }
    var vectorType = ValueType.Key;
    var typed = keys == null;
    for (var i = start; i < _stack.length; i += step) {
      final elemWidth = _stack[i].elementWidth(_offset, i + prefixElements);
      if (elemWidth.index > bitWidth.index) {
        bitWidth = elemWidth;
      }
      if (i == start) {
        vectorType = _stack[i].type;
        typed &= ValueTypeUtils.isTypedVectorElement(vectorType);
      } else {
        if (vectorType != _stack[i].type) {
          typed = false;
        }
      }
    }
    final byteWidth = _align(bitWidth);
    final fix = typed & ValueTypeUtils.isNumber(vectorType) &&
        vecLength >= 2 &&
        vecLength <= 4;
    if (keys != null) {
      _writeStackValue(keys, byteWidth);
      _writeUInt(1 << keys.width.index, byteWidth);
    }
    if (fix == false) {
      _writeUInt(vecLength, byteWidth);
    }
    final vecOffset = _offset;
    for (var i = start; i < _stack.length; i += step) {
      _writeStackValue(_stack[i], byteWidth);
    }
    if (typed == false) {
      for (var i = start; i < _stack.length; i += step) {
        _writeUInt(_stack[i].storedPackedType(), 1);
      }
    }
    if (keys != null) {
      return _StackValue.withOffset(vecOffset, ValueType.Map, bitWidth);
    }
    if (typed) {
      final vType =
          ValueTypeUtils.toTypedVector(vectorType, fix ? vecLength : 0);
      return _StackValue.withOffset(vecOffset, vType, bitWidth);
    }
    return _StackValue.withOffset(vecOffset, ValueType.Vector, bitWidth);
  }

  void _endVector(_StackPointer pointer) {
    final vecLength = _stack.length - pointer.stackPosition;
    final vec = _createVector(pointer.stackPosition, vecLength, 1);
    _stack.removeRange(pointer.stackPosition, _stack.length);
    _stack.add(vec);
  }

  void _sortKeysAndEndMap(_StackPointer pointer) {
    if (((_stack.length - pointer.stackPosition) & 1) == 1) {
      throw StateError(
          'The stack needs to hold key value pairs (even number of elements). Check if you combined [addKey] with add... method calls properly.');
    }

    var sorted = true;
    for (var i = pointer.stackPosition; i < _stack.length - 2; i += 2) {
      if (_shouldFlip(_stack[i], _stack[i + 2])) {
        sorted = false;
        break;
      }
    }

    if (sorted == false) {
      for (var i = pointer.stackPosition; i < _stack.length; i += 2) {
        var flipIndex = i;
        for (var j = i + 2; j < _stack.length; j += 2) {
          if (_shouldFlip(_stack[flipIndex], _stack[j])) {
            flipIndex = j;
          }
        }
        if (flipIndex != i) {
          var k = _stack[flipIndex];
          var v = _stack[flipIndex + 1];
          _stack[flipIndex] = _stack[i];
          _stack[flipIndex + 1] = _stack[i + 1];
          _stack[i] = k;
          _stack[i + 1] = v;
        }
      }
    }
    _endMap(pointer);
  }

  void _endMap(_StackPointer pointer) {
    final vecLength = (_stack.length - pointer.stackPosition) >> 1;
    final offsets = <int>[];
    for (var i = pointer.stackPosition; i < _stack.length; i += 2) {
      offsets.add(_stack[i].offset!);
    }
    final keysHash = _KeysHash(offsets);
    _StackValue? keysStackValue;
    if (_keyVectorCache.containsKey(keysHash)) {
      keysStackValue = _keyVectorCache[keysHash];
    } else {
      keysStackValue = _createVector(pointer.stackPosition, vecLength, 2);
      _keyVectorCache[keysHash] = keysStackValue;
    }
    final vec =
        _createVector(pointer.stackPosition + 1, vecLength, 2, keysStackValue);
    _stack.removeRange(pointer.stackPosition, _stack.length);
    _stack.add(vec);
  }

  bool _shouldFlip(_StackValue v1, _StackValue v2) {
    if (v1.type != ValueType.Key || v2.type != ValueType.Key) {
      throw StateError(
          'Stack values are not keys $v1 | $v2. Check if you combined [addKey] with add... method calls properly.');
    }

    late int c1, c2;
    var index = 0;
    do {
      c1 = _buffer.getUint8(v1.offset! + index);
      c2 = _buffer.getUint8(v2.offset! + index);
      if (c2 < c1) return true;
      if (c1 < c2) return false;
      index += 1;
    } while (c1 != 0 && c2 != 0);
    return false;
  }

  int _align(BitWidth width) {
    final byteWidth = BitWidthUtil.toByteWidth(width);
    _offset += BitWidthUtil.paddingSize(_offset, byteWidth);
    return byteWidth;
  }

  void _writeStackValue(_StackValue value, int byteWidth) {
    final newOffset = _newOffset(byteWidth);
    if (value.isOffset) {
      final relativeOffset = _offset - value.offset!;
      if (byteWidth == 8 || relativeOffset < (1 << (byteWidth * 8))) {
        _writeUInt(relativeOffset, byteWidth);
      } else {
        throw StateError(
            'Unexpected size $byteWidth. This might be a bug. Please create an issue https://github.com/google/flatbuffers/issues/new');
      }
    } else {
      _pushBuffer(value.asU8List(BitWidthUtil.fromByteWidth(byteWidth)));
    }
    _offset = newOffset;
  }

  void _writeUInt(int value, int byteWidth) {
    final newOffset = _newOffset(byteWidth);
    _pushUInt(value, BitWidthUtil.fromByteWidth(byteWidth));
    _offset = newOffset;
  }

  int _newOffset(int newValueSize) {
    final newOffset = _offset + newValueSize;
    var size = _buffer.lengthInBytes;
    final prevSize = size;
    while (size < newOffset) {
      size <<= 1;
    }
    if (prevSize < size) {
      final newBuf = ByteData(size);
      newBuf.buffer.asUint8List().setAll(0, _buffer.buffer.asUint8List());
    }
    return newOffset;
  }

  void _pushInt(int value, BitWidth width) {
    switch (width) {
      case BitWidth.width8:
        _buffer.setInt8(_offset, value);
        break;
      case BitWidth.width16:
        _buffer.setInt16(_offset, value, Endian.little);
        break;
      case BitWidth.width32:
        _buffer.setInt32(_offset, value, Endian.little);
        break;
      case BitWidth.width64:
        _buffer.setInt64(_offset, value, Endian.little);
        break;
    }
  }

  void _pushUInt(int value, BitWidth width) {
    switch (width) {
      case BitWidth.width8:
        _buffer.setUint8(_offset, value);
        break;
      case BitWidth.width16:
        _buffer.setUint16(_offset, value, Endian.little);
        break;
      case BitWidth.width32:
        _buffer.setUint32(_offset, value, Endian.little);
        break;
      case BitWidth.width64:
        _buffer.setUint64(_offset, value, Endian.little);
        break;
    }
  }

  void _pushBuffer(List<int> value) {
    _buffer.buffer.asUint8List().setAll(_offset, value);
  }
}

class _StackValue {
  late Object _value;
  int? _offset;
  final ValueType _type;
  final BitWidth _width;

  _StackValue.withNull()
      : _type = ValueType.Null,
        _width = BitWidth.width8;

  _StackValue.withInt(int value)
      : _type = ValueType.Int,
        _width = BitWidthUtil.width(value),
        _value = value;

  _StackValue.withBool(bool value)
      : _type = ValueType.Bool,
        _width = BitWidth.width8,
        _value = value;

  _StackValue.withDouble(double value)
      : _type = ValueType.Float,
        _width = BitWidthUtil.width(value),
        _value = value;

  _StackValue.withOffset(int value, ValueType type, BitWidth width)
      : _offset = value,
        _type = type,
        _width = width;

  BitWidth storedWidth({BitWidth width = BitWidth.width8}) {
    return ValueTypeUtils.isInline(_type)
        ? BitWidthUtil.max(_width, width)
        : _width;
  }

  int storedPackedType({BitWidth width = BitWidth.width8}) {
    return ValueTypeUtils.packedType(_type, storedWidth(width: width));
  }

  BitWidth elementWidth(int size, int index) {
    if (ValueTypeUtils.isInline(_type)) return _width;
    final offset = _offset!;
    for (var i = 0; i < 4; i++) {
      final width = 1 << i;
      final bitWidth = BitWidthUtil.uwidth(size +
          BitWidthUtil.paddingSize(size, width) +
          index * width -
          offset);
      if (1 << bitWidth.index == width) {
        return bitWidth;
      }
    }
    throw StateError(
        'Element is of unknown. Size: $size at index: $index. This might be a bug. Please create an issue https://github.com/google/flatbuffers/issues/new');
  }

  List<int> asU8List(BitWidth width) {
    if (ValueTypeUtils.isNumber(_type)) {
      if (_type == ValueType.Float) {
        if (width == BitWidth.width32) {
          final result = ByteData(4);
          result.setFloat32(0, _value as double, Endian.little);
          return result.buffer.asUint8List();
        } else {
          final result = ByteData(8);
          result.setFloat64(0, _value as double, Endian.little);
          return result.buffer.asUint8List();
        }
      } else {
        switch (width) {
          case BitWidth.width8:
            final result = ByteData(1);
            result.setInt8(0, _value as int);
            return result.buffer.asUint8List();
          case BitWidth.width16:
            final result = ByteData(2);
            result.setInt16(0, _value as int, Endian.little);
            return result.buffer.asUint8List();
          case BitWidth.width32:
            final result = ByteData(4);
            result.setInt32(0, _value as int, Endian.little);
            return result.buffer.asUint8List();
          case BitWidth.width64:
            final result = ByteData(8);
            result.setInt64(0, _value as int, Endian.little);
            return result.buffer.asUint8List();
        }
      }
    }
    if (_type == ValueType.Null) {
      final result = ByteData(1);
      result.setInt8(0, 0);
      return result.buffer.asUint8List();
    }
    if (_type == ValueType.Bool) {
      final result = ByteData(1);
      result.setInt8(0, _value as bool ? 1 : 0);
      return result.buffer.asUint8List();
    }

    throw StateError(
        'Unexpected type: $_type. This might be a bug. Please create an issue https://github.com/google/flatbuffers/issues/new');
  }

  ValueType get type {
    return _type;
  }

  BitWidth get width {
    return _width;
  }

  bool get isOffset {
    return !ValueTypeUtils.isInline(_type);
  }

  int? get offset => _offset;

  bool get isFloat32 {
    return _type == ValueType.Float && _width == BitWidth.width32;
  }
}

class _StackPointer {
  int stackPosition;
  bool isVector;

  _StackPointer(this.stackPosition, this.isVector);
}

class _KeysHash {
  final List<int> keys;

  const _KeysHash(this.keys);

  @override
  bool operator ==(Object other) {
    if (other is _KeysHash) {
      if (keys.length != other.keys.length) return false;
      for (var i = 0; i < keys.length; i++) {
        if (keys[i] != other.keys[i]) return false;
      }
      return true;
    }
    return false;
  }

  @override
  int get hashCode {
    var result = 17;
    for (var i = 0; i < keys.length; i++) {
      result = result * 23 + keys[i];
    }
    return result;
  }
}
