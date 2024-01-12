# ICICLE CUDA to Golang Binding Guide

This guide provides instructions on how to compile CUDA code using the provided Makefile, and then how to use the resulting shared libraries to bind Golang to ICICLE's CUDA code.

## Prerequisites

To compile the CUDA files, you will need:

- CUDA toolkit installed. The Makefile assumes CUDA is installed in `/usr/local/cuda`. If CUDA is installed in a different location, please adjust the `CUDA_ROOT_DIR` variable accordingly.
- A compatible GPU and corresponding driver installed on your machine.

## Structure of the Makefile

The Makefile is designed to compile CUDA files for three curves: BN254, BLS12_381, and BLS12_377. The source files are located in the `icicle/curves/` directory.

## Compiling CUDA Code

1. Navigate to the directory containing the Makefile in your terminal.
2. To compile all curve libraries, use the `make all` command. This will create three shared libraries: `libbn254.so`, `libbls12_381.so`, and `libbls12_377.so`.
3. If you want to compile a specific curve, you can do so by specifying the target. For example, to compile only the BN254 curve, use `make libbn254.so`. Replace `libbn254.so` with `libbls12_381.so` or `libbls12_377.so` to compile those curves instead.

The resulting `.so` files are the compiled shared libraries for each curve.

## Golang Binding

The shared libraries produced from the CUDA code compilation are used to bind Golang to ICICLE's CUDA code.

1. These shared libraries (`libbn254.so`, `libbls12_381.so`, `libbls12_377.so`) can be imported in your Go project to leverage the GPU accelerated functionalities provided by ICICLE. 

2. In your Go project, you can use `cgo` to link these shared libraries. Here's a basic example on how you can use `cgo` to link these libraries:

```go
/*
#cgo LDFLAGS: -L/path/to/shared/libs -lbn254 -lbls12_381 -lbls12_377
#include "icicle.h" // make sure you use the correct header file(s)
*/
import "C"

func main() {
    // Now you can call the C functions from the ICICLE libraries.
    // Note that C function calls are prefixed with 'C.' in Go code.
}
```

Replace `/path/to/shared/libs` with the actual path where the shared libraries are located on your system.

## Cleaning up

If you want to remove the compiled files, you can use the `make clean` command. This will remove the `libbn254.so`, `libbls12_381.so`, and `libbls12_377.so` files.

## Common issues

### Cannot find shared library

In some cases you may encounter the following error, despite exporting the correct `LD_LIBRARY_PATH`.

```
/usr/local/go/pkg/tool/linux_amd64/link: running gcc failed: exit status 1
/usr/bin/ld: cannot find -lbn254: No such file or directory
/usr/bin/ld: cannot find -lbn254: No such file or directory
/usr/bin/ld: cannot find -lbn254: No such file or directory
/usr/bin/ld: cannot find -lbn254: No such file or directory
/usr/bin/ld: cannot find -lbn254: No such file or directory
collect2: error: ld returned 1 exit status
```

This is normally fixed by exporting the path to the shared library location in the following way: `export CGO_LDFLAGS="-L/<path_to_shared_lib>/"`
