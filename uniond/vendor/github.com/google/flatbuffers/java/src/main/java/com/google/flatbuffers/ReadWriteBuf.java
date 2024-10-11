package com.google.flatbuffers;

/**
 * Interface to represent a read-write buffer. This interface will be used to access and write
 * FlexBuffers message.
 */
public interface ReadWriteBuf extends ReadBuf {

    /**
     * Clears (resets) the buffer so that it can be reused. Write position will be set to the
     * start.
     */
    void clear();

    /**
     * Put a boolean into the buffer at {@code writePosition()} . Booleans as stored as single
     * byte. Write position will be incremented.
     * @return boolean element
     */
    void putBoolean(boolean value);

    /**
     * Put an array of bytes into the buffer at {@code writePosition()}. Write position will be
     * incremented.
     * @param value the data to be copied
     * @param start initial position on value to be copied
     * @param length amount of bytes to be copied
     */
    void put (byte[] value, int start, int length);

    /**
     * Write a byte into the buffer at {@code writePosition()}. Write position will be
     * incremented.
     */
    void put(byte value);

    /**
     * Write a 16-bit into in the buffer at {@code writePosition()}. Write position will be
     * incremented.
     */
    void putShort(short value);

    /**
     * Write a 32-bit into in the buffer at {@code writePosition()}. Write position will be
     * incremented.
     */
    void putInt(int value);

    /**
     * Write a 64-bit into in the buffer at {@code writePosition()}. Write position will be
     * incremented.
     */
    void putLong(long value);

    /**
     * Write a 32-bit float into the buffer at {@code writePosition()}. Write position will be
     * incremented.
     */
    void putFloat(float value);

    /**
     * Write a 64-bit float into the buffer at {@code writePosition()}. Write position will be
     * incremented.
     */
    void putDouble(double value);

    /**
     * Write boolean into a given position on the buffer. Booleans as stored as single byte.
     * @param index position of the element in buffer
     */
    void setBoolean(int index, boolean value);

    /**
     * Read a byte from data.
     * @param index position of the element in the buffer
     * @return a byte
     */
    void set(int index, byte value);

    /**
     * Write an array of bytes into the buffer.
     * @param index initial position of the buffer to be written
     * @param value the data to be copied
     * @param start initial position on value to be copied
     * @param length amount of bytes to be copied
     */
    void set(int index, byte[] value, int start, int length);

    /**
     * Read a short from data.
     * @param index position of the element in ReadBuf
     * @return a short
     */
    void setShort(int index, short value);

    /**
     * Read a 32-bit int from data.
     * @param index position of the element in ReadBuf
     * @return an int
     */
    void setInt(int index, int value);

    /**
     * Read a 64-bit long from data.
     * @param index position of the element in ReadBuf
     * @return a long
     */
    void setLong(int index, long value);

    /**
     * Read a 32-bit float from data.
     * @param index position of the element in ReadBuf
     * @return a float
     */
    void setFloat(int index, float value);

    /**
     * Read a 64-bit float from data.
     * @param index position of the element in ReadBuf
     * @return a double
     */
    void setDouble(int index, double value);


    int writePosition();
    /**
     * Defines the size of the message in the buffer. It also determines last position that buffer
     * can be read or write. Last byte to be accessed is in position {@code limit() -1}.
     * @return indicate last position
     */
    int limit();

    /**
     * Request capacity of the buffer. In case buffer is already larger
     * than the requested, this method will just return true. Otherwise
     * It might try to resize the buffer.
     *
     * @return true if buffer is able to offer
     * the requested capacity
     */
    boolean requestCapacity(int capacity);
}
