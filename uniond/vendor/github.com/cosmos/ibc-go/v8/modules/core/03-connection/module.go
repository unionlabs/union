package connection

import (
	"github.com/cosmos/gogoproto/grpc"
	"github.com/spf13/cobra"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/client/cli"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
)

// Name returns the IBC connection ICS name.
func Name() string {
	return types.SubModuleName
}

// GetQueryCmd returns the root query command for the IBC connections.
func GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// RegisterQueryService registers the gRPC query service for IBC connections.
func RegisterQueryService(server grpc.Server, queryServer types.QueryServer) {
	types.RegisterQueryServer(server, queryServer)
}
