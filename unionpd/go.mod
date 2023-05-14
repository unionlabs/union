module cometbls-prover

go 1.20

require (
	github.com/cometbft/cometbft v0.37.0
	github.com/consensys/gnark v0.7.2-0.20230418172633-f83323bdf138
	github.com/consensys/gnark-crypto v0.9.2-0.20230329155745-a57dcc3b53de
	github.com/holiman/uint256 v1.2.2
	github.com/spf13/cobra v1.7.0
	golang.org/x/crypto v0.7.0
	google.golang.org/grpc v1.54.0
	google.golang.org/protobuf v1.28.2-0.20220831092852-f930b1dc76e8
)

require (
	github.com/bits-and-blooms/bitset v1.5.0 // indirect
	github.com/blang/semver/v4 v4.0.0 // indirect
	github.com/btcsuite/btcd/btcec/v2 v2.3.2 // indirect
	github.com/consensys/bavard v0.1.13 // indirect
	github.com/cosmos/gogoproto v1.4.6 // indirect
	github.com/decred/dcrd/dcrec/secp256k1/v4 v4.1.0 // indirect
	github.com/fxamacker/cbor/v2 v2.4.0 // indirect
	github.com/golang/protobuf v1.5.3 // indirect
	github.com/google/go-cmp v0.5.9 // indirect
	github.com/google/pprof v0.0.0-20230309165930-d61513b1440d // indirect
	github.com/inconshreveable/mousetrap v1.1.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.16 // indirect
	github.com/mmcloughlin/addchain v0.4.0 // indirect
	github.com/petermattis/goid v0.0.0-20180202154549-b0b1615b78e5 // indirect
	github.com/rs/zerolog v1.29.0 // indirect
	github.com/sasha-s/go-deadlock v0.3.1 // indirect
	github.com/spf13/pflag v1.0.5 // indirect
	github.com/x448/float16 v0.8.4 // indirect
	golang.org/x/exp v0.0.0-20230310171629-522b1b587ee0 // indirect
	golang.org/x/net v0.8.0 // indirect
	golang.org/x/sys v0.6.0 // indirect
	golang.org/x/text v0.8.0 // indirect
	google.golang.org/genproto v0.0.0-20230216225411-c8e22ba71e44 // indirect
	rsc.io/tmplfunc v0.0.3 // indirect
)

replace (
	github.com/cometbft/cometbft => github.com/unionfi/cometbls v0.0.0-20230410071201-f19ae296cf17
	github.com/consensys/gnark => github.com/hussein-aitlahcen/gnark v0.0.0-20230419121246-e325d86969ea
	github.com/consensys/gnark-crypto => github.com/hussein-aitlahcen/gnark-crypto v0.0.0-20230419121058-59d3a8050fda
	github.com/cosmos/cosmos-sdk => github.com/unionfi/cosmos-sdk v0.0.0-20230410074509-021566a5aba4
)
