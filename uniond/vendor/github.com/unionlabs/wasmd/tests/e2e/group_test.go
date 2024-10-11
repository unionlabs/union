package e2e_test

import (
	"testing"
	"time"

	"cosmossdk.io/math/unsafe"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"

	"github.com/CosmWasm/wasmd/tests/e2e"
	"github.com/CosmWasm/wasmd/tests/ibctesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestGroupWithContract(t *testing.T) {
	// Given a group with a contract as only member
	// When  contract submits a proposal with try_execute
	// Then	 the payload msg is executed

	coord := ibctesting.NewCoordinator(t, 1)
	chain := coord.GetChain(ibctesting.GetChainID(1))
	contractAddr := e2e.InstantiateStargateReflectContract(t, chain)
	chain.Fund(contractAddr, sdkmath.NewIntFromUint64(1_000_000_000))

	members := []group.MemberRequest{
		{
			Address:  contractAddr.String(),
			Weight:   "1",
			Metadata: "my contract",
		},
	}
	msg, err := group.NewMsgCreateGroupWithPolicy(
		chain.SenderAccount.GetAddress().String(),
		members,
		"my group",
		"my metadata",
		false,
		group.NewPercentageDecisionPolicy("1", time.Second, 0),
	)
	require.NoError(t, err)
	rsp, err := chain.SendMsgs(msg)
	require.NoError(t, err)

	var createRsp group.MsgCreateGroupWithPolicyResponse
	chain.UnwrapExecTXResult(rsp, &createRsp)
	groupID, policyAddr := createRsp.GroupId, sdk.MustAccAddressFromBech32(createRsp.GroupPolicyAddress)
	require.NotEmpty(t, groupID)
	chain.Fund(policyAddr, sdkmath.NewIntFromUint64(1_000_000_000))
	// and a proposal submitted
	recipientAddr := sdk.AccAddress(rand.Bytes(address.Len))

	payload := []sdk.Msg{banktypes.NewMsgSend(policyAddr, recipientAddr, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.OneInt())))}
	propMsg, err := group.NewMsgSubmitProposal(policyAddr.String(), []string{contractAddr.String()}, payload, "my proposal", group.Exec_EXEC_TRY, "my title", "my description")
	require.NoError(t, err)

	rsp = e2e.MustExecViaStargateReflectContract(t, chain, contractAddr, propMsg)
	var execRsp types.MsgExecuteContractResponse
	chain.UnwrapExecTXResult(rsp, &execRsp)

	var groupRsp group.MsgSubmitProposalResponse
	require.NoError(t, chain.Codec.Unmarshal(execRsp.Data, &groupRsp))
	// require.NotEmpty(t, groupRsp.ProposalId)

	// and coins received
	recipientBalance := chain.Balance(recipientAddr, sdk.DefaultBondDenom)
	expBalanceAmount := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.OneInt())
	assert.Equal(t, expBalanceAmount.String(), recipientBalance.String())
}

func TestGroupWithNewReflectContract(t *testing.T) {
	// Given a group with a contract as only member
	// When  contract submits a proposal with try_execute
	// Then	 the payload msg is executed

	coord := ibctesting.NewCoordinator(t, 1)
	chain := coord.GetChain(ibctesting.GetChainID(1))
	contractAddr := e2e.InstantiateReflectContract(t, chain)
	chain.Fund(contractAddr, sdkmath.NewIntFromUint64(1_000_000_000))

	members := []group.MemberRequest{
		{
			Address:  contractAddr.String(),
			Weight:   "1",
			Metadata: "my contract",
		},
	}
	msg, err := group.NewMsgCreateGroupWithPolicy(
		chain.SenderAccount.GetAddress().String(),
		members,
		"my group",
		"my metadata",
		false,
		group.NewPercentageDecisionPolicy("1", time.Second, 0),
	)
	require.NoError(t, err)
	rsp, err := chain.SendMsgs(msg)
	require.NoError(t, err)

	var createRsp group.MsgCreateGroupWithPolicyResponse
	chain.UnwrapExecTXResult(rsp, &createRsp)
	groupID, policyAddr := createRsp.GroupId, sdk.MustAccAddressFromBech32(createRsp.GroupPolicyAddress)
	require.NotEmpty(t, groupID)
	chain.Fund(policyAddr, sdkmath.NewIntFromUint64(1_000_000_000))
	// and a proposal submitted
	recipientAddr := sdk.AccAddress(rand.Bytes(address.Len))

	payload := []sdk.Msg{banktypes.NewMsgSend(policyAddr, recipientAddr, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.OneInt())))}
	propMsg, err := group.NewMsgSubmitProposal(policyAddr.String(), []string{contractAddr.String()}, payload, "my proposal", group.Exec_EXEC_TRY, "my title", "my description")
	require.NoError(t, err)

	rsp = e2e.MustExecViaAnyReflectContract(t, chain, contractAddr, propMsg)
	var execRsp types.MsgExecuteContractResponse
	chain.UnwrapExecTXResult(rsp, &execRsp)

	var groupRsp group.MsgSubmitProposalResponse
	require.NoError(t, chain.Codec.Unmarshal(execRsp.Data, &groupRsp))

	// and coins received
	recipientBalance := chain.Balance(recipientAddr, sdk.DefaultBondDenom)
	expBalanceAmount := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.OneInt())
	assert.Equal(t, expBalanceAmount.String(), recipientBalance.String())
}
