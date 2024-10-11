module cosmossdk.io/store

go 1.23

toolchain go1.23.0

require (
	cosmossdk.io/core v1.0.0-alpha.2
	cosmossdk.io/core/testing v0.0.0-00010101000000-000000000000
	cosmossdk.io/errors v1.0.1
	cosmossdk.io/log v1.4.1
	cosmossdk.io/math v1.3.0
	github.com/cometbft/cometbft v1.0.0-rc1
	github.com/cometbft/cometbft/api v1.0.0-rc.1
	github.com/cosmos/cosmos-proto v1.0.0-beta.5
	github.com/cosmos/gogoproto v1.7.0
	github.com/cosmos/iavl v1.0.0-beta.1.0.20240813194616-eb5078efcf9e
	github.com/cosmos/ics23/go v0.11.0
	github.com/hashicorp/go-hclog v1.6.3
	github.com/hashicorp/go-metrics v0.5.3
	github.com/hashicorp/go-plugin v1.6.1
	github.com/hashicorp/golang-lru v1.0.2
	github.com/stretchr/testify v1.9.0
	github.com/tidwall/btree v1.7.0
	go.uber.org/mock v0.4.0
	golang.org/x/exp v0.0.0-20240506185415-9bf2ced13842
	google.golang.org/grpc v1.66.2
	google.golang.org/protobuf v1.34.2
	gotest.tools/v3 v3.5.1
)

require (
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/emicklei/dot v1.6.2 // indirect
	github.com/fatih/color v1.17.0 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/golang/protobuf v1.5.4 // indirect
	github.com/google/go-cmp v0.6.0 // indirect
	github.com/hashicorp/go-immutable-radix v1.3.1 // indirect
	github.com/hashicorp/go-uuid v1.0.1 // indirect
	github.com/hashicorp/yamux v0.1.1 // indirect
	github.com/jhump/protoreflect v1.17.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/mitchellh/go-testing-interface v1.14.1 // indirect
	github.com/oklog/run v1.1.0 // indirect
	github.com/petermattis/goid v0.0.0-20221215004737-a150e88a970d // indirect
	github.com/pkg/errors v0.9.1 // indirect
	github.com/pmezard/go-difflib v1.0.1-0.20181226105442-5d4384ee4fb2 // indirect
	github.com/rogpeppe/go-internal v1.12.0 // indirect
	github.com/rs/zerolog v1.33.0 // indirect
	golang.org/x/crypto v0.27.0 // indirect
	golang.org/x/net v0.29.0 // indirect
	golang.org/x/sys v0.25.0 // indirect
	golang.org/x/text v0.18.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240903143218-8af14fe29dc1 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace cosmossdk.io/core => ../core

replace cosmossdk.io/core/testing => ../core/testing
