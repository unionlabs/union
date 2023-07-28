# CBOR Codec in Go

[![](https://github.com/fxamacker/images/raw/master/cbor/v2.4.0/fxamacker_cbor_banner.png)](#cbor-library-in-go)

[![](https://github.com/fxamacker/cbor/workflows/ci/badge.svg)](https://github.com/fxamacker/cbor/actions?query=workflow%3Aci)
[![](https://github.com/fxamacker/cbor/workflows/cover%20%E2%89%A598%25/badge.svg)](https://github.com/fxamacker/cbor/actions?query=workflow%3A%22cover+%E2%89%A598%25%22)
[![](https://github.com/fxamacker/cbor/workflows/linters/badge.svg)](https://github.com/fxamacker/cbor/actions?query=workflow%3Alinters)
[![CodeQL](https://github.com/fxamacker/cbor/actions/workflows/codeql-analysis.yml/badge.svg)](https://github.com/fxamacker/cbor/actions/workflows/codeql-analysis.yml)
[![](https://img.shields.io/badge/fuzzing-3%2B%20billion%20execs-44c010)](#fuzzing-and-code-coverage)
[![Go Report Card](https://goreportcard.com/badge/github.com/fxamacker/cbor)](https://goreportcard.com/report/github.com/fxamacker/cbor)
[![](https://img.shields.io/badge/go-%3E%3D%201.12-blue)](#cbor-library-installation)

[__fxamacker/cbor__](https://github.com/fxamacker/cbor) is a modern [CBOR](https://tools.ietf.org/html/rfc8949) codec in [Go](https://golang.org).  It's like `encoding/json` for CBOR with time-saving features.  It balances [security](https://github.com/fxamacker/cbor/#cbor-security), usability, [speed](https://github.com/fxamacker/cbor/#cbor-performance), data size, program size, and other competing factors.

Features include CBOR tags, duplicate map key detection, float64→32→16, and Go struct tags (`toarray`, `keyasint`, `omitempty`).  API is close to `encoding/json` plus predefined CBOR options like Core Deterministic Encoding, Preferred Serialization, CTAP2, etc.

Using CBOR [Preferred Serialization](https://www.rfc-editor.org/rfc/rfc8949.html#name-preferred-serialization) with Go struct tags (`toarray`, `keyasint`, `omitempty`) reduces programming effort and creates smaller encoded data size.

fxamacker/cbor has 98% coverage and is fuzz tested.  It won't exhaust RAM decoding 9 bytes of bad CBOR data.  It's used by Arm Ltd., Berlin Institute of Health at Charité, Chainlink, ConsenSys, Dapper Labs, Duo Labs (cisco), EdgeX Foundry, Mozilla, Netherlands (govt), Oasis Labs, Taurus SA, Teleport, and others.

Install with `go get github.com/fxamacker/cbor/v2` and `import "github.com/fxamacker/cbor/v2"`.  
See [Quick Start](#quick-start) to save time.

## What is CBOR?

[CBOR](https://tools.ietf.org/html/rfc8949) is a concise binary data format inspired by [JSON](https://www.json.org) and [MessagePack](https://msgpack.org).  CBOR is defined in [RFC 8949](https://tools.ietf.org/html/rfc8949) (December 2020) which obsoletes [RFC 7049](https://tools.ietf.org/html/rfc7049) (October 2013).  

CBOR is an [Internet Standard](https://en.wikipedia.org/wiki/Internet_Standard) by [IETF](https://www.ietf.org).  It's used in other standards like [WebAuthn](https://en.wikipedia.org/wiki/WebAuthn) by [W3C](https://www.w3.org), [COSE (RFC 8152)](https://tools.ietf.org/html/rfc8152), [CWT (RFC 8392)](https://tools.ietf.org/html/rfc8392), [CDDL (RFC 8610)](https://datatracker.ietf.org/doc/html/rfc8610) and [more](CBOR_GOLANG.md).

[Reasons for choosing CBOR](https://github.com/fxamacker/cbor/wiki/Why-CBOR) vary by project.  Some projects replaced protobuf, encoding/json, encoding/gob, etc. with CBOR.  For example, by replacing protobuf with CBOR in gRPC.

## Why fxamacker/cbor?

fxamacker/cbor balances competing factors such as speed, size, safety, usability, maintainability, and etc.

- Killer features include Go struct tags like `toarray`, `keyasint`, etc.  They reduce encoded data size, improve speed, and reduce programming effort. For example, `toarray` automatically translates a Go struct to/from a CBOR array.

- Modern CBOR features include Core Deterministic Encoding and Preferred Encoding. Other features include CBOR tags, big.Int, float64→32→16, an API like `encoding/json`, and more.

- Security features include the option to detect duplicate map keys and options to set various max limits. And it's designed to make concurrent use of CBOR options easy and free from side-effects.  

- To prevent crashes, it has been fuzz-tested since before release 1.0 and code coverage is kept above 98%.

- For portability and safety, it avoids using `unsafe`, which makes it portable and protected by Go1's compatibility guidelines.  

- For performance, it uses safe optimizations.  When used properly, fxamacker/cbor can be faster than CBOR codecs that rely on `unsafe`.  However, speed is only one factor and should be considered together with other competing factors.

## CBOR Security

__fxamacker/cbor__ is secure.  It rejects malformed CBOR data and has an option to detect duplicate map keys.  It doesn't crash when decoding bad CBOR data. It has extensive tests, coverage-guided fuzzing, data validation, and avoids Go's `unsafe` package.

Decoding 9 or 10 bytes of malformed CBOR data shouldn't exhaust memory. For example,  
`[]byte{0x9B, 0x00, 0x00, 0x42, 0xFA, 0x42, 0xFA, 0x42, 0xFA, 0x42}`

|     | Decode bad 10 bytes to interface{} | Decode bad 10 bytes to []byte |
| :--- | :------------------ | :--------------- |
| fxamacker/cbor<br/>1.0-2.3 | 49.44 ns/op, 24 B/op, 2 allocs/op* | 51.93 ns/op, 32 B/op, 2 allocs/op* |
| ugorji/go 1.2.6 | ⚠️ 45021 ns/op, 262852 B/op, 7 allocs/op | 💥 runtime: out of memory: cannot allocate |
| ugorji/go 1.1-1.1.7 | 💥 runtime: out of memory: cannot allocate | 💥 runtime: out of memory: cannot allocate|

*Speed and memory are for latest codec version listed in the row (compiled with Go 1.17.5).

fxamacker/cbor CBOR safety settings include: MaxNestedLevels, MaxArrayElements, MaxMapPairs, and IndefLength.

For more info, see:
 - [RFC 8949 Section 10 (Security Considerations)](https://tools.ietf.org/html/rfc8949#section-10) or [RFC 7049 Section 8](https://tools.ietf.org/html/rfc7049#section-8).
 - [Go warning](https://golang.org/pkg/unsafe/), "Packages that import unsafe may be non-portable and are not protected by the Go 1 compatibility guidelines."

## CBOR Performance

__fxamacker/cbor__ is fast without sacrificing security. It can be faster than libraries relying on `unsafe` package.

![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_speed_comparison.svg?sanitize=1 "CBOR speed comparison chart")

__Click to expand:__

<details>
  <summary> 👉 CBOR Program Size Comparison </summary><p>

__fxamacker/cbor__ produces smaller programs without sacrificing features.
  
![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_size_comparison.svg?sanitize=1 "CBOR program size comparison chart")

</details>

<details><summary> 👉 fxamacker/cbor 2.3.0 (safe) vs ugorji/go 1.2.6 (unsafe)</summary><p>

fxamacker/cbor 2.3.0 (not using `unsafe`) is faster than ugorji/go 1.2.6 (using `unsafe`).

```
benchstat results/bench-ugorji-go-count20.txt results/bench-fxamacker-cbor-count20.txt 
name                                 old time/op    new time/op    delta
DecodeCWTClaims-8                      1.08µs ± 0%    0.67µs ± 0%  -38.10%  (p=0.000 n=16+20)
DecodeCOSE/128-Bit_Symmetric_Key-8      715ns ± 0%     501ns ± 0%  -29.97%  (p=0.000 n=20+19)
DecodeCOSE/256-Bit_Symmetric_Key-8      722ns ± 0%     507ns ± 0%  -29.72%  (p=0.000 n=19+18)
DecodeCOSE/ECDSA_P256_256-Bit_Key-8    1.11µs ± 0%    0.83µs ± 0%  -25.27%  (p=0.000 n=19+20)
DecodeWebAuthn-8                        880ns ± 0%     727ns ± 0%  -17.31%  (p=0.000 n=18+20)
EncodeCWTClaims-8                       785ns ± 0%     388ns ± 0%  -50.51%  (p=0.000 n=20+20)
EncodeCOSE/128-Bit_Symmetric_Key-8      973ns ± 0%     433ns ± 0%  -55.45%  (p=0.000 n=20+19)
EncodeCOSE/256-Bit_Symmetric_Key-8      974ns ± 0%     435ns ± 0%  -55.37%  (p=0.000 n=20+19)
EncodeCOSE/ECDSA_P256_256-Bit_Key-8    1.14µs ± 0%    0.55µs ± 0%  -52.10%  (p=0.000 n=19+19)
EncodeWebAuthn-8                        564ns ± 0%     450ns ± 1%  -20.18%  (p=0.000 n=18+20)

name                                 old alloc/op   new alloc/op   delta
DecodeCWTClaims-8                        744B ± 0%      160B ± 0%  -78.49%  (p=0.000 n=20+20)
DecodeCOSE/128-Bit_Symmetric_Key-8       792B ± 0%      232B ± 0%  -70.71%  (p=0.000 n=20+20)
DecodeCOSE/256-Bit_Symmetric_Key-8       816B ± 0%      256B ± 0%  -68.63%  (p=0.000 n=20+20)
DecodeCOSE/ECDSA_P256_256-Bit_Key-8      905B ± 0%      344B ± 0%  -61.99%  (p=0.000 n=20+20)
DecodeWebAuthn-8                       1.56kB ± 0%    0.99kB ± 0%  -36.41%  (p=0.000 n=20+20)
EncodeCWTClaims-8                      1.35kB ± 0%    0.18kB ± 0%  -86.98%  (p=0.000 n=20+20)
EncodeCOSE/128-Bit_Symmetric_Key-8     1.95kB ± 0%    0.22kB ± 0%  -88.52%  (p=0.000 n=20+20)
EncodeCOSE/256-Bit_Symmetric_Key-8     1.95kB ± 0%    0.24kB ± 0%  -87.70%  (p=0.000 n=20+20)
EncodeCOSE/ECDSA_P256_256-Bit_Key-8    1.95kB ± 0%    0.32kB ± 0%  -83.61%  (p=0.000 n=20+20)
EncodeWebAuthn-8                       1.30kB ± 0%    1.09kB ± 0%  -16.56%  (p=0.000 n=20+20)

name                                 old allocs/op  new allocs/op  delta
DecodeCWTClaims-8                        6.00 ± 0%      6.00 ± 0%     ~     (all equal)
DecodeCOSE/128-Bit_Symmetric_Key-8       4.00 ± 0%      4.00 ± 0%     ~     (all equal)
DecodeCOSE/256-Bit_Symmetric_Key-8       4.00 ± 0%      4.00 ± 0%     ~     (all equal)
DecodeCOSE/ECDSA_P256_256-Bit_Key-8      7.00 ± 0%      7.00 ± 0%     ~     (all equal)
DecodeWebAuthn-8                         5.00 ± 0%      5.00 ± 0%     ~     (all equal)
EncodeCWTClaims-8                        4.00 ± 0%      2.00 ± 0%  -50.00%  (p=0.000 n=20+20)
EncodeCOSE/128-Bit_Symmetric_Key-8       6.00 ± 0%      2.00 ± 0%  -66.67%  (p=0.000 n=20+20)
EncodeCOSE/256-Bit_Symmetric_Key-8       6.00 ± 0%      2.00 ± 0%  -66.67%  (p=0.000 n=20+20)
EncodeCOSE/ECDSA_P256_256-Bit_Key-8      6.00 ± 0%      2.00 ± 0%  -66.67%  (p=0.000 n=20+20)
EncodeWebAuthn-8                         4.00 ± 0%      2.00 ± 0%  -50.00%  (p=0.000 n=20+20)
```
 </details>

Benchmarks used Go 1.17.5, linux_amd64, and data from [RFC 8392 Appendix A.1](https://tools.ietf.org/html/rfc8392#appendix-A.1).  Default build options were used for all CBOR libraries.  Library init code was put outside the benchmark loop for all libraries compared.

## CBOR API

__fxamacker/cbor__ is easy to use.  It provides standard API and interfaces.

__Standard API__.  Function signatures identical to [`encoding/json`](https://golang.org/pkg/encoding/json/) include:  
`Marshal`, `Unmarshal`, `NewEncoder`, `NewDecoder`, `(*Encoder).Encode`, and `(*Decoder).Decode`.

__Standard Interfaces__.  Custom encoding and decoding is handled by implementing:  
`BinaryMarshaler`, `BinaryUnmarshaler`, `Marshaler`, and `Unmarshaler`.

__Predefined Encoding Options__.  Encoding options are easy to use and are customizable.

```go
func CoreDetEncOptions() EncOptions {}              // RFC 8949 Core Deterministic Encoding
func PreferredUnsortedEncOptions() EncOptions {}    // RFC 8949 Preferred Serialization
func CanonicalEncOptions() EncOptions {}            // RFC 7049 Canonical CBOR
func CTAP2EncOptions() EncOptions {}                // FIDO2 CTAP2 Canonical CBOR
```

fxamacker/cbor designed to simplify concurrency.  CBOR options can be used without creating unintended runtime side-effects.

## Go Struct Tags

__fxamacker/cbor__ provides Go struct tags like __`toarray`__ and __`keyasint`__ to save time and reduce encoded size of data.

<br>

![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_struct_tags_api.svg?sanitize=1 "CBOR API and Go Struct Tags")

## CBOR Features

__fxamacker/cbor__ is a full-featured CBOR encoder and decoder.

|   | CBOR Feature  | Description  |
| :--- | :--- | :--- |
| ☑️ | CBOR tags | API supports built-in and user-defined tags.  |
| ☑️ | Preferred serialization | Integers encode to fewest bytes. Optional float64 → float32 → float16. |
| ☑️ | Map key sorting | Unsorted, length-first (Canonical CBOR), and bytewise-lexicographic (CTAP2). |
| ☑️ | Duplicate map keys | Always forbid for encoding and option to allow/forbid for decoding.   |
| ☑️ | Indefinite length data | Option to allow/forbid for encoding and decoding. |
| ☑️ | Well-formedness | Always checked and enforced. |
| ☑️ | Basic validity checks | Check UTF-8 validity and optionally check duplicate map keys. |
| ☑️ | Security considerations | Prevent integer overflow and resource exhaustion (RFC 8949 Section 10). |

## CBOR Library Installation

fxamacker/cbor supports Go 1.12 and newer versions.  Init the Go module, go get v2, and begin coding.

```
go mod init github.com/my_name/my_repo
go get github.com/fxamacker/cbor/v2
```

```go
import "github.com/fxamacker/cbor/v2"  // imports as cbor
```

## Quick Start
🛡️ Use Go's `io.LimitReader` to limit size when decoding very large or indefinite size data.

Import using "/v2" like this: `import "github.com/fxamacker/cbor/v2"`, and  
it will import version 2.x as package "cbor" (when using Go modules).

Functions with identical signatures to encoding/json include:  
`Marshal`, `Unmarshal`, `NewEncoder`, `NewDecoder`, `(*Encoder).Encode`, `(*Decoder).Decode`.

__Default Mode__  

If default options are acceptable, package level functions can be used for encoding and decoding.

```go
b, err := cbor.Marshal(v)        // encode v to []byte b
err := cbor.Unmarshal(b, &v)     // decode []byte b to v
encoder := cbor.NewEncoder(w)    // create encoder with io.Writer w
decoder := cbor.NewDecoder(r)    // create decoder with io.Reader r
```

__Modes__

If you need to use options or CBOR tags, then you'll want to create a mode.

"Mode" means defined way of encoding or decoding -- it links the standard API to your CBOR options and CBOR tags.  This way, you don't pass around options and the API remains identical to `encoding/json`.

EncMode and DecMode are interfaces created from EncOptions or DecOptions structs.  
For example, `em, err := cbor.EncOptions{...}.EncMode()` or `em, err := cbor.CanonicalEncOptions().EncMode()`.

EncMode and DecMode use immutable options so their behavior won't accidentally change at runtime.  Modes are reusable, safe for concurrent use, and allow fast parallelism.

__Creating and Using Encoding Modes__

💡 Avoid using init().  For best performance, reuse EncMode and DecMode after creating them.

Most apps will probably create one EncMode and DecMode before init().  There's no limit and each can use different options.

```go
// Create EncOptions using either struct literal or a function.
opts := cbor.CanonicalEncOptions()

// If needed, modify opts. For example: opts.Time = cbor.TimeUnix

// Create reusable EncMode interface with immutable options, safe for concurrent use.
em, err := opts.EncMode()   

// Use EncMode like encoding/json, with same function signatures.
b, err := em.Marshal(v)      // encode v to []byte b

encoder := em.NewEncoder(w)  // create encoder with io.Writer w
err := encoder.Encode(v)     // encode v to io.Writer w
```

Both `em.Marshal(v)` and `encoder.Encode(v)` use encoding options specified during creation of encoding mode `em`.

__Creating Modes With CBOR Tags__

A TagSet is used to specify CBOR tags.
 
```go
em, err := opts.EncMode()                  // no tags
em, err := opts.EncModeWithTags(ts)        // immutable tags
em, err := opts.EncModeWithSharedTags(ts)  // mutable shared tags
```

TagSet and all modes using it are safe for concurrent use.  Equivalent API is available for DecMode.

__Predefined Encoding Options__

```go
func CoreDetEncOptions() EncOptions {}              // RFC 8949 Core Deterministic Encoding
func PreferredUnsortedEncOptions() EncOptions {}    // RFC 8949 Preferred Serialization
func CanonicalEncOptions() EncOptions {}            // RFC 7049 Canonical CBOR
func CTAP2EncOptions() EncOptions {}                // FIDO2 CTAP2 Canonical CBOR
```

The empty curly braces prevent a syntax highlighting bug on GitHub, please ignore them.

__Struct Tags (keyasint, toarray, omitempty)__

The `keyasint`, `toarray`, and `omitempty` struct tags make it easy to use compact CBOR message formats.  Internet standards often use CBOR arrays and CBOR maps with int keys to save space.

The following sections provide more info:

* [Struct Tags](#struct-tags-1)
* [Decoding Options](#decoding-options)
* [Encoding Options](#encoding-options)
* [API](#api) 
* [Usage](#usage) 

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Features

### Standard API

Many function signatures are identical to encoding/json, including:  
`Marshal`, `Unmarshal`, `NewEncoder`, `NewDecoder`, `(*Encoder).Encode`, `(*Decoder).Decode`.

`RawMessage` can be used to delay CBOR decoding or precompute CBOR encoding, like `encoding/json`.

Standard interfaces allow user-defined types to have custom CBOR encoding and decoding.  They include:  
`BinaryMarshaler`, `BinaryUnmarshaler`, `Marshaler`, and `Unmarshaler`.

`Marshaler` and `Unmarshaler` interfaces are satisfied by `MarshalCBOR` and `UnmarshalCBOR` functions using same params and return types as Go's MarshalJSON and UnmarshalJSON.

### Struct Tags

Support "cbor" and "json" keys in Go's struct tags. If both are specified for the same field, then "cbor" is used.

* a different field name can be specified, like encoding/json.
* `omitempty` omits (ignores) field if value is empty, like encoding/json.
* `-` always omits (ignores) field, like encoding/json.
* `keyasint` treats fields as elements of CBOR maps with specified int key.
* `toarray` treats fields as elements of CBOR arrays.

See [Struct Tags](#struct-tags-1) for more info.

### CBOR Tags (New in v2.1)

There are three categories of CBOR tags:

* __Default built-in CBOR tags__ currently include tag numbers 0 (Standard Date/Time), 1 (Epoch Date/Time), 2 (Unsigned Bignum), 3 (Negative Bignum), 55799 (Self-Described CBOR).  

* __Optional built-in CBOR tags__ may be provided in the future via build flags or optional package(s) to help reduce bloat.

* __User-defined CBOR tags__ are easy by using TagSet to associate tag numbers to user-defined Go types.

### Preferred Serialization

Preferred serialization encodes integers and floating-point values using the fewest bytes possible.

* Integers are always encoded using the fewest bytes possible.
* Floating-point values can optionally encode from float64->float32->float16 when values fit.

### Compact Data Size

The combination of preferred serialization and struct tags (toarray, keyasint, omitempty) allows very compact data size.

### Predefined Encoding Options

Easy-to-use functions (no params) return preset EncOptions struct:  
`CanonicalEncOptions`, `CTAP2EncOptions`, `CoreDetEncOptions`, `PreferredUnsortedEncOptions`

### Encoding Options

Integers always encode to the shortest form that preserves value.  By default, time values are encoded without tags.

Encoding of other data types and map key sort order are determined by encoder options.

| EncOptions | Available Settings (defaults listed first)
| :--- | :--- |
| Sort | **SortNone**, SortLengthFirst, SortBytewiseLexical <br/> Aliases: SortCanonical, SortCTAP2, SortCoreDeterministic |
| Time | **TimeUnix**, TimeUnixMicro, TimeUnixDynamic, TimeRFC3339, TimeRFC3339Nano |
| TimeTag | **EncTagNone**, EncTagRequired |
| ShortestFloat | **ShortestFloatNone**, ShortestFloat16  |
| BigIntConvert | **BigIntConvertShortest**, BigIntConvertNone |
| InfConvert | **InfConvertFloat16**, InfConvertNone |
| NaNConvert | **NaNConvert7e00**, NaNConvertNone, NaNConvertQuiet, NaNConvertPreserveSignal |
| IndefLength | **IndefLengthAllowed**, IndefLengthForbidden  |
| TagsMd | **TagsAllowed**, TagsForbidden |

See [Options](#options) section for details about each setting.

### Decoding Options

| DecOptions | Available Settings (defaults listed first)  |
| :--- | :--- |
| TimeTag | **DecTagIgnored**, DecTagOptional, DecTagRequired |
| DupMapKey | **DupMapKeyQuiet**, DupMapKeyEnforcedAPF |
| IntDec | **IntDecConvertNone**, IntDecConvertSigned |
| IndefLength | **IndefLengthAllowed**, IndefLengthForbidden |
| TagsMd | **TagsAllowed**, TagsForbidden |
| ExtraReturnErrors | **ExtraDecErrorNone**, ExtraDecErrorUnknownField |
| MaxNestedLevels | **32**, can be set to [4, 256] |
| MaxArrayElements | **131072**, can be set to [16, 2147483647] |
| MaxMapPairs | **131072**, can be set to [16, 2147483647] |

See [Options](#options) section for details about each setting.

### Additional Features

* Decoder always checks for invalid UTF-8 string errors.
* Decoder always decodes in-place to slices, maps, and structs.
* Decoder tries case-sensitive first and falls back to case-insensitive field name match when decoding to structs. 
* Decoder supports decoding registered CBOR tag data to interface types. 
* Both encoder and decoder support indefinite length CBOR data (["streaming"](https://tools.ietf.org/html/rfc7049#section-2.2)).
* Both encoder and decoder correctly handles nil slice, map, pointer, and interface values.

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Standards
This library is a full-featured generic CBOR [(RFC 8949)](https://tools.ietf.org/html/rfc8949) encoder and decoder.  Notable CBOR features include:

|   | CBOR Feature  | Description  |
| :--- | :--- | :--- |
| ☑️ | CBOR tags | API supports built-in and user-defined tags.  |
| ☑️ | Preferred serialization | Integers encode to fewest bytes. Optional float64 → float32 → float16. |
| ☑️ | Map key sorting | Unsorted, length-first (Canonical CBOR), and bytewise-lexicographic (CTAP2). |
| ☑️ | Duplicate map keys | Always forbid for encoding and option to allow/forbid for decoding.   |
| ☑️ | Indefinite length data | Option to allow/forbid for encoding and decoding. |
| ☑️ | Well-formedness | Always checked and enforced. |
| ☑️ | Basic validity checks | Check UTF-8 validity and optionally check duplicate map keys. |
| ☑️ | Security considerations | Prevent integer overflow and resource exhaustion (RFC 8949 Section 10). |

See the Features section for list of [Encoding Options](#encoding-options) and [Decoding Options](#decoding-options).

Known limitations are noted in the [Limitations section](#limitations). 

Go nil values for slices, maps, pointers, etc. are encoded as CBOR null.  Empty slices, maps, etc. are encoded as empty CBOR arrays and maps.

Decoder checks for all required well-formedness errors, including all "subkinds" of syntax errors and too little data.

After well-formedness is verified, basic validity errors are handled as follows:

* Invalid UTF-8 string: Decoder always checks and returns invalid UTF-8 string error.
* Duplicate keys in a map: Decoder has options to ignore or enforce rejection of duplicate map keys.

When decoding well-formed CBOR arrays and maps, decoder saves the first error it encounters and continues with the next item.  Options to handle this differently may be added in the future.

By default, decoder treats time values of floating-point NaN and Infinity as if they are CBOR Null or CBOR Undefined.

See [Options](#options) section for detailed settings or [Features](#features) section for a summary of options.

__Click to expand topic:__

<details>
 <summary>Duplicate Map Keys</summary><p>

This library provides options for fast detection and rejection of duplicate map keys based on applying a Go-specific data model to CBOR's extended generic data model in order to determine duplicate vs distinct map keys. Detection relies on whether the CBOR map key would be a duplicate "key" when decoded and applied to the user-provided Go map or struct. 

`DupMapKeyQuiet` turns off detection of duplicate map keys. It tries to use a "keep fastest" method by choosing either "keep first" or "keep last" depending on the Go data type.

`DupMapKeyEnforcedAPF` enforces detection and rejection of duplidate map keys. Decoding stops immediately and returns `DupMapKeyError` when the first duplicate key is detected. The error includes the duplicate map key and the index number. 

APF suffix means "Allow Partial Fill" so the destination map or struct can contain some decoded values at the time of error. It is the caller's responsibility to respond to the `DupMapKeyError` by discarding the partially filled result if that's required by their protocol.

</details>

<details>
 <summary>Tag Validity</summary><p>

This library checks tag validity for built-in tags (currently tag numbers 0, 1, 2, 3, and 55799):

* Inadmissible type for tag content 
* Inadmissible value for tag content

Unknown tag data items (not tag number 0, 1, 2, 3, or 55799) are handled in two ways:

* When decoding into an empty interface, unknown tag data item will be decoded into `cbor.Tag` data type, which contains tag number and tag content.  The tag content will be decoded into the default Go data type for the CBOR data type.
* When decoding into other Go types, unknown tag data item is decoded into the specified Go type.  If Go type is registered with a tag number, the tag number can optionally be verified.

Decoder also has an option to forbid tag data items (treat any tag data item as error) which is specified by protocols such as CTAP2 Canonical CBOR.  

For more information, see [decoding options](#decoding-options-1) and [tag options](#tag-options).

</details>

## Limitations

If any of these limitations prevent you from using this library, please open an issue along with a link to your project.

* CBOR `Undefined` (0xf7) value decodes to Go's `nil` value.  CBOR `Null` (0xf6) more closely matches Go's `nil`.
* CBOR map keys with data types not supported by Go for map keys are ignored and an error is returned after continuing to decode remaining items.  
* When using io.Reader interface to read very large or indefinite length CBOR data, Go's `io.LimitReader` should be used to limit size.
* When decoding registered CBOR tag data to interface type, decoder creates a pointer to registered Go type matching CBOR tag number.  Requiring a pointer for this is a Go limitation. 

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## API
Many function signatures are identical to Go's encoding/json, such as:  
`Marshal`, `Unmarshal`, `NewEncoder`, `NewDecoder`, `(*Encoder).Encode`, and `(*Decoder).Decode`.

Interfaces identical or comparable to Go's encoding, encoding/json, or encoding/gob include:  
`Marshaler`, `Unmarshaler`, `BinaryMarshaler`, and `BinaryUnmarshaler`.

Like `encoding/json`, `RawMessage` can be used to delay CBOR decoding or precompute CBOR encoding.

"Mode" in this API means defined way of encoding or decoding -- it links the standard API to CBOR options and CBOR tags.

EncMode and DecMode are interfaces created from EncOptions or DecOptions structs.  
For example, `em, err := cbor.EncOptions{...}.EncMode()` or `em, err := cbor.CanonicalEncOptions().EncMode()`.

EncMode and DecMode use immutable options so their behavior won't accidentally change at runtime.  Modes are intended to be reused and are safe for concurrent use.

__API for Default Mode__

If default options are acceptable, then you don't need to create EncMode or DecMode.

```go
Marshal(v interface{}) ([]byte, error)
NewEncoder(w io.Writer) *Encoder

Unmarshal(data []byte, v interface{}) error
NewDecoder(r io.Reader) *Decoder
```

__API for Creating & Using Encoding Modes__

```go
// EncMode interface uses immutable options and is safe for concurrent use.
type EncMode interface {
	Marshal(v interface{}) ([]byte, error)
	NewEncoder(w io.Writer) *Encoder
	EncOptions() EncOptions  // returns copy of options
}

// EncOptions specifies encoding options.
type EncOptions struct {
...
}

// EncMode returns an EncMode interface created from EncOptions.
func (opts EncOptions) EncMode() (EncMode, error) {}

// EncModeWithTags returns EncMode with options and tags that are both immutable. 
func (opts EncOptions) EncModeWithTags(tags TagSet) (EncMode, error) {}

// EncModeWithSharedTags returns EncMode with immutable options and mutable shared tags. 
func (opts EncOptions) EncModeWithSharedTags(tags TagSet) (EncMode, error) {}
```

The empty curly braces prevent a syntax highlighting bug, please ignore them.

__API for Predefined Encoding Options__

```go
func CoreDetEncOptions() EncOptions {}              // RFC 8949 Core Deterministic Encoding
func PreferredUnsortedEncOptions() EncOptions {}    // RFC 8949 Preferred Serialization
func CanonicalEncOptions() EncOptions {}            // RFC 7049 Canonical CBOR
func CTAP2EncOptions() EncOptions {}                // FIDO2 CTAP2 Canonical CBOR
```

__API for Creating & Using Decoding Modes__

```go
// DecMode interface uses immutable options and is safe for concurrent use.
type DecMode interface {
	Unmarshal(data []byte, v interface{}) error
	NewDecoder(r io.Reader) *Decoder
	DecOptions() DecOptions  // returns copy of options
}

// DecOptions specifies decoding options.
type DecOptions struct {
...
}

// DecMode returns a DecMode interface created from DecOptions.
func (opts DecOptions) DecMode() (DecMode, error) {}

// DecModeWithTags returns DecMode with options and tags that are both immutable. 
func (opts DecOptions) DecModeWithTags(tags TagSet) (DecMode, error) {}

// DecModeWithSharedTags returns DecMode with immutable options and mutable shared tags. 
func (opts DecOptions) DecModeWithSharedTags(tags TagSet) (DecMode, error) {}
```

The empty curly braces prevent a syntax highlighting bug, please ignore them.

__API for Using CBOR Tags__

`TagSet` can be used to associate user-defined Go type(s) to tag number(s).  It's also used to create EncMode or DecMode. For example, `em := EncOptions{...}.EncModeWithTags(ts)` or `em := EncOptions{...}.EncModeWithSharedTags(ts)`. This allows every standard API exported by em (like `Marshal` and `NewEncoder`) to use the specified tags automatically.

`Tag` and `RawTag` can be used to encode/decode a tag number with a Go value, but `TagSet` is generally recommended.

```go
type TagSet interface {
    // Add adds given tag number(s), content type, and tag options to TagSet.
    Add(opts TagOptions, contentType reflect.Type, num uint64, nestedNum ...uint64) error

    // Remove removes given tag content type from TagSet.
    Remove(contentType reflect.Type)    
}
```

`Tag` and `RawTag` types can also be used to encode/decode tag number with Go value.

```go
type Tag struct {
    Number  uint64
    Content interface{}
}

type RawTag struct {
    Number  uint64
    Content RawMessage
}
```

See [API docs (godoc.org)](https://godoc.org/github.com/fxamacker/cbor) for more details and more functions.  See [Usage section](#usage) for usage and code examples.

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Options

Struct tags, decoding options, and encoding options.

### Struct Tags

This library supports both "cbor" and "json" key for some (not all) struct tags.  If "cbor" and "json" keys are both present for the same field, then "cbor" key will be used.

| Key | Format Str | Scope | Description |
| --- | ---------- | ----- | ------------|
| cbor or json | "myName" | field | Name of field to use such as "myName", etc. like encoding/json. |
| cbor or json | ",omitempty" | field | Omit (ignore) this field if value is empty, like encoding/json. |
| cbor or json | "-" | field | Omit (ignore) this field always, like encoding/json. |
| cbor | ",keyasint" | field | Treat field as an element of CBOR map with specified int as key. |
| cbor | ",toarray" | struct | Treat each field as an element of CBOR array. This automatically disables "omitempty" and "keyasint" for all fields in the struct. |

The "keyasint" struct tag requires an integer key to be specified:

```
type myStruct struct {
    MyField     int64    `cbor:"-1,keyasint,omitempty'`
    OurField    string   `cbor:"0,keyasint,omitempty"`
    FooField    Foo      `cbor:"5,keyasint,omitempty"`
    BarField    Bar      `cbor:"hello,omitempty"`
    ...
}
```

The "toarray" struct tag requires a special field "_" (underscore) to indicate "toarray" applies to the entire struct:

```
type myStruct struct {
    _           struct{}    `cbor:",toarray"`
    MyField     int64
    OurField    string
    ...
}
```

__Click to expand:__

<details>
  <summary>Example Using CBOR Web Tokens</summary><p>
   
![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_struct_tags_api.svg?sanitize=1 "CBOR API and Go Struct Tags")

</details>

### Decoding Options

| DecOptions.TimeTag | Description |
| ------------------ | ----------- |
| DecTagIgnored (default) | Tag numbers are ignored (if present) for time values. |
| DecTagOptional | Tag numbers are only checked for validity if present for time values. |
| DecTagRequired | Tag numbers must be provided for time values except for CBOR Null and CBOR Undefined. |

The following CBOR time values are decoded as Go's "zero time instant":

* CBOR Null
* CBOR Undefined
* CBOR floating-point NaN
* CBOR floating-point Infinity

Go's `time` package provides `IsZero` function, which reports whether t represents "zero time instant"  
(January 1, year 1, 00:00:00 UTC).

<br>

| DecOptions.DupMapKey | Description |
| -------------------- | ----------- |
| DupMapKeyQuiet (default) | turns off detection of duplicate map keys. It uses a "keep fastest" method by choosing either "keep first" or "keep last" depending on the Go data type. |
| DupMapKeyEnforcedAPF | enforces detection and rejection of duplidate map keys. Decoding stops immediately and returns `DupMapKeyError` when the first duplicate key is detected. The error includes the duplicate map key and the index number. |

`DupMapKeyEnforcedAPF` uses "Allow Partial Fill" so the destination map or struct can contain some decoded values at the time of error.  Users can respond to the `DupMapKeyError` by discarding the partially filled result if that's required by their protocol.

<br>

| DecOptions.IntDec | Description |
| ------------------ | ----------- |
| IntDecConvertNone (default) | When decoding to Go interface{}, CBOR positive int (major type 0) decode to uint64 value, and CBOR negative int (major type 1) decode to int64 value. |
| IntDecConvertSigned | When decoding to Go interface{}, CBOR positive/negative int (major type 0 and 1) decode to int64 value. |

If `IntDecConvertedSigned` is used and value overflows int64, UnmarshalTypeError is returned.

<br>

| DecOptions.IndefLength | Description |
| ---------------------- | ----------- |
|IndefLengthAllowed (default) | allow indefinite length data |
|IndefLengthForbidden | forbid indefinite length data |

<br>

| DecOptions.TagsMd | Description |
| ----------------- | ----------- |
|TagsAllowed (default) | allow CBOR tags (major type 6) |
|TagsForbidden | forbid CBOR tags (major type 6) |

<br>

| DecOptions.ExtraReturnErrors | Description |
| ----------------- | ----------- |
|ExtraDecErrorNone (default) | no extra decoding errors.  E.g. ignore unknown fields if encountered. |
|ExtraDecErrorUnknownField | return error if unknown field is encountered |

<br>

| DecOptions.MaxNestedLevels | Description |
| -------------------------- | ----------- |
| 32 (default) | allowed setting is [4, 256] |

<br>

| DecOptions.MaxArrayElements | Description |
| --------------------------- | ----------- |
| 131072 (default) | allowed setting is [16, 2147483647] |

<br>

| DecOptions.MaxMapPairs | Description |
| ---------------------- | ----------- |
| 131072 (default) | allowed setting is [16, 2147483647] |

### Encoding Options

__Integers always encode to the shortest form that preserves value__.  Encoding of other data types and map key sort order are determined by encoding options.

These functions are provided to create and return a modifiable EncOptions struct with predefined settings.

| Predefined EncOptions | Description |
| --------------------- | ----------- |
| CanonicalEncOptions() |[Canonical CBOR (RFC 7049 Section 3.9)](https://tools.ietf.org/html/rfc7049#section-3.9). |
| CTAP2EncOptions() |[CTAP2 Canonical CBOR (FIDO2 CTAP2)](https://fidoalliance.org/specs/fido-v2.0-id-20180227/fido-client-to-authenticator-protocol-v2.0-id-20180227.html#ctap2-canonical-cbor-encoding-form). |
| PreferredUnsortedEncOptions() |Unsorted, encode float64->float32->float16 when values fit, NaN values encoded as float16 0x7e00. |
| CoreDetEncOptions() |PreferredUnsortedEncOptions() + map keys are sorted bytewise lexicographic. |

<br>

| EncOptions.Sort | Description |
| --------------- | ----------- |
| SortNone (default) |No sorting for map keys. |
| SortLengthFirst |Length-first map key ordering. |
| SortBytewiseLexical |Bytewise lexicographic map key ordering [(RFC 8949 Section 4.2.1)](https://datatracker.ietf.org/doc/html/rfc8949#section-4.2.1).|
| SortCanonical |(alias) Same as SortLengthFirst [(RFC 7049 Section 3.9)](https://tools.ietf.org/html/rfc7049#section-3.9) |
| SortCTAP2 |(alias) Same as SortBytewiseLexical [(CTAP2 Canonical CBOR)](https://fidoalliance.org/specs/fido-v2.0-id-20180227/fido-client-to-authenticator-protocol-v2.0-id-20180227.html#ctap2-canonical-cbor-encoding-form). |
| SortCoreDeterministic |(alias) Same as SortBytewiseLexical [(RFC 8949 Section 4.2.1)](https://datatracker.ietf.org/doc/html/rfc8949#section-4.2.1). |

<br>

| EncOptions.Time | Description |
| --------------- | ----------- |
| TimeUnix (default) | (seconds) Encode as integer. |
| TimeUnixMicro | (microseconds) Encode as floating-point.  ShortestFloat option determines size. |
| TimeUnixDynamic | (seconds or microseconds) Encode as integer if time doesn't have fractional seconds, otherwise encode as floating-point rounded to microseconds. |
| TimeRFC3339 | (seconds) Encode as RFC 3339 formatted string. |
| TimeRFC3339Nano | (nanoseconds) Encode as RFC3339 formatted string. |

<br>

| EncOptions.TimeTag | Description |
| ------------------ | ----------- |
| EncTagNone (default) | Tag number will not be encoded for time values. |
| EncTagRequired | Tag number (0 or 1) will be encoded unless time value is undefined/zero-instant. |

By default, undefined (zero instant) time values will encode as CBOR Null without tag number for both EncTagNone and EncTagRequired.  Although CBOR Undefined might be technically more correct for EncTagRequired, CBOR Undefined might not be supported by other generic decoders and it isn't supported by JSON.

Go's `time` package provides `IsZero` function, which reports whether t represents the zero time instant, January 1, year 1, 00:00:00 UTC. 

<br>

| EncOptions.BigIntConvert | Description |
| ------------------------ | ----------- |
| BigIntConvertShortest (default) | Encode big.Int as CBOR integer if value fits. |
| BigIntConvertNone | Encode big.Int as CBOR bignum (tag 2 or 3). |

<br>

__Floating-Point Options__

Encoder has 3 types of options for floating-point data: ShortestFloatMode, InfConvertMode, and NaNConvertMode.

| EncOptions.ShortestFloat | Description |
| ------------------------ | ----------- |
| ShortestFloatNone (default) | No size conversion. Encode float32 and float64 to CBOR floating-point of same bit-size. |
| ShortestFloat16 | Encode float64 -> float32 -> float16 ([IEEE 754 binary16](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)) when values fit. |

Conversions for infinity and NaN use InfConvert and NaNConvert settings.

| EncOptions.InfConvert | Description |
| --------------------- | ----------- |
| InfConvertFloat16 (default) | Convert +- infinity to float16 since they always preserve value (recommended) |
| InfConvertNone |Don't convert +- infinity to other representations -- used by CTAP2 Canonical CBOR |

<br>

| EncOptions.NaNConvert | Description |
| --------------------- | ----------- |
| NaNConvert7e00 (default) | Encode to 0xf97e00 (CBOR float16 = 0x7e00) -- used by RFC 8949 Preferred Encoding, etc. |
| NaNConvertNone | Don't convert NaN to other representations -- used by CTAP2 Canonical CBOR. |
| NaNConvertQuiet | Force quiet bit = 1 and use shortest form that preserves NaN payload. |
| NaNConvertPreserveSignal | Convert to smallest form that preserves value (quit bit unmodified and NaN payload preserved). |

<br>

| EncOptions.IndefLength | Description |
| ---------------------- | ----------- |
|IndefLengthAllowed (default) | allow indefinite length data |
|IndefLengthForbidden | forbid indefinite length data |

<br>

| EncOptions.TagsMd | Description |
| ----------------- | ----------- |
|TagsAllowed (default) | allow CBOR tags (major type 6) |
|TagsForbidden | forbid CBOR tags (major type 6) |


### Tag Options

TagOptions specifies how encoder and decoder handle tag number registered with TagSet.

| TagOptions.DecTag | Description |
| ------------------ | ----------- |
| DecTagIgnored (default) | Tag numbers are ignored (if present). |
| DecTagOptional | Tag numbers are only checked for validity if present. |
| DecTagRequired | Tag numbers must be provided except for CBOR Null and CBOR Undefined. |

<br>

| TagOptions.EncTag | Description |
| ------------------ | ----------- |
| EncTagNone (default) | Tag number will not be encoded. |
| EncTagRequired | Tag number will be encoded. |
	
<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Usage
🛡️ Use Go's `io.LimitReader` to limit size when decoding very large or indefinite size data.

Functions with identical signatures to encoding/json include:  
`Marshal`, `Unmarshal`, `NewEncoder`, `NewDecoder`, `(*Encoder).Encode`, `(*Decoder).Decode`.

__Default Mode__  

If default options are acceptable, package level functions can be used for encoding and decoding.

```go
b, err := cbor.Marshal(v)        // encode v to []byte b

err := cbor.Unmarshal(b, &v)     // decode []byte b to v

encoder := cbor.NewEncoder(w)    // create encoder with io.Writer w

decoder := cbor.NewDecoder(r)    // create decoder with io.Reader r
```

__Modes__

If you need to use options or CBOR tags, then you'll want to create a mode.

"Mode" means defined way of encoding or decoding -- it links the standard API to your CBOR options and CBOR tags.  This way, you don't pass around options and the API remains identical to `encoding/json`.

EncMode and DecMode are interfaces created from EncOptions or DecOptions structs.  
For example, `em, err := cbor.EncOptions{...}.EncMode()` or `em, err := cbor.CanonicalEncOptions().EncMode()`.

EncMode and DecMode use immutable options so their behavior won't accidentally change at runtime.  Modes are reusable, safe for concurrent use, and allow fast parallelism.

__Creating and Using Encoding Modes__

EncMode is an interface ([API](#api)) created from EncOptions struct.  EncMode uses immutable options after being created and is safe for concurrent use.  For best performance, EncMode should be reused.

```go
// Create EncOptions using either struct literal or a function.
opts := cbor.CanonicalEncOptions()

// If needed, modify opts. For example: opts.Time = cbor.TimeUnix

// Create reusable EncMode interface with immutable options, safe for concurrent use.
em, err := opts.EncMode()   

// Use EncMode like encoding/json, with same function signatures.
b, err := em.Marshal(v)      // encode v to []byte b

encoder := em.NewEncoder(w)  // create encoder with io.Writer w
err := encoder.Encode(v)     // encode v to io.Writer w
```

__Struct Tags (keyasint, toarray, omitempty)__

The `keyasint`, `toarray`, and `omitempty` struct tags make it easy to use compact CBOR message formats.  Internet standards often use CBOR arrays and CBOR maps with int keys to save space.

<hr>

![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_struct_tags_api.svg?sanitize=1 "CBOR API and Struct Tags")

<hr>

__Decoding CWT (CBOR Web Token)__ using `keyasint` and `toarray` struct tags:

```go
// Signed CWT is defined in RFC 8392
type signedCWT struct {
	_           struct{} `cbor:",toarray"`
	Protected   []byte
	Unprotected coseHeader
	Payload     []byte
	Signature   []byte
}

// Part of COSE header definition
type coseHeader struct {
	Alg int    `cbor:"1,keyasint,omitempty"`
	Kid []byte `cbor:"4,keyasint,omitempty"`
	IV  []byte `cbor:"5,keyasint,omitempty"`
}

// data is []byte containing signed CWT

var v signedCWT
if err := cbor.Unmarshal(data, &v); err != nil {
	return err
}
```

__Encoding CWT (CBOR Web Token)__ using `keyasint` and `toarray` struct tags:

```go
// Use signedCWT struct defined in "Decoding CWT" example.

var v signedCWT
...
if data, err := cbor.Marshal(v); err != nil {
	return err
}
```

__Encoding and Decoding CWT (CBOR Web Token) with CBOR Tags__

```go
// Use signedCWT struct defined in "Decoding CWT" example.

// Create TagSet (safe for concurrency).
tags := cbor.NewTagSet()
// Register tag COSE_Sign1 18 with signedCWT type.
tags.Add(	
	cbor.TagOptions{EncTag: cbor.EncTagRequired, DecTag: cbor.DecTagRequired}, 
	reflect.TypeOf(signedCWT{}), 
	18)

// Create DecMode with immutable tags.
dm, _ := cbor.DecOptions{}.DecModeWithTags(tags)

// Unmarshal to signedCWT with tag support.
var v signedCWT
if err := dm.Unmarshal(data, &v); err != nil {
	return err
}

// Create EncMode with immutable tags.
em, _ := cbor.EncOptions{}.EncModeWithTags(tags)

// Marshal signedCWT with tag number.
if data, err := cbor.Marshal(v); err != nil {
	return err
}
```

For more examples, see [examples_test.go](example_test.go).

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Comparisons

Comparisons are between this newer library and a well-known library that had 1,000+ stars before this library was created.  Default build settings for each library were used for all comparisons.

__This library is safer__.  Small malicious CBOR messages are rejected quickly before they exhaust system resources.

Decoding 9 or 10 bytes of malformed CBOR data shouldn't exhaust memory. For example,  
`[]byte{0x9B, 0x00, 0x00, 0x42, 0xFA, 0x42, 0xFA, 0x42, 0xFA, 0x42}`

|     | Decode bad 10 bytes to interface{} | Decode bad 10 bytes to []byte |
| :--- | :------------------ | :--------------- |
| fxamacker/cbor<br/>1.0-2.3 | 49.44 ns/op, 24 B/op, 2 allocs/op* | 51.93 ns/op, 32 B/op, 2 allocs/op* |
| ugorji/go 1.2.6 | ⚠️ 45021 ns/op, 262852 B/op, 7 allocs/op | 💥 runtime: out of memory: cannot allocate |
| ugorji/go 1.1.0-1.1.7 | 💥 runtime: out of memory: cannot allocate | 💥 runtime: out of memory: cannot allocate|

*Speed and memory are for latest codec version listed in the row (compiled with Go 1.17.5).

fxamacker/cbor CBOR safety settings include: MaxNestedLevels, MaxArrayElements, MaxMapPairs, and IndefLength.

__This library is smaller__. Programs like senmlCat can be 4 MB smaller by switching to this library.  Programs using more complex CBOR data types can be 9.2 MB smaller.

![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_size_comparison.svg?sanitize=1 "CBOR speed comparison chart")


__This library is faster__ for encoding and decoding CBOR Web Token (CWT).  However, speed is only one factor and it can vary depending on data types and sizes.  Unlike the other library, this one doesn't use Go's ```unsafe``` package or code gen.

![alt text](https://github.com/fxamacker/images/raw/master/cbor/v2.3.0/cbor_speed_comparison.svg?sanitize=1 "CBOR speed comparison chart")

__This library uses less memory__ for encoding and decoding CBOR Web Token (CWT) using test data from RFC 8392 A.1.

|  | fxamacker/cbor 2.3 | ugorji/go 1.2.6 |
| :--- | :--- | :--- | 
| Encode CWT | 0.18 kB/op &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 2 allocs/op | 1.35 kB/op &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 4 allocs/op |
| Decode CWT | 160 bytes/op &nbsp;&nbsp;&nbsp; 6 allocs/op | 744 bytes/op &nbsp;&nbsp;&nbsp; 6 allocs/op |

Running your own benchmarks is highly recommended.  Use your most common data structures and data sizes.

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Benchmarks

Go structs are faster than maps with string keys:

* decoding into struct is >28% faster than decoding into map.
* encoding struct is >35% faster than encoding map.

Go structs with `keyasint` struct tag are faster than maps with integer keys:

* decoding into struct is >28% faster than decoding into map.
* encoding struct is >34% faster than encoding map.

Go structs with `toarray` struct tag are faster than slice:

* decoding into struct is >15% faster than decoding into slice.
* encoding struct is >12% faster than encoding slice.

Doing your own benchmarks is highly recommended.  Use your most common message sizes and data types.

See [Benchmarks for fxamacker/cbor](CBOR_BENCHMARKS.md).

## Fuzzing and Code Coverage

__Over 375 tests__ must pass on 4 architectures before tagging a release.  They include all RFC 7049 and RFC 8949 examples, bugs found by fuzzing, maliciously crafted CBOR data, and over 87 tests with malformed data.  There's some overlap in the tests but it isn't a high priority to trim tests.

__Code coverage__ must not fall below 95% when tagging a release.  Code coverage is above 98% (`go test -cover`) for cbor v2.3 which is among the highest for libraries (in Go) of this type.

__Coverage-guided fuzzing__ must pass 1+ billion execs using a large corpus before tagging a release.  Fuzzing is usually continued after the release is tagged and is manually stopped after reaching 1-3 billion execs.  Fuzzing uses a customized version of [dvyukov/go-fuzz](https://github.com/dvyukov/go-fuzz).

To prevent delays to release schedules, fuzzing is not restarted for a release if changes are limited to ci, docs, and comments.

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)

## Versions and API Changes
This project uses [Semantic Versioning](https://semver.org), so the API is always backwards compatible unless the major version number changes.  

These functions have signatures identical to encoding/json and they will likely never change even after major new releases:  
`Marshal`, `Unmarshal`, `NewEncoder`, `NewDecoder`, `(*Encoder).Encode`, and `(*Decoder).Decode`.

Newly added API documented as "subject to change" are excluded from SemVer.

Newly added API in the master branch that has never been release tagged are excluded from SemVer.

## Code of Conduct 
This project has adopted the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).  Contact [faye.github@gmail.com](mailto:faye.github@gmail.com) with any questions or comments.

## Contributing
Please refer to [How to Contribute](CONTRIBUTING.md).

## Security Policy
Security fixes are provided for the latest released version of fxamacker/cbor.

For the full text of the Security Policy, see [SECURITY.md](SECURITY.md).

## Disclaimers
Phrases like "no crashes", "doesn't crash", and "is secure" mean there are no known crash bugs in the latest version based on results of unit tests and coverage-guided fuzzing.  They don't imply the software is 100% bug-free or 100% invulnerable to all known and unknown attacks.

Please read the license for additional disclaimers and terms.

## Special Thanks

__Making this library better__  

* Stefan Tatschner for using this library in [sep](https://rumpelsepp.org/projects/sep), being the 1st to discover my CBOR library, requesting time.Time in issue #1, and submitting this library in a [PR to cbor.io](https://github.com/cbor/cbor.github.io/pull/56) on Aug 12, 2019.
* Yawning Angel for using this library to [oasis-core](https://github.com/oasislabs/oasis-core), and requesting BinaryMarshaler in issue #5.
* Jernej Kos for requesting RawMessage in issue #11 and offering feedback on v2.1 API for CBOR tags.
* ZenGround0 for using this library in [go-filecoin](https://github.com/filecoin-project/go-filecoin), filing "toarray" bug in issue #129, and requesting  
CBOR BSTR <--> Go array in #133.
* Keith Randall for [fixing Go bugs and providing workarounds](https://github.com/golang/go/issues/36400) so we don't have to wait for new versions of Go.

__Help clarifying CBOR RFC 7049 or 7049bis (7049bis is the draft of RFC 8949)__

* Carsten Bormann for RFC 7049 (CBOR), adding this library to cbor.io, his fast confirmation to my RFC 7049 errata, approving my pull request to 7049bis, and his patience when I misread a line in 7049bis.
* Laurence Lundblade for his help on the IETF mailing list for 7049bis and for pointing out on a CBORbis issue that CBOR Undefined might be problematic translating to JSON.
* Jeffrey Yasskin for his help on the IETF mailing list for 7049bis.

__Words of encouragement and support__

* Jakob Borg for his words of encouragement about this library at Go Forum.  This is especially appreciated in the early stages when there's a lot of rough edges.


## License 
Copyright © 2019-2022 [Faye Amacker](https://github.com/fxamacker).  

fxamacker/cbor is licensed under the MIT License.  See [LICENSE](LICENSE) for the full license text.  

<hr>

⚓  [Quick Start](#quick-start) • [Features](#features) • [Standards](#standards) • [API](#api) • [Options](#options) • [Usage](#usage) • [Fuzzing](#fuzzing-and-code-coverage) • [License](#license)
