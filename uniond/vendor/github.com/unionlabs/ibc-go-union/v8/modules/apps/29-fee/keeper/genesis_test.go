package keeper_test

import (
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestInitGenesis() {
	packetID := channeltypes.NewPacketID(ibctesting.MockFeePort, ibctesting.FirstChannelID, 1)

	genesisState := types.GenesisState{
		IdentifiedFees: []types.IdentifiedPacketFees{
			{
				PacketId: packetID,
				PacketFees: []types.PacketFee{
					{
						Fee:           types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee),
						RefundAddress: suite.chainA.SenderAccount.GetAddress().String(),
						Relayers:      nil,
					},
				},
			},
		},
		FeeEnabledChannels: []types.FeeEnabledChannel{
			{
				PortId:    ibctesting.MockFeePort,
				ChannelId: ibctesting.FirstChannelID,
			},
		},
		RegisteredPayees: []types.RegisteredPayee{
			{
				Relayer:   suite.chainA.SenderAccount.GetAddress().String(),
				Payee:     suite.chainB.SenderAccount.GetAddress().String(),
				ChannelId: ibctesting.FirstChannelID,
			},
		},
		RegisteredCounterpartyPayees: []types.RegisteredCounterpartyPayee{
			{
				Relayer:           suite.chainA.SenderAccount.GetAddress().String(),
				CounterpartyPayee: suite.chainB.SenderAccount.GetAddress().String(),
				ChannelId:         ibctesting.FirstChannelID,
			},
		},
	}

	suite.chainA.GetSimApp().IBCFeeKeeper.InitGenesis(suite.chainA.GetContext(), genesisState)

	// check fee
	feesInEscrow, found := suite.chainA.GetSimApp().IBCFeeKeeper.GetFeesInEscrow(suite.chainA.GetContext(), packetID)
	suite.Require().True(found)
	suite.Require().Equal(genesisState.IdentifiedFees[0].PacketFees, feesInEscrow.PacketFees)

	// check fee is enabled
	isEnabled := suite.chainA.GetSimApp().IBCFeeKeeper.IsFeeEnabled(suite.chainA.GetContext(), ibctesting.MockFeePort, ibctesting.FirstChannelID)
	suite.Require().True(isEnabled)

	// check payee addresses
	payeeAddr, found := suite.chainA.GetSimApp().IBCFeeKeeper.GetPayeeAddress(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), ibctesting.FirstChannelID)
	suite.Require().True(found)
	suite.Require().Equal(genesisState.RegisteredPayees[0].Payee, payeeAddr)

	// check relayers
	counterpartyPayeeAddr, found := suite.chainA.GetSimApp().IBCFeeKeeper.GetCounterpartyPayeeAddress(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), ibctesting.FirstChannelID)
	suite.Require().True(found)
	suite.Require().Equal(genesisState.RegisteredCounterpartyPayees[0].CounterpartyPayee, counterpartyPayeeAddr)
}

func (suite *KeeperTestSuite) TestExportGenesis() {
	// set fee enabled
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeeEnabled(suite.chainA.GetContext(), ibctesting.MockFeePort, ibctesting.FirstChannelID)

	// setup & escrow the packet fee
	refundAcc := suite.chainA.SenderAccount.GetAddress()
	packetID := channeltypes.NewPacketID(ibctesting.MockFeePort, ibctesting.FirstChannelID, 1)
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)

	packetFee := types.NewPacketFee(fee, refundAcc.String(), []string{})
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees([]types.PacketFee{packetFee}))

	// set payee address
	suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
		suite.chainA.GetContext(),
		suite.chainA.SenderAccount.GetAddress().String(),
		suite.chainB.SenderAccount.GetAddress().String(),
		ibctesting.FirstChannelID,
	)

	// set counterparty payee address
	suite.chainA.GetSimApp().IBCFeeKeeper.SetCounterpartyPayeeAddress(
		suite.chainA.GetContext(),
		suite.chainA.SenderAccount.GetAddress().String(),
		suite.chainB.SenderAccount.GetAddress().String(),
		ibctesting.FirstChannelID,
	)

	// set forward relayer address
	suite.chainA.GetSimApp().IBCFeeKeeper.SetRelayerAddressForAsyncAck(suite.chainA.GetContext(), packetID, suite.chainA.SenderAccount.GetAddress().String())

	// export genesis
	genesisState := suite.chainA.GetSimApp().IBCFeeKeeper.ExportGenesis(suite.chainA.GetContext())

	// check fee enabled
	suite.Require().Equal(ibctesting.FirstChannelID, genesisState.FeeEnabledChannels[0].ChannelId)
	suite.Require().Equal(ibctesting.MockFeePort, genesisState.FeeEnabledChannels[0].PortId)

	// check fee
	suite.Require().Equal(packetID, genesisState.IdentifiedFees[0].PacketId)
	suite.Require().Equal(fee, genesisState.IdentifiedFees[0].PacketFees[0].Fee)
	suite.Require().Equal(refundAcc.String(), genesisState.IdentifiedFees[0].PacketFees[0].RefundAddress)
	suite.Require().Equal([]string(nil), genesisState.IdentifiedFees[0].PacketFees[0].Relayers)

	// check forward relayer addresses
	suite.Require().Equal(suite.chainA.SenderAccount.GetAddress().String(), genesisState.ForwardRelayers[0].Address)
	suite.Require().Equal(packetID, genesisState.ForwardRelayers[0].PacketId)

	// check payee addresses
	suite.Require().Equal(suite.chainA.SenderAccount.GetAddress().String(), genesisState.RegisteredPayees[0].Relayer)
	suite.Require().Equal(suite.chainB.SenderAccount.GetAddress().String(), genesisState.RegisteredPayees[0].Payee)
	suite.Require().Equal(ibctesting.FirstChannelID, genesisState.RegisteredPayees[0].ChannelId)

	// check registered counterparty payee addresses
	suite.Require().Equal(suite.chainA.SenderAccount.GetAddress().String(), genesisState.RegisteredCounterpartyPayees[0].Relayer)
	suite.Require().Equal(suite.chainB.SenderAccount.GetAddress().String(), genesisState.RegisteredCounterpartyPayees[0].CounterpartyPayee)
	suite.Require().Equal(ibctesting.FirstChannelID, genesisState.RegisteredCounterpartyPayees[0].ChannelId)
}
