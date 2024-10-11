# Annotated Flatbuffer Binary

This directory demonstrates the ability of flatc to annotate binary flatbuffers 
with helpful annotations. The resulting annotated flatbuffer binary (afb)
contains all the binary data with line-by-line annotations.

## Usage

Given a `schema` in either plain-text (.fbs) or already compiled to a binary
schema (.bfbs) and `binary` file(s) that was created by the `schema`.

```sh
flatc --annotate {schema_file} -- {binary_file}...
```

### Example

The following command should produce `annotated_binary.afb` in this directory:

```sh
cd tests\annotated_binary
..\..\flatc --annotate annotated_binary.fbs -- annotated_binary.bin
```

The `annotated_binary.bin` is the flatbufer binary of the data contained within
 `annotated_binary.json`, which was made by the following command:

```sh
..\..\flatc -b annotated_binary.fbs annotated_binary.json
```

## Text Format

Currently there is a built-in text-based format for outputting the annotations.
The `annotated_binary.afb` is an example of the text format of a binary
`annotated_binary.bin` and the `annotated_binary.fbs` (or 
`annotated_binary.bfbs`) schema.

The file is ordered in increasing the offsets from the beginning of the binary.
The offset is the 1st column, expressed in hexadecimal format (e.g. `+0x003c`).

### Binary Sections

Binary sections are comprised of contigious [binary regions](#binary-regions) 
that are logically grouped together. For example, a binary section may be a
single instance of a flatbuffer `Table` or its `vtable`. The sections may be 
labelled with the name of the associated type, as defined in the input schema.

Example of a `vtable` Binary Section that is associated with the user-defined
`AnnotateBinary.Bar` table.

```
vtable (AnnotatedBinary.Bar):
  +0x00A0 | 08 00                   | uint16_t   | 0x0008 (8)                         | size of this vtable
  +0x00A2 | 13 00                   | uint16_t   | 0x0013 (19)                        | size of referring table
  +0x00A4 | 08 00                   | VOffset16  | 0x0008 (8)                         | offset to field `a` (id: 0)
  +0x00A6 | 04 00                   | VOffset16  | 0x0004 (4)                         | offset to field `b` (id: 1)
```

### Binary Regions

Binary regions are contigious bytes regions that are grouped together to form 
some sort of value, e.g. a `scalar` or an array of scalars. A binary region may
be split up over multiple text lines, if the size of the region is large.

Looking at an example binary region:

```
vtable (AnnotatedBinary.Bar):
  +0x00A0 | 08 00                   | uint16_t   | 0x0008 (8)                         | size of this vtable
```

The first column (`+0x00A0`) is the offset to this region from the beginning of
the buffer. 

The second column are the raw bytes (hexadecimal) that make up this
region. These are expressed in the little-endian format that flatbuffers uses 
for the wire format.

The third column is the type to interpret the bytes as. Some types are special
to flatbuffer internals (e.g. `SOffet32`, `Offset32`, and `VOffset16`) which are
used by flatbuffers to point to various offsetes. The other types are specified
as C++-like types which are the standard fix-width scalars. For the above
example, the type is `uint16_t` which is a 16-bit unsigned integer type.

The fourth column shows the raw bytes as a compacted, big-endian value. The raw
bytes are duplicated in this fashion since it is more intutive to read the data
in the big-endian format (e.g., `0x0008`). This value is followed by the decimal
representation of the value (e.g., `(8)`). (For strings, the raw string value
is shown instead). 

The fifth column is a textual comment on what the value is. As much metadata as
known is provided.

#### Offsets

If the type in the 3rd column is of an absolute offset (`SOffet32` or 
`Offset32`), the fourth column also shows an `Loc: +0x025A` value which shows 
where in the binary this region is pointing to. These values are absolute from
the beginning of the file, their calculation from the raw value in the 4th
column depends on the context.
