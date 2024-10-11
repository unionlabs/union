# Results

* [Curve25519 Field Inversion](#curve25519-field-inversion)
* [NIST P-256 Field Inversion](#nist-p-256-field-inversion)
* [NIST P-384 Field Inversion](#nist-p-384-field-inversion)
* [secp256k1 (Bitcoin) Field Inversion](#secp256k1-bitcoin-field-inversion)
* [Curve25519 Scalar Inversion](#curve25519-scalar-inversion)
* [NIST P-256 Scalar Inversion](#nist-p-256-scalar-inversion)
* [NIST P-384 Scalar Inversion](#nist-p-384-scalar-inversion)
* [secp256k1 (Bitcoin) Scalar Inversion](#secp256k1-bitcoin-scalar-inversion)
* [M-221 Field Inversion](#m-221-field-inversion)
* [E-222 Field Inversion](#e-222-field-inversion)
* [Curve1174 Field Inversion](#curve1174-field-inversion)
* [E-382 Field Inversion](#e-382-field-inversion)
* [M-383/Curve383187 Field Inversion](#m-383curve383187-field-inversion)
* [Curve41417 Field Inversion](#curve41417-field-inversion)
* [M-511 Field Inversion](#m-511-field-inversion)
* [NIST P-192 Field Inversion](#nist-p-192-field-inversion)
* [NIST P-224 Field Inversion](#nist-p-224-field-inversion)
* [Goldilocks Field Inversion](#goldilocks-field-inversion)
* [secp192k1 Field Inversion](#secp192k1-field-inversion)
* [secp224k1 Field Inversion](#secp224k1-field-inversion)


## Curve25519 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^255-19` |
| _d_ | `2` |
| _N_-_d_ | `7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeb` |
| Length | 266 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |
| Best Known | 265 |
| Delta | +1 |


Addition chain produced by `addchain`:

```go
_10       = 2*1
_11       = 1 + _10
_1100     = _11 << 2
_1111     = _11 + _1100
_11110000 = _1111 << 4
_11111111 = _1111 + _11110000
x10       = _11111111 << 2 + _11
x20       = x10 << 10 + x10
x30       = x20 << 10 + x10
x60       = x30 << 30 + x30
x120      = x60 << 60 + x60
x240      = x120 << 120 + x120
x250      = x240 << 10 + x10
return      (x250 << 2 + 1) << 3 + _11
```

## NIST P-256 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^256-2^224+2^192+2^96-1` |
| _d_ | `3` |
| _N_-_d_ | `ffffffff00000001000000000000000000000000fffffffffffffffffffffffc` |
| Length | 266 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |
| Best Known | 266 |
| Delta | +0 |


Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_1100   = _11 << 2
_1111   = _11 + _1100
_111100 = _1111 << 2
_111111 = _11 + _111100
x12     = _111111 << 6 + _111111
x24     = x12 << 12 + x12
x30     = x24 << 6 + _111111
x32     = x30 << 2 + _11
i232    = ((x32 << 32 + 1) << 128 + x32) << 32
return    ((x32 + i232) << 30 + x30) << 2
```

## NIST P-384 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^384-2^128-2^96+2^32-1` |
| _d_ | `3` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000fffffffc` |
| Length | 397 |
| Algorithm | `opt(runs(heuristic(use_first(halving,approximation))))` |
| Best Known | 396 |
| Delta | +1 |


Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_110    = 2*_11
_111    = 1 + _110
_111000 = _111 << 3
_111111 = _111 + _111000
x12     = _111111 << 6 + _111111
x24     = x12 << 12 + x12
x30     = x24 << 6 + _111111
x31     = 2*x30 + 1
x32     = 2*x31 + 1
x63     = x32 << 31 + x31
x126    = x63 << 63 + x63
x252    = x126 << 126 + x126
x255    = x252 << 3 + _111
return    ((x255 << 33 + x32) << 94 + x30) << 2
```

## secp256k1 (Bitcoin) Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f` |
| _d_ | `3` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2c` |
| Length | 269 |
| Algorithm | `opt(runs(heuristic(use_first(halving,delta_largest))))` |
| Best Known | 269 |
| Delta | +0 |


Addition chain produced by `addchain`:

```go
_10      = 2*1
_11      = 1 + _10
_1100    = _11 << 2
_1111    = _11 + _1100
_11110   = 2*_1111
_11111   = 1 + _11110
_1111100 = _11111 << 2
_1111111 = _11 + _1111100
x11      = _1111111 << 4 + _1111
x22      = x11 << 11 + x11
x27      = x22 << 5 + _11111
x54      = x27 << 27 + x27
x108     = x54 << 54 + x54
x216     = x108 << 108 + x108
x223     = x216 << 7 + _1111111
i266     = ((x223 << 23 + x22) << 5 + 1) << 3
return     (_11 + i266) << 2
```

## Curve25519 Scalar Inversion

| Property | Value |
| --- | ----- |
| _N_ | `1000000000000000000000000000000014def9dea2f79cd65812631a5cf5d3ed` |
| _d_ | `2` |
| _N_-_d_ | `1000000000000000000000000000000014def9dea2f79cd65812631a5cf5d3eb` |
| Length | 283 |
| Algorithm | `opt(dictionary(hybrid(4,0),continued_fractions(binary)))` |
| Best Known | 284 |
| Delta | -1 |


Addition chain produced by `addchain`:

```go
_10       = 2*1
_11       = 1 + _10
_100      = 1 + _11
_1000     = 2*_100
_1010     = _10 + _1000
_1011     = 1 + _1010
_10000    = 2*_1000
_10110    = 2*_1011
_100000   = _1010 + _10110
_100110   = _10000 + _10110
_1000000  = 2*_100000
_1010000  = _10000 + _1000000
_1010011  = _11 + _1010000
_1100011  = _10000 + _1010011
_1100111  = _100 + _1100011
_1101011  = _100 + _1100111
_10010011 = _1000000 + _1010011
_10010111 = _100 + _10010011
_10111101 = _100110 + _10010111
_11010011 = _10110 + _10111101
_11100111 = _1010000 + _10010111
_11101011 = _100 + _11100111
_11110101 = _1010 + _11101011
i161      = ((_1011 + _11110101) << 126 + _1010011) << 9 + _10
i180      = ((_11110101 + i161) << 7 + _1100111) << 9 + _11110101
i210      = ((i180 << 11 + _10111101) << 8 + _11100111) << 9
i233      = ((_1101011 + i210) << 6 + _1011) << 14 + _10010011
i264      = ((i233 << 10 + _1100011) << 9 + _10010111) << 10
return      ((_11110101 + i264) << 8 + _11010011) << 8 + _11101011
```

## NIST P-256 Scalar Inversion

| Property | Value |
| --- | ----- |
| _N_ | `ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551` |
| _d_ | `2` |
| _N_-_d_ | `ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc63254f` |
| Length | 294 |
| Algorithm | `opt(dictionary(hybrid(8,16),heuristic(use_first(halving,delta_largest))))` |
| Best Known | 292 |
| Delta | +2 |


Addition chain produced by `addchain`:

```go
_10       = 2*1
_100      = 2*_10
_101      = 1 + _100
_110      = 1 + _101
_1001     = _100 + _101
_1111     = _110 + _1001
_10010    = 2*_1001
_10101    = _110 + _1111
_11000    = _110 + _10010
_11010    = _10 + _11000
_101111   = _10101 + _11010
_111000   = _1001 + _101111
_111101   = _101 + _111000
_111111   = _10 + _111101
_1001111  = _10010 + _111101
_1100001  = _10010 + _1001111
_1100011  = _10 + _1100001
_1110011  = _10010 + _1100001
_1110111  = _100 + _1110011
_1111101  = _110 + _1110111
_10010101 = _11000 + _1111101
_10100111 = _10010 + _10010101
_10101101 = _110 + _10100111
_11100101 = _111000 + _10101101
_11111111 = _11010 + _11100101
x16       = _11111111 << 8 + _11111111
x32       = x16 << 16 + x16
i133      = ((x32 << 48 + x16) << 16 + x16) << 16
i158      = ((x16 + i133) << 16 + x16) << 6 + _101111
i186      = ((i158 << 9 + _1110011) << 8 + _1111101) << 9
i206      = ((_10101101 + i186) << 8 + _10100111) << 9 + _101111
i236      = ((i206 << 8 + _111101) << 11 + _1001111) << 9
i257      = ((_1110111 + i236) << 10 + _11100101) << 8 + _1100001
i286      = ((i257 << 7 + _111111) << 10 + _1100011) << 10
return      (_10010101 + i286) << 6 + _1111
```

## NIST P-384 Scalar Inversion

| Property | Value |
| --- | ----- |
| _N_ | `ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973` |
| _d_ | `2` |
| _N_-_d_ | `ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52971` |
| Length | 434 |
| Algorithm | `opt(dictionary(hybrid(4,0),continued_fractions(dichotomic)))` |
| Best Known | 433 |
| Delta | +1 |


Addition chain produced by `addchain`:

```go
_10      = 2*1
_11      = 1 + _10
_101     = _10 + _11
_111     = _10 + _101
_1001    = _10 + _111
_1011    = _10 + _1001
_1101    = _10 + _1011
_1111    = _10 + _1101
_11110   = 2*_1111
_11111   = 1 + _11110
_1111100 = _11111 << 2
i14      = _1111100 << 2
i26      = (i14 << 3 + _1111100) << 7 + i14
i42      = i26 << 15 + i26
x64      = i42 << 30 + i42 + _1111
x128     = x64 << 64 + x64
x192     = x128 << 64 + x64
x194     = x192 << 2 + _11
i225     = ((x194 << 6 + _111) << 3 + _11) << 7
i235     = 2*((_1101 + i225) << 6 + _1101) + 1
i258     = ((i235 << 11 + _11111) << 2 + 1) << 8
i269     = ((_1101 + i258) << 2 + _11) << 6 + _1011
i286     = ((i269 << 4 + _111) << 6 + _11111) << 5
i308     = ((_1011 + i286) << 10 + _1101) << 9 + _1101
i323     = ((i308 << 4 + _1011) << 6 + _1001) << 3
i340     = ((1 + i323) << 7 + _1011) << 7 + _101
i357     = ((i340 << 5 + _111) << 5 + _1111) << 5
i369     = ((_1011 + i357) << 4 + _1011) << 5 + _111
i387     = ((i369 << 3 + _11) << 7 + _11) << 6
i397     = ((_1011 + i387) << 4 + _101) << 3 + _11
i413     = ((i397 << 4 + _11) << 4 + _11) << 6
i427     = ((_101 + i413) << 5 + _101) << 6 + _1011
return     (2*i427 + 1) << 4 + 1
```

## secp256k1 (Bitcoin) Scalar Inversion

| Property | Value |
| --- | ----- |
| _N_ | `fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141` |
| _d_ | `2` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd036413f` |
| Length | 293 |
| Algorithm | `opt(dictionary(hybrid(4,0),continued_fractions(dichotomic)))` |
| Best Known | 290 |
| Delta | +3 |


Addition chain produced by `addchain`:

```go
_10       = 2*1
_11       = 1 + _10
_101      = _10 + _11
_111      = _10 + _101
_1001     = _10 + _111
_1011     = _10 + _1001
_1101     = _10 + _1011
_110100   = _1101 << 2
_111111   = _1011 + _110100
_1111110  = 2*_111111
_1111111  = 1 + _1111110
_11111110 = 2*_1111111
_11111111 = 1 + _11111110
i17       = _11111111 << 3
i19       = i17 << 2
i21       = i19 << 2
i39       = (i21 << 6 + i19) << 10 + i21
x63       = (i39 << 4 + i17) << 28 + i39 + _1111111
x64       = 2*x63 + 1
x127      = x64 << 63 + x63
i154      = ((x127 << 5 + _1011) << 3 + _101) << 4
i166      = ((_101 + i154) << 4 + _111) << 5 + _1101
i181      = ((i166 << 2 + _11) << 5 + _111) << 6
i193      = ((_1101 + i181) << 5 + _1011) << 4 + _1101
i214      = ((i193 << 3 + 1) << 6 + _101) << 10
i230      = ((_111 + i214) << 4 + _111) << 9 + _11111111
i247      = ((i230 << 5 + _1001) << 6 + _1011) << 4
i261      = ((_1101 + i247) << 5 + _11) << 6 + _1101
i283      = ((i261 << 10 + _1101) << 4 + _1001) << 6
return      (1 + i283) << 8 + _111111
```

## M-221 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^221-3` |
| _d_ | `2` |
| _N_-_d_ | `1ffffffffffffffffffffffffffffffffffffffffffffffffffffffb` |
| Length | 231 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_1100   = _11 << 2
_1111   = _11 + _1100
_111100 = _1111 << 2
_111111 = _11 + _111100
x10     = _111111 << 4 + _1111
x20     = x10 << 10 + x10
x26     = x20 << 6 + _111111
x52     = x26 << 26 + x26
x104    = x52 << 52 + x52
x208    = x104 << 104 + x104
x218    = x208 << 10 + x10
return    x218 << 3 + _11
```

## E-222 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^222-117` |
| _d_ | `2` |
| _N_-_d_ | `3fffffffffffffffffffffffffffffffffffffffffffffffffffff89` |
| Length | 233 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10      = 2*1
_11      = 1 + _10
_110     = 2*_11
_111     = 1 + _110
_111000  = _111 << 3
_111111  = _111 + _111000
_1111110 = 2*_111111
_1111111 = 1 + _1111110
x13      = _1111111 << 6 + _111111
x26      = x13 << 13 + x13
x52      = x26 << 26 + x26
x104     = x52 << 52 + x52
x208     = x104 << 104 + x104
x215     = x208 << 7 + _1111111
return     (x215 << 4 + 1) << 3 + 1
```

## Curve1174 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^251-9` |
| _d_ | `2` |
| _N_-_d_ | `7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5` |
| Length | 263 |
| Algorithm | `opt(dictionary(hybrid(3,64),continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10      = 2*1
_11      = 1 + _10
_110     = 2*_11
_111     = 1 + _110
_111000  = _111 << 3
_111111  = _111 + _111000
_1111110 = 2*_111111
_1111111 = 1 + _1111110
x14      = _1111111 << 7 + _1111111
x15      = 2*x14 + 1
x30      = x15 << 15 + x15
x60      = x30 << 30 + x30
x120     = x60 << 60 + x60
x240     = x120 << 120 + x120
x247     = x240 << 7 + _1111111
return     (x247 << 2 + 1) << 2 + 1
```

## E-382 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^382-105` |
| _d_ | `2` |
| _N_-_d_ | `3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff95` |
| Length | 395 |
| Algorithm | `opt(dictionary(hybrid(5,0),continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10       = 2*1
_11       = 1 + _10
_110      = 2*_11
_111      = 1 + _110
_1110     = 2*_111
_10101    = _111 + _1110
_101010   = 2*_10101
_111111   = _10101 + _101010
_1111110  = 2*_111111
_11111100 = 2*_1111110
i11       = 2*_11111100
i25       = (i11 << 5 + _11111100) << 7 + i11
i51       = (i25 << 5 + _11111100) << 19 + i25
i101      = (i51 << 5 + _11111100) << 43 + i51
i199      = (i101 << 4 + _1111110) << 92 + i101
x375      = i199 << 186 + i199 + _111
return      x375 << 7 + _10101
```

## M-383/Curve383187 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^383-187` |
| _d_ | `2` |
| _N_-_d_ | `7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff43` |
| Length | 396 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10       = 2*1
_11       = 1 + _10
_1100     = _11 << 2
_1111     = _11 + _1100
_11110000 = _1111 << 4
_11111111 = _1111 + _11110000
x16       = _11111111 << 8 + _11111111
x20       = x16 << 4 + _1111
x22       = x20 << 2 + _11
x44       = x22 << 22 + x22
x88       = x44 << 44 + x44
x176      = x88 << 88 + x88
x352      = x176 << 176 + x176
x374      = x352 << 22 + x22
x375      = 2*x374 + 1
return      (x375 << 2 + 1) << 6 + _11
```

## Curve41417 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^414-17` |
| _d_ | `2` |
| _N_-_d_ | `3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed` |
| Length | 426 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_1100   = _11 << 2
_1111   = _11 + _1100
_111100 = _1111 << 2
_111111 = _11 + _111100
x12     = _111111 << 6 + _111111
x24     = x12 << 12 + x12
x48     = x24 << 24 + x24
x96     = x48 << 48 + x48
x192    = x96 << 96 + x96
x384    = x192 << 192 + x192
x408    = x384 << 24 + x24
x409    = 2*x408 + 1
return    (x409 << 3 + _11) << 2 + 1
```

## M-511 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^511-187` |
| _d_ | `2` |
| _N_-_d_ | `7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff43` |
| Length | 525 |
| Algorithm | `opt(runs(continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10       = 2*1
_11       = 1 + _10
_1100     = _11 << 2
_1111     = _11 + _1100
_111100   = _1111 << 2
_111111   = _11 + _111100
_11111100 = _111111 << 2
_11111111 = _11 + _11111100
x16       = _11111111 << 8 + _11111111
x22       = x16 << 6 + _111111
x30       = x22 << 8 + _11111111
x60       = x30 << 30 + x30
x120      = x60 << 60 + x60
x240      = x120 << 120 + x120
x480      = x240 << 240 + x240
x502      = x480 << 22 + x22
x503      = 2*x502 + 1
return      (x503 << 2 + 1) << 6 + _11
```

## NIST P-192 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^192-2^64-1` |
| _d_ | `2` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffefffffffffffffffd` |
| Length | 203 |
| Algorithm | `opt(dictionary(hybrid(2,0),continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_110    = 2*_11
_111    = 1 + _110
_111000 = _111 << 3
_111111 = _111 + _111000
x12     = _111111 << 6 + _111111
x15     = x12 << 3 + _111
x30     = x15 << 15 + x15
x60     = x30 << 30 + x30
x62     = x60 << 2 + _11
x124    = x62 << 62 + x62
x127    = x124 << 3 + _111
return    (x127 << 63 + x62) << 2 + 1
```

## NIST P-224 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^224-2^96+1` |
| _d_ | `2` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffeffffffffffffffffffffffff` |
| Length | 234 |
| Algorithm | `opt(runs(heuristic(use_first(halving,approximation))))` |

Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_110    = 2*_11
_111    = 1 + _110
_111000 = _111 << 3
_111111 = _111 + _111000
x12     = _111111 << 6 + _111111
x14     = x12 << 2 + _11
x17     = x14 << 3 + _111
x31     = x17 << 14 + x14
x48     = x31 << 17 + x17
x96     = x48 << 48 + x48
x127    = x96 << 31 + x31
return    x127 << 97 + x96
```

## Goldilocks Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `2^448-2^224-1` |
| _d_ | `2` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffffffffffffffffffffffffffffffffffffffffffffffffffffd` |
| Length | 460 |
| Algorithm | `opt(runs(heuristic(use_first(halving,approximation))))` |

Addition chain produced by `addchain`:

```go
_10     = 2*1
_11     = 1 + _10
_110    = 2*_11
_111    = 1 + _110
_111000 = _111 << 3
_111111 = _111 + _111000
x12     = _111111 << 6 + _111111
x24     = x12 << 12 + x12
i34     = x24 << 6
x30     = _111111 + i34
x48     = i34 << 18 + x24
x96     = x48 << 48 + x48
x192    = x96 << 96 + x96
x222    = x192 << 30 + x30
x223    = 2*x222 + 1
return    (x223 << 223 + x222) << 2 + 1
```

## secp192k1 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `fffffffffffffffffffffffffffffffffffffffeffffee37` |
| _d_ | `2` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffffffffffeffffee35` |
| Length | 205 |
| Algorithm | `opt(dictionary(hybrid(3,0),continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_10      = 2*1
_11      = 1 + _10
_101     = _10 + _11
_111     = _10 + _101
_11100   = _111 << 2
_11111   = _11 + _11100
_1111100 = _11111 << 2
_1111111 = _11 + _1111100
i15      = _1111111 << 5
x19      = i15 << 7 + i15 + _11111
i31      = x19 << 7
i51      = i31 << 19 + i31
i90      = i51 << 38 + i51
x159     = i90 << 76 + i90 + _1111111
i199     = ((x159 << 20 + x19) << 4 + _111) << 5
return     (_11 + i199) << 4 + _101
```

## secp224k1 Field Inversion

| Property | Value |
| --- | ----- |
| _N_ | `fffffffffffffffffffffffffffffffffffffffffffffffeffffe56d` |
| _d_ | `2` |
| _N_-_d_ | `fffffffffffffffffffffffffffffffffffffffffffffffeffffe56b` |
| Length | 238 |
| Algorithm | `opt(dictionary(hybrid(5,0),continued_fractions(dichotomic)))` |

Addition chain produced by `addchain`:

```go
_100     = 1 << 2
_101     = 1 + _100
_10100   = _101 << 2
_10101   = 1 + _10100
_101010  = 2*_10101
_111111  = _10101 + _101010
_1111110 = 2*_111111
x12      = _1111110 << 5 + _111111
x19      = x12 << 7 + _1111110 + 1
i25      = 2*x19
i45      = i25 << 19 + i25
x57      = i45 << 18 + x19
i104     = x57 << 39 + i45
x191     = i104 << 95 + i104 + 1
i235     = ((x191 << 20 + x19) << 7 + _10101) << 5
return     2*(_10101 + i235) + 1
```


