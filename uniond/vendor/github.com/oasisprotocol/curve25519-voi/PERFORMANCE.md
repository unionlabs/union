### curve25519-voi performance

All measurements were taken on an Intel(R) Core(TM) i7-10510U @ 1.80GHz
with turbo disabled, using Go 1.17 as the toolchain.  As there are
multiple definitions of "Ed25519 signature verification", curve25519-voi
was configured for each benchmark to use as similar verification semantics
as possible where relevant.

| Implementation      | Version/Commit                           |
| ------------------- | ---------------------------------------- |
| crypto/ed25519      | 1.17                                     |
| x/crypto/curve25519 | v0.0.0-20210817164053-32db794688a5       |
| circl               | v1.0.1-0.20210810200100-62142fc919e5     |
| ed25519consensus    | v0.0.0-20210430192048-0962ce16b305       |
| curve25519-voi      | ff25a2a1612da873cc1335ae53ac7b38c5e8f2bb |

##### Native

```
name \ time/op            stdlib       circl          ed25519consensus  voi
X25519/ScalarBaseMult      107µs ± 0%      44µs ± 0%                      24µs ± 0%
X25519/ScalarMult          107µs ± 0%      65µs ± 0%                     106µs ± 0%
Ed25519/NewKeyFromSeed    42.8µs ± 0%    54.3µs ± 0%                    24.3µs ± 0%
Ed25519/Sign              52.7µs ± 0%    56.5µs ± 0%                    27.5µs ± 0%
Ed25519/Verify             125µs ± 0%     121µs ± 1%                      75µs ± 1%
Ed25519/Verify_ZIP215                                       129µs ± 0%    74µs ± 0%
Ed25519/VerifyBatch_1                                       160µs ± 1%   104µs ± 0%
Ed25519/VerifyBatch_8                                       571µs ± 0%   398µs ± 0%
Ed25519/VerifyBatch_16                                     1.05ms ± 2%  0.73ms ± 0%
Ed25519/VerifyBatch_32                                     1.98ms ± 0%  1.40ms ± 0%
Ed25519/VerifyBatch_64                                     3.87ms ± 0%  2.74ms ± 1%
Ed25519/VerifyBatch_128                                    7.64ms ± 0%  5.19ms ± 1%
Ed25519/VerifyBatch_256                                    15.3ms ± 0%   9.3ms ± 0%
Ed25519/VerifyBatch_512                                    30.7ms ± 0%  17.1ms ± 1%
Ed25519/VerifyBatch_1024                                   61.8ms ± 1%  31.7ms ± 0%

name \ alloc/op           stdlib       circl          ed25519consensus  voi
X25519/ScalarBaseMult      0.00B          0.00B                          0.00B
X25519/ScalarMult          0.00B          0.00B                          0.00B
Ed25519/NewKeyFromSeed     0.00B          0.00B                          0.00B
Ed25519/Sign               0.00B        416.00B ± 0%                    64.00B ± 0%
Ed25519/Verify             0.00B       4784.00B ± 0%                     0.00B
Ed25519/Verify_ZIP215                                       0.00B        0.00B
Ed25519/VerifyBatch_1                                      5.65kB ± 0%  5.82kB ± 0%
Ed25519/VerifyBatch_8                                      31.9kB ± 0%  36.4kB ± 0%
Ed25519/VerifyBatch_16                                     69.3kB ± 0%  70.3kB ± 0%
Ed25519/VerifyBatch_32                                      130kB ± 0%   146kB ± 0%
Ed25519/VerifyBatch_64                                      254kB ± 0%   283kB ± 0%
Ed25519/VerifyBatch_128                                     497kB ± 0%   334kB ± 0%
Ed25519/VerifyBatch_256                                     970kB ± 0%   521kB ± 0%
Ed25519/VerifyBatch_512                                    1.93MB ± 0%  0.90MB ± 0%
Ed25519/VerifyBatch_1024                                   3.82MB ± 0%  1.61MB ± 0%

name \ allocs/op          stdlib       circl          ed25519consensus  voi
X25519/ScalarBaseMult       0.00           0.00                           0.00
X25519/ScalarMult           0.00           0.00                           0.00
Ed25519/NewKeyFromSeed      0.00           0.00                           0.00
Ed25519/Sign                0.00           4.00 ± 0%                      1.00 ± 0%
Ed25519/Verify              0.00          15.00 ± 0%                      0.00
Ed25519/Verify_ZIP215                                        0.00         0.00
Ed25519/VerifyBatch_1                                        9.00 ± 0%   14.00 ± 0%
Ed25519/VerifyBatch_8                                        26.0 ± 0%    31.0 ± 0%
Ed25519/VerifyBatch_16                                       43.0 ± 0%    48.0 ± 0%
Ed25519/VerifyBatch_32                                       76.0 ± 0%    81.0 ± 0%
Ed25519/VerifyBatch_64                                        141 ± 0%     146 ± 0%
Ed25519/VerifyBatch_128                                       270 ± 0%     205 ± 0%
Ed25519/VerifyBatch_256                                       527 ± 0%     206 ± 0%
Ed25519/VerifyBatch_512                                     1.04k ± 0%   0.21k ± 0%
Ed25519/VerifyBatch_1024                                    2.06k ± 0%   0.21k ± 0%
```

