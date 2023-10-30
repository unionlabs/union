# What are these files?

These files are the DWARF data generated for (an early version of) this
library. Each file corresponds is a section from the built library's object
file. By splitting the sections out to their own files, we don't need to worry
about cross platform and cross object file format issues when running examples.

# Updating and adding new sections

## macOS

Use `otool` to list the sections of a binary:

```
$ otool -l path/to/binary
```

You should see output similar to this:

```
Load command 0
      cmd LC_SEGMENT_64
  cmdsize 72
  segname __PAGEZERO
   vmaddr 0x0000000000000000
   vmsize 0x0000000100000000
  fileoff 0
 filesize 0
  maxprot 0x00000000
 initprot 0x00000000
   nsects 0
    flags 0x0
Load command 1
      cmd LC_SEGMENT_64
  cmdsize 712
  segname __TEXT
   vmaddr 0x0000000100000000
   vmsize 0x00000000001b7000
  fileoff 0
 filesize 1798144
  maxprot 0x00000007
 initprot 0x00000005
   nsects 8
    flags 0x0
Section
  sectname __text
   segname __TEXT
      addr 0x0000000100000a50
      size 0x0000000000170716
    offset 2640
     align 2^4 (16)
    reloff 0
    nreloc 0
     flags 0x80000400
 reserved1 0
 reserved2 0
```

Etc.

Find the `Section` entry of the section you'd like to isolate. For example, if
you're looking for `eh_frame`, find an entry like this:

```
Section
  sectname __eh_frame
   segname __TEXT
      addr 0x0000000100192f38
      size 0x00000000000240c8
    offset 1650488
     align 2^3 (8)
    reloff 0
    nreloc 0
     flags 0x00000000
 reserved1 0
 reserved2 0
```

Then use `dd` to copy `size` bytes starting from `offset`:

```
$ dd bs=1 skip=1650488 count=$(printf "%d" 0x00000000000240c8) if=path/to/binary of=fixtures/self/eh_frame
```

Finally, use `otool` and `hexdump` to verify that the isolated section has the
same data as the section within the binary:

```
$ otool -s __TEXT __eh_frame path/to/binary | head
path/to/binary:
Contents of (__TEXT,__eh_frame) section
0000000100192f38	14 00 00 00 00 00 00 00 01 7a 52 00 01 78 10 01
0000000100192f48	10 0c 07 08 90 01 00 00 24 00 00 00 1c 00 00 00
0000000100192f58	f8 da e6 ff ff ff ff ff 66 00 00 00 00 00 00 00
0000000100192f68	00 41 0e 10 86 02 43 0d 06 00 00 00 00 00 00 00
0000000100192f78	1c 00 00 00 00 00 00 00 01 7a 50 4c 52 00 01 78
0000000100192f88	10 07 9b 9d 40 02 00 10 10 0c 07 08 90 01 00 00
0000000100192f98	2c 00 00 00 24 00 00 00 20 db e6 ff ff ff ff ff
0000000100192fa8	8d 00 00 00 00 00 00 00 08 37 e7 fd ff ff ff ff

$ otool -s __TEXT __eh_frame path/to/binary | tail
00000001001b6f68	9a 0a 00 00 00 00 00 00 00 41 0e 10 86 02 43 0d
00000001001b6f78	06 50 83 07 8c 06 8d 05 8e 04 8f 03 00 00 00 00
00000001001b6f88	24 00 00 00 7c 0e 00 00 30 a0 fb ff ff ff ff ff
00000001001b6f98	15 00 00 00 00 00 00 00 00 41 0e 10 86 02 43 0d
00000001001b6fa8	06 00 00 00 00 00 00 00 24 00 00 00 a4 0e 00 00
00000001001b6fb8	28 a0 fb ff ff ff ff ff 1c 00 00 00 00 00 00 00
00000001001b6fc8	00 41 0e 10 86 02 43 0d 06 00 00 00 00 00 00 00
00000001001b6fd8	24 00 00 00 cc 0e 00 00 20 a0 fb ff ff ff ff ff
00000001001b6fe8	66 01 00 00 00 00 00 00 00 41 0e 10 86 02 43 0d
00000001001b6ff8	06 00 00 00 00 00 00 00
```

This should be the same, ignoring the leading offsets:

```
$ hexdump fixtures/self/eh_frame | head
0000000 14 00 00 00 00 00 00 00 01 7a 52 00 01 78 10 01
0000010 10 0c 07 08 90 01 00 00 24 00 00 00 1c 00 00 00
0000020 f8 da e6 ff ff ff ff ff 66 00 00 00 00 00 00 00
0000030 00 41 0e 10 86 02 43 0d 06 00 00 00 00 00 00 00
0000040 1c 00 00 00 00 00 00 00 01 7a 50 4c 52 00 01 78
0000050 10 07 9b 9d 40 02 00 10 10 0c 07 08 90 01 00 00
0000060 2c 00 00 00 24 00 00 00 20 db e6 ff ff ff ff ff
0000070 8d 00 00 00 00 00 00 00 08 37 e7 fd ff ff ff ff
0000080 ff 41 0e 10 86 02 43 0d 06 00 00 00 00 00 00 00
0000090 24 00 00 00 94 00 00 00 80 db e6 ff ff ff ff ff

$ hexdump fixtures/self/eh_frame | tail
0024040 06 50 83 07 8c 06 8d 05 8e 04 8f 03 00 00 00 00
0024050 24 00 00 00 7c 0e 00 00 30 a0 fb ff ff ff ff ff
0024060 15 00 00 00 00 00 00 00 00 41 0e 10 86 02 43 0d
0024070 06 00 00 00 00 00 00 00 24 00 00 00 a4 0e 00 00
0024080 28 a0 fb ff ff ff ff ff 1c 00 00 00 00 00 00 00
0024090 00 41 0e 10 86 02 43 0d 06 00 00 00 00 00 00 00
00240a0 24 00 00 00 cc 0e 00 00 20 a0 fb ff ff ff ff ff
00240b0 66 01 00 00 00 00 00 00 00 41 0e 10 86 02 43 0d
00240c0 06 00 00 00 00 00 00 00
```

## Linux

Something like this:

```
objcopy --dump-section .eh_frame=fixtures/self/eh_frame path/to/binary
```
