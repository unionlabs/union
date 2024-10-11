package integration

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"testing"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	govkeeper "cosmossdk.io/x/gov/keeper"
	govtypes "cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var (
	CyberpunkCapabilities = []string{"staking", "mask", "stargate", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3", "cosmwasm_1_4"}
	ReflectCapabilities   = []string{"staking", "mask", "stargate", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3", "cosmwasm_1_4", "cosmwasm_2_0"}
)

func TestLoadStoredGovV1Beta1LegacyTypes(t *testing.T) {
	capabilities := make([]string, len(ReflectCapabilities)+1)
	copy(capabilities, ReflectCapabilities)
	capabilities = append(capabilities, "iterator")
	pCtx, keepers := keeper.CreateTestInput(t, false, capabilities)
	k := keepers.WasmKeeper
	keepers.GovKeeper.SetLegacyRouter(v1beta1.NewRouter().
		AddRoute(types.ModuleName, keeper.NewLegacyWasmProposalHandler(k, types.EnableAllProposals)),
	)
	myAddress := keeper.RandomAccountAddress(t)
	keepers.Faucet.Fund(pCtx, myAddress, sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewIntFromUint64(100_000_000)))
	keepers.Faucet.Fund(pCtx, myAddress, sdk.NewCoin("denom", sdkmath.NewIntFromUint64(100_000_000)))

	reflectExample := keeper.InstantiateReflectExampleContract(t, pCtx, keepers)
	burnerExample := keeper.StoreBurnerExampleContract(t, pCtx, keepers)
	hackatomExample := keeper.InstantiateHackatomExampleContract(t, pCtx, keepers)

	type StealMsg struct {
		Recipient string     `json:"recipient"`
		Amount    []sdk.Coin `json:"amount"`
	}
	stealMsg := struct {
		Steal StealMsg `json:"steal_funds"`
	}{Steal: StealMsg{
		Recipient: myAddress.String(),
		Amount:    []sdk.Coin{sdk.NewInt64Coin("denom", 75)},
	}}
	stealMsgBz := must(json.Marshal(stealMsg))

	specs := map[string]struct {
		legacyContent v1beta1.Content
	}{
		"store code": {
			legacyContent: &types.StoreCodeProposal{ //nolint:staticcheck
				Title:        "Foo",
				Description:  "Bar",
				Source:       "https://example.com/",
				Builder:      "cosmwasm/workspace-optimizer:v0.12.8",
				RunAs:        myAddress.String(),
				WASMByteCode: testdata.HackatomContractWasm(),
				CodeHash:     must(hex.DecodeString(testdata.ChecksumHackatom)),
			},
		},
		"instantiate": {
			legacyContent: &types.InstantiateContractProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				RunAs:       myAddress.String(),
				Admin:       myAddress.String(),
				CodeID:      reflectExample.CodeID,
				Label:       "testing",
				Msg:         []byte("{}"),
			},
		},
		"instantiate2": {
			legacyContent: &types.InstantiateContract2Proposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				RunAs:       myAddress.String(),
				Admin:       myAddress.String(),
				CodeID:      reflectExample.CodeID,
				Label:       "testing",
				Msg:         []byte("{}"),
				Salt:        []byte("mySalt"),
			},
		},
		"store and instantiate": {
			legacyContent: &types.StoreAndInstantiateContractProposal{ //nolint:staticcheck
				Title:        "Foo",
				Description:  "Bar",
				RunAs:        myAddress.String(),
				WASMByteCode: testdata.ReflectContractWasm(),
				Admin:        myAddress.String(),
				Label:        "testing",
				Msg:          []byte("{}"),
				Source:       "https://example.com/",
				Builder:      "cosmwasm/workspace-optimizer:v0.12.8",
				CodeHash:     reflectExample.Checksum,
			},
		},
		"migrate": {
			legacyContent: &types.MigrateContractProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				Contract:    reflectExample.Contract.String(),
				CodeID:      burnerExample.CodeID,
				Msg:         []byte(fmt.Sprintf(`{"payout": "%s"}`, myAddress)),
			},
		},
		"execute": {
			legacyContent: &types.ExecuteContractProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				Contract:    reflectExample.Contract.String(),
				RunAs:       reflectExample.CreatorAddr.String(),
				Msg: must(json.Marshal(testdata.ReflectHandleMsg{
					Reflect: &testdata.ReflectPayload{
						Msgs: []wasmvmtypes.CosmosMsg{{
							Bank: &wasmvmtypes.BankMsg{
								Send: &wasmvmtypes.SendMsg{
									ToAddress: myAddress.String(),
									Amount:    []wasmvmtypes.Coin{{Denom: "denom", Amount: "100"}},
								},
							},
						}},
					},
				})),
			},
		},
		"sudo": {
			&types.SudoContractProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				Contract:    hackatomExample.Contract.String(),
				Msg:         stealMsgBz,
			},
		},
		"update admin": {
			legacyContent: &types.UpdateAdminProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				Contract:    reflectExample.Contract.String(),
				NewAdmin:    myAddress.String(),
			},
		},
		"clear admin": {
			legacyContent: &types.ClearAdminProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				Contract:    reflectExample.Contract.String(),
			},
		},
		"pin codes": {
			legacyContent: &types.PinCodesProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				CodeIDs:     []uint64{reflectExample.CodeID},
			},
		},
		"unpin codes": {
			legacyContent: &types.UnpinCodesProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				CodeIDs:     []uint64{reflectExample.CodeID},
			},
		},
		"update instantiate config": {
			legacyContent: &types.UpdateInstantiateConfigProposal{ //nolint:staticcheck
				Title:       "Foo",
				Description: "Bar",
				AccessConfigUpdates: []types.AccessConfigUpdate{
					{CodeID: reflectExample.CodeID, InstantiatePermission: types.AllowNobody},
				},
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := pCtx.CacheContext()
			propID := mustSubmitAndExecuteLegacyProposal(t, ctx, spec.legacyContent, myAddress.String(), keepers)
			// when
			proposal, err := keepers.GovKeeper.Proposals.Get(ctx, propID)
			// then
			require.NoError(t, err)
			require.Len(t, proposal.Messages, 1)
			assert.NotNil(t, proposal.Messages[0].GetCachedValue())
		})
	}
}

