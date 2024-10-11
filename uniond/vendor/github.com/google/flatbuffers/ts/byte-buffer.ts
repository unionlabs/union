import { FILE_IDENTIFIER_LENGTH, SIZEOF_INT } from "./constants.js";
import { int32, isLittleEndian, float32, float64 } from "./utils.js";
import { Offset, Table, IGeneratedObject, IUnpackableObject } from "./types.js";
import { Encoding } from "./encoding.js";

export class ByteBuffer {
    private position_ = 0;
    private text_decoder_ = new TextDecoder();
  
    /**
     * Create a new ByteBuffer with a given array of bytes (`Uint8Array`)
     */
    constructor(private bytes_: Uint8Array) { }
  
    /**
     * Create and allocate a new ByteBuffer with a given size.
     */
    static allocate(byte_size: number): ByteBuffer {
      return new ByteBuffer(new Uint8Array(byte_size));
    }
  
    clear(): void {
      this.position_ = 0;
    }
  
    /**
     * Get the underlying `Uint8Array`.
     */
    bytes(): Uint8Array {
      return this.bytes_;
    }
  
    /**
     * Get the buffer's position.
     */
    position(): number {
      return this.position_;
    }
  
    /**
     * Set the buffer's position.
     */
    setPosition(position: number): void {
      this.position_ = position;
    }
  
    /**
     * Get the buffer's capacity.
     */
    capacity(): number {
      return this.bytes_.length;
    }
  
    readInt8(offset: number): number {
      return this.readUint8(offset) << 24 >> 24;
    }
  
    readUint8(offset: number): number {
      return this.bytes_[offset];
    }
  
    readInt16(offset: number): number {
      return this.readUint16(offset) << 16 >> 16;
    }
  
    readUint16(offset: number): number {
      return this.bytes_[offset] | this.bytes_[offset + 1] << 8;
    }
  
    readInt32(offset: number): number {
      return this.bytes_[offset] | this.bytes_[offset + 1] << 8 | this.bytes_[offset + 2] << 16 | this.bytes_[offset + 3] << 24;
    }
  
    readUint32(offset: number): number {
      return this.readInt32(offset) >>> 0;
    }
  
    readInt64(offset: number): bigint {
      return BigInt.asIntN(64, BigInt(this.readUint32(offset)) + (BigInt(this.readUint32(offset + 4)) << BigInt(32)));
    }
  
    readUint64(offset: number): bigint {
      return BigInt.asUintN(64, BigInt(this.readUint32(offset)) + (BigInt(this.readUint32(offset + 4)) << BigInt(32)));
    }
  
    readFloat32(offset: number): number {
      int32[0] = this.readInt32(offset);
      return float32[0];
    }
  
    readFloat64(offset: number): number {
      int32[isLittleEndian ? 0 : 1] = this.readInt32(offset);
      int32[isLittleEndian ? 1 : 0] = this.readInt32(offset + 4);
      return float64[0];
    }
  
    writeInt8(offset: number, value: number): void {
      this.bytes_[offset] = value;
    }
  
    writeUint8(offset: number, value: number): void {
      this.bytes_[offset] = value;
    }
  
    writeInt16(offset: number, value: number): void {
      this.bytes_[offset] = value;
      this.bytes_[offset + 1] = value >> 8;
    }
  
    writeUint16(offset: number, value: number): void {
      this.bytes_[offset] = value;
      this.bytes_[offset + 1] = value >> 8;
    }
  
    writeInt32(offset: number, value: number): void {
      this.bytes_[offset] = value;
      this.bytes_[offset + 1] = value >> 8;
      this.bytes_[offset + 2] = value >> 16;
      this.bytes_[offset + 3] = value >> 24;
    }
  
    writeUint32(offset: number, value: number): void {
      this.bytes_[offset] = value;
      this.bytes_[offset + 1] = value >> 8;
      this.bytes_[offset + 2] = value >> 16;
      this.bytes_[offset + 3] = value >> 24;
    }
  
    writeInt64(offset: number, value: bigint): void {
      this.writeInt32(offset, Number(BigInt.asIntN(32, value)));
      this.writeInt32(offset + 4, Number(BigInt.asIntN(32, value >> BigInt(32))));
    }
  
    writeUint64(offset: number, value: bigint): void {
      this.writeUint32(offset, Number(BigInt.asUintN(32, value)));
      this.writeUint32(offset + 4, Number(BigInt.asUintN(32, value >> BigInt(32))));
    }
  
    writeFloat32(offset: number, value: number): void {
      float32[0] = value;
      this.writeInt32(offset, int32[0]);
    }
  
    writeFloat64(offset: number, value: number): void {
      float64[0] = value;
      this.writeInt32(offset, int32[isLittleEndian ? 0 : 1]);
      this.writeInt32(offset + 4, int32[isLittleEndian ? 1 : 0]);
    }
  
