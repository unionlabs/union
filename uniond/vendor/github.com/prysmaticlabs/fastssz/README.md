
# FastSSZ

Clone:

```
$ git clone git@github.com:prysmaticlabs/fastssz.git
```

Download the eth2.0 spec tests

```
$ make get-spec-tests
```

Regenerate the test spec encodings:

```
$ make build-spec-tests
```

Generate encodings for a specific package:

```
$ go run sszgen/*.go --path ./ethereumapis/eth/v1alpha1 [--objs BeaconBlock,Eth1Data]
```

Optionally, you can specify the objs you want to generate. Otherwise, it will generate encodings for all structs in the package. Note that if a struct does not have 'ssz' tags when required (i.e size of arrays), the generator will fail.

By default, it generates a file with the prefix '_encoding.go' for each file that contains a generated struct. Optionally, you can combine all the outputs in a single file with the 'output' flag.

```
$ go run sszgen/*.go --path ./ethereumapis/eth/v1alpha1 --output ./ethereumapis/eth/v1alpha1/encoding.go
```

Test the spectests:

```
$ go test -v ./spectests/... -run TestSpec
```

Run the fuzzer:

```
$ FUZZ_TESTS=True go test -v ./spectests/... -run TestFuzz
```

To install the generator run:

```
$ go get github.com/prysmaticlabs/fastssz/sszgen
```

Benchmark (BeaconBlock):

```
$ go test -v ./spectests/... -run=XXX -bench=.
goos: linux
goarch: amd64
pkg: github.com/prysmaticlabs/fastssz/spectests
BenchmarkMarshalGoSSZ-4       	    1366	    753160 ns/op	  115112 B/op	    8780 allocs/op
BenchmarkMarshalFast-4        	  240765	      5093 ns/op	   18432 B/op	       1 allocs/op
BenchmarkMarshalSuperFast-4   	  377835	      3041 ns/op	       0 B/op	       0 allocs/op
BenchmarkUnMarshalGoSSZ-4     	     847	   1395097 ns/op	  144608 B/op	    8890 allocs/op
BenchmarkUnMarshalFast-4      	   43824	     27190 ns/op	   31024 B/op	     577 allocs/op
PASS
ok  	github.com/prysmaticlabs/fastssz/spectests	6.608s
```

# Package reference

To reference a struct from another package use the '--include' flag to point to that package.

Example:

```
$ go run sszgen/*.go --path ./example2
$ go run sszgen/*.go --path ./example 
[ERR]: could not find struct with name 'Checkpoint'
$ go run sszgen/*.go --path ./example --include ./example2
```

There are some caveats required to use this functionality.
- If multiple input paths import the same package, all of them need to import it with the same alias if any.
- If the folder of the package is not the same as the name of the package, any input file that imports this package needs to do it with an alias.
