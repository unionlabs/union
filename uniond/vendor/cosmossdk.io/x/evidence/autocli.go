package evidence

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	evidencev1beta1 "cosmossdk.io/api/cosmos/evidence/v1beta1"

	"github.com/cosmos/cosmos-sdk/version"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: evidencev1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:      "Evidence",
					Use:            "evidence [hash]",
					Short:          "Query for evidence by hash",
					Example:        fmt.Sprintf("%s query evidence DF0C23E8634E480F84B9D5674A7CDC9816466DEC28A3358F73260F68D28D7660", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "hash"}},
				},
				{
					RpcMethod: "AllEvidence",
					Use:       "list",
					Short:     "Query all (paginated) submitted evidence",
					Example:   fmt.Sprintf("%s query evidence --page=2 --page-limit=50", version.AppName),
				},
			},
		},
	}
}
