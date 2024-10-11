# secp256k1

 This package is copied from https://github.com/ethereum/go-ethereum/tree/8fddf27a989e246659fd018ea9be37b2b4f55326/crypto/secp256k1

 Unlike the rest of go-ethereum it is [3-clause BSD](https://opensource.org/licenses/BSD-3-Clause) licensed so compatible with our Apache2.0 license. We opt to copy in here rather than depend on go-ethereum to avoid issues with vendoring of the GPL parts of that repository by downstream.

## Duplicate Symbols

If a project is importing [go-ethereum](https://github.com/ethereum/go-ethereum) and the Cosmos SDK, cgo secp256k1 will only work on linux operating systems due to duplicated symbols. If you are testing on a mac, we recommend using a docker container or something similar. 

To avoid duplicate symbol errors `ldflags` must be set to allow for multiple definitions. 

#### Gcc

 + `go build -tags libsecp256k1_sdk  -ldflags=all="-extldflags=-Wl,--allow-multiple-definition"`

#### Clang

 + `go build -tags libsecp256k1_sdk -ldflags=all="-extldflags=-zmuldefs"`