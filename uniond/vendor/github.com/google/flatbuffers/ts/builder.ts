import { ByteBuffer } from "./byte-buffer.js"
import { SIZEOF_SHORT, SIZE_PREFIX_LENGTH, SIZEOF_INT, FILE_IDENTIFIER_LENGTH } from "./constants.js"
import { Offset, IGeneratedObject } from "./types.js"

export class Builder {
    private bb: ByteBuffer
    /** Remaining space in the ByteBuffer. */
    private space: number
    /** Minimum alignment encountered so far. */
    private minalign = 1
    /** The vtable for the current table. */
    private vtable: number[] | null = null
    /** The amount of fields we're actually using. */
    private vtable_in_use = 0
    /** Whether we are currently serializing a table. */
    private isNested = false;
    /** Starting offset of the current struct/table. */
    private object_start = 0
    /** List of offsets of all vtables. */
    private vtables: number[] = []
    /** For the current vector being built. */
    private vector_num_elems = 0 
    /** False omits default values from the serialized data */
    private force_defaults = false;
    
    private string_maps: Map<string | Uint8Array, number> | null = null;
    private text_encoder = new TextEncoder();
  
    /**
     * Create a FlatBufferBuilder.
     */
    constructor(opt_initial_size?: number) {
      let initial_size: number;
  
      if (!opt_initial_size) {
        initial_size = 1024;
      } else {
        initial_size = opt_initial_size;
      }
  
      /**
       * @type {ByteBuffer}
       * @private
       */
      this.bb = ByteBuffer.allocate(initial_size);
      this.space = initial_size;
    }
  
  
    clear(): void {
      this.bb.clear();
      this.space = this.bb.capacity();
      this.minalign = 1;
      this.vtable = null;
      this.vtable_in_use = 0;
      this.isNested = false;
      this.object_start = 0;
      this.vtables = [];
      this.vector_num_elems = 0;
      this.force_defaults = false;
      this.string_maps = null;
    }
  
    /**
     * In order to save space, fields that are set to their default value
     * don't get serialized into the buffer. Forcing defaults provides a
     * way to manually disable this optimization.
     *
     * @param forceDefaults true always serializes default values
     */
    forceDefaults(forceDefaults: boolean): void {
      this.force_defaults = forceDefaults;
    }
  
    /**
     * Get the ByteBuffer representing the FlatBuffer. Only call this after you've
     * called finish(). The actual data starts at the ByteBuffer's current position,
     * not necessarily at 0.
     */
    dataBuffer(): ByteBuffer {
      return this.bb;
    }
  
    /**
     * Get the bytes representing the FlatBuffer. Only call this after you've
     * called finish().
     */
    asUint8Array(): Uint8Array {
      return this.bb.bytes().subarray(this.bb.position(), this.bb.position() + this.offset());
    }
  
    /**
     * Prepare to write an element of `size` after `additional_bytes` have been
     * written, e.g. if you write a string, you need to align such the int length
     * field is aligned to 4 bytes, and the string data follows it directly. If all
     * you need to do is alignment, `additional_bytes` will be 0.
     *
     * @param size This is the of the new element to write
     * @param additional_bytes The padding size
     */
    prep(size: number, additional_bytes: number): void {
      // Track the biggest thing we've ever aligned to.
      if (size > this.minalign) {
        this.minalign = size;
      }
  
      // Find the amount of alignment needed such that `size` is properly
      // aligned after `additional_bytes`
      const align_size = ((~(this.bb.capacity() - this.space + additional_bytes)) + 1) & (size - 1);
  
      // Reallocate the buffer if needed.
      while (this.space < align_size + size + additional_bytes) {
        const old_buf_size = this.bb.capacity();
        this.bb = Builder.growByteBuffer(this.bb);
        this.space += this.bb.capacity() - old_buf_size;
      }
  
      this.pad(align_size);
    }
  
    pad(byte_size: number): void {
      for (let i = 0; i < byte_size; i++) {
        this.bb.writeInt8(--this.space, 0);
      }
    }
  
    writeInt8(value: number): void {
      this.bb.writeInt8(this.space -= 1, value);
    }
  
    writeInt16(value: number): void {
      this.bb.writeInt16(this.space -= 2, value);
    }
  
    writeInt32(value: number): void {
      this.bb.writeInt32(this.space -= 4, value);
    }
  
    writeInt64(value: bigint): void {
      this.bb.writeInt64(this.space -= 8, value);
    }
  
    writeFloat32(value: number): void {
      this.bb.writeFloat32(this.space -= 4, value);
    }
  
    writeFloat64(value: number): void {
      this.bb.writeFloat64(this.space -= 8, value);
    }
  
