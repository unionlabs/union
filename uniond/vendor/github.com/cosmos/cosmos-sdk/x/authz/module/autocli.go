package authz

import (
	"fmt"

	authzv1beta1 "cosmossdk.io/api/cosmos/authz/v1beta1"
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"

	"github.com/cosmos/cosmos-sdk/version"
	bank "github.com/cosmos/cosmos-sdk/x/bank/types"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: authzv1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Grants",
					Use:       "grants [granter-addr] [grantee-addr] <msg-type-url>",
					Short:     "Query grants for a granter-grantee pair and optionally a msg-type-url",
					Long:      "Query authorization grants for a granter-grantee pair. If msg-type-url is set, it will select grants only for that msg type.",
					Example:   fmt.Sprintf("%s query authz grants cosmos1skj.. cosmos1skjwj.. %s", version.AppName, bank.SendAuthorization{}.MsgTypeURL()),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "granter"},
						{ProtoField: "grantee"},
						{ProtoField: "msg_type_url", Optional: true},
					},
				},
				{
					RpcMethod: "GranterGrants",
					Use:       "grants-by-granter [granter-addr]",
					Short:     "Query authorization grants granted by granter",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "granter"},
					},
				},
				{
					RpcMethod: "GranteeGrants",
					Use:       "grants-by-grantee [grantee-addr]",
					Short:     "Query authorization grants granted to a grantee",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "grantee"},
					},
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service:              authzv1beta1.Msg_ServiceDesc.ServiceName,
			EnhanceCustomCommand: false, // use custom commands only until v0.51
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Exec",
					Use:       "exec [msg-json-file] --from [grantee]",
					Short:     "Execute tx on behalf of granter account",
					Example:   fmt.Sprintf("$ %s tx authz exec msg.json --from grantee\n $ %[1]s tx bank send [granter] [recipient] [amount] --generate-only | jq .body.messages > msg.json && %[1]s tx authz exec msg.json --from grantee", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "msgs", Varargs: true},
					},
				},
				{
					RpcMethod: "Revoke",
					Use:       "revoke [grantee] [msg-type-url] --from [granter]",
					Short:     `Revoke authorization from a granter to a grantee`,
					Example: fmt.Sprintf(`%s tx authz revoke cosmos1skj.. %s --from=cosmos1skj..`,
						version.AppName, bank.SendAuthorization{}.MsgTypeURL()),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "grantee"},
						{ProtoField: "msg_type_url"},
					},
				},
			},
		},
	}
}
