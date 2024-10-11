package com.google.flatbuffers;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;

public class ByteBufferReadWriteBuf implements ReadWriteBuf {

  private final ByteBuffer buffer;

  public ByteBufferReadWriteBuf(ByteBuffer bb) {
    this.buffer = bb;
    this.buffer.order(ByteOrder.LITTLE_ENDIAN);
  }

  @Override
  public void clear() {
    buffer.clear();
  }

  @Override
  public boolean getBoolean(int index) {
    return get(index) != 0;
  }

  @Override
  public byte get(int index) {
    return buffer.get(index);
  }

  @Override
  public short getShort(int index) {
    return buffer.getShort(index);
  }

  @Override
  public int getInt(int index) {
    return buffer.getInt(index);
  }

  @Override
  public long getLong(int index) {
    return buffer.getLong(index);
  }

  @Override
  public float getFloat(int index) {
    return buffer.getFloat(index);
  }

  @Override
  public double getDouble(int index) {
    return buffer.getDouble(index);
  }

  @Override
  public String getString(int start, int size) {
    return Utf8Safe.decodeUtf8Buffer(buffer, start, size);
  }

  @Override
  public byte[] data() {
    return buffer.array();
  }

  @Override
  public void putBoolean(boolean value) {
    buffer.put(value ? (byte)1 : (byte)0);
  }

  @Override
  public void put(byte[] value, int start, int length) {
    buffer.put(value, start, length);
  }

  @Override
  public void put(byte value) {
    buffer.put(value);
  }

  @Override
  public void putShort(short value) {
    buffer.putShort(value);
  }

  @Override
  public void putInt(int value) {
    buffer.putInt(value);
  }

  @Override
  public void putLong(long value) {
    buffer.putLong(value);
  }

  @Override
  public void putFloat(float value) {
    buffer.putFloat(value);
  }

  @Override
  public void putDouble(double value) {
    buffer.putDouble(value);
  }

  @Override
  public void setBoolean(int index, boolean value) {
    set(index, value ? (byte)1 : (byte)0);
  }

  @Override
  public void set(int index, byte value) {
    requestCapacity(index + 1);
    buffer.put(index, value);
  }

  @Override
  public void set(int index, byte[] value, int start, int length) {
    requestCapacity(index + (length - start));
    int curPos = buffer.position();
    buffer.position(index);
    buffer.put(value, start, length);
    buffer.position(curPos);
  }

  @Override
  public void setShort(int index, short value) {
    requestCapacity(index + 2);
    buffer.putShort(index, value);
  }

  @Override
  public void setInt(int index, int value) {
    requestCapacity(index + 4);
    buffer.putInt(index, value);
  }

  @Override
  public void setLong(int index, long value) {
    requestCapacity(index + 8);
    buffer.putLong(index, value);
  }

  @Override
  public void setFloat(int index, float value) {
    requestCapacity(index + 4);
    buffer.putFloat(index, value);
  }

  @Override
  public void setDouble(int index, double value) {
    requestCapacity(index + 8);
    buffer.putDouble(index, value);
  }

  @Override
  public int writePosition() {
    return buffer.position();
  }

  @Override
  public int limit() {
    return buffer.limit();
  }

  @Override
  public boolean requestCapacity(int capacity) {
    return capacity <= buffer.limit();
  }

}