    /**
     * Add an `int8` to the buffer, properly aligned, and grows the buffer (if necessary).
     * @param value The `int8` to add the buffer.
     */
    addInt8(value: number): void {
      this.prep(1, 0);
      this.writeInt8(value);
    }
  
    /**
     * Add an `int16` to the buffer, properly aligned, and grows the buffer (if necessary).
     * @param value The `int16` to add the buffer.
     */
    addInt16(value: number): void {
      this.prep(2, 0);
      this.writeInt16(value);
    }
  
    /**
     * Add an `int32` to the buffer, properly aligned, and grows the buffer (if necessary).
     * @param value The `int32` to add the buffer.
     */
    addInt32(value: number): void {
      this.prep(4, 0);
      this.writeInt32(value);
    }
  
    /**
     * Add an `int64` to the buffer, properly aligned, and grows the buffer (if necessary).
     * @param value The `int64` to add the buffer.
     */
    addInt64(value: bigint): void {
      this.prep(8, 0);
      this.writeInt64(value);
    }
  
    /**
     * Add a `float32` to the buffer, properly aligned, and grows the buffer (if necessary).
     * @param value The `float32` to add the buffer.
     */
    addFloat32(value: number): void {
      this.prep(4, 0);
      this.writeFloat32(value);
    }
  
    /**
     * Add a `float64` to the buffer, properly aligned, and grows the buffer (if necessary).
     * @param value The `float64` to add the buffer.
     */
    addFloat64(value: number): void {
      this.prep(8, 0);
      this.writeFloat64(value);
    }
  
    addFieldInt8(voffset: number, value: number, defaultValue: number|null): void {
      if (this.force_defaults || value != defaultValue) {
        this.addInt8(value);
        this.slot(voffset);
      }
    }
  
    addFieldInt16(voffset: number, value: number, defaultValue: number|null): void {
      if (this.force_defaults || value != defaultValue) {
        this.addInt16(value);
        this.slot(voffset);
      }
    }
  
    addFieldInt32(voffset: number, value: number, defaultValue: number|null): void {
      if (this.force_defaults || value != defaultValue) {
        this.addInt32(value);
        this.slot(voffset);
      }
    }
  
    addFieldInt64(voffset: number, value: bigint, defaultValue: bigint|null): void {
      if (this.force_defaults || value !== defaultValue) {
        this.addInt64(value);
        this.slot(voffset);
      }
    }
  
    addFieldFloat32(voffset: number, value: number, defaultValue: number|null): void {
      if (this.force_defaults || value != defaultValue) {
        this.addFloat32(value);
        this.slot(voffset);
      }
    }
  
    addFieldFloat64(voffset: number, value: number, defaultValue: number|null): void {
      if (this.force_defaults || value != defaultValue) {
        this.addFloat64(value);
        this.slot(voffset);
      }
    }
  
    addFieldOffset(voffset: number, value: Offset, defaultValue: Offset): void {
      if (this.force_defaults || value != defaultValue) {
        this.addOffset(value);
        this.slot(voffset);
      }
    }
  
    /**
     * Structs are stored inline, so nothing additional is being added. `d` is always 0.
     */
    addFieldStruct(voffset: number, value: Offset, defaultValue: Offset): void {
      if (value != defaultValue) {
        this.nested(value);
        this.slot(voffset);
      }
    }
  
    /**
     * Structures are always stored inline, they need to be created right
     * where they're used.  You'll get this assertion failure if you
     * created it elsewhere.
     */
    nested(obj: Offset): void {
      if (obj != this.offset()) {
        throw new TypeError('FlatBuffers: struct must be serialized inline.');
      }
    }
  
    /**
     * Should not be creating any other object, string or vector
     * while an object is being constructed
     */
    notNested(): void {
      if (this.isNested) {
        throw new TypeError('FlatBuffers: object serialization must not be nested.');
      }
    }
  
    /**
     * Set the current vtable at `voffset` to the current location in the buffer.
     */
    slot(voffset: number): void {
      if (this.vtable !== null)
        this.vtable[voffset] = this.offset();
    }
  
    /**
     * @returns Offset relative to the end of the buffer.
     */
    offset(): Offset {
      return this.bb.capacity() - this.space;
    }
  
