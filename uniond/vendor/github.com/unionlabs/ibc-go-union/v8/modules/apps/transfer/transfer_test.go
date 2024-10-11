package transfer_test

import (
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type TransferTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
	chainC *ibctesting.TestChain
}

func (suite *TransferTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 3)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
	suite.chainC = suite.coordinator.GetChain(ibctesting.GetChainID(3))
}

func NewTransferPath(chainA, chainB *ibctesting.TestChain) *ibctesting.Path {
	path := ibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig.PortID = ibctesting.TransferPort
	path.EndpointB.ChannelConfig.PortID = ibctesting.TransferPort
	path.EndpointA.ChannelConfig.Version = types.Version
	path.EndpointB.ChannelConfig.Version = types.Version

	return path
}

// Constructs the following sends based on the established channels/connections
// 1 - from chainA to chainB
// 2 - from chainB to chainC
// 3 - from chainC to chainB
func (suite *TransferTestSuite) TestHandleMsgTransfer() {
	// setup between chainA and chainB
	// NOTE:
	// pathAtoB.EndpointA = endpoint on chainA
	// pathAtoB.EndpointB = endpoint on chainB
	pathAtoB := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(pathAtoB)

	originalBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)
	timeoutHeight := clienttypes.NewHeight(1, 110)

	amount, ok := sdkmath.NewIntFromString("9223372036854775808") // 2^63 (one above int64)
	suite.Require().True(ok)
	coinToSendToB := sdk.NewCoin(sdk.DefaultBondDenom, amount)

	// send from chainA to chainB
	msg := types.NewMsgTransfer(pathAtoB.EndpointA.ChannelConfig.PortID, pathAtoB.EndpointA.ChannelID, coinToSendToB, suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), timeoutHeight, 0, "")
	res, err := suite.chainA.SendMsgs(msg)
	suite.Require().NoError(err) // message committed

	packet, err := ibctesting.ParsePacketFromEvents(res.Events)
	suite.Require().NoError(err)

	// relay send
	err = pathAtoB.RelayPacket(packet)
	suite.Require().NoError(err) // relay committed

	// check that module account escrow address has locked the tokens
	escrowAddress := types.GetEscrowAddress(packet.GetSourcePort(), packet.GetSourceChannel())
	balance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), escrowAddress, sdk.DefaultBondDenom)
	suite.Require().Equal(coinToSendToB, balance)

	// check that voucher exists on chain B
	voucherDenomTrace := types.ParseDenomTrace(types.GetPrefixedDenom(packet.GetDestPort(), packet.GetDestChannel(), sdk.DefaultBondDenom))
	balance = suite.chainB.GetSimApp().BankKeeper.GetBalance(suite.chainB.GetContext(), suite.chainB.SenderAccount.GetAddress(), voucherDenomTrace.IBCDenom())
	coinSentFromAToB := types.GetTransferCoin(pathAtoB.EndpointB.ChannelConfig.PortID, pathAtoB.EndpointB.ChannelID, sdk.DefaultBondDenom, amount)
	suite.Require().Equal(coinSentFromAToB, balance)

	// setup between chainB to chainC
	// NOTE:
	// pathBtoC.EndpointA = endpoint on chainB
	// pathBtoC.EndpointB = endpoint on chainC
	pathBtoC := ibctesting.NewTransferPath(suite.chainB, suite.chainC)
	suite.coordinator.Setup(pathBtoC)

	// send from chainB to chainC
	msg = types.NewMsgTransfer(pathBtoC.EndpointA.ChannelConfig.PortID, pathBtoC.EndpointA.ChannelID, coinSentFromAToB, suite.chainB.SenderAccount.GetAddress().String(), suite.chainC.SenderAccount.GetAddress().String(), timeoutHeight, 0, "")
	res, err = suite.chainB.SendMsgs(msg)
	suite.Require().NoError(err) // message committed

	packet, err = ibctesting.ParsePacketFromEvents(res.Events)
	suite.Require().NoError(err)

	err = pathBtoC.RelayPacket(packet)
	suite.Require().NoError(err) // relay committed

	// NOTE: fungible token is prefixed with the full trace in order to verify the packet commitment
	fullDenomPath := types.GetPrefixedDenom(pathBtoC.EndpointB.ChannelConfig.PortID, pathBtoC.EndpointB.ChannelID, voucherDenomTrace.GetFullDenomPath())

	// check that the balance is updated on chainC
	coinSentFromBToC := sdk.NewCoin(types.ParseDenomTrace(fullDenomPath).IBCDenom(), amount)
	balance = suite.chainC.GetSimApp().BankKeeper.GetBalance(suite.chainC.GetContext(), suite.chainC.SenderAccount.GetAddress(), coinSentFromBToC.Denom)
	suite.Require().Equal(coinSentFromBToC, balance)

	// check that balance on chain B is empty
	balance = suite.chainB.GetSimApp().BankKeeper.GetBalance(suite.chainB.GetContext(), suite.chainB.SenderAccount.GetAddress(), coinSentFromBToC.Denom)
	suite.Require().Zero(balance.Amount.Int64())

	// send from chainC back to chainB
	msg = types.NewMsgTransfer(pathBtoC.EndpointB.ChannelConfig.PortID, pathBtoC.EndpointB.ChannelID, coinSentFromBToC, suite.chainC.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), timeoutHeight, 0, "")
	res, err = suite.chainC.SendMsgs(msg)
	suite.Require().NoError(err) // message committed

	packet, err = ibctesting.ParsePacketFromEvents(res.Events)
	suite.Require().NoError(err)

	err = pathBtoC.RelayPacket(packet)
	suite.Require().NoError(err) // relay committed

	// check that balance on chain A is updated
	balance = suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)
	suite.Require().Equal(originalBalance.SubAmount(amount).Amount.Int64(), balance.Amount.Int64())

	// check that balance on chain B has the transferred amount
	balance = suite.chainB.GetSimApp().BankKeeper.GetBalance(suite.chainB.GetContext(), suite.chainB.SenderAccount.GetAddress(), coinSentFromAToB.Denom)
	suite.Require().Equal(coinSentFromAToB, balance)

	// check that module account escrow address is empty
	escrowAddress = types.GetEscrowAddress(packet.GetDestPort(), packet.GetDestChannel())
	balance = suite.chainB.GetSimApp().BankKeeper.GetBalance(suite.chainB.GetContext(), escrowAddress, coinSentFromAToB.Denom)
	suite.Require().Zero(balance.Amount.Int64())

	// check that balance on chain C is empty
	balance = suite.chainC.GetSimApp().BankKeeper.GetBalance(suite.chainC.GetContext(), suite.chainC.SenderAccount.GetAddress(), voucherDenomTrace.IBCDenom())
	suite.Require().Zero(balance.Amount.Int64())
}

func TestTransferTestSuite(t *testing.T) {
	testifysuite.Run(t, new(TransferTestSuite))
}
