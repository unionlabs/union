package keeper_test

import (
	"fmt"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channelkeeper "github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

var (
	defaultRecvFee    = sdk.Coins{sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdkmath.NewInt(100)}}
	defaultAckFee     = sdk.Coins{sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdkmath.NewInt(200)}}
	defaultTimeoutFee = sdk.Coins{sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdkmath.NewInt(300)}}
	invalidCoins      = sdk.Coins{sdk.Coin{Denom: "invalidDenom", Amount: sdkmath.NewInt(100)}}
)

type KeeperTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
	chainC *ibctesting.TestChain

	path     *ibctesting.Path
	pathAToC *ibctesting.Path
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 3)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
	suite.chainC = suite.coordinator.GetChain(ibctesting.GetChainID(3))

	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	mockFeeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
	path.EndpointA.ChannelConfig.Version = mockFeeVersion
	path.EndpointB.ChannelConfig.Version = mockFeeVersion
	path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
	path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
	suite.path = path

	path = ibctesting.NewPath(suite.chainA, suite.chainC)
	path.EndpointA.ChannelConfig.Version = mockFeeVersion
	path.EndpointB.ChannelConfig.Version = mockFeeVersion
	path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
	path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
	suite.pathAToC = path
}

func TestKeeperTestSuite(t *testing.T) {
	testifysuite.Run(t, new(KeeperTestSuite))
}

// helper function
func lockFeeModule(chain *ibctesting.TestChain) {
	ctx := chain.GetContext()
	storeKey := chain.GetSimApp().GetKey(types.ModuleName)
	store := ctx.KVStore(storeKey)
	store.Set(types.KeyLocked(), []byte{1})
}

func (suite *KeeperTestSuite) TestEscrowAccountHasBalance() {
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)

	suite.Require().False(suite.chainA.GetSimApp().IBCFeeKeeper.EscrowAccountHasBalance(suite.chainA.GetContext(), fee.Total()))

	// set fee in escrow account
	err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), types.ModuleName, fee.Total())
	suite.Require().Nil(err)

	suite.Require().True(suite.chainA.GetSimApp().IBCFeeKeeper.EscrowAccountHasBalance(suite.chainA.GetContext(), fee.Total()))

	// increase ack fee
	fee.AckFee = fee.AckFee.Add(defaultAckFee...)
	suite.Require().False(suite.chainA.GetSimApp().IBCFeeKeeper.EscrowAccountHasBalance(suite.chainA.GetContext(), fee.Total()))
}

func (suite *KeeperTestSuite) TestGetSetPayeeAddress() {
	suite.coordinator.Setup(suite.path)

	payeeAddr, found := suite.chainA.GetSimApp().IBCFeeKeeper.GetPayeeAddress(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), suite.path.EndpointA.ChannelID)
	suite.Require().False(found)
	suite.Require().Empty(payeeAddr)

	suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
		suite.chainA.GetContext(),
		suite.chainA.SenderAccounts[0].SenderAccount.GetAddress().String(),
		suite.chainA.SenderAccounts[1].SenderAccount.GetAddress().String(),
		suite.path.EndpointA.ChannelID,
	)

	payeeAddr, found = suite.chainA.GetSimApp().IBCFeeKeeper.GetPayeeAddress(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), suite.path.EndpointA.ChannelID)
	suite.Require().True(found)
	suite.Require().Equal(suite.chainA.SenderAccounts[1].SenderAccount.GetAddress().String(), payeeAddr)
}

func (suite *KeeperTestSuite) TestFeesInEscrow() {
	suite.coordinator.Setup(suite.path)

	// escrow five fees for packet sequence 1
	packetID := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 1)
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)

	packetFee := types.NewPacketFee(fee, suite.chainA.SenderAccount.GetAddress().String(), nil)
	packetFees := []types.PacketFee{packetFee, packetFee, packetFee, packetFee, packetFee}

	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees(packetFees))

	// retrieve the fees in escrow and assert the length of PacketFees
	feesInEscrow, found := suite.chainA.GetSimApp().IBCFeeKeeper.GetFeesInEscrow(suite.chainA.GetContext(), packetID)
	suite.Require().True(found)
	suite.Require().Len(feesInEscrow.PacketFees, 5, fmt.Sprintf("expected length 5, but got %d", len(feesInEscrow.PacketFees)))

	// delete fees for packet sequence 1
	suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeesInEscrow(suite.chainA.GetContext(), packetID)
	hasFeesInEscrow := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
	suite.Require().False(hasFeesInEscrow)
}

