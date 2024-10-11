module cosmossdk.io/collections

go 1.21

require (
	cosmossdk.io/core v0.12.0
	cosmossdk.io/core/testing v0.0.0-00010101000000-000000000000
	github.com/stretchr/testify v1.9.0
	pgregory.net/rapid v1.1.0
)

require (
	github.com/cosmos/gogoproto v1.5.0 // indirect
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/google/go-cmp v0.6.0 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/tidwall/btree v1.7.0 // indirect
	golang.org/x/exp v0.0.0-20240222234643-814bf88cf225 // indirect
	golang.org/x/net v0.27.0 // indirect
	golang.org/x/sys v0.22.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240709173604-40e1e62336c5 // indirect
	google.golang.org/grpc v1.64.1 // indirect
	google.golang.org/protobuf v1.34.2 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace (
	cosmossdk.io/core => ../core
	cosmossdk.io/core/testing => ../core/testing
)
