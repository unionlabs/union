module galois

go 1.22.5

toolchain go1.23.1

require (
	github.com/cometbft/cometbft v0.38.2
	github.com/cometbft/cometbft/api v1.0.0-rc.1
	github.com/consensys/gnark v0.7.2-0.20230418172633-f83323bdf138
	github.com/consensys/gnark-crypto v0.12.2-0.20240215234832-d72fcb379d3e
	github.com/rs/zerolog v1.33.0
	github.com/spf13/cobra v1.8.1
	github.com/stretchr/testify v1.9.0
	golang.org/x/net v0.28.0
	google.golang.org/grpc v1.66.0
	google.golang.org/protobuf v1.34.2
)

require (
	github.com/bits-and-blooms/bitset v1.8.0 // indirect
	github.com/blang/semver/v4 v4.0.0 // indirect
	github.com/btcsuite/btcd/btcec/v2 v2.3.4 // indirect
	github.com/consensys/bavard v0.1.13 // indirect
	github.com/cosmos/gogoproto v1.7.0 // indirect
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/decred/dcrd/dcrec/secp256k1/v4 v4.2.0 // indirect
	github.com/fxamacker/cbor/v2 v2.5.0 // indirect
	github.com/go-kit/log v0.2.1 // indirect
	github.com/go-logfmt/logfmt v0.6.0 // indirect
	github.com/golang/protobuf v1.5.4 // indirect
	github.com/google/go-cmp v0.6.0 // indirect
	github.com/google/pprof v0.0.0-20230817174616-7a8ec2ada47b // indirect
	github.com/holiman/uint256 v1.3.1 // indirect
	github.com/inconshreveable/mousetrap v1.1.0 // indirect
	github.com/ingonyama-zk/icicle v0.0.0-20230928131117-97f0079e5c71 // indirect
	github.com/ingonyama-zk/iciclegnark v0.1.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/mmcloughlin/addchain v0.4.0 // indirect
	github.com/oasisprotocol/curve25519-voi v0.0.0-20230904125328-1f23a7beb09a // indirect
	github.com/petermattis/goid v0.0.0-20240813172612-4fcff4a6cae7 // indirect
	github.com/pkg/errors v0.9.1 // indirect
	github.com/pmezard/go-difflib v1.0.1-0.20181226105442-5d4384ee4fb2 // indirect
	github.com/sasha-s/go-deadlock v0.3.5 // indirect
	github.com/spf13/pflag v1.0.5 // indirect
	github.com/supranational/blst v0.3.13 // indirect
	github.com/tunabay/go-bitarray v1.3.1 // indirect
	github.com/x448/float16 v0.8.4 // indirect
	golang.org/x/crypto v0.26.0 // indirect
	golang.org/x/exp v0.0.0-20231110203233-9a3e6036ecaa // indirect
	golang.org/x/sync v0.8.0 // indirect
	golang.org/x/sys v0.24.0 // indirect
	golang.org/x/text v0.17.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240604185151-ef581f913117 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
	rsc.io/tmplfunc v0.0.3 // indirect
)

replace (
	github.com/cometbft/cometbft => github.com/unionlabs/cometbft v0.0.0-20241014115136-9ea090e8aeb8
	github.com/cometbft/cometbft/api => github.com/unionlabs/cometbft/api v0.0.0-20241014115136-9ea090e8aeb8
	github.com/consensys/gnark => github.com/consensys/gnark v0.9.2-0.20240312175655-ce0186ef32c1
	// Fork of gnark crypto until https://github.com/ConsenSys/gnark-crypto/pull/314 is merged
	github.com/consensys/gnark-crypto => github.com/unionlabs/gnark-crypto v0.0.0-20240112093739-635c1b6963c6
	github.com/cosmos/gogoproto => github.com/cosmos/gogoproto v1.4.11
	github.com/tunabay/go-bitarray => github.com/poisonphang/go-bitarray v0.0.0-20240912214703-d6127bb4d1bd
)