    /**
     * Return the file identifier.   Behavior is undefined for FlatBuffers whose
     * schema does not include a file_identifier (likely points at padding or the
     * start of a the root vtable).
     */
    getBufferIdentifier(): string {
      if (this.bytes_.length < this.position_ + SIZEOF_INT +
          FILE_IDENTIFIER_LENGTH) {
        throw new Error(
            'FlatBuffers: ByteBuffer is too short to contain an identifier.');
      }
      let result = "";
      for (let i = 0; i < FILE_IDENTIFIER_LENGTH; i++) {
        result += String.fromCharCode(
            this.readInt8(this.position_ + SIZEOF_INT + i));
      }
      return result;
    }
  
    /**
     * Look up a field in the vtable, return an offset into the object, or 0 if the
     * field is not present.
     */
    __offset(bb_pos: number, vtable_offset: number): Offset {
      const vtable = bb_pos - this.readInt32(bb_pos);
      return vtable_offset < this.readInt16(vtable) ? this.readInt16(vtable + vtable_offset) : 0;
    }
  
    /**
     * Initialize any Table-derived type to point to the union at the given offset.
     */
    __union(t: Table, offset: number): Table {
      t.bb_pos = offset + this.readInt32(offset);
      t.bb = this;
      return t;
    }
  
    /**
     * Create a JavaScript string from UTF-8 data stored inside the FlatBuffer.
     * This allocates a new string and converts to wide chars upon each access.
     *
     * To avoid the conversion to string, pass Encoding.UTF8_BYTES as the
     * "optionalEncoding" argument. This is useful for avoiding conversion when
     * the data will just be packaged back up in another FlatBuffer later on.
     *
     * @param offset
     * @param opt_encoding Defaults to UTF16_STRING
     */
    __string(offset: number, opt_encoding?: Encoding): string | Uint8Array {
      offset += this.readInt32(offset);
      const length = this.readInt32(offset);
      offset += SIZEOF_INT;
      const utf8bytes = this.bytes_.subarray(offset, offset + length);
      if (opt_encoding === Encoding.UTF8_BYTES)
        return utf8bytes;
      else
        return this.text_decoder_.decode(utf8bytes);
    }
  
    /**
     * Handle unions that can contain string as its member, if a Table-derived type then initialize it, 
     * if a string then return a new one
     * 
     * WARNING: strings are immutable in JS so we can't change the string that the user gave us, this 
     * makes the behaviour of __union_with_string different compared to __union
     */
    __union_with_string(o: Table | string, offset: number) : Table | string {
      if(typeof o === 'string') {
        return this.__string(offset) as string;
      } 
      return this.__union(o, offset);
    }
  
    /**
     * Retrieve the relative offset stored at "offset"
     */
    __indirect(offset: Offset): Offset {
      return offset + this.readInt32(offset);
    }
  
    /**
     * Get the start of data of a vector whose offset is stored at "offset" in this object.
     */
    __vector(offset: Offset): Offset {
      return offset + this.readInt32(offset) + SIZEOF_INT; // data starts after the length
    }
  
    /**
     * Get the length of a vector whose offset is stored at "offset" in this object.
     */
    __vector_len(offset: Offset): Offset {
      return this.readInt32(offset + this.readInt32(offset));
    }
  
    __has_identifier(ident: string): boolean {
      if (ident.length != FILE_IDENTIFIER_LENGTH) {
        throw new Error('FlatBuffers: file identifier must be length ' +
                        FILE_IDENTIFIER_LENGTH);
      }
      for (let i = 0; i < FILE_IDENTIFIER_LENGTH; i++) {
        if (ident.charCodeAt(i) != this.readInt8(this.position() + SIZEOF_INT + i)) {
          return false;
        }
      }
      return true;
    }

    /**
     * A helper function for generating list for obj api
     */
    createScalarList<T>(listAccessor: (i: number) => T | null, listLength: number): T[] {
      const ret: T[]  = [];
      for(let i = 0; i < listLength; ++i) {
        const val = listAccessor(i);
        if(val !== null) {
          ret.push(val);
        }
      }
      return ret;
    }

    /**
     * A helper function for generating list for obj api
     * @param listAccessor function that accepts an index and return data at that index
     * @param listLength listLength
     * @param res result list
     */
    createObjList<T1 extends IUnpackableObject<T2>, T2 extends IGeneratedObject>(listAccessor: (i: number) => T1 | null, listLength: number): T2[] {
      const ret: T2[] = [];
      for(let i = 0; i < listLength; ++i) {
        const val = listAccessor(i);
        if(val !== null) {
          ret.push(val.unpack());
        }
      }
      return ret;
    }
  }
