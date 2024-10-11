module github.com/cosmos/gogoproto

go 1.19

require (
	github.com/golang/protobuf v1.5.3
	github.com/google/go-cmp v0.6.0
	github.com/tendermint/go-amino v0.16.0
	google.golang.org/grpc v1.62.1
	google.golang.org/protobuf v1.32.0
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/google/gofuzz v1.2.0 // indirect
	github.com/stretchr/testify v1.8.4 // indirect
	golang.org/x/net v0.20.0 // indirect
	golang.org/x/sys v0.16.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240123012728-ef4313101c80 // indirect
)

// API changed in an incompatible way
retract v1.4.8