    /**
     * Doubles the size of the backing ByteBuffer and copies the old data towards
     * the end of the new buffer (since we build the buffer backwards).
     *
     * @param bb The current buffer with the existing data
     * @returns A new byte buffer with the old data copied
     * to it. The data is located at the end of the buffer.
     *
     * uint8Array.set() formally takes {Array<number>|ArrayBufferView}, so to pass
     * it a uint8Array we need to suppress the type check:
     * @suppress {checkTypes}
     */
    static growByteBuffer(bb: ByteBuffer): ByteBuffer {
      const old_buf_size = bb.capacity();
  
      // Ensure we don't grow beyond what fits in an int.
      if (old_buf_size & 0xC0000000) {
        throw new Error('FlatBuffers: cannot grow buffer beyond 2 gigabytes.');
      }
  
      const new_buf_size = old_buf_size << 1;
      const nbb = ByteBuffer.allocate(new_buf_size);
      nbb.setPosition(new_buf_size - old_buf_size);
      nbb.bytes().set(bb.bytes(), new_buf_size - old_buf_size);
      return nbb;
    }
  
    /**
     * Adds on offset, relative to where it will be written.
     *
     * @param offset The offset to add.
     */
    addOffset(offset: Offset): void {
      this.prep(SIZEOF_INT, 0); // Ensure alignment is already done.
      this.writeInt32(this.offset() - offset + SIZEOF_INT);
    }
  
    /**
     * Start encoding a new object in the buffer.  Users will not usually need to
     * call this directly. The FlatBuffers compiler will generate helper methods
     * that call this method internally.
     */
    startObject(numfields: number): void {
      this.notNested();
      if (this.vtable == null) {
        this.vtable = [];
      }
      this.vtable_in_use = numfields;
      for (let i = 0; i < numfields; i++) {
        this.vtable[i] = 0; // This will push additional elements as needed
      }
      this.isNested = true;
      this.object_start = this.offset();
    }
  
    /**
     * Finish off writing the object that is under construction.
     *
     * @returns The offset to the object inside `dataBuffer`
     */
    endObject(): Offset {
      if (this.vtable == null || !this.isNested) {
        throw new Error('FlatBuffers: endObject called without startObject');
      }
  
      this.addInt32(0);
      const vtableloc = this.offset();
  
      // Trim trailing zeroes.
      let i = this.vtable_in_use - 1;
      // eslint-disable-next-line no-empty
      for (; i >= 0 && this.vtable[i] == 0; i--) {}
      const trimmed_size = i + 1;
  
      // Write out the current vtable.
      for (; i >= 0; i--) {
        // Offset relative to the start of the table.
        this.addInt16(this.vtable[i] != 0 ? vtableloc - this.vtable[i] : 0);
      }
  
      const standard_fields = 2; // The fields below:
      this.addInt16(vtableloc - this.object_start);
      const len = (trimmed_size + standard_fields) * SIZEOF_SHORT;
      this.addInt16(len);
  
      // Search for an existing vtable that matches the current one.
      let existing_vtable = 0;
      const vt1 = this.space;
    outer_loop:
      for (i = 0; i < this.vtables.length; i++) {
        const vt2 = this.bb.capacity() - this.vtables[i];
        if (len == this.bb.readInt16(vt2)) {
          for (let j = SIZEOF_SHORT; j < len; j += SIZEOF_SHORT) {
            if (this.bb.readInt16(vt1 + j) != this.bb.readInt16(vt2 + j)) {
              continue outer_loop;
            }
          }
          existing_vtable = this.vtables[i];
          break;
        }
      }
  
      if (existing_vtable) {
        // Found a match:
        // Remove the current vtable.
        this.space = this.bb.capacity() - vtableloc;
  
        // Point table to existing vtable.
        this.bb.writeInt32(this.space, existing_vtable - vtableloc);
      } else {
        // No match:
        // Add the location of the current vtable to the list of vtables.
        this.vtables.push(this.offset());
  
        // Point table to current vtable.
        this.bb.writeInt32(this.bb.capacity() - vtableloc, this.offset() - vtableloc);
      }
  
      this.isNested = false;
      return vtableloc as Offset;
    }
  
    /**
     * Finalize a buffer, poiting to the given `root_table`.
     */
    finish(root_table: Offset, opt_file_identifier?: string, opt_size_prefix?: boolean): void {
      const size_prefix = opt_size_prefix ? SIZE_PREFIX_LENGTH : 0;
      if (opt_file_identifier) {
        const file_identifier = opt_file_identifier;
        this.prep(this.minalign, SIZEOF_INT +
          FILE_IDENTIFIER_LENGTH + size_prefix);
        if (file_identifier.length != FILE_IDENTIFIER_LENGTH) {
          throw new TypeError('FlatBuffers: file identifier must be length ' +
            FILE_IDENTIFIER_LENGTH);
        }
        for (let i = FILE_IDENTIFIER_LENGTH - 1; i >= 0; i--) {
          this.writeInt8(file_identifier.charCodeAt(i));
        }
      }
      this.prep(this.minalign, SIZEOF_INT + size_prefix);
      this.addOffset(root_table);
      if (size_prefix) {
        this.addInt32(this.bb.capacity() - this.space);
      }
      this.bb.setPosition(this.space);
    }
  
