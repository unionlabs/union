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
 
### Conversion from/to `big.Int` and other formats

```
BenchmarkSetFromBig/1word-6                     253798280                4.84 ns/op            0 B/op          0 allocs/op
BenchmarkSetFromBig/2words-6                    242738034                5.00 ns/op            0 B/op          0 allocs/op
BenchmarkSetFromBig/3words-6                    233704105                5.22 ns/op            0 B/op          0 allocs/op
BenchmarkSetFromBig/4words-6                    192542544                5.70 ns/op            0 B/op          0 allocs/op
BenchmarkSetFromBig/overflow-6                  212680123                6.05 ns/op            0 B/op          0 allocs/op
BenchmarkToBig/1word-6                          14953528                81.6 ns/op            64 B/op          2 allocs/op
BenchmarkToBig/2words-6                         15932970                85.1 ns/op            64 B/op          2 allocs/op
BenchmarkToBig/3words-6                         15629001                77.0 ns/op            64 B/op          2 allocs/op
BenchmarkToBig/4words-6                         14525355                78.0 ns/op            64 B/op          2 allocs/op
BenchmarkSetBytes/generic-6                      5386718               230 ns/op               0 B/op          0 allocs/op
BenchmarkSetBytes/specific-6                     9418405               130 ns/op               0 B/op          0 allocs/op
BenchmarkRLPEncoding-6                             82531             13085 ns/op           11911 B/op        255 allocs/op

```
### Math operations

