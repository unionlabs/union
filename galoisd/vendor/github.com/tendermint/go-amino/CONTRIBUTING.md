# Contributing

Thank you for considering making contributions to go-amino! This repository follows the [contribution guidelines] of 
tendermint and the corresponding [coding repo]. Please take a look if you are not already familiar with those.

Besides what you can find in aforementioned resources, there are a few things to consider specific to go-amino. 
They are outlined below.

## Compatibility

### Protobuf

Amino aims to be and stay [protobuf] compatible. Please, ensure that any change you add retains protobuf compatibility.
Basic compatibility is ensured by tests. To not introduce a protobuf dependency, these tests are not run with every test 
run, though. You need to turn on a [build flag] to build and run those tests.

### Tendermint

Please ensure that tendermint still passes all tests when run with your changes. You can do so by cloning [tendermint].
Then update the dependency to your commit (or release) in the corresponding [Gopkg.toml] and run `dep ensure -v` to get 
tendermint build with your amino version. Finally, run `make test` (all in the tendermint project directory).


## Fuzzers

Amino is fuzzed using several fuzzers. At least run [gofuzz] by running the command:
```
make test
```
This is what circle-ci will also run for you.
 
Ideally, run the more in-depth [go-fuzzer], too. They are currently not run by circel-ci and we need to run it manually 
for any substantial change.
If go-fuzzer isn't installed on your system, make sure to run:
```
go get -u github.com/dvyukov/go-fuzz/go-fuzz-build github.com/dvyukov/go-fuzz/go-fuzz
```

The fuzzers are run by:
```
make gofuzz_json
```
and
```
make gofuzz_binary
```
respectively. Both fuzzers will run in an endless loop and you have to quit them manually. They will output 
any problems (crashers) on the commandline. You'll find details of those crashers in the project directories 
`tests/fuzz/binary/crashers` and `tests/fuzz/json/crashers` respectively. 

If you find a crasher related to your changes please fix it, or file an issue containing the crasher information.


[contribution guidelines]: https://github.com/tendermint/tendermint/blob/master/CONTRIBUTING.md
[coding repo]: https://github.com/tendermint/coding
[gofuzz]: https://github.com/google/gofuzz
[go-fuzzer]: https://github.com/dvyukov/go-fuzz
[protobuf]: https://developers.google.com/protocol-buffers/
[build flag]: https://github.com/tendermint/go-amino/blob/faa6e731944e2b7b6a46ad202902851e8ce85bee/tests/proto3/proto3_compat_test.go#L1
[tendermint]: https://github.com/tendermint/tendermint/
[Gopkg.toml]: https://github.com/tendermint/tendermint/blob/master/Gopkg.toml


