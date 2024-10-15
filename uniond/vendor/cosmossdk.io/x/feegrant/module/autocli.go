package module

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	feegrantv1beta1 "cosmossdk.io/api/cosmos/feegrant/v1beta1"

	"github.com/cosmos/cosmos-sdk/version"
)

func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: feegrantv1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Allowance",
					Use:       "grant <granter> <grantee>",
					Short:     "Query details of a single grant",
					Long:      "Query details for a grant. You can find the fee-grant of a granter and grantee.",
					Example:   fmt.Sprintf(`$ %s query feegrant grant [granter] [grantee]`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "granter"},
						{ProtoField: "grantee"},
					},
				},
				{
					RpcMethod: "Allowances",
					Use:       "grants-by-grantee <grantee>",
					Short:     "Query all grants of a grantee",
					Long:      "Queries all the grants for a grantee address.",
					Example:   fmt.Sprintf(`$ %s query feegrant grants-by-grantee [grantee]`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "grantee"},
					},
				},
				{
					RpcMethod: "AllowancesByGranter",
					Use:       "grants-by-granter <granter>",
					Short:     "Query all grants by a granter",
					Example:   fmt.Sprintf(`$ %s query feegrant grants-by-granter [granter]`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "granter"},
					},
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: feegrantv1beta1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "RevokeAllowance",
					Use:       "revoke <granter> <grantee>",
					Short:     "Revoke a fee grant",
					Long:      "Revoke fee grant from a granter to a grantee. Note, the '--from' flag is ignored as it is implied from [granter]",
					Example:   fmt.Sprintf(`$ %s tx feegrant revoke [granter] [grantee]`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "granter"},
						{ProtoField: "grantee"},
					},
				},
				{
					RpcMethod: "PruneAllowances",
					Use:       "prune",
					Short:     "Prune expired allowances",
					Long:      "Prune up to 75 expired allowances in order to reduce the size of the store when the number of expired allowances is large.",
					Example:   fmt.Sprintf(`$ %s tx feegrant prune --from [mykey]`, version.AppName),
				},
			},
			EnhanceCustomCommand: true,
		},
	}
}
