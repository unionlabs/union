# CBOR Benchmarks for fxamacker/cbor 

See [bench_test.go](bench_test.go).

Benchmarks on Feb. 22, 2020 with cbor v2.2.0:
* [Go builtin types](#go-builtin-types)
* [Go structs](#go-structs)
* [Go structs with "keyasint" struct tag](#go-structs-with-keyasint-struct-tag)
* [Go structs with "toarray" struct tag](#go-structs-with-toarray-struct-tag)
* [COSE data](#cose-data)
* [CWT claims data](#cwt-claims-data)
* [SenML data](#SenML-data)

## Go builtin types

Benchmarks use data representing the following values:

* Boolean: `true`
* Positive integer: `18446744073709551615`
* Negative integer: `-1000`
* Float: `-4.1`
* Byte string: `h'0102030405060708090a0b0c0d0e0f101112131415161718191a'`
* Text string: `"The quick brown fox jumps over the lazy dog"`
* Array: `[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26]`
* Map: `{"a": "A", "b": "B", "c": "C", "d": "D", "e": "E", "f": "F", "g": "G", "h": "H", "i": "I", "j": "J", "l": "L", "m": "M", "n": "N"}}`

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshal/CBOR_bool_to_Go_interface_{}-2 | 110 ns/op | 16 B/op | 1 allocs/op
BenchmarkUnmarshal/CBOR_bool_to_Go_bool-2 | 99.3 ns/op | 1 B/op | 1 allocs/op
BenchmarkUnmarshal/CBOR_positive_int_to_Go_interface_{}-2 | 135 ns/op | 24 B/op | 2 allocs/op
BenchmarkUnmarshal/CBOR_positive_int_to_Go_uint64-2 | 116 ns/op | 8 B/op | 1 allocs/op
BenchmarkUnmarshal/CBOR_negative_int_to_Go_interface_{}-2 | 133 ns/op | 24 B/op | 2 allocs/op
BenchmarkUnmarshal/CBOR_negative_int_to_Go_int64-2 | 113 ns/op | 8 B/op | 1 allocs/op
BenchmarkUnmarshal/CBOR_float_to_Go_interface_{}-2 | 137 ns/op | 24 B/op | 2 allocs/op
BenchmarkUnmarshal/CBOR_float_to_Go_float64-2 | 115 ns/op | 8 B/op | 1 allocs/op
BenchmarkUnmarshal/CBOR_bytes_to_Go_interface_{}-2 | 179 ns/op | 80 B/op | 3 allocs/op
BenchmarkUnmarshal/CBOR_bytes_to_Go_[]uint8-2 | 194 ns/op | 64 B/op | 2 allocs/op
BenchmarkUnmarshal/CBOR_text_to_Go_interface_{}-2 | 209 ns/op | 80 B/op | 3 allocs/op
BenchmarkUnmarshal/CBOR_text_to_Go_string-2 | 193 ns/op | 64 B/op | 2 allocs/op
BenchmarkUnmarshal/CBOR_array_to_Go_interface_{}-2 |1068 ns/op | 672 B/op | 29 allocs/op
BenchmarkUnmarshal/CBOR_array_to_Go_[]int-2 | 1073 ns/op | 272 B/op | 3 allocs/op
BenchmarkUnmarshal/CBOR_map_to_Go_interface_{}-2 | 2926 ns/op | 1420 B/op | 30 allocs/op
BenchmarkUnmarshal/CBOR_map_to_Go_map[string]interface_{}-2 | 3755 ns/op | 965 B/op | 19 allocs/op
BenchmarkUnmarshal/CBOR_map_to_Go_map[string]string-2 | 2586 ns/op | 740 B/op | 5 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshal/Go_bool_to_CBOR_bool-2 | 86.1 ns/op	| 1 B/op | 1 allocs/op
BenchmarkMarshal/Go_uint64_to_CBOR_positive_int-2 | 97.0 ns/op | 16 B/op | 1 allocs/op
BenchmarkMarshal/Go_int64_to_CBOR_negative_int-2 | 90.3 ns/op | 3 B/op | 1 allocs/op
BenchmarkMarshal/Go_float64_to_CBOR_float-2 | 97.9 ns/op	| 16 B/op | 1 allocs/op
BenchmarkMarshal/Go_[]uint8_to_CBOR_bytes-2 | 121 ns/op | 32 B/op	| 1 allocs/op
BenchmarkMarshal/Go_string_to_CBOR_text-2 | 115 ns/op | 48 B/op | 1 allocs/op
BenchmarkMarshal/Go_[]int_to_CBOR_array-2 | 529 ns/op | 32 B/op	| 1 allocs/op
BenchmarkMarshal/Go_map[string]string_to_CBOR_map-2 | 2115 ns/op | 576 B/op | 28 allocs/op

## Go structs

Benchmarks use struct and map[string]interface{} representing the following value:

```
{
    "T":    true,
    "Ui":   uint(18446744073709551615),
    "I":    -1000,
    "F":    -4.1,
    "B":    []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26},
    "S":    "The quick brown fox jumps over the lazy dog",
    "Slci": []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26},
    "Mss":  map[string]string{"a": "A", "b": "B", "c": "C", "d": "D", "e": "E", "f": "F", "g": "G", "h": "H", "i": "I", "j": "J", "l": "L", "m": "M", "n": "N"},
}
```

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshal/CBOR_map_to_Go_map[string]interface{}-2 | 6221 ns/op | 2621 B/op | 73 allocs/op
BenchmarkUnmarshal/CBOR_map_to_Go_struct-2 | 4458 ns/op | 1172 B/op | 10 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshal/Go_map[string]interface{}_to_CBOR_map-2 | 4441 ns/op | 1072 B/op | 45 allocs/op
BenchmarkMarshal/Go_struct_to_CBOR_map-2 | 2866 ns/op | 720 B/op | 28 allocs/op

## Go structs with "keyasint" struct tag

Benchmarks use struct (with keyasint struct tag) and map[int]interface{} representing the following value:

```
{
    1: true,
    2: uint(18446744073709551615),
    3: -1000,
    4: -4.1,
    5: []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26},
    6: "The quick brown fox jumps over the lazy dog",
    7: []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26},
    8: map[string]string{"a": "A", "b": "B", "c": "C", "d": "D", "e": "E", "f": "F", "g": "G", "h": "H", "i": "I", "j": "J", "l": "L", "m": "M", "n": "N"},
}
```

Struct type with keyasint struct tag is used to handle CBOR map with integer keys.

```
type T struct {
	T    bool              `cbor:"1,keyasint"`
	Ui   uint              `cbor:"2,keyasint"`
	I    int               `cbor:"3,keyasint"`
	F    float64           `cbor:"4,keyasint"`
	B    []byte            `cbor:"5,keyasint"`
	S    string            `cbor:"6,keyasint"`
	Slci []int             `cbor:"7,keyasint"`
	Mss  map[string]string `cbor:"8,keyasint"`
}
```

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshal/CBOR_map_to_Go_map[int]interface{}-2| 6030 ns/op | 2517 B/op | 70 allocs/op
BenchmarkUnmarshal/CBOR_map_to_Go_struct_keyasint-2 | 4332 ns/op | 1173 B/op | 10 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshal/Go_map[int]interface{}_to_CBOR_map-2 | 4348 ns/op | 992 B/op | 45 allocs/op
BenchmarkMarshal/Go_struct_keyasint_to_CBOR_map-2 | 2847 ns/op | 704 B/op | 28 allocs/op

## Go structs with "toarray" struct tag

Benchmarks use struct (with toarray struct tag) and []interface{} representing the following value:

```
[
    true,
    uint(18446744073709551615),
    -1000,
    -4.1,
    []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26},
    "The quick brown fox jumps over the lazy dog",
    []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26},
    map[string]string{"a": "A", "b": "B", "c": "C", "d": "D", "e": "E", "f": "F", "g": "G", "h": "H", "i": "I", "j": "J", "l": "L", "m": "M", "n": "N"}
]
```

Struct type with toarray struct tag is used to handle CBOR array.

```
type T struct {
	_    struct{} `cbor:",toarray"`
	T    bool
	Ui   uint
	I    int
	F    float64
	B    []byte
	S    string
	Slci []int
	Mss  map[string]string
}
```

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshal/CBOR_array_to_Go_[]interface{}-2 | 4863 ns/op | 2404 B/op | 67 allocs/op
BenchmarkUnmarshal/CBOR_array_to_Go_struct_toarray-2 | 4173 ns/op | 1164 B/op | 9 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshal/Go_[]interface{}_to_CBOR_map-2 | 3240 ns/op | 704 B/op | 28 allocs/op
BenchmarkMarshal/Go_struct_toarray_to_CBOR_array-2 | 2823 ns/op | 704 B/op | 28 allocs/op

## COSE data

Benchmarks use COSE data from https://tools.ietf.org/html/rfc8392#appendix-A section A.2

```
// 128-Bit Symmetric COSE_Key
{
    / k /   -1: h'231f4c4d4d3051fdc2ec0a3851d5b383'
    / kty /  1: 4 / Symmetric /,
    / kid /  2: h'53796d6d6574726963313238' / 'Symmetric128' /,
    / alg /  3: 10 / AES-CCM-16-64-128 /
}
// 256-Bit Symmetric COSE_Key 
{
    / k /   -1: h'403697de87af64611c1d32a05dab0fe1fcb715a86ab435f1
                ec99192d79569388'
    / kty /  1: 4 / Symmetric /,
    / kid /  4: h'53796d6d6574726963323536' / 'Symmetric256' /,
    / alg /  3: 4 / HMAC 256/64 /
}
// ECDSA 256-Bit COSE Key
{
    / d /   -4: h'6c1382765aec5358f117733d281c1c7bdc39884d04a45a1e
                6c67c858bc206c19',
    / y /   -3: h'60f7f1a780d8a783bfb7a2dd6b2796e8128dbbcef9d3d168
                db9529971a36e7b9',
    / x /   -2: h'143329cce7868e416927599cf65a34f3ce2ffda55a7eca69
                ed8919a394d42f0f',
    / crv / -1: 1 / P-256 /,
    / kty /  1: 2 / EC2 /,
    / kid /  2: h'4173796d6d657472696345434453413
                23536' / 'AsymmetricECDSA256' /,
    / alg /  3: -7 / ECDSA 256 /
}
```

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshalCOSE/128-Bit_Symmetric_Key-2 | 562 ns/op | 240 B/op | 4 allocs/op
BenchmarkUnmarshalCOSE/256-Bit_Symmetric_Key-2 | 568 ns/op | 256 B/op | 4 allocs/op
BenchmarkUnmarshalCOSE/ECDSA_P256_256-Bit_Key-2 | 968 ns/op | 360 B/op | 7 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshalCOSE/128-Bit_Symmetric_Key-2 | 523 ns/op | 224 B/op | 2 allocs/op
BenchmarkMarshalCOSE/256-Bit_Symmetric_Key-2 | 521 ns/op | 240 B/op | 2 allocs/op
BenchmarkMarshalCOSE/ECDSA_P256_256-Bit_Key-2 | 668 ns/op | 320 B/op | 2 allocs/op

## CWT claims data

Benchmarks use CTW claims data from https://tools.ietf.org/html/rfc8392#appendix-A section A.1

```
{
    / iss / 1: "coap://as.example.com",
    / sub / 2: "erikw",
    / aud / 3: "coap://light.example.com",
    / exp / 4: 1444064944,
    / nbf / 5: 1443944944,
    / iat / 6: 1443944944,
    / cti / 7: h'0b71'
}
```

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshalCWTClaims-2 | 765 ns/op | 176 B/op | 6 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshalCWTClaims-2 | 451 ns/op | 176 B/op | 2 allocs/op

## SenML data

Benchmarks use SenML data from https://tools.ietf.org/html/rfc8428#section-6

```
[
    {-2: "urn:dev:ow:10e2073a0108006:", -3: 1276020076.001, -4: "A", -1: 5, 0: "voltage", 1: "V", 2: 120.1},
    {0: "current", 6: -5, 2: 1.2}, 
    {0: "current", 6: -4, 2: 1.3},
    {0: "current", 6: -3, 2: 1.4}, 
    {0: "current", 6: -2, 2: 1.5},
    {0: "current", 6: -1, 2: 1.6}, 
    {0: "current", 6: 0, 2: 1.7}
]
```

Decoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkUnmarshalSenML-2 | 3106 ns/op | 1544 B/op | 18 allocs/op

Encoding Benchmark | Time | Memory | Allocs 
--- | ---: | ---: | ---:
BenchmarkMarshalSenML-2 | 2976 ns/op | 272 B/op	| 2 allocs/op