    /**
     * Finalize a size prefixed buffer, pointing to the given `root_table`.
     */
    finishSizePrefixed(this: Builder, root_table: Offset, opt_file_identifier?: string): void {
      this.finish(root_table, opt_file_identifier, true);
    }
  
    /**
     * This checks a required field has been set in a given table that has
     * just been constructed.
     */
    requiredField(table: Offset, field: number): void {
      const table_start = this.bb.capacity() - table;
      const vtable_start = table_start - this.bb.readInt32(table_start);
      const ok = field < this.bb.readInt16(vtable_start) &&
                 this.bb.readInt16(vtable_start + field) != 0;
  
      // If this fails, the caller will show what field needs to be set.
      if (!ok) {
        throw new TypeError('FlatBuffers: field ' + field + ' must be set');
      }
    }
  
    /**
     * Start a new array/vector of objects.  Users usually will not call
     * this directly. The FlatBuffers compiler will create a start/end
     * method for vector types in generated code.
     *
     * @param elem_size The size of each element in the array
     * @param num_elems The number of elements in the array
     * @param alignment The alignment of the array
     */
    startVector(elem_size: number, num_elems: number, alignment: number): void {
      this.notNested();
      this.vector_num_elems = num_elems;
      this.prep(SIZEOF_INT, elem_size * num_elems);
      this.prep(alignment, elem_size * num_elems); // Just in case alignment > int.
    }
  
    /**
     * Finish off the creation of an array and all its elements. The array must be
     * created with `startVector`.
     *
     * @returns The offset at which the newly created array
     * starts.
     */
    endVector(): Offset {
      this.writeInt32(this.vector_num_elems);
      return this.offset();
    }
  
    /**
     * Encode the string `s` in the buffer using UTF-8. If the string passed has 
     * already been seen, we return the offset of the already written string
     *
     * @param s The string to encode
     * @return The offset in the buffer where the encoded string starts
     */
    createSharedString(s: string | Uint8Array): Offset {
      if (!s) { return 0 }
  
      if (!this.string_maps) {
        this.string_maps = new Map();
      }
  
      if (this.string_maps.has(s)) {
        return this.string_maps.get(s) as Offset
      }
      const offset = this.createString(s)
      this.string_maps.set(s, offset)
      return offset
    }
  
    /**
     * Encode the string `s` in the buffer using UTF-8. If a Uint8Array is passed
     * instead of a string, it is assumed to contain valid UTF-8 encoded data.
     *
     * @param s The string to encode
     * @return The offset in the buffer where the encoded string starts
     */
    createString(s: string | Uint8Array | null | undefined): Offset {
      if (s === null || s === undefined) {
        return 0;
      }

      let utf8: string | Uint8Array | number[];
      if (s instanceof Uint8Array) {
        utf8 = s;
      } else {
        utf8 = this.text_encoder.encode(s);
      }
  
      this.addInt8(0);
      this.startVector(1, utf8.length, 1);
      this.bb.setPosition(this.space -= utf8.length);
      this.bb.bytes().set(utf8, this.space);
      return this.endVector();
    }
  
    /**
     * Create a byte vector.
     *
     * @param v The bytes to add
     * @returns The offset in the buffer where the byte vector starts
     */
    createByteVector(v: Uint8Array | null | undefined): Offset {
      if (v === null || v === undefined) {
        return 0;
      }

      this.startVector(1, v.length, 1);
      this.bb.setPosition(this.space -= v.length);
      this.bb.bytes().set(v, this.space);
      return this.endVector();
    }
  
    /**
     * A helper function to pack an object
     * 
     * @returns offset of obj
     */
    createObjectOffset(obj: string | IGeneratedObject | null): Offset {
      if(obj === null) {
        return 0
      }
  
      if(typeof obj === 'string') {
        return this.createString(obj);
      } else {
        return obj.pack(this);
      }
    }
  
    /**
     * A helper function to pack a list of object
     * 
     * @returns list of offsets of each non null object
     */
    createObjectOffsetList(list: (string | IGeneratedObject)[]): Offset[] {
      const ret: number[] = [];
  
      for(let i = 0; i < list.length; ++i) {
        const val = list[i];
  
        if(val !== null) {
          ret.push(this.createObjectOffset(val));
        } else {
          throw new TypeError(
            'FlatBuffers: Argument for createObjectOffsetList cannot contain null.'); 
        }
      }
      
      return ret;
    }
  
    createStructOffsetList(list: (string | IGeneratedObject)[], startFunc: (builder: Builder, length: number) => void): Offset {
      startFunc(this, list.length);
      this.createObjectOffsetList(list.slice().reverse());
      return this.endVector();
    }
  }
