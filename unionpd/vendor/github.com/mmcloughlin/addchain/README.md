<p align="center">
  <img src="logo.svg" width="40%" border="0" alt="addchain" />
  <br />
  <img src="https://img.shields.io/github/workflow/status/mmcloughlin/addchain/ci/master.svg?style=flat-square" alt="Build Status" />
  <a href="https://pkg.go.dev/github.com/mmcloughlin/addchain"><img src="https://img.shields.io/badge/doc-reference-007d9b?logo=go&style=flat-square" alt="go.dev" /></a>
  <a href="https://goreportcard.com/report/github.com/mmcloughlin/addchain"><img src="https://goreportcard.com/badge/github.com/mmcloughlin/addchain?style=flat-square" alt="Go Report Card" /></a>
  <a href="https://doi.org/10.5281/zenodo.5622943"><img src="https://img.shields.io/badge/DOI-10.5281%2Fzenodo.5622943-007ec6?style=flat-square" alt="DOI: 10.5281/zenodo.5622943" /></a>
</p>

<p align="center">Cryptographic Addition Chain Generation in Go</p>

`addchain` generates short addition chains for exponents of cryptographic
interest with [results](#results) rivaling the best hand-optimized chains.
Intended as a building block in elliptic curve or other cryptographic code
generators.

* Suite of algorithms from academic research: continued fractions,
  dictionary-based and Bos-Coster heuristics
* Custom run-length techniques exploit structure of cryptographic exponents
  with excellent results on Solinas primes
* Generic optimization methods eliminate redundant operations
* Simple domain-specific language for addition chain computations
* Command-line interface or library
* Code generation and templated output support

## Table of Contents

* [Background](#background)
* [Results](#results)
* [Usage](#usage)
  * [Command-line Interface](#command-line-interface)
  * [Library](#library)
* [Algorithms](#algorithms)
  * [Binary](#binary)
  * [Continued Fractions](#continued-fractions)
  * [Bos-Coster Heuristics](#bos-coster-heuristics)
  * [Dictionary](#dictionary)
  * [Runs](#runs)
  * [Optimization](#optimization)
* [Citing](#citing)
* [Thanks](#thanks)
* [Contributing](#contributing)
* [License](#license)


## Background

An [_addition chain_](https://en.wikipedia.org/wiki/Addition_chain) for a
target integer _n_ is a sequence of numbers starting at 1 and ending at _n_
such that every term is a sum of two numbers appearing earlier in the
sequence. For example, an addition chain for 29 is

```
1, 2, 4, 8, 9, 17, 25, 29
```

Addition chains arise in the optimization of exponentiation algorithms with
fixed exponents. For example, the addition chain above corresponds to the
following sequence of multiplications to compute <code>x<sup>29</sup></code>

<pre>
 x<sup>2</sup> = x<sup>1</sup> * x<sup>1</sup>
 x<sup>4</sup> = x<sup>2</sup> * x<sup>2</sup>
 x<sup>8</sup> = x<sup>4</sup> * x<sup>4</sup>
 x<sup>9</sup> = x<sup>1</sup> * x<sup>8</sup>
x<sup>17</sup> = x<sup>8</sup> * x<sup>9</sup>
x<sup>25</sup> = x<sup>8</sup> * x<sup>17</sup>
x<sup>29</sup> = x<sup>4</sup> * x<sup>25</sup>
</pre>

An exponentiation algorithm for a fixed exponent _n_ reduces to finding a
_minimal length addition chain_ for _n_. This is especially relevent in
cryptography where exponentiation by huge fixed exponents forms a
performance-critical component of finite-field arithmetic. In particular,
constant-time inversion modulo a prime _p_ is performed by computing
<code>x<sup>p-2</sup> (mod p)</code>, thanks to [Fermat's Little
Theorem](https://en.wikipedia.org/wiki/Fermat%27_little_theorem). Square root
also reduces to exponentiation for some prime moduli. Finding short addition
chains for these exponents is one important part of high-performance finite
field implementations required for elliptic curve cryptography or RSA.

Minimal addition chain search is famously hard. No practical optimal
algorithm is known, especially for cryptographic exponents of size 256-bits
and up. Given its importance for the performance of cryptographic
implementations, implementers devote significant effort to hand-tune addition
chains. The goal of the `addchain` project is to match or exceed the best
hand-optimized addition chains using entirely automated approaches, building
on extensive academic research and applying new tweaks that exploit the
unique nature of cryptographic exponents.

## Results

The following table shows the results of the `addchain` library on popular
cryptographic exponents. For each one we also show the length of the [best
known hand-optimized addition chain](https://briansmith.org/ecc-inversion-addition-chains-01), and the
delta from the library result.

| Name | This Library | Best Known | Delta |
| ---- | -----------: | ---------: | ----: |
| [Curve25519 Field Inversion](doc/results.md#curve25519-field-inversion) | 266 | 265 | +1 |
| [NIST P-256 Field Inversion](doc/results.md#nist-p-256-field-inversion) | 266 | 266 | **+0** |
| [NIST P-384 Field Inversion](doc/results.md#nist-p-384-field-inversion) | 397 | 396 | +1 |
| [secp256k1 (Bitcoin) Field Inversion](doc/results.md#secp256k1-bitcoin-field-inversion) | 269 | 269 | **+0** |
| [Curve25519 Scalar Inversion](doc/results.md#curve25519-scalar-inversion) | 283 | 284 | **-1** |
| [NIST P-256 Scalar Inversion](doc/results.md#nist-p-256-scalar-inversion) | 294 | 292 | +2 |
| [NIST P-384 Scalar Inversion](doc/results.md#nist-p-384-scalar-inversion) | 434 | 433 | +1 |
| [secp256k1 (Bitcoin) Scalar Inversion](doc/results.md#secp256k1-bitcoin-scalar-inversion) | 293 | 290 | +3 |


See [full results listing](doc/results.md) for more detail and
results for less common exponents.

These results demonstrate that `addchain` is competitive with hand-optimized
chains, often with equivalent or better performance. Even when `addchain` is
slightly sub-optimal, it can still be considered valuable since it fully
automates a laborious manual process. As such, `addchain` can be trusted to
produce high quality results in an automated code generation tool.

## Usage

### Command-line Interface

Install a pre-compiled [release
binary](https://github.com/mmcloughlin/addchain/releases):

```
curl -sSfL https://git.io/addchain | sh -s -- -b /usr/local/bin
```

Alternatively build from source:

```
go install github.com/mmcloughlin/addchain/cmd/addchain@latest
```

Search for a curve25519 field inversion addition chain with:

```sh
addchain search '2^255 - 19 - 2'
```

Output:

```
addchain: expr: "2^255 - 19 - 2"
addchain: hex: 7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeb
addchain: dec: 57896044618658097711785492504343953926634992332820282019728792003956564819947
addchain: best: opt(runs(continued_fractions(dichotomic)))
addchain: cost: 266
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

Next, you can [generate code from this addition chain](doc/gen.md).

### Library

Install:

```
go get -u github.com/mmcloughlin/addchain
```

Algorithms all conform to the [`alg.ChainAlgorithm`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg#ChainAlgorithm) or
[`alg.SequenceAlgorithm`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg#SequenceAlgorithm) interfaces and can be used directly. However the
most user-friendly method uses the [`alg/ensemble`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/ensemble) package to
instantiate a sensible default set of algorithms and the [`alg/exec`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/exec)
helper to execute them in parallel. The following code uses this method to
find an addition chain for curve25519 field inversion:

```go
func Example() {
	// Target number: 2²⁵⁵ - 21.
	n := new(big.Int)
	n.SetString("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeb", 16)

	// Default ensemble of algorithms.
	algorithms := ensemble.Ensemble()

	// Use parallel executor.
	ex := exec.NewParallel()
	results := ex.Execute(n, algorithms)

	// Output best result.
	best := 0
	for i, r := range results {
		if r.Err != nil {
			log.Fatal(r.Err)
		}
		if len(results[i].Program) < len(results[best].Program) {
			best = i
		}
	}
	r := results[best]
	fmt.Printf("best: %d\n", len(r.Program))
	fmt.Printf("algorithm: %s\n", r.Algorithm)

	// Output:
	// best: 266
	// algorithm: opt(runs(continued_fractions(dichotomic)))
}
```

## Algorithms

This section summarizes the algorithms implemented by `addchain` along with
references to primary literature. See the [bibliography](doc/bibliography.md)
for the complete references list.

### Binary

The [`alg/binary`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/binary) package implements the addition chain equivalent
of the basic [square-and-multiply exponentiation
method](https://en.wikipedia.org/wiki/Exponentiation_by_squaring). It is
included for completeness, but is almost always outperformed by more advanced
algorithms below.

### Continued Fractions

The [`alg/contfrac`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/contfrac) package implements the continued fractions
methods for addition sequence search introduced by
Bergeron-Berstel-Brlek-Duboc in 1989 and later extended. This approach
utilizes a decomposition of an addition chain akin to continued fractions,
namely

```
(1,..., k,..., n) = (1,...,n mod k,..., k) ⊗ (1,..., n/k) ⊕ (n mod k).
```

for certain special operators ⊗ and ⊕. This
decomposition lends itself to a recursive algorithm for efficient addition
sequence search, with results dependent on the _strategy_ for choosing the
auxillary integer _k_. The [`alg/contfrac`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/contfrac) package provides a
laundry list of strategies from the literature: binary, co-binary,
dichotomic, dyadic, fermat, square-root and total.

#### References

* F Bergeron, J Berstel, S Brlek and C Duboc. Addition chains using continued fractions. Journal of Algorithms. 1989. http://www-igm.univ-mlv.fr/~berstel/Articles/1989AdditionChainDuboc.pdf
* Bergeron, F., Berstel, J. and Brlek, S. Efficient computation of addition chains. Journal de theorie des nombres de Bordeaux. 1994. http://www.numdam.org/item/JTNB_1994__6_1_21_0
* Amadou Tall and Ali Yassin Sanghare. Efficient computation of addition-subtraction chains using generalized continued Fractions. Cryptology ePrint Archive, Report 2013/466. 2013. https://eprint.iacr.org/2013/466
* Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve Cryptography, chapter 9. 2006. http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf

### Bos-Coster Heuristics

Bos and Coster described an iterative algorithm for efficient addition
sequence generation in which at each step a heuristic proposes new numbers
for the sequence in such a way that the _maximum_ number always decreases.
The [original Bos-Coster paper](https://link.springer.com/content/pdf/10.1007/0-387-34805-0_37.pdf) defined four
heuristics: Approximation, Divison, Halving and Lucas. Package
[`alg/heuristic`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/heuristic) implements a variation on these heuristics:

* **Approximation:** looks for two elements a, b in the current sequence with sum close to the largest element.
* **Halving:** applies when the target is at least twice as big as the next largest, and if so it will propose adding a sequence of doublings.
* **Delta Largest:** proposes adding the delta between the largest two entries in the current sequence.

Divison and Lucas are not implemented due to disparities in the literature
about their precise definition and poor results from early experiments.
Furthermore, this library does not apply weights to the heuristics as
suggested in the paper, rather it simply uses the first that applies. However
both of these remain [possible avenues for
improvement](https://github.com/mmcloughlin/addchain/issues/26).

#### References

* Bos, Jurjen and Coster, Matthijs. Addition Chain Heuristics. In Advances in Cryptology --- CRYPTO' 89 Proceedings, pages 400--407. 1990. https://link.springer.com/content/pdf/10.1007/0-387-34805-0_37.pdf
* Riad S. Wahby. kwantam/addchain. Github Repository. Apache License, Version 2.0. 2018. https://github.com/kwantam/addchain
* Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve Cryptography, chapter 9. 2006. http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf
* Ayan Nandy. Modifications of Bos and Coster’s Heuristics in search of a shorter addition chain for faster exponentiation. Masters thesis, Indian Statistical Institute Kolkata. 2011. http://library.isical.ac.in:8080/jspui/bitstream/10263/6441/1/DISS-285.pdf
* F. L. Ţiplea, S. Iftene, C. Hriţcu, I. Goriac, R. Gordân and E. Erbiceanu. MpNT: A Multi-Precision Number Theory Package, Number Theoretical Algorithms (I). Technical Report TR03-02, Faculty of Computer Science, "Alexandru Ioan Cuza" University, Iasi. 2003. https://profs.info.uaic.ro/~tr/tr03-02.pdf
* Stam, Martijn. Speeding up subgroup cryptosystems. PhD thesis, Technische Universiteit Eindhoven. 2003. https://cr.yp.to/bib/2003/stam-thesis.pdf

### Dictionary

Dictionary methods decompose the binary representation of a target integer _n_ into a set of dictionary _terms_, such that _n_
may be written as a sum

<pre>
n = ∑ 2<sup>e<sub>i</sub></sup> d<sub>i</sub>
</pre>

for exponents _e_ and elements _d_ from a dictionary _D_. Given such a decomposition we can construct an addition chain for _n_ by

1. Find a short addition _sequence_ containing every element of the dictionary _D_. Continued fractions and Bos-Coster heuristics can be used here.
2. Build _n_ from the dictionary terms according to the sum decomposition.

The efficiency of this approach boils down to the decomposition method. The [`alg/dict`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/dict) package provides:

* **Fixed Window:** binary representation of _n_ is broken into fixed _k_-bit windows
* **Sliding Window**: break _n_ into _k_-bit windows, skipping zeros where possible
* **Run Length**: decompose _n_ into runs of 1s up to a maximal length
* **Hybrid**: mix of sliding window and run length methods

#### References

* Martin Otto. Brauer addition-subtraction chains. PhD thesis, Universitat Paderborn. 2001. http://www.martin-otto.de/publications/docs/2001_MartinOtto_Diplom_BrauerAddition-SubtractionChains.pdf
* Kunihiro, Noboru and Yamamoto, Hirosuke. New Methods for Generating Short Addition Chains. IEICE Transactions on Fundamentals of Electronics Communications and Computer Sciences. 2000. https://pdfs.semanticscholar.org/b398/d10faca35af9ce5a6026458b251fd0a5640c.pdf
* Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve Cryptography, chapter 9. 2006. http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf

### Runs

The runs algorithm is a custom variant of the dictionary approach that
decomposes a target into runs of ones. It leverages the observation that
building a dictionary consisting of runs of 1s of lengths
<code>l<sub>1</sub>, l<sub>2</sub>, ..., l<sub>k</sub></code> can itself be
reduced to:

1. Find an addition sequence containing the run lengths
   <code>l<sub>i</sub></code>. As with dictionary approaches we can use
   Bos-Coster heuristics and continued fractions here. However here we have the
   advantage that the <code>l<sub>i</sub></code> are typically very _small_,
   meaning that a wider range of algorithms can be brought to bear.
2. Use the addition sequence for the run lengths <code>l<sub>i</sub></code>
   to build an addition sequence for the runs themselves
   <code>r(l<sub>i</sub>)</code> where <code>r(e) = 2<sup>e</sup>-1</code>. See
   [`dict.RunsChain`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/dict#RunsChain).

This approach has proved highly effective against cryptographic exponents
which frequently exhibit binary structure, such as those derived from
[Solinas primes](https://en.wikipedia.org/wiki/Solinas_prime).

> I have not seen this method discussed in the literature. Please help me find references to prior art if you know any.

### Optimization

Close inspection of addition chains produced by other algorithms revealed
cases of redundant computation. This motivated a final optimization pass over
addition chains to remove unecessary steps. The [`alg/opt`](https://pkg.go.dev/github.com/mmcloughlin/addchain/alg/opt) package
implements the following optimization:

1. Determine _all possible_ ways each element can be computed from those prior.
2. Count how many times each element is used where it is the _only possible_ way of computing that entry.
3. Prune elements that are always used in computations that have an alternative.

These micro-optimizations were vital in closing the gap between `addchain`'s
automated approaches and hand-optimized chains. This technique is reminiscent
of basic passes in optimizing compilers, raising the question of whether
other [compiler optimizations could apply to addition
chains](https://github.com/mmcloughlin/addchain/issues/24)?

> I have not seen this method discussed in the literature. Please help me find references to prior art if you know any.

## Citing

If you use `addchain` in your research a citation would be appreciated.
Citing a specific release is preferred, since they are [archived on
Zenodo](https://doi.org/10.5281/zenodo.4625263) and assigned a DOI. Please use the
following BibTeX to cite the most recent [0.4.0
release](https://github.com/mmcloughlin/addchain/releases/tag/v0.4.0).

```bib
@misc{addchain,
    title        = {addchain: Cryptographic Addition Chain Generation in Go},
    author       = {Michael B. McLoughlin},
    year         = 2021,
    month        = oct,
    howpublished = {Repository \url{https://github.com/mmcloughlin/addchain}},
    version      = {0.4.0},
    license      = {BSD 3-Clause License},
    doi          = {10.5281/zenodo.5622943},
    url          = {https://doi.org/10.5281/zenodo.5622943},
}
```

If you need to cite a currently unreleased version please consider [filing an
issue](https://github.com/mmcloughlin/addchain/issues/new) to request a new
release, or to discuss an appropriate format for the citation.

## Thanks

Thank you to [Tom Dean](https://web.stanford.edu/~trdean/), [Riad
Wahby](https://wahby.org/), [Brian Smith](https://briansmith.org/) and
[str4d](https://github.com/str4d) for advice and encouragement. Thanks also to
[Damian Gryski](https://github.com/dgryski) and [Martin
Glancy](https://twitter.com/mglancy) for review.

## Contributing

Contributions to `addchain` are welcome:

* [Submit bug reports](https://github.com/mmcloughlin/addchain/issues/new) to
  the issues page.
* Suggest [test cases](https://github.com/mmcloughlin/addchain/blob/e6c070065205efcaa02627ab1b23e8ce6aeea1db/internal/results/results.go#L62)
  or update best-known hand-optimized results.
* Pull requests accepted. Please discuss in the [issues section](https://github.com/mmcloughlin/addchain/issues)
  before starting significant work.

## License

`addchain` is available under the [BSD 3-Clause License](LICENSE).
