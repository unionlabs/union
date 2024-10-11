package fee_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// Integration test to ensure ics29 works with ics20
func (suite *FeeTestSuite) TestFeeTransfer() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	feeTransferVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: transfertypes.Version}))
	path.EndpointA.ChannelConfig.Version = feeTransferVersion
	path.EndpointB.ChannelConfig.Version = feeTransferVersion
	path.EndpointA.ChannelConfig.PortID = transfertypes.PortID
	path.EndpointB.ChannelConfig.PortID = transfertypes.PortID

	suite.coordinator.Setup(path)

	// set up coin & ics20 packet
	coin := ibctesting.TestCoin
	fee := types.Fee{
		RecvFee:    defaultRecvFee,
		AckFee:     defaultAckFee,
		TimeoutFee: defaultTimeoutFee,
	}

	msgs := []sdk.Msg{
		types.NewMsgPayPacketFee(fee, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, suite.chainA.SenderAccount.GetAddress().String(), nil),
		transfertypes.NewMsgTransfer(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, coin, suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), clienttypes.NewHeight(1, 100), 0, ""),
	}
	res, err := suite.chainA.SendMsgs(msgs...)
	suite.Require().NoError(err) // message committed

	// after incentivizing the packets
	originalChainASenderAccountBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), ibctesting.TestCoin.Denom))

	packet, err := ibctesting.ParsePacketFromEvents(res.Events)
	suite.Require().NoError(err)

	// register counterparty address on chainB
	// relayerAddress is address of sender account on chainB, but we will use it on chainA
	// to differentiate from the chainA.SenderAccount for checking successful relay payouts
	relayerAddress := suite.chainB.SenderAccount.GetAddress()

	msgRegister := types.NewMsgRegisterCounterpartyPayee(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, suite.chainB.SenderAccount.GetAddress().String(), relayerAddress.String())
	_, err = suite.chainB.SendMsgs(msgRegister)
	suite.Require().NoError(err) // message committed

	// relay packet
	err = path.RelayPacket(packet)
	suite.Require().NoError(err) // relay committed

	// ensure relayers got paid
	// relayer for forward relay: chainB.SenderAccount
	// relayer for reverse relay: chainA.SenderAccount

	// check forward relay balance
	suite.Require().Equal(
		fee.RecvFee,
		sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainB.SenderAccount.GetAddress(), ibctesting.TestCoin.Denom)),
	)

	suite.Require().Equal(
		fee.AckFee, // ack fee paid, no refund needed since timeout_fee = recv_fee + ack_fee
		sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), ibctesting.TestCoin.Denom)).Sub(originalChainASenderAccountBalance[0]))
}

func (suite *FeeTestSuite) TestTransferFeeUpgrade() {
	var path *ibctesting.Path

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {},
			nil,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			// configure the initial path to create a regular transfer channel
			path.EndpointA.ChannelConfig.PortID = transfertypes.PortID
			path.EndpointB.ChannelConfig.PortID = transfertypes.PortID
			path.EndpointA.ChannelConfig.Version = transfertypes.Version
			path.EndpointB.ChannelConfig.Version = transfertypes.Version

			suite.coordinator.Setup(path)

			// configure the channel upgrade to upgrade to an incentivized fee enabled transfer channel
			upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: transfertypes.Version}))
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

			tc.malleate()

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeConfirm()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeOpen()
			suite.Require().NoError(err)

			expPass := tc.expError == nil
			if expPass {
				channelA := path.EndpointA.GetChannel()
				suite.Require().Equal(upgradeVersion, channelA.Version)

				channelB := path.EndpointB.GetChannel()
				suite.Require().Equal(upgradeVersion, channelB.Version)

				isFeeEnabled := suite.chainA.GetSimApp().IBCFeeKeeper.IsFeeEnabled(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(isFeeEnabled)

				isFeeEnabled = suite.chainB.GetSimApp().IBCFeeKeeper.IsFeeEnabled(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(isFeeEnabled)

				fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
				msgs := []sdk.Msg{
					types.NewMsgPayPacketFee(fee, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, suite.chainA.SenderAccount.GetAddress().String(), nil),
					transfertypes.NewMsgTransfer(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, ibctesting.TestCoin, suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), clienttypes.NewHeight(1, 100), 0, ""),
				}

				res, err := suite.chainA.SendMsgs(msgs...)
				suite.Require().NoError(err) // message committed

				feeEscrowAddr := suite.chainA.GetSimApp().AuthKeeper.GetModuleAddress(types.ModuleName)
				escrowBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), feeEscrowAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(escrowBalance.Amount, fee.Total().AmountOf(sdk.DefaultBondDenom))

				packet, err := ibctesting.ParsePacketFromEvents(res.Events)
				suite.Require().NoError(err)

				err = path.RelayPacket(packet)
				suite.Require().NoError(err) // relay committed

				escrowBalance = suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), feeEscrowAddr, sdk.DefaultBondDenom)
				suite.Require().True(escrowBalance.IsZero())
			} else {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnesidedFeeMiddlewareTransferHandshake() {
	RemoveFeeMiddleware(suite.chainB) // remove fee middleware from chainB

	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	feeTransferVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: transfertypes.Version}))
	path.EndpointA.ChannelConfig.Version = feeTransferVersion // this will be renegotiated by the Try step
	path.EndpointB.ChannelConfig.Version = ""                 // this will be overwritten by the Try step
	path.EndpointA.ChannelConfig.PortID = transfertypes.PortID
	path.EndpointB.ChannelConfig.PortID = transfertypes.PortID

	suite.coordinator.Setup(path)

	suite.Require().Equal(path.EndpointA.ChannelConfig.Version, transfertypes.Version)
	suite.Require().Equal(path.EndpointB.ChannelConfig.Version, transfertypes.Version)
}
