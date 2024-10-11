package com.google.flatbuffers;

import java.util.Arrays;

/**
 * Implements {@code ReadBuf} using an array of bytes
 * as a backing storage. Using array of bytes are
 * usually faster than {@code ByteBuffer}.
 *
 * This class is not thread-safe, meaning that
 * it must operate on a single thread. Operating from
 * multiple thread leads into a undefined behavior
 */
public class ArrayReadWriteBuf implements ReadWriteBuf {

  private byte[] buffer;
  private int writePos;

  public ArrayReadWriteBuf() {
    this(10);
  }

  public ArrayReadWriteBuf(int initialCapacity) {
    this(new byte[initialCapacity]);
  }

  public ArrayReadWriteBuf(byte[] buffer) {
    this.buffer = buffer;
    this.writePos = 0;
  }

  public ArrayReadWriteBuf(byte[] buffer, int startPos) {
    this.buffer = buffer;
    this.writePos = startPos;
  }

  @Override
  public void clear() {
    this.writePos = 0;
  }

  @Override
  public boolean getBoolean(int index) {
    return buffer[index] != 0;
  }

  @Override
  public byte get(int index) {
    return buffer[index];
  }

  @Override
  public short getShort(int index) {
    return (short) ((buffer[index+ 1] << 8) | (buffer[index] & 0xff));
  }

  @Override
  public int getInt(int index) {
    return (((buffer[index + 3]) << 24) |
      ((buffer[index + 2] & 0xff) << 16) |
      ((buffer[index + 1] & 0xff) << 8) |
      ((buffer[index] & 0xff)));
  }

  @Override
  public long getLong(int index) {
    return ((((long) buffer[index++] & 0xff)) |
      (((long) buffer[index++] & 0xff) << 8) |
      (((long) buffer[index++] & 0xff) << 16) |
      (((long) buffer[index++] & 0xff) << 24) |
      (((long) buffer[index++] & 0xff) << 32) |
      (((long) buffer[index++] & 0xff) << 40) |
      (((long) buffer[index++] & 0xff) << 48) |
      (((long) buffer[index]) << 56));
  }

  @Override
  public float getFloat(int index) {
    return Float.intBitsToFloat(getInt(index));
  }

  @Override
  public double getDouble(int index) {
    return Double.longBitsToDouble(getLong(index));
  }

  @Override
  public String getString(int start, int size) {
    return Utf8Safe.decodeUtf8Array(buffer, start, size);
  }

  @Override
  public byte[] data() {
    return buffer;
  }


  @Override
  public void putBoolean(boolean value) {
      setBoolean(writePos, value);
      writePos++;
  }

  @Override
  public void put(byte[] value, int start, int length) {
    set(writePos, value, start, length);
    writePos+=length;
  }

  @Override
  public void put(byte value) {
    set(writePos, value);
    writePos++;
  }

  @Override
  public void putShort(short value) {
    setShort(writePos, value);
    writePos +=2;
  }

  @Override
  public void putInt(int value) {
    setInt(writePos, value);
    writePos +=4;
  }

  @Override
  public void putLong(long value) {
    setLong(writePos, value);
    writePos +=8;
  }

  @Override
  public void putFloat(float value) {
    setFloat(writePos, value);
    writePos +=4;
  }

  @Override
  public void putDouble(double value) {
    setDouble(writePos, value);
    writePos +=8;
  }

  @Override
  public void setBoolean(int index, boolean value) {
    set(index, value ? (byte)1 : (byte)0);
  }

  @Override
  public void set(int index, byte value) {
    requestCapacity(index + 1);
    buffer[index] = value;
  }

  @Override
  public void set(int index, byte[] toCopy, int start, int length) {
    requestCapacity(index + (length - start));
    System.arraycopy(toCopy, start, buffer, index, length);
  }

  @Override
  public void setShort(int index, short value) {
    requestCapacity(index + 2);

    buffer[index++] = (byte) ((value) & 0xff);
    buffer[index  ] = (byte) ((value >> 8) & 0xff);
  }

  @Override
  public void setInt(int index, int value) {
    requestCapacity(index + 4);

    buffer[index++] = (byte) ((value) & 0xff);
    buffer[index++] = (byte) ((value >>  8) & 0xff);
    buffer[index++] = (byte) ((value >> 16) & 0xff);
    buffer[index  ] = (byte) ((value >> 24) & 0xff);
  }

  @Override
  public void setLong(int index, long value) {
    requestCapacity(index + 8);

    int i = (int) value;
    buffer[index++] = (byte) ((i) & 0xff);
    buffer[index++] = (byte) ((i >>  8) & 0xff);
    buffer[index++] = (byte) ((i >> 16) & 0xff);
    buffer[index++] = (byte) ((i >> 24) & 0xff);
    i = (int) (value >> 32);
    buffer[index++] = (byte) ((i) & 0xff);
    buffer[index++] = (byte) ((i >>  8) & 0xff);
    buffer[index++] = (byte) ((i >> 16) & 0xff);
    buffer[index  ] = (byte) ((i >> 24) & 0xff);
  }

  @Override
  public void setFloat(int index, float value) {
    requestCapacity(index + 4);

    int iValue = Float.floatToRawIntBits(value);
    buffer[index++] = (byte) ((iValue) & 0xff);
    buffer[index++] = (byte) ((iValue >>  8) & 0xff);
    buffer[index++] = (byte) ((iValue >> 16) & 0xff);
    buffer[index  ] = (byte) ((iValue >> 24) & 0xff);
  }

  @Override
  public void setDouble(int index, double value) {
    requestCapacity(index + 8);

    long lValue = Double.doubleToRawLongBits(value);
    int i = (int) lValue;
    buffer[index++] = (byte) ((i) & 0xff);
    buffer[index++] = (byte) ((i >>  8) & 0xff);
    buffer[index++] = (byte) ((i >> 16) & 0xff);
    buffer[index++] = (byte) ((i >> 24) & 0xff);
    i = (int) (lValue >> 32);
    buffer[index++] = (byte) ((i) & 0xff);
    buffer[index++] = (byte) ((i >>  8) & 0xff);
    buffer[index++] = (byte) ((i >> 16) & 0xff);
    buffer[index  ] = (byte) ((i >> 24) & 0xff);
  }

  @Override
  public int limit() {
    return writePos;
  }

  @Override
  public int writePosition() {
    return writePos;
  }

  @Override
  public boolean requestCapacity(int capacity) {
    if (capacity < 0) {
      throw new IllegalArgumentException("Capacity may not be negative (likely a previous int overflow)");
    }
    if (buffer.length >= capacity) {
      return true;
    }
    // implemented in the same growing fashion as ArrayList
    int oldCapacity = buffer.length;
    int newCapacity = oldCapacity + (oldCapacity >> 1);
    if (newCapacity < capacity) {  // Note: this also catches newCapacity int overflow
      newCapacity = capacity;
    }
    buffer = Arrays.copyOf(buffer, newCapacity);
    return true;
  }
}
