package e2e

import (
	"bytes"
	"encoding/base64"
	"fmt"
	"testing"
	"time"

	ibcfee "github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	ibctransfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types" //nolint:staticcheck
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"

	"github.com/CosmWasm/wasmd/app"
	wasmibctesting "github.com/CosmWasm/wasmd/tests/ibctesting"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestIBCFeesTransfer(t *testing.T) {
	// scenario:
	// given 2 chains
	//   with an ics-20 channel established
	// when an ics-29 fee is attached to an ibc package
	// then the relayer's payee is receiving the fee(s) on success
	marshaler := app.MakeEncodingConfig(t).Codec
	coord := wasmibctesting.NewCoordinator(t, 2)
	chainA := coord.GetChain(wasmibctesting.GetChainID(1))
	chainB := coord.GetChain(wasmibctesting.GetChainID(2))

	actorChainA := sdk.AccAddress(chainA.SenderPrivKey.PubKey().Address())
	actorChainB := sdk.AccAddress(chainB.SenderPrivKey.PubKey().Address())
	receiver := sdk.AccAddress(bytes.Repeat([]byte{1}, address.Len))
	payee := sdk.AccAddress(bytes.Repeat([]byte{2}, address.Len))
	oneToken := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(1)))

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: string(marshaler.MustMarshalJSON(&ibcfee.Metadata{FeeVersion: ibcfee.Version, AppVersion: ibctransfertypes.Version})),
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: string(marshaler.MustMarshalJSON(&ibcfee.Metadata{FeeVersion: ibcfee.Version, AppVersion: ibctransfertypes.Version})),
		Order:   channeltypes.UNORDERED,
	}
	// with an ics-20 transfer channel setup between both chains
	coord.Setup(path)
	appA := chainA.App.(*app.WasmApp)
	require.True(t, appA.IBCFeeKeeper.IsFeeEnabled(chainA.GetContext(), ibctransfertypes.PortID, path.EndpointA.ChannelID))
	// and with a payee registered on both chains
	_, err := chainA.SendMsgs(ibcfee.NewMsgRegisterPayee(ibctransfertypes.PortID, path.EndpointA.ChannelID, actorChainA.String(), payee.String()))
	require.NoError(t, err)
	_, err = chainB.SendMsgs(ibcfee.NewMsgRegisterCounterpartyPayee(ibctransfertypes.PortID, path.EndpointB.ChannelID, actorChainB.String(), payee.String()))
	require.NoError(t, err)

	// when a transfer package is sent
	transferCoin := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(1))
	ibcPayloadMsg := ibctransfertypes.NewMsgTransfer(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, transferCoin, actorChainA.String(), receiver.String(), clienttypes.Height{}, uint64(time.Now().Add(time.Minute).UnixNano()), "testing")
	ibcPackageFee := ibcfee.NewFee(oneToken, oneToken, sdk.Coins{})
	feeMsg := ibcfee.NewMsgPayPacketFee(ibcPackageFee, ibctransfertypes.PortID, path.EndpointA.ChannelID, actorChainA.String(), nil)
	_, err = chainA.SendMsgs(feeMsg, ibcPayloadMsg)
	require.NoError(t, err)
	pendingIncentivisedPackages := appA.IBCFeeKeeper.GetIdentifiedPacketFeesForChannel(chainA.GetContext(), ibctransfertypes.PortID, path.EndpointA.ChannelID)
	assert.Len(t, pendingIncentivisedPackages, 1)

	// and packages relayed
	require.NoError(t, coord.RelayAndAckPendingPackets(path))

	// then
	expBalance := ibctransfertypes.GetTransferCoin(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, transferCoin.Denom, transferCoin.Amount)
	gotBalance := chainB.Balance(receiver, expBalance.Denom)
	assert.Equal(t, expBalance.String(), gotBalance.String())
	payeeBalance := chainA.AllBalances(payee)
	assert.Equal(t, oneToken.Add(oneToken...).String(), payeeBalance.String())

	// and with a payee registered for chain B to A
	_, err = chainA.SendMsgs(ibcfee.NewMsgRegisterCounterpartyPayee(ibctransfertypes.PortID, path.EndpointA.ChannelID, actorChainA.String(), payee.String()))
	require.NoError(t, err)
	_, err = chainB.SendMsgs(ibcfee.NewMsgRegisterPayee(ibctransfertypes.PortID, path.EndpointB.ChannelID, actorChainB.String(), payee.String()))
	require.NoError(t, err)

	// and transfer from B to A
	ibcPayloadMsg = ibctransfertypes.NewMsgTransfer(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, transferCoin, actorChainB.String(), receiver.String(), clienttypes.Height{}, uint64(time.Now().Add(time.Minute).UnixNano()), "more testing")
	ibcPackageFee = ibcfee.NewFee(oneToken, oneToken, sdk.Coins{})
	feeMsg = ibcfee.NewMsgPayPacketFee(ibcPackageFee, ibctransfertypes.PortID, path.EndpointB.ChannelID, actorChainB.String(), nil)
	_, err = chainB.SendMsgs(feeMsg, ibcPayloadMsg)
	require.NoError(t, err)
	appB := chainB.App.(*app.WasmApp)
	pendingIncentivisedPackages = appB.IBCFeeKeeper.GetIdentifiedPacketFeesForChannel(chainB.GetContext(), ibctransfertypes.PortID, path.EndpointB.ChannelID)
	assert.Len(t, pendingIncentivisedPackages, 1)

	// when packages relayed
	require.NoError(t, coord.RelayAndAckPendingPackets(path))

	// then
	expBalance = ibctransfertypes.GetTransferCoin(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, transferCoin.Denom, transferCoin.Amount)
	gotBalance = chainA.Balance(receiver, expBalance.Denom)
	assert.Equal(t, expBalance.String(), gotBalance.String())
	payeeBalance = chainB.AllBalances(payee)
	assert.Equal(t, sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(2)).String(), payeeBalance.String())
}