func (suite *KeeperTestSuite) TestIsLocked() {
	ctx := suite.chainA.GetContext()
	suite.Require().False(suite.chainA.GetSimApp().IBCFeeKeeper.IsLocked(ctx))

	lockFeeModule(suite.chainA)

	suite.Require().True(suite.chainA.GetSimApp().IBCFeeKeeper.IsLocked(ctx))
}

func (suite *KeeperTestSuite) TestGetIdentifiedPacketFeesForChannel() {
	suite.coordinator.Setup(suite.path)

	// escrow a fee
	refundAcc := suite.chainA.SenderAccount.GetAddress()
	packetID1 := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 1)
	packetID2 := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 2)
	packetID5 := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 51)

	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)

	// escrow the packet fee
	packetFee := types.NewPacketFee(fee, refundAcc.String(), []string{})
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID1, types.NewPacketFees([]types.PacketFee{packetFee}))
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID2, types.NewPacketFees([]types.PacketFee{packetFee}))
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID5, types.NewPacketFees([]types.PacketFee{packetFee}))

	// set fees in escrow for packetIDs on different channel
	diffChannel := "channel-1"
	diffPacketID1 := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, diffChannel, 1)
	diffPacketID2 := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, diffChannel, 2)
	diffPacketID5 := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, diffChannel, 5)
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), diffPacketID1, types.NewPacketFees([]types.PacketFee{packetFee}))
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), diffPacketID2, types.NewPacketFees([]types.PacketFee{packetFee}))
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), diffPacketID5, types.NewPacketFees([]types.PacketFee{packetFee}))

	expectedFees := []types.IdentifiedPacketFees{
		{
			PacketId: packetID1,
			PacketFees: []types.PacketFee{
				{
					Fee:           fee,
					RefundAddress: refundAcc.String(),
					Relayers:      nil,
				},
			},
		},
		{
			PacketId: packetID2,
			PacketFees: []types.PacketFee{
				{
					Fee:           fee,
					RefundAddress: refundAcc.String(),
					Relayers:      nil,
				},
			},
		},
		{
			PacketId: packetID5,
			PacketFees: []types.PacketFee{
				{
					Fee:           fee,
					RefundAddress: refundAcc.String(),
					Relayers:      nil,
				},
			},
		},
	}

	identifiedFees := suite.chainA.GetSimApp().IBCFeeKeeper.GetIdentifiedPacketFeesForChannel(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)
	suite.Require().Len(identifiedFees, len(expectedFees))
	suite.Require().Equal(identifiedFees, expectedFees)
}

func (suite *KeeperTestSuite) TestGetAllIdentifiedPacketFees() {
	suite.coordinator.Setup(suite.path)

	// escrow a fee
	refundAcc := suite.chainA.SenderAccount.GetAddress()
	packetID := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 1)
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)

	// escrow the packet fee
	packetFee := types.NewPacketFee(fee, refundAcc.String(), []string{})
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees([]types.PacketFee{packetFee}))

	expectedFees := []types.IdentifiedPacketFees{
		{
			PacketId: packetID,
			PacketFees: []types.PacketFee{
				{
					Fee:           fee,
					RefundAddress: refundAcc.String(),
					Relayers:      nil,
				},
			},
		},
	}

	identifiedFees := suite.chainA.GetSimApp().IBCFeeKeeper.GetAllIdentifiedPacketFees(suite.chainA.GetContext())
	suite.Require().Len(identifiedFees, len(expectedFees))
	suite.Require().Equal(identifiedFees, expectedFees)
}

