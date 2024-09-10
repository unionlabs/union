module IBC::bcs_utils {
    use std::string::String;
    use std::from_bcs;
    use std::vector;

    const E_INVALID_PREFIX: u64 = 9000;

    struct BcsBuf has drop {
        inner: vector<u8>,
        cursor: u64,
    }

    public fun new(buf: vector<u8>): BcsBuf {
        BcsBuf {
            inner: buf,
            cursor: 0
        }
    }

    public fun remaining(buf: &BcsBuf): vector<u8> {
        vector::slice(&buf.inner, buf.cursor, vector::length(&buf.inner) - 1)
    }

    /// Parses the length prefix of a variable length type such as vector and string
    ///
    /// Returns the parsed length and the number of bytes read
    fun parse_length_prefix(buf: &BcsBuf): (u32, u64) {
        let value: u64 = 0;
        let shift = 0;
        let i = buf.cursor;
        let first_pos = i;
        while (shift < 32) {
            let byte = vector::borrow(&buf.inner, i);  
            let digit = *byte & 0x7f;
            value = value | ((digit as u64) << shift);
            if (digit == *byte) {
                if (shift > 0 && digit == 0) {
                    abort E_INVALID_PREFIX
                };

                return ((value as u32), i + 1 - first_pos)
            };

            i = i + 1;
            shift = shift + 7;
        };

        abort E_INVALID_PREFIX
    }

    /// Peel a variable-length length prefix of variable-sized types
    public fun peel_length_prefix(buf: &mut BcsBuf): u32 {
        let (length, n_read) = parse_length_prefix(buf);
        buf.cursor = buf.cursor + n_read;
        length
    }

    /// Peel a u32
    public fun peel_u32(buf: &mut BcsBuf): u32 {
        buf.cursor = buf.cursor + 4;
        from_bcs::to_u32(vector::slice(&buf.inner, buf.cursor - 4, buf.cursor))
    }

    /// Peel a u64
    public fun peel_u64(buf: &mut BcsBuf): u64 {
        buf.cursor = buf.cursor + 8;
        from_bcs::to_u64(vector::slice(&buf.inner, buf.cursor - 8, buf.cursor))
    }

    /// Peel a string
    public fun peel_string(buf: &mut BcsBuf): String {
        let (length, n_read) = parse_length_prefix(buf);
        buf.cursor = buf.cursor + n_read + (length as u64);
        from_bcs::to_string(vector::slice(&buf.inner, buf.cursor - (length as u64) - n_read, buf.cursor))
    }

    /// Peel a vector<u8>
    public fun peel_bytes(buf: &mut BcsBuf): vector<u8> {
        let (length, n_read) = parse_length_prefix(buf);
        buf.cursor = buf.cursor + n_read + (length as u64);
        from_bcs::to_bytes(vector::slice(&buf.inner, buf.cursor - (length as u64) - n_read, buf.cursor))
    }

    // /// Peel a vector<u8>
    public fun peel_fixed_bytes(buf: &mut BcsBuf, length: u32): vector<u8> {
        buf.cursor = buf.cursor + (length as u64);
        vector::slice(&buf.inner, buf.cursor - (length as u64), buf.cursor)
    }

    /// Peel a vector of T
    public inline fun peel_vector<T>(buf: &mut BcsBuf, parse_fn: |&mut BcsBuf|T): vector<T> {
        let length = peel_length_prefix(buf);
        let i = 0;
        let vec: vector<T> = vector::empty();
        while (i < length) {
            vector::push_back(&mut vec, parse_fn(buf));
            i = i + 1;
        };
        vec
    } 
}