func TestIBCFeesWasm(t *testing.T) {
	// scenario:
	// given 2 chains with cw20-ibc on chain A and native ics20 module on B
	//   and an ibc channel established
	// when an ics-29 fee is attached to an ibc package
	// then the relayer's payee is receiving the fee(s) on success
	marshaler := app.MakeEncodingConfig(t).Codec
	coord := wasmibctesting.NewCoordinator(t, 2)
	chainA := coord.GetChain(wasmibctesting.GetChainID(1))
	chainB := coord.GetChain(ibctesting.GetChainID(2))
	actorChainA := sdk.AccAddress(chainA.SenderPrivKey.PubKey().Address())
	actorChainB := sdk.AccAddress(chainB.SenderPrivKey.PubKey().Address())

	// setup chain A
	codeID := chainA.StoreCodeFile("./testdata/cw20_base.wasm.gz").CodeID

	initMsg := []byte(fmt.Sprintf(`{"decimals": 6, "name": "test", "symbol":"ALX", "initial_balances": [{"address": %q,"amount":"100000000"}] }`, actorChainA.String()))
	cw20ContractAddr := chainA.InstantiateContract(codeID, initMsg)

	initMsg = []byte(fmt.Sprintf(`{"default_timeout": 360, "gov_contract": %q, "allowlist":[{"contract":%q}]}`, actorChainA.String(), cw20ContractAddr.String()))
	codeID = chainA.StoreCodeFile("./testdata/cw20_ics20.wasm.gz").CodeID
	ibcContractAddr := chainA.InstantiateContract(codeID, initMsg)
	ibcContractPortID := chainA.ContractInfo(ibcContractAddr).IBCPortID

	payee := sdk.AccAddress(bytes.Repeat([]byte{2}, address.Len))
	oneToken := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(1)))

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibcContractPortID,
		Version: string(marshaler.MustMarshalJSON(&ibcfee.Metadata{FeeVersion: ibcfee.Version, AppVersion: ibctransfertypes.Version})),
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: string(marshaler.MustMarshalJSON(&ibcfee.Metadata{FeeVersion: ibcfee.Version, AppVersion: ibctransfertypes.Version})),
		Order:   channeltypes.UNORDERED,
	}
	// with an ics-29 fee enabled channel setup between both chains
	coord.Setup(path)
	appA := chainA.App.(*app.WasmApp)
	appB := chainB.App.(*app.WasmApp)
	require.True(t, appA.IBCFeeKeeper.IsFeeEnabled(chainA.GetContext(), ibcContractPortID, path.EndpointA.ChannelID))
	require.True(t, appB.IBCFeeKeeper.IsFeeEnabled(chainB.GetContext(), ibctransfertypes.PortID, path.EndpointB.ChannelID))
	// and with a payee registered for A -> B
	_, err := chainA.SendMsgs(ibcfee.NewMsgRegisterPayee(ibcContractPortID, path.EndpointA.ChannelID, actorChainA.String(), payee.String()))
	require.NoError(t, err)
	_, err = chainB.SendMsgs(ibcfee.NewMsgRegisterCounterpartyPayee(ibctransfertypes.PortID, path.EndpointB.ChannelID, actorChainB.String(), payee.String()))
	require.NoError(t, err)

	// when a transfer package is sent from ics20 contract on A to B
	transfer := base64.StdEncoding.EncodeToString([]byte(fmt.Sprintf(`{"channel": %q, "remote_address": %q}`, path.EndpointA.ChannelID, actorChainB.String())))
	exec := []byte(fmt.Sprintf(`{"send":{"contract": %q, "amount": "100", "msg": %q}}`, ibcContractAddr.String(), transfer))
	execMsg := wasmtypes.MsgExecuteContract{
		Sender:   actorChainA.String(),
		Contract: cw20ContractAddr.String(),
		Msg:      exec,
	}
	ibcPackageFee := ibcfee.NewFee(oneToken, oneToken, sdk.Coins{})
	feeMsg := ibcfee.NewMsgPayPacketFee(ibcPackageFee, ibcContractPortID, path.EndpointA.ChannelID, actorChainA.String(), nil)
	_, err = chainA.SendMsgs(feeMsg, &execMsg)
	require.NoError(t, err)
	pendingIncentivisedPackages := appA.IBCFeeKeeper.GetIdentifiedPacketFeesForChannel(chainA.GetContext(), ibcContractPortID, path.EndpointA.ChannelID)
	assert.Len(t, pendingIncentivisedPackages, 1)

	// and packages relayed
	require.NoError(t, coord.RelayAndAckPendingPackets(path))

	// then
	// on chain A
	gotCW20Balance, err := appA.WasmKeeper.QuerySmart(chainA.GetContext(), cw20ContractAddr, []byte(fmt.Sprintf(`{"balance":{"address": %q}}`, actorChainA.String())))
	require.NoError(t, err)
	assert.JSONEq(t, `{"balance":"99999900"}`, string(gotCW20Balance))
	payeeBalance := chainA.AllBalances(payee)
	assert.Equal(t, sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(2)).String(), payeeBalance.String())
	// and on chain B
	pendingIncentivisedPackages = appA.IBCFeeKeeper.GetIdentifiedPacketFeesForChannel(chainA.GetContext(), ibcContractPortID, path.EndpointA.ChannelID)
	assert.Len(t, pendingIncentivisedPackages, 0)
	expBalance := ibctransfertypes.GetTransferCoin(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, "cw20:"+cw20ContractAddr.String(), sdkmath.NewInt(100))
	gotBalance := chainB.Balance(actorChainB, expBalance.Denom)
	assert.Equal(t, expBalance.String(), gotBalance.String(), chainB.AllBalances(actorChainB))

	// and with a payee registered for chain B to A
	_, err = chainA.SendMsgs(ibcfee.NewMsgRegisterCounterpartyPayee(ibcContractPortID, path.EndpointA.ChannelID, actorChainA.String(), payee.String()))
	require.NoError(t, err)
	_, err = chainB.SendMsgs(ibcfee.NewMsgRegisterPayee(ibctransfertypes.PortID, path.EndpointB.ChannelID, actorChainB.String(), payee.String()))
	require.NoError(t, err)

	// and when sent back from chain B to A
	ibcPayloadMsg := ibctransfertypes.NewMsgTransfer(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, gotBalance, actorChainB.String(), actorChainA.String(), clienttypes.Height{}, uint64(time.Now().Add(time.Minute).UnixNano()), "even more tests")
	ibcPackageFee = ibcfee.NewFee(oneToken, oneToken, sdk.Coins{})
	feeMsg = ibcfee.NewMsgPayPacketFee(ibcPackageFee, ibctransfertypes.PortID, path.EndpointB.ChannelID, actorChainB.String(), nil)
	_, err = chainB.SendMsgs(feeMsg, ibcPayloadMsg)
	require.NoError(t, err)
	pendingIncentivisedPackages = appB.IBCFeeKeeper.GetIdentifiedPacketFeesForChannel(chainB.GetContext(), ibctransfertypes.PortID, path.EndpointB.ChannelID)
	assert.Len(t, pendingIncentivisedPackages, 1)

	// when packages relayed
	require.NoError(t, coord.RelayAndAckPendingPackets(path))

	// then
	// on chain A
	gotCW20Balance, err = appA.WasmKeeper.QuerySmart(chainA.GetContext(), cw20ContractAddr, []byte(fmt.Sprintf(`{"balance":{"address": %q}}`, actorChainA.String())))
	require.NoError(t, err)
	assert.JSONEq(t, `{"balance":"100000000"}`, string(gotCW20Balance))
	// and on chain B
	payeeBalance = chainB.AllBalances(payee)
	assert.Equal(t, sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(2)).String(), payeeBalance.String())
}
