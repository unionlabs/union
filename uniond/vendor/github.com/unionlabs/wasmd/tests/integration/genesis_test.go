package integration

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestInitGenesis(t *testing.T) {
	data := setupTest(t)

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := data.faucet.NewFundedRandomAccount(data.ctx, deposit.Add(deposit...)...)
	fred := data.faucet.NewFundedRandomAccount(data.ctx, topUp...)

	msg := types.MsgStoreCode{
		Sender:       creator.String(),
		WASMByteCode: testContract,
	}
	h := data.msgServiceRouter.Handler(&msg)
	q := data.grpcQueryRouter

	err := msg.ValidateBasic()
	require.NoError(t, err)

	res, err := h(data.ctx, &msg)
	require.NoError(t, err)
	assertStoreCodeResponse(t, res.Data, 1)

	bob := keyPubAddr()
	initMsg := initMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	instMsg := types.MsgInstantiateContract{
		Sender: creator.String(),
		CodeID: firstCodeID,
		Msg:    initMsgBz,
		Funds:  deposit,
		Label:  "testing",
	}
	h = data.msgServiceRouter.Handler(&instMsg)
	res, err = h(data.ctx, &instMsg)
	require.NoError(t, err)
	contractBech32Addr := parseInitResponse(t, res.Data)

	execMsg := types.MsgExecuteContract{
		Sender:   fred.String(),
		Contract: contractBech32Addr,
		Msg:      []byte(`{"release":{}}`),
		Funds:    topUp,
	}
	h = data.msgServiceRouter.Handler(&execMsg)
	res, err = h(data.ctx, &execMsg)
	require.NoError(t, err)
	// from https://github.com/CosmWasm/cosmwasm/blob/master/contracts/hackatom/src/contract.rs#L167
	assertExecuteResponse(t, res.Data, []byte{0xf0, 0x0b, 0xaa})

	// ensure all contract state is as after init
	assertCodeList(t, q, data.ctx, 1, data.encConf.Codec)
	assertCodeBytes(t, q, data.ctx, 1, testContract, data.encConf.Codec)

	assertContractList(t, q, data.ctx, 1, []string{contractBech32Addr}, data.encConf.Codec)
	assertContractInfo(t, q, data.ctx, contractBech32Addr, 1, creator, data.encConf.Codec)
	assertContractState(t, q, data.ctx, contractBech32Addr, state{
		Verifier:    fred.String(),
		Beneficiary: bob.String(),
		Funder:      creator.String(),
	}, data.encConf.Codec)

	// export into genstate
	genState := keeper.ExportGenesis(data.ctx, &data.keeper)

	// create new app to import genstate into
	newData := setupTest(t)
	q2 := newData.grpcQueryRouter

	// initialize new app with genstate
	_, err = keeper.InitGenesis(newData.ctx, &newData.keeper, *genState)
	require.NoError(t, err)

	// run same checks again on newdata, to make sure it was reinitialized correctly
	assertCodeList(t, q2, newData.ctx, 1, data.encConf.Codec)
	assertCodeBytes(t, q2, newData.ctx, 1, testContract, data.encConf.Codec)

	assertContractList(t, q2, newData.ctx, 1, []string{contractBech32Addr}, data.encConf.Codec)
	assertContractInfo(t, q2, newData.ctx, contractBech32Addr, 1, creator, data.encConf.Codec)
	assertContractState(t, q2, newData.ctx, contractBech32Addr, state{
		Verifier:    fred.String(),
		Beneficiary: bob.String(),
		Funder:      creator.String(),
	}, data.encConf.Codec)
}