`uint256`:
```
Benchmark_Add/single/uint256-6                  575308741                2.19 ns/op            0 B/op          0 allocs/op
Benchmark_Sub/single/uint256-6                  551694393                2.71 ns/op            0 B/op          0 allocs/op
Benchmark_Sub/single/uint256_of-6               405466652                2.52 ns/op            0 B/op          0 allocs/op
BenchmarkMul/single/uint256-6                   147034321                8.19 ns/op            0 B/op          0 allocs/op
BenchmarkMulOverflow/single/uint256-6           45344761                25.4 ns/op             0 B/op          0 allocs/op
BenchmarkSquare/single/uint256-6                196272379                6.14 ns/op            0 B/op          0 allocs/op
Benchmark_Exp/large/uint256-6                     374550              3199 ns/op               0 B/op          0 allocs/op
Benchmark_Exp/small/uint256-6                    4426760               270 ns/op               0 B/op          0 allocs/op
BenchmarkDiv/small/uint256-6                    94629267                12.5 ns/op             0 B/op          0 allocs/op
BenchmarkDiv/mod64/uint256-6                    17367373                67.6 ns/op             0 B/op          0 allocs/op
BenchmarkDiv/mod128/uint256-6                   10192484               130 ns/op               0 B/op          0 allocs/op
BenchmarkDiv/mod192/uint256-6                   10936984               107 ns/op               0 B/op          0 allocs/op
BenchmarkDiv/mod256/uint256-6                   13436908                93.5 ns/op             0 B/op          0 allocs/op
BenchmarkMod/small/uint256-6                    80138805                15.2 ns/op             0 B/op          0 allocs/op
BenchmarkMod/mod64/uint256-6                    17065768                72.1 ns/op             0 B/op          0 allocs/op
BenchmarkMod/mod128/uint256-6                    9469146               123 ns/op               0 B/op          0 allocs/op
BenchmarkMod/mod192/uint256-6                   11193145               115 ns/op               0 B/op          0 allocs/op
BenchmarkMod/mod256/uint256-6                   12896706                93.1 ns/op             0 B/op          0 allocs/op
BenchmarkAddMod/small/uint256-6                 62187169                21.0 ns/op             0 B/op          0 allocs/op
BenchmarkAddMod/mod64/uint256-6                 15169026                82.5 ns/op             0 B/op          0 allocs/op
BenchmarkAddMod/mod128/uint256-6                 8460835               144 ns/op               0 B/op          0 allocs/op
BenchmarkAddMod/mod192/uint256-6                 9273334               141 ns/op               0 B/op          0 allocs/op
BenchmarkAddMod/mod256/uint256-6                10145329               113 ns/op               0 B/op          0 allocs/op
BenchmarkMulMod/small/uint256-6                 26673195                42.3 ns/op             0 B/op          0 allocs/op
BenchmarkMulMod/mod64/uint256-6                 10133446               125 ns/op               0 B/op          0 allocs/op
BenchmarkMulMod/mod128/uint256-6                 4955551               229 ns/op               0 B/op          0 allocs/op
BenchmarkMulMod/mod192/uint256-6                 5210977               220 ns/op               0 B/op          0 allocs/op
BenchmarkMulMod/mod256/uint256-6                 5527972               220 ns/op               0 B/op          0 allocs/op
Benchmark_SDiv/large/uint256-6                   9823093               124 ns/op               0 B/op          0 allocs/op
```
vs `big.Int`
```
Benchmark_Add/single/big-6                      45798462                25.0 ns/op             0 B/op          0 allocs/op
Benchmark_Sub/single/big-6                      51314886                23.7 ns/op             0 B/op          0 allocs/op
BenchmarkMul/single/big-6                       14101502                75.9 ns/op             0 B/op          0 allocs/op
BenchmarkMulOverflow/single/big-6               15774238                81.5 ns/op             0 B/op          0 allocs/op
BenchmarkSquare/single/big-6                    16739438                71.5 ns/op             0 B/op          0 allocs/op
Benchmark_Exp/large/big-6                          41250             42132 ns/op           18144 B/op        189 allocs/op
Benchmark_Exp/small/big-6                         130993             10813 ns/op            7392 B/op         77 allocs/op
BenchmarkDiv/small/big-6                        18169453                70.8 ns/op             8 B/op          1 allocs/op
BenchmarkDiv/mod64/big-6                         7500694               147 ns/op               8 B/op          1 allocs/op
BenchmarkDiv/mod128/big-6                        3075676               370 ns/op              80 B/op          1 allocs/op
BenchmarkDiv/mod192/big-6                        3908166               307 ns/op              80 B/op          1 allocs/op
BenchmarkDiv/mod256/big-6                        4416366               252 ns/op              80 B/op          1 allocs/op
BenchmarkMod/small/big-6                        19958649                70.8 ns/op             8 B/op          1 allocs/op
BenchmarkMod/mod64/big-6                         6718828               167 ns/op              64 B/op          1 allocs/op
BenchmarkMod/mod128/big-6                        3347608               349 ns/op              64 B/op          1 allocs/op
BenchmarkMod/mod192/big-6                        4072453               293 ns/op              48 B/op          1 allocs/op
BenchmarkMod/mod256/big-6                        4545860               254 ns/op               8 B/op          1 allocs/op
BenchmarkAddMod/small/big-6                     13976365                79.6 ns/op             8 B/op          1 allocs/op
BenchmarkAddMod/mod64/big-6                      5799034               208 ns/op              77 B/op          1 allocs/op
BenchmarkAddMod/mod128/big-6                     2998821               409 ns/op              64 B/op          1 allocs/op
BenchmarkAddMod/mod192/big-6                     3420640               351 ns/op              61 B/op          1 allocs/op
BenchmarkAddMod/mod256/big-6                     4124067               298 ns/op              40 B/op          1 allocs/op
BenchmarkMulMod/small/big-6                     14748193                85.8 ns/op             8 B/op          1 allocs/op
BenchmarkMulMod/mod64/big-6                      3524833               420 ns/op              96 B/op          1 allocs/op
BenchmarkMulMod/mod128/big-6                     1851936               637 ns/op              96 B/op          1 allocs/op
BenchmarkMulMod/mod192/big-6                     2028134               584 ns/op              80 B/op          1 allocs/op
BenchmarkMulMod/mod256/big-6                     2125716               576 ns/op              80 B/op          1 allocs/op
Benchmark_SDiv/large/big-6                       1658139               848 ns/op             312 B/op          6 allocs/op
```

