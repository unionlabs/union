/*
Package intmath provides a simple integer math library.

The standard Go math library uses the float64 type for everything. 
This often requires casting both input and result, resulting in 
more verbose and (possibly) slower code. This library aims to 
extend the Go math library by providing math packages for the int,
uint, int32, uint32, int64 and uint64 types.

The intmath package itself is just a dummy package used for 
documentation, testing and benchmarking - the real library is
in the subpackages. Import the relevant type to use math
functions tailored to that type.

Note that the dedicated FPU in your computer can be so fast that 
casting to and from float64 would be faster than using integer 
math. In other words, the standard math library can in theory 
outperform these libraries despite repeated casting to and from
float64. 

Similarly, some code in u64 and i64 may be optimised to run faster
on 32bit systems, at a minor cost for 64bit systems. All else being
equal, the code in the library will favour this behaviour 
(see for example the u64.Log2 source). 

Benchmarks are included so you can easily test the differences for
yourself. They assume a uniform distribution of input.

Not all math functions will be copied, as not all of them make 
sense in an integer math context. At the moment, the following 
functions have been implemented:

 Abs(x T) T		// only for signed types, obviously
 Max(x, y T) T
 Min(x, y T) T
 GCD(a, b T) T		// adapted from github.com/cznic/mathutil
 Log10(n T) T		// adapted from graphics.stanford.edu/~seander/bithacks.html
 Log2(n T) (r T)		// adapted from graphics.stanford.edu/~seander/bithacks.html
 Pow(x, y T) (r T)
 Sqrt(x T) (r T)		// adapted from Hacker's Delight
 Cbrt(n T) T		// adapted from Hacker's Delight
 BitCount(v T) T		// adapted from Sonia Keys' library
 TrailingZeros(v T) T	// adapted from Sonia Key's library
 Is64bits() bool		// only for intgr and uint libraries, obviously

As a general rule, all functions take and return types matching
the type of the library. Functions in u64 only take and return
uint64, for example. Think of it as fake function overloading:
the functions have the same name in every library, but the
library name determines the type. Compare the following code
snippets using the standard math library, a hypothetical library
throwing all types in the same library, and this library:

 c := int(math.Pow(float64(a), float64(b)))
 k := uint32(Math.Pow(float64(i), float64(j)))
 z := uint64(math.Pow(float64(x), float64(y)))

 c := intmath.Pow(a, b))
 k := intmath.PowUint32(i, j)
 z := intmath.PowUint64(x, y)

 c := intgr.Pow(a, b)
 k := u32.Pow(i, j)
 z := u64.Pow(x, y)

Note that the last example is the least verbose, yet it is 
immediately clear what the types of c, k and z are. If there ever
will be exceptions to this general rule of input and return types
the functions will be listed here and the reasoning behind making 
an exception will be explained in the documentation.

As mentioned before, the speed of modern FPUs should not be 
underestimated. Benchmarking is advised to see if your system 
actually benefits form this library. To do this run go test:

 go test code.google.com/p/intmath -test.bench="Benchmark*"

Some benchmarks also include alternate implementations for
different functions. These might just be more effective on your 
architecture.
*/
package intmath
