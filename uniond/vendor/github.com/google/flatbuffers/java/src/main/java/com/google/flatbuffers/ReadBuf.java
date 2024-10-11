package com.google.flatbuffers;

/**
 *  Represent a chunk of data, where FlexBuffers will read from.
 */
public interface ReadBuf {

  /**
   * Read boolean from data. Booleans as stored as single byte
   * @param index position of the element in ReadBuf
   * @return boolean element
   */
  boolean getBoolean(int index);

  /**
   * Read a byte from data.
   * @param index position of the element in ReadBuf
   * @return a byte
   */
  byte get(int index);

  /**
   * Read a short from data.
   * @param index position of the element in ReadBuf
   * @return a short
   */
  short getShort(int index);

  /**
   * Read a 32-bit int from data.
   * @param index position of the element in ReadBuf
   * @return an int
   */
  int getInt(int index);

  /**
   * Read a 64-bit long from data.
   * @param index position of the element in ReadBuf
   * @return a long
   */
  long getLong(int index);

  /**
   * Read a 32-bit float from data.
   * @param index position of the element in ReadBuf
   * @return a float
   */
  float getFloat(int index);

  /**
   * Read a 64-bit float from data.
   * @param index position of the element in ReadBuf
   * @return a double
   */
  double getDouble(int index);

  /**
   * Read an UTF-8 string from data.
   * @param start initial element of the string
   * @param size size of the string in bytes.
   * @return a {@code String}
   */
  String getString(int start, int size);

  /**
   * Expose ReadBuf as an array of bytes.
   * This method is meant to be as efficient as possible, so for a array-backed ReadBuf, it should
   * return its own internal data. In case access to internal data is not possible,
   * a copy of the data into an array of bytes might occur.
   * @return ReadBuf as an array of bytes
   */
  byte[] data();

  /**
   * Defines the size of the message in the buffer. It also determines last position that buffer
   * can be read. Last byte to be accessed is in position {@code limit() -1}.
   * @return indicate last position
   */
  int limit();

}
