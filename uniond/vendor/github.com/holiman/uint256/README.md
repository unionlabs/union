# Fixed size 256-bit math library

This is a library specialized at replacing the big.Int library for math based on 256-bit types, used by both 
[go-ethereum](https://github.com/ethereum/go-ethereum) and [turbo-geth](https://github.com/ledgerwatch/turbo-geth).

[![Go Reference](https://pkg.go.dev/badge/github.com/holiman/uint256.svg)](https://pkg.go.dev/github.com/holiman/uint256)
[![codecov](https://codecov.io/gh/holiman/uint256/branch/master/graph/badge.svg?token=LHs7xL99wQ)](https://codecov.io/gh/holiman/uint256)
[![DeepSource](https://deepsource.io/gh/holiman/uint256.svg/?label=active+issues&token=CNJRIm7wXZdOM9xKKH4hXUKd)](https://deepsource.io/gh/holiman/uint256/?ref=repository-badge)

## Benchmarks

Current benchmarks, with tests ending with `big` being the standard `big.Int` library, and `uint256` being this library. 

### Current status

- As of 2020-03-18, `uint256` wins over `math/big` in every single case, often with orders of magnitude.
- And as of release `0.1.0`, the `uint256` library is alloc-free. 
- With the `1.0.0` release, it also has `100%` test coverage.

### Bitwise

| benchmark |  `u256` time/op | `big` time/op | time diff |  `u256` B/op|  `big` B/op | B diff| `u256` allocs/op | `big` allocs/op | allocs diff |
|--|--|--|--|--|--|--|--|--|--|
| And/single | 2.03ns | 8.46ns | `-76.04%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Or/single | 2.03ns | 10.98ns | `-81.51%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Xor/single | 2.03ns | 12.19ns | `-83.34%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Rsh/n_eq_0 | 5.00ns | 48.11ns | `-89.61%` | 0 | 64 | `-100.00%` | 0 | 1 | `-100.00%`|
| Rsh/n_gt_0 | 10.42ns | 48.41ns | `-78.48%` | 0 | 64 | `-100.00%` | 0 | 1 | `-100.00%`|
| Rsh/n_gt_64 | 6.94ns | 52.39ns | `-86.76%` | 0 | 64 | `-100.00%` | 0 | 1 | `-100.00%`|
| Rsh/n_gt_128 | 5.49ns | 44.21ns | `-87.59%` | 0 | 48 | `-100.00%` | 0 | 1 | `-100.00%`|
| Rsh/n_gt_192 | 3.43ns | 28.71ns | `-88.04%` | 0 | 8 | `-100.00%` | 0 | 1 | `-100.00%`|
| Lsh/n_eq_0 | 4.89ns | 40.49ns | `-87.92%` | 0 | 64 | `-100.00%` | 0 | 1 | `-100.00%`|
| Lsh/n_gt_0 | 10.14ns | 53.25ns | `-80.96%` | 0 | 80 | `-100.00%` | 0 | 1 | `-100.00%`|
| Lsh/n_gt_64 | 7.50ns | 53.92ns | `-86.08%` | 0 | 80 | `-100.00%` | 0 | 1 | `-100.00%`|
| Lsh/n_gt_128 | 5.39ns | 56.86ns | `-90.52%` | 0 | 96 | `-100.00%` | 0 | 1 | `-100.00%`|
| Lsh/n_gt_192 | 3.90ns | 57.61ns | `-93.23%` | 0 | 96 | `-100.00%` | 0 | 1 | `-100.00%`|

### Conversions

| benchmark |  `u256` time/op | `big` time/op | time diff |  `u256` B/op|  `big` B/op | B diff| `u256` allocs/op | `big` allocs/op | allocs diff |
|--|--|--|--|--|--|--|--|--|--|
| FromHexString | 116.70ns | 861.30ns | `-86.45%` | 32 | 88 | `-63.64%` | 1 | 3 | `-66.67%`|
| FromDecimalString | 7973.00ns | 32850.00ns | `-75.73%` | 0 | 2464 | `-100.00%` | 0 | 77 | `-100.00%`|
| Float64/Float64 | 2366.00ns | 28483.00ns | `-91.69%` | 0 | 23424 | `-100.00%` | 0 | 510 | `-100.00%`|
| EncodeHex/large | 95.26ns | 184.30ns | `-48.31%` | 80 | 140 | `-42.86%` | 1 | 2 | `-50.00%`|
| Decimal/ToDecimal | 67384.00ns | 83431.00ns | `-19.23%` | 11344 | 31920 | `-64.46%` | 248 | 594 | `-58.25%`|
| Decimal/ToPrettyDecimal | 84208.00ns | 123953.00ns | `-32.06%` | 14720 | 61376 | `-76.02%` | 251 | 1100 | `-77.18%`|

### Math

| benchmark |  `u256` time/op | `big` time/op | time diff |  `u256` B/op|  `big` B/op | B diff| `u256` allocs/op | `big` allocs/op | allocs diff |
|--|--|--|--|--|--|--|--|--|--|
| Add/single | 1.99ns | 18.11ns | `-89.02%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Sub/single | 2.00ns | 17.19ns | `-88.35%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Mul/single | 12.10ns | 57.69ns | `-79.03%` | 0 | 0 | `~` | 0 | 0 | `~`|
| SDiv/large | 94.64ns | 642.10ns | `-85.26%` | 0 | 312 | `-100.00%` | 0 | 6 | `-100.00%`|
| Sqrt/single | 594.90ns | 1844.00ns | `-67.74%` | 0 | 528 | `-100.00%` | 0 | 7 | `-100.00%`|
| Square/single | 12.49ns | 56.10ns | `-77.74%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Cmp/single | 4.78ns | 12.79ns | `-62.61%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Div/small | 12.91ns | 48.31ns | `-73.28%` | 0 | 8 | `-100.00%` | 0 | 1 | `-100.00%`|
| Div/mod64 | 65.77ns | 111.20ns | `-40.85%` | 0 | 8 | `-100.00%` | 0 | 1 | `-100.00%`|
| Div/mod128 | 93.67ns | 301.40ns | `-68.92%` | 0 | 80 | `-100.00%` | 0 | 1 | `-100.00%`|
| Div/mod192 | 86.52ns | 263.80ns | `-67.20%` | 0 | 80 | `-100.00%` | 0 | 1 | `-100.00%`|
| Div/mod256 | 77.17ns | 252.80ns | `-69.47%` | 0 | 80 | `-100.00%` | 0 | 1 | `-100.00%`|
| AddMod/small | 13.84ns | 48.16ns | `-71.26%` | 0 | 4 | `-100.00%` | 0 | 0 | `~`|
| AddMod/mod64 | 22.83ns | 57.58ns | `-60.35%` | 0 | 11 | `-100.00%` | 0 | 0 | `~`|
| AddMod/mod128 | 48.31ns | 145.00ns | `-66.68%` | 0 | 12 | `-100.00%` | 0 | 0 | `~`|
| AddMod/mod192 | 47.26ns | 160.00ns | `-70.46%` | 0 | 12 | `-100.00%` | 0 | 0 | `~`|
| AddMod/mod256 | 14.44ns | 143.20ns | `-89.92%` | 0 | 12 | `-100.00%` | 0 | 0 | `~`|
| MulMod/small | 40.81ns | 63.14ns | `-35.37%` | 0 | 8 | `-100.00%` | 0 | 1 | `-100.00%`|
| MulMod/mod64 | 91.66ns | 112.60ns | `-18.60%` | 0 | 48 | `-100.00%` | 0 | 1 | `-100.00%`|
| MulMod/mod128 | 136.00ns | 362.70ns | `-62.50%` | 0 | 128 | `-100.00%` | 0 | 2 | `-100.00%`|
| MulMod/mod192 | 151.70ns | 421.20ns | `-63.98%` | 0 | 144 | `-100.00%` | 0 | 2 | `-100.00%`|
| MulMod/mod256 | 188.80ns | 489.20ns | `-61.41%` | 0 | 176 | `-100.00%` | 0 | 2 | `-100.00%`|
| Exp/small | 433.70ns | 7490.00ns | `-94.21%` | 0 | 7392 | `-100.00%` | 0 | 77 | `-100.00%`|
| Exp/large | 5145.00ns | 23043.00ns | `-77.67%` | 0 | 18144 | `-100.00%` | 0 | 189 | `-100.00%`|
| Mod/mod64 | 70.94ns | 118.90ns | `-40.34%` | 0 | 64 | `-100.00%` | 0 | 1 | `-100.00%`|
| Mod/small | 14.82ns | 46.27ns | `-67.97%` | 0 | 8 | `-100.00%` | 0 | 1 | `-100.00%`|
| Mod/mod128 | 99.58ns | 286.90ns | `-65.29%` | 0 | 64 | `-100.00%` | 0 | 1 | `-100.00%`|
| Mod/mod192 | 95.31ns | 269.30ns | `-64.61%` | 0 | 48 | `-100.00%` | 0 | 1 | `-100.00%`|
| Mod/mod256 | 84.13ns | 222.90ns | `-62.26%` | 0 | 8 | `-100.00%` | 0 | 1 | `-100.00%`|
| MulOverflow/single | 27.68ns | 57.52ns | `-51.88%` | 0 | 0 | `~` | 0 | 0 | `~`|
| MulDivOverflow/small | 2.83ns | 22.97ns | `-87.69%` | 0 | 0 | `~` | 0 | 0 | `~`|
| MulDivOverflow/div64 | 2.85ns | 23.13ns | `-87.66%` | 0 | 0 | `~` | 0 | 0 | `~`|
| MulDivOverflow/div128 | 3.14ns | 31.29ns | `-89.96%` | 0 | 2 | `-100.00%` | 0 | 0 | `~`|
| MulDivOverflow/div192 | 3.17ns | 30.28ns | `-89.54%` | 0 | 2 | `-100.00%` | 0 | 0 | `~`|
| MulDivOverflow/div256 | 3.54ns | 35.56ns | `-90.06%` | 0 | 5 | `-100.00%` | 0 | 0 | `~`|
| Lt/small | 3.12ns | 10.43ns | `-70.06%` | 0 | 0 | `~` | 0 | 0 | `~`|
| Lt/large | 3.49ns | 10.32ns | `-66.18%` | 0 | 0 | `~` | 0 | 0 | `~`|

### Other

| benchmark |  `u256` time/op | `u256` B/op| `u256` allocs/op |
|--|--|--|--|
| SetBytes/generic | 104.30ns | 0 | 0 ||
| Sub/single/uint256_of | 1.99ns | 0 | 0 ||
| SetFromBig/overflow | 3.57ns | 0 | 0 ||
| ToBig/2words | 73.84ns | 64 | 2 ||
| SetBytes/specific | 45.81ns | 0 | 0 ||
| ScanScientific | 674.10ns | 0 | 0 ||
| SetFromBig/4words | 3.82ns | 0 | 0 ||
| ToBig/4words | 68.82ns | 64 | 2 ||
| RLPEncoding | 11917.00ns | 11911 | 255 ||
| SetFromBig/1word | 2.99ns | 0 | 0 ||
| SetFromBig/3words | 4.12ns | 0 | 0 ||
| ToBig/3words | 70.12ns | 64 | 2 ||
| ToBig/1word | 75.38ns | 64 | 2 ||
| MulMod/mod256/uint256r | 77.11ns | 0 | 0 ||
| HashTreeRoot | 11.27ns | 0 | 0 ||
| SetFromBig/2words | 3.90ns | 0 | 0 ||

## Helping out

If you're interested in low-level algorithms and/or doing optimizations for shaving off nanoseconds, then this is certainly for you!

### Implementation work

Choose an operation, and optimize the s**t out of it!

A few rules, though, to help your PR get approved:

- Do not optimize for 'best-case'/'most common case' at the expense of worst-case. 
- We'll hold off on go assembly for a while, until the algos and interfaces are finished in a 'good enough' first version. After that, it's assembly time. 

### Doing benchmarks

To do a simple benchmark for everything, do

```
go test -run - -bench . -benchmem

```

To see the difference between a branch and master, for a particular benchmark, do

```
git checkout master
go test -run - -bench Benchmark_Lsh -benchmem -count=10 > old.txt

git checkout opt_branch
go test -run - -bench Benchmark_Lsh -benchmem -count=10 > new.txt

benchstat old.txt new.txt

```