func mustSubmitAndExecuteLegacyProposal(t *testing.T, ctx sdk.Context, content v1beta1.Content, myActorAddress string, keepers keeper.TestKeepers) uint64 {
	t.Helper()
	govAuthority := keepers.AccountKeeper.GetModuleAddress(govtypes.ModuleName).String()
	msgServer := govkeeper.NewMsgServerImpl(keepers.GovKeeper)
	// ignore all submit events
	contentMsg, rsp, err := submitLegacyProposal(t, ctx.WithEventManager(sdk.NewEventManager()), content, myActorAddress, govAuthority, msgServer)
	require.NoError(t, err)

	_, err = msgServer.ExecLegacyContent(ctx, v1.NewMsgExecLegacyContent(contentMsg.Content, govAuthority))
	require.NoError(t, err)
	return rsp.ProposalId
}

// does not fail on submit proposal
func submitLegacyProposal(t *testing.T, ctx sdk.Context, content v1beta1.Content, myActorAddress, govAuthority string, msgServer v1.MsgServer) (*v1.MsgExecLegacyContent, *v1.MsgSubmitProposalResponse, error) {
	t.Helper()
	contentMsg, err := v1.NewLegacyContent(content, govAuthority)
	require.NoError(t, err)

	proposal, err := v1.NewMsgSubmitProposal(
		[]sdk.Msg{contentMsg},
		sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewIntFromUint64(1_000_000))),
		myActorAddress,
		"",
		content.GetTitle(),
		content.GetDescription(),
		false,
	)
	require.NoError(t, err)

	// when stored
	rsp, err := msgServer.SubmitProposal(ctx, proposal)
	return contentMsg, rsp, err
}