Notes:
 * circl is alone in the use of BMI2 for `X25519/ScalarMult`.
 * As the system used for benchmarking has AVX2, a subtantial fraction
   of the curve25519-voi routines benchmarked are written in assembly
   language.

##### purego (Assembly optimizations disabled)

```
name \ time/op            stdlib       circl          ed25519consensus  voi
X25519/ScalarBaseMult      160µs ± 0%      68µs ± 2%                      47µs ± 0%
X25519/ScalarMult          160µs ± 0%     108µs ± 0%                     111µs ± 0%
Ed25519/NewKeyFromSeed    58.3µs ± 1%    60.8µs ± 0%                    47.8µs ± 1%
Ed25519/Sign              73.3µs ± 1%    63.4µs ± 0%                    50.9µs ± 0%
Ed25519/Verify             181µs ± 0%     133µs ± 0%                     121µs ± 0%
Ed25519/Verify_ZIP215                                       191µs ± 0%   109µs ± 1%
Ed25519/VerifyBatch_1                                       231µs ± 0%   163µs ± 1%
Ed25519/VerifyBatch_8                                       823µs ± 0%   591µs ± 0%
Ed25519/VerifyBatch_16                                     1.51ms ± 0%  1.08ms ± 1%
Ed25519/VerifyBatch_32                                     2.86ms ± 0%  2.05ms ± 0%
Ed25519/VerifyBatch_64                                     5.60ms ± 2%  4.00ms ± 0%
Ed25519/VerifyBatch_128                                    11.0ms ± 0%   7.9ms ± 0%
Ed25519/VerifyBatch_256                                    22.0ms ± 0%  14.0ms ± 0%
Ed25519/VerifyBatch_512                                    44.0ms ± 0%  25.5ms ± 0%
Ed25519/VerifyBatch_1024                                   88.1ms ± 0%  46.4ms ± 0%

name \ alloc/op           stdlib       circl          ed25519consensus  voi
X25519/ScalarBaseMult      0.00B          0.00B                          0.00B
X25519/ScalarMult          0.00B          0.00B                          0.00B
Ed25519/NewKeyFromSeed     0.00B          0.00B                          0.00B
Ed25519/Sign               0.00B        416.00B ± 0%                    64.00B ± 0%
Ed25519/Verify             0.00B       4784.00B ± 0%                     0.00B
Ed25519/Verify_ZIP215                                       0.00B        0.00B
Ed25519/VerifyBatch_1                                      5.65kB ± 0%  5.82kB ± 0%
Ed25519/VerifyBatch_8                                      31.9kB ± 0%  36.4kB ± 0%
Ed25519/VerifyBatch_16                                     69.3kB ± 0%  70.3kB ± 0%
Ed25519/VerifyBatch_32                                      130kB ± 0%   146kB ± 0%
Ed25519/VerifyBatch_64                                      254kB ± 0%   283kB ± 0%
Ed25519/VerifyBatch_128                                     497kB ± 0%   334kB ± 0%
Ed25519/VerifyBatch_256                                     970kB ± 0%   521kB ± 0%
Ed25519/VerifyBatch_512                                    1.93MB ± 0%  0.90MB ± 0%
Ed25519/VerifyBatch_1024                                   3.82MB ± 0%  1.61MB ± 0%

name \ allocs/op          stdlib       circl          ed25519consensus  voi
X25519/ScalarBaseMult       0.00           0.00                           0.00
X25519/ScalarMult           0.00           0.00                           0.00
Ed25519/NewKeyFromSeed      0.00           0.00                           0.00
Ed25519/Sign                0.00           4.00 ± 0%                      1.00 ± 0%
Ed25519/Verify              0.00          15.00 ± 0%                      0.00
Ed25519/Verify_ZIP215                                        0.00         0.00
Ed25519/VerifyBatch_1                                        9.00 ± 0%   14.00 ± 0%
Ed25519/VerifyBatch_8                                        26.0 ± 0%    31.0 ± 0%
Ed25519/VerifyBatch_16                                       43.0 ± 0%    48.0 ± 0%
Ed25519/VerifyBatch_32                                       76.0 ± 0%    81.0 ± 0%
Ed25519/VerifyBatch_64                                        141 ± 0%     146 ± 0%
Ed25519/VerifyBatch_128                                       270 ± 0%     205 ± 0%
Ed25519/VerifyBatch_256                                       527 ± 0%     206 ± 0%
Ed25519/VerifyBatch_512                                     1.04k ± 0%   0.21k ± 0%
Ed25519/VerifyBatch_1024                                    2.06k ± 0%   0.21k ± 0%
```

