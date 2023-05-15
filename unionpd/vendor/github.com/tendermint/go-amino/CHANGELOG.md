# Changelog

## [Unreleased]

## 0.16.0 (September 11, 2020)

IMPROVEMENTS:
 - Add support for `Un/MarshalAminoJSON` override: if a type implements
 `Un/MarshalAminoJSON`, then amino will use these methods for JSON un/marshalling
 ([#323]).

[#323]: https://github.com/tendermint/go-amino/pull/323

## 0.15.1 (October 10, 2019)

### IMPROVEMENTS:

- [#289](https://github.com/tendermint/go-amino/issues/289) Return Boolean pointers as non-nil (@nicolasbrugneaux)
- [#282](https://github.com/tendermint/go-amino/pull/282) Improve efficiency of `VarintSize()`(@rickyyangz)

## 0.15.0 (May 2, 2018)

BREAKING CHANGE:

- JSON: Skip null fields on decoding when the `json:"omitempty"` tag is set.
  This matches the encoding behaviour ([#261]).

IMPROVEMENTS:

- Amino becomes a go-module (requires go 1.11) but keeps dep support for backwards compatibility ([#255]).

[#255]: https://github.com/tendermint/go-amino/pull/255
[#261]: https://github.com/tendermint/go-amino/issues/261

## 0.14.1 (November 6, 2018)

IMPROVEMENTS:

- go-amino compiles again on 32-bit platforms ([#242])

[#242]: https://github.com/tendermint/go-amino/pull/242

## 0.14.0 (October 26, 2018)

BREAKING CHANGE:

- Switch default encoding of unsigned ints (`int`, `int32`, `int64`) to be on par with [proto3's] variable length
  encoding (of `int32`, `int64`) ([#237])

[proto3's]: https://developers.google.com/protocol-buffers/docs/proto#scalar
[#237]: https://github.com/tendermint/go-amino/issues/237

## 0.13.0 (October 15, 2018)

BREAKING CHANGE:

- `time.Time`: validate seconds since 1970 are in valid range during encoding; i.e. in the interval [-62135596800, 253402300800)
- `time.Time`: match encoding of time.Time to protobuf's well known type [Timestamp]([#224])
- Rename `MarshalBinary` to `MarshalBinaryLengthPrefixed` ([#222])

[timestamp]: https://github.com/protocolbuffers/protobuf/blob/d2980062c859649523d5fd51d6b55ab310e47482/src/google/protobuf/timestamp.proto#L123-L135

IMPROVEMENTS:

- Add `MustUnmarshalJSON` and `MustMarshalJSON` that panic if an error occurs ([#228])
- Optimize performance by removing `defer`s in `getTypeInfo` ([#227] by @ValarDragon)
- Optimize performance by removing unnecessary allocation in `UvarintSize` ([#225] by [@ValarDragon])
- Add test that shows compatibility with stdlib (JSON time encoding) ([#131] by [@odeke-em])

[#224]: https://github.com/tendermint/go-amino/pull/224
[#222]: https://github.com/tendermint/go-amino/pull/222
[#228]: https://github.com/tendermint/go-amino/pull/228
[#227]: https://github.com/tendermint/go-amino/pull/227
[#225]: https://github.com/tendermint/go-amino/pull/225
[#131]: https://github.com/tendermint/go-amino/pull/131
[@valardragon]: https://github.com/ValarDragon
[@odeke-em]: https://github.com/odeke-em

## 0.12.0 (August 4, 2018)

BREAKING CHANGE:

- Write empty (non-nil) struct pointers, unless (is list element and empty_elements isn't set) #206

## 0.11.1 (July 17, 2018)

IMPROVEMENTS:

- Remove dependency on tmlibs/common

## 0.11.0 (June 19, 2018)

BREAKING CHANGE:

- Do not encode zero values in `EncodeTime`
  (to match proto3's behaviour) (#178, #190)
- Do not encode empty structs, unless explicitly enforced
  via `amino:"write_empty"` (to match proto3's behaviour) (#179)

IMPROVEMENTS:

- DecodeInt{8, 16} negative limit checks (#125)

## 0.10.1 (June 15, 2018)

FEATURE:

- [aminoscan] aminoscan --color will print ASCII bytes in different colors

BUG FIXES:

- do not err if prefix bytes are exactly 4 (for registered types)

## 0.10.0 (June 12, 2018)

BREAKING CHANGE:

- 100% Proto3 compatibility for primitive types, repeated fields, and embedded structs/messages.
- BigEndian -> LittleEndian
- [u]int[64/32] is (signed) Varint by default, "fixed32" and "fixed64" to use 4 and 8 byte types.
- Amino:JSON [u]int64 and ints are strings.
- Enforce UTC timezone for JSON encoding of time.

## 0.9.11 (May 27, 2018)

NEW FEATURES:

- Seal() on a codec to prevent further modifications. #150
- Global Marshal/Unmarshal methods on a sealed codec with nothing registered.

## 0.9.10 (May 10, 2018)

BREAKING CHANGE:

- Amino:JSON encoding of interfaces use the registered concrete type name, not the disfix bytes.

## 0.9.9 (May 1, 2018)

BUG FIXES:

- MarshalAmino/UnmarshalAmino actually works (sorry!)

## 0.9.8 (April 26, 2018)

NEW FEATURES:

- DeepCopy() copies any Amino object (with support for .DeepCopy() and
  .MarshalAmino/UnmarshalAmino().

## 0.9.7 (April 25, 2019)

FEATURES:

- Add MustUnmarshalBinary and MustUnmarshalBinaryBare to the Codec
  - both methods are analogous to their marshalling counterparts
  - both methods will panic in case of an error
- MarshalJSONIndent

## 0.9.6 (April 5, 2018)

IMPROVEMENTS:

- map[string]<any> support for Amino:JSON

## 0.9.5 (April 5, 2018)

BREAKING CHANGE:

- Skip encoding of "void" (nil/empty) struct fields and list elements, esp empty strings

IMPROVEMENTS:

- Better error message with empty inputs

## 0.9.4 (April 3, 2018)

BREAKING CHANGE:

- Treat empty slices and nil the same in binary

IMPROVEMENTS:

- Add indenting to aminoscan

BUG FIXES:

- JSON omitempty fix.

## 0.9.2 (Mar 24, 2018)

BUG FIXES:

- Fix UnmarshalBinaryReader consuming too much from bufio.
- Fix UnmarshalBinaryReader obeying limit.

## 0.9.1 (Mar 24, 2018)

BUG FIXES:

- Fix UnmarshalBinaryReader returned n

## 0.9.0 (Mar 10, 2018)

BREAKING CHANGE:

- wire -> amino
- Protobuf-like encoding
- MarshalAmino/UnmarshalAmino

## 0.8.0 (Jan 28, 2018)

BREAKING CHANGE:

- New Disamb/Prefix system
- Marshal/Unmarshal Binary/JSON
- JSON is a shim but PR incoming

## 0.7.2 (Dec 5, 2017)

IMPROVEMENTS:

- data: expose Marshal and Unmarshal methods on `Bytes` to support protobuf
- nowriter: start adding new interfaces for improved technical language and organization

BUG FIXES:

- fix incorrect byte write count for integers

## 0.7.1 (Oct 27, 2017)

BUG FIXES:

- dont use nil for empty byte array (undoes fix from 0.7.0 pending further analysis)

## 0.7.0 (Oct 26, 2017)

BREAKING CHANGE:

- time: panic on encode, error on decode for times before 1970
- rm codec.go

IMPROVEMENTS:

- various additional comments, guards, and checks

BUG FIXES:

- fix default encoding of time and bytes
- don't panic on ReadTime
- limit the amount of memory that can be allocated

## 0.6.2 (May 18, 2017)

FEATURES:

- `github.com/tendermint/go-data` -> `github.com/tendermint/go-wire/data`

IMPROVEMENTS:

- Update imports for new `tmlibs` repository

## 0.6.1 (April 18, 2017)

FEATURES:

- Size functions: ByteSliceSize, UvarintSize
- CLI tool
- Expression DSL
- New functions for bools: ReadBool, WriteBool, GetBool, PutBool
- ReadJSONBytes function

IMPROVEMENTS:

- Makefile
- Use arrays instead of slices
- More testing
- Allow omitempty to work on non-comparable types

BUG FIXES:

- Allow time parsing for seconds, milliseconds, and microseconds
- Stop overflows in ReadBinaryBytes

## 0.6.0 (January 18, 2016)

BREAKING CHANGES:

FEATURES:

IMPROVEMENTS:

BUG FIXES:

## Prehistory