func (suite *KeeperTestSuite) TestGetAllFeeEnabledChannels() {
	validPortID := "ibcmoduleport"
	// set two channels enabled
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeeEnabled(suite.chainA.GetContext(), ibctesting.MockFeePort, ibctesting.FirstChannelID)
	suite.chainA.GetSimApp().IBCFeeKeeper.SetFeeEnabled(suite.chainA.GetContext(), validPortID, ibctesting.FirstChannelID)

	expectedCh := []types.FeeEnabledChannel{
		{
			PortId:    validPortID,
			ChannelId: ibctesting.FirstChannelID,
		},
		{
			PortId:    ibctesting.MockFeePort,
			ChannelId: ibctesting.FirstChannelID,
		},
	}

	ch := suite.chainA.GetSimApp().IBCFeeKeeper.GetAllFeeEnabledChannels(suite.chainA.GetContext())
	suite.Require().Len(ch, len(expectedCh))
	suite.Require().Equal(ch, expectedCh)
}

func (suite *KeeperTestSuite) TestGetAllPayees() {
	var expectedPayees []types.RegisteredPayee

	for i := 0; i < 3; i++ {
		suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
			suite.chainA.GetContext(),
			suite.chainA.SenderAccounts[i].SenderAccount.GetAddress().String(),
			suite.chainB.SenderAccounts[i].SenderAccount.GetAddress().String(),
			ibctesting.FirstChannelID,
		)

		registeredPayee := types.RegisteredPayee{
			Relayer:   suite.chainA.SenderAccounts[i].SenderAccount.GetAddress().String(),
			Payee:     suite.chainB.SenderAccounts[i].SenderAccount.GetAddress().String(),
			ChannelId: ibctesting.FirstChannelID,
		}

		expectedPayees = append(expectedPayees, registeredPayee)
	}

	registeredPayees := suite.chainA.GetSimApp().IBCFeeKeeper.GetAllPayees(suite.chainA.GetContext())
	suite.Require().Len(registeredPayees, len(expectedPayees))
	suite.Require().ElementsMatch(expectedPayees, registeredPayees)
}

func (suite *KeeperTestSuite) TestGetAllCounterpartyPayees() {
	relayerAddr := suite.chainA.SenderAccount.GetAddress().String()
	counterpartyPayee := suite.chainB.SenderAccount.GetAddress().String()

	suite.chainA.GetSimApp().IBCFeeKeeper.SetCounterpartyPayeeAddress(suite.chainA.GetContext(), relayerAddr, counterpartyPayee, ibctesting.FirstChannelID)

	expectedCounterpartyPayee := []types.RegisteredCounterpartyPayee{
		{
			Relayer:           relayerAddr,
			CounterpartyPayee: counterpartyPayee,
			ChannelId:         ibctesting.FirstChannelID,
		},
	}

	counterpartyPayeeAddr := suite.chainA.GetSimApp().IBCFeeKeeper.GetAllCounterpartyPayees(suite.chainA.GetContext())
	suite.Require().Len(counterpartyPayeeAddr, len(expectedCounterpartyPayee))
	suite.Require().Equal(counterpartyPayeeAddr, expectedCounterpartyPayee)
}

func (suite *KeeperTestSuite) TestWithICS4Wrapper() {
	suite.SetupTest()

	// test if the ics4 wrapper is the channel keeper initially
	ics4Wrapper := suite.chainA.GetSimApp().IBCFeeKeeper.GetICS4Wrapper()

	_, isChannelKeeper := ics4Wrapper.(channelkeeper.Keeper)
	suite.Require().True(isChannelKeeper)
	_, isFeeKeeper := ics4Wrapper.(keeper.Keeper)
	suite.Require().False(isFeeKeeper)

	// set the ics4 wrapper to itself (don't do this in production)
	suite.chainA.GetSimApp().IBCFeeKeeper.WithICS4Wrapper(suite.chainA.GetSimApp().IBCFeeKeeper)
	ics4Wrapper = suite.chainA.GetSimApp().IBCFeeKeeper.GetICS4Wrapper()

	_, isFeeKeeper = ics4Wrapper.(keeper.Keeper)
	suite.Require().True(isFeeKeeper)
	_, isChannelKeeper = ics4Wrapper.(channelkeeper.Keeper)
	suite.Require().False(isChannelKeeper)
}
