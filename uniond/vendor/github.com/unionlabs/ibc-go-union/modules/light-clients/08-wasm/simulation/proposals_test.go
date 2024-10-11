package simulation_test

import (
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/simulation"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func TestProposalMsgs(t *testing.T) {
	// initialize parameters
	s := rand.NewSource(1)
	r := rand.New(s)

	ctx := sdk.NewContext(nil, cmtproto.Header{}, true, nil)
	accounts := simtypes.RandomAccounts(r, 3)

	// execute ProposalMsgs function
	weightedProposalMsgs := simulation.ProposalMsgs()
	require.Equal(t, 1, len(weightedProposalMsgs))
	w0 := weightedProposalMsgs[0]

	require.Equal(t, simulation.OpWeightMsgStoreCode, w0.AppParamsKey())
	require.Equal(t, simulation.DefaultWeightMsgStoreCode, w0.DefaultWeight())

	msg := w0.MsgSimulatorFn()(r, ctx, accounts)
	msgStoreCode, ok := msg.(*types.MsgStoreCode)
	require.True(t, ok)

	require.Equal(t, sdk.AccAddress(address.Module("gov")).String(), msgStoreCode.Signer)
	require.Equal(t, msgStoreCode.WasmByteCode, []byte{0x01})
}
