# Tests for Annotated Binaries

## Invalid Binary Tests

The following is a collection of manually corrupted binaries based off of
`..\annotated_binary.bin`. Each file changes some offset or length/size entry to
point so an invalid spot, and the generated annotated binaries demonstrate that
those corruptions can be spotted.

Each of these files were ran with the following command:

```sh
cd .tests/annotated_binary
../../flatc -annotate annotated_binary.fbs tests/{binary_file}...
```

### `invalid_root_offset.bin`

Changed first two bytes from `4400` to `FFFF` which produces an offset larger
than the binary.

### `invalid_root_table_vtable_offset.bin`

Changed two bytes at 0x0044 from `3A00` to `FFFF` which points to an offset
outside the binary.

### `invalid_root_table_too_short.bin`

Truncated the file to 0x46 bytes, as that cuts into the vtable offset field of
the root table.

```sh
truncate annotated_binary.bin --size=70 >> invalid_root_table_too_short.bin
```

### `invalid_vtable_size.bin`

Changed two bytes at 0x000A from `3A00` to `FFFF` which size is larger than the
binary.

### `invalid_vtable_size_short.bin`

Changed two bytes at 0x000A from `3A00` to `0100` which size is smaller than the
minimum size of 4 bytes.

### `invalid_vtable_ref_table_size.bin`

Changed two bytes at 0x000C from `6800` to `FFFF` which size is larger than the
binary.

### `invalid_vtable_ref_table_size_short.bin`

Changed two bytes at 0x000C from `6800` to `0100` which size is smaller than 
the minimum size of 4 bytes.

### `invalid_vtable_field_offset.bin`

Changed two bytes at 0x0016 from `1000` to `FFFF` which points to a field larger
than the binary.

### `invalid_table_field_size.bin`

Truncated the file to 0x52 bytes, as that cuts a Uint32t value in half.

### `invalid_table_field_offset.bin`

Truncated the file to 0x96 bytes, as that cuts a UOffset32 value in half. Also,
changed two bytes at 0x90 from `DC00` to `FFFF` which points to a section larger
than the binary.

### `invalid_string_length_cut_short.bin`

Truncated the file to 0xAD bytes, as that cuts string length Uint32t value in 
half.

### `invalid_string_length.bin`

Changed two bytes at 0x00AC from `0500` to `FFFF` which is a string length
larger than the binary.

### `invalid_vector_length_cut_short.bin`

Truncated the file to 0x0136 bytes, as that cuts vector length Uint32t value in 
half.

### `invalid_struct_field_cut_short.bin`

Truncated the file to 0x5d bytes, as that cuts struct field value in half.

### `invalid_struct_array_field_cut_short.bin`

Truncated the file to 0x6A bytes, as that cuts struct array field value in half.

### `invalid_vector_structs_cut_short.bin`

Truncated the file to 0x0154 bytes, as that cuts into a vector of structs.

### `invalid_vector_tables_cut_short.bin`

Truncated the file to 0x01DE bytes, as that cuts into a vector of table offsets.

### `invalid_vector_strings_cut_short.bin`

Truncated the file to 0x0176 bytes, as that cuts into a vector of string
offsets.

### `invalid_vector_scalars_cut_short.bin`

Truncated the file to 0x01C1 bytes, as that cuts into a vector of scalars 
values.

### `invalid_vector_unions_cut_short.bin`

Truncated the file to 0x01DE bytes, as that cuts into a vector of union offset 
values.

### `invalid_union_type_value.bin`

Changed one byte at 0x004D from `02` to `FF` which is a union type value that is
larger than the enum.

### `invalid_vector_union_type_value.bin`

Changed one byte at 0x0131 from `02` to `FF` which is a vector union type value 
that is larger than the enum.