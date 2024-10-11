# intmath
Migrated from code.google.com/p/intmath

##About
This library extends the Go math library with math packages for the int, uint, int32, uint32, int64 and uint64 types. The standard Go math library uses float64 for everything. This often requires casting both input and result, resulting in more verbose and (possibly) slower code.

Note that the dedicated FPU in your computer can be so fast that casting to and from float64 would be faster than using integer math for certain operations (most notably calculating the square root).

## Installing
    go get github.com/thomaso-mirodin/intmath/

## Usage

[![GoDoc](https://godoc.org/ggithub.com/thomaso-mirodin/intmath?status.svg)](https://godoc.org/github.com/thomaso-mirodin/intmath)

The root intmath package is only used for godoc documentation, and testing and benchmarking. Import the relevant sub-package to use math functions tailored to the type of choice:

    package main

    import (
        "github.com/thomaso-mirodin/intmath/intgr"
    )

    func main() {
        a := 3
        b := 4
        c := intgr.Sqrt(a*a + b*b)
        println(c)
    }

Functions take and return types matching the type of the package (with some obvious exceptions). Think of it as the good bits of function overloading without the bad: the functions have the same name in every library, but the package name determines the type. Compare the code examples using the standard math library, a hypothetical library throwing all types in the same library, and this library:

    c := int(math.Pow(float64(a), float64(b)))
    k := uint32(Math.Pow(float64(i), float64(j)))
    z := uint64(math.Pow(float64(x), float64(y)))

    c := intmath.Pow(a, b))
    k := intmath.PowUint32(i, j)
    z := intmath.PowUint64(x, y)

    c := intgr.Pow(a, b)
    k := u32.Pow(i, j)
    z := u64.Pow(x, y)

Note that the last example is the least verbose, yet it is immediately clear what the types of c, k and z are. For those who feel that package names like "i64" make the code less clear because they do not explain what the library does, Jan Mercl's mathutil package also offers everything this library does in terms of math functions, and more (for now).

## Implemented
Not all functions in the Go math library will be copied, as not all of them make sense in an integer math context. At the moment the following functions have been implemented:

    Abs(x T) T            // only for signed types, obviously
    Max(x, y T) T
    Min(x, y T) T
    GCD(a, b T) T         // adapted from github.com/cznic/mathutil
    Log2(n T) (r T)       // adapted from graphics.stanford.edu/~seander/bithacks.html
    Log10(n T) T          // adapted from graphics.stanford.edu/~seander/bithacks.html
    Pow(x, y T) (r T)
    Cbrt(n T) T           // adapted from Hacker's Delight
    Sqrt(x T) (r T)       // adapted from Hacker's Delight
    BitCount(v T) T       // adapted from Sonia Keys' library
    TrailingZeros(v T) T  // adapted from Sonia Keys' library
    Is64bits() bool       // only for intgr and uint libraries, obviously

## Speed and Benchmarking
Some code in u64 and i64 may be optimised to run faster on 32bit systems, at a minor cost for 64bit systems. All else being equal, the code in the library will favour this behaviour (see for example the u64.Log2 source). Addendum: it seems that it's possible to make Go source files that only compile for 6g or 8g. Will update.

As mentioned before, the speed of modern FPUs should not be underestimated. Benchmarking is advised to see if your system actually benefits form this library. To do this run go test:

    go test github.com/thomaso-mirodin/intmath -test.bench="Benchmark*"

The benchmarks assume a uniform distribution of input. Some functions also include benchmarks for alternate implementations. These might just be more effective on your architecture.