##### Precomputed verification

curve25519-voi supports expanding public keys, to accelerate repeated
Ed25519 signature verification with the same public key.  This will be a
net performance gain if a public key will be used to verify signatures
more than once, and costs approximately 1.47 KiB of memory per public key.

The batch verification code is written to attempt to leverage pre-computation
where possible, and will convert non-expanded public keys to expanded form
internally by default unless the batch grows over the size supported by
the precomputed multiscalar multiply.

```
name                      old time/op    new time/op    delta
Ed25519/NewExpandedPublicKey               11.3µs ± 0%
Ed25519/Verify              75.4µs ± 1%    66.6µs ± 1%  -11.68%  (p=0.008 n=5+5)
Ed25519/Verify_ZIP215       73.9µs ± 0%    64.2µs ± 0%  -13.07%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_1        104µs ± 0%      93µs ± 0%  -10.59%  (p=0.016 n=5+4)
Ed25519/VerifyBatch_8        398µs ± 0%     307µs ± 0%  -22.99%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_16       733µs ± 0%     552µs ± 0%  -24.74%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_32      1.40ms ± 0%    1.04ms ± 1%  -26.03%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_64      2.74ms ± 1%    1.99ms ± 0%  -27.28%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_128     5.19ms ± 1%    3.91ms ± 0%  -24.68%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_256     9.32ms ± 0%    7.07ms ± 0%  -24.07%  (p=0.016 n=4+5)
Ed25519/VerifyBatch_512     17.1ms ± 1%    13.1ms ± 1%  -23.54%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_1024    31.7ms ± 0%    24.0ms ± 1%  -24.29%  (p=0.008 n=5+5)

name                      old alloc/op   new alloc/op   delta
Ed25519/NewExpandedPublicKey               1.50kB ± 0%
Ed25519/Verify               0.00B          0.00B          ~     (all equal)
Ed25519/Verify_ZIP215        0.00B          0.00B          ~     (all equal)
Ed25519/VerifyBatch_1       5.82kB ± 0%    4.31kB ± 0%  -25.86%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_8       36.4kB ± 0%    24.4kB ± 0%  -33.01%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_16      70.3kB ± 0%    46.2kB ± 0%     ~     (p=0.079 n=4+5)
Ed25519/VerifyBatch_32       146kB ± 0%      98kB ± 0%  -33.00%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_64       283kB ± 0%     187kB ± 0%  -33.98%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_128      334kB ± 0%     193kB ± 0%  -42.32%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_256      521kB ± 0%     380kB ± 0%  -27.14%  (p=0.000 n=5+4)
Ed25519/VerifyBatch_512      899kB ± 0%     757kB ± 0%  -15.73%  (p=0.000 n=4+5)
Ed25519/VerifyBatch_1024    1.61MB ± 0%    1.47MB ± 0%   -8.76%  (p=0.000 n=4+5)

name                      old allocs/op  new allocs/op  delta
Ed25519/NewExpandedPublicKey                 2.00 ± 0%
Ed25519/Verify                0.00           0.00          ~     (all equal)
Ed25519/Verify_ZIP215         0.00           0.00          ~     (all equal)
Ed25519/VerifyBatch_1         14.0 ± 0%      12.0 ± 0%  -14.29%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_8         31.0 ± 0%      15.0 ± 0%  -51.61%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_16        48.0 ± 0%      16.0 ± 0%  -66.67%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_32        81.0 ± 0%      17.0 ± 0%  -79.01%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_64         146 ± 0%        18 ± 0%  -87.67%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_128        205 ± 0%        17 ± 0%  -91.71%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_256        206 ± 0%        18 ± 0%  -91.26%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_512        207 ± 0%        19 ± 0%  -90.82%  (p=0.008 n=5+5)
Ed25519/VerifyBatch_1024       208 ± 0%        20 ± 0%  -90.38%  (p=0.008 n=5+5)
```