### Boolean logic
`uint256`
```
Benchmark_And/single/uint256-6                  571318570                2.13 ns/op            0 B/op          0 allocs/op
Benchmark_Or/single/uint256-6                   500672864                2.09 ns/op            0 B/op          0 allocs/op
Benchmark_Xor/single/uint256-6                  575198724                2.24 ns/op            0 B/op          0 allocs/op
Benchmark_Cmp/single/uint256-6                  400446943                3.09 ns/op            0 B/op          0 allocs/op
BenchmarkLt/large/uint256-6                     322143085                3.50 ns/op            0 B/op          0 allocs/op
BenchmarkLt/small/uint256-6                     351231680                3.33 ns/op            0 B/op          0 allocs/op
```
vs `big.Int`
```
Benchmark_And/single/big-6                      78524395                16.2 ns/op             0 B/op          0 allocs/op
Benchmark_Or/single/big-6                       65390958                20.5 ns/op             0 B/op          0 allocs/op
Benchmark_Xor/single/big-6                      58333172                20.6 ns/op             0 B/op          0 allocs/op
Benchmark_Cmp/single/big-6                      144781878                8.37 ns/op            0 B/op          0 allocs/op
BenchmarkLt/large/big-6                         95643212                13.8 ns/op             0 B/op          0 allocs/op
BenchmarkLt/small/big-6                         84561792                14.6 ns/op             0 B/op          0 allocs/op
```

### Bitwise shifts

`uint256`:
```
Benchmark_Lsh/n_eq_0/uint256-6                  291558974                3.96 ns/op            0 B/op          0 allocs/op
Benchmark_Lsh/n_gt_192/uint256-6                208429646                5.80 ns/op            0 B/op          0 allocs/op
Benchmark_Lsh/n_gt_128/uint256-6                151857447                6.90 ns/op            0 B/op          0 allocs/op
Benchmark_Lsh/n_gt_64/uint256-6                 124543732                9.55 ns/op            0 B/op          0 allocs/op
Benchmark_Lsh/n_gt_0/uint256-6                  100000000               11.2 ns/op             0 B/op          0 allocs/op
Benchmark_Rsh/n_eq_0/uint256-6                  296913555                4.08 ns/op            0 B/op          0 allocs/op
Benchmark_Rsh/n_gt_192/uint256-6                212698939                5.52 ns/op            0 B/op          0 allocs/op
Benchmark_Rsh/n_gt_128/uint256-6                157391629                7.59 ns/op            0 B/op          0 allocs/op
Benchmark_Rsh/n_gt_64/uint256-6                 124916373                9.46 ns/op            0 B/op          0 allocs/op
Benchmark_Rsh/n_gt_0/uint256-6                  100000000               11.5 ns/op 
```
vs `big.Int`:
```
Benchmark_Lsh/n_eq_0/big-6                      21387698                78.6 ns/op            64 B/op          1 allocs/op
Benchmark_Lsh/n_gt_192/big-6                    15645853                73.9 ns/op            96 B/op          1 allocs/op
Benchmark_Lsh/n_gt_128/big-6                    15954750                75.0 ns/op            96 B/op          1 allocs/op
Benchmark_Lsh/n_gt_64/big-6                     16771413                81.3 ns/op            80 B/op          1 allocs/op
Benchmark_Lsh/n_gt_0/big-6                      17118044                70.7 ns/op            80 B/op          1 allocs/op
Benchmark_Rsh/n_eq_0/big-6                      21585044                65.5 ns/op            64 B/op          1 allocs/op
Benchmark_Rsh/n_gt_192/big-6                    28313300                42.3 ns/op             8 B/op          1 allocs/op
Benchmark_Rsh/n_gt_128/big-6                    21191526                58.1 ns/op            48 B/op          1 allocs/op
Benchmark_Rsh/n_gt_64/big-6                     15906076                69.0 ns/op            64 B/op          1 allocs/op
Benchmark_Rsh/n_gt_0/big-6                      19234408                93.0 ns/op            64 B/op          1 allocs/op
```
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
