package fee_test

import (
	"context"
	"fmt"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	ibcfee "github.com/cosmos/ibc-go/v8/modules/apps/29-fee"
	feekeeper "github.com/cosmos/ibc-go/v8/modules/apps/29-fee/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

var (
	defaultRecvFee    = sdk.Coins{sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdkmath.NewInt(100)}}
	defaultAckFee     = sdk.Coins{sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdkmath.NewInt(200)}}
	defaultTimeoutFee = sdk.Coins{sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdkmath.NewInt(300)}}
)

// Tests OnChanOpenInit on ChainA
func (suite *FeeTestSuite) TestOnChanOpenInit() {
	testCases := []struct {
		name         string
		version      string
		expPass      bool
		isFeeEnabled bool
	}{
		{
			"success - valid fee middleware and mock version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version})),
			true,
			true,
		},
		{
			"success - fee version not included, only perform mock logic",
			ibcmock.Version,
			true,
			false,
		},
		{
			"invalid fee middleware version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: "invalid-ics29-1", AppVersion: ibcmock.Version})),
			false,
			false,
		},
		{
			"invalid mock version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: "invalid-mock-version"})),
			false,
			false,
		},
		{
			"mock version not wrapped",
			types.Version,
			false,
			false,
		},
		{
			"passing an empty string returns default version",
			"",
			true,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			// reset suite
			suite.SetupTest()
			suite.coordinator.SetupConnections(suite.path)

			// setup mock callback
			suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanOpenInit = func(ctx context.Context, order channeltypes.Order, connectionHops []string,
				portID, channelID string, chanCap *capabilitytypes.Capability,
				counterparty channeltypes.Counterparty, version string,
			) (string, error) {
				if version != ibcmock.Version {
					return "", fmt.Errorf("incorrect mock version")
				}
				return ibcmock.Version, nil
			}

			suite.path.EndpointA.ChannelID = ibctesting.FirstChannelID

			counterparty := channeltypes.NewCounterparty(suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID)
			channel := &channeltypes.Channel{
				State:          channeltypes.INIT,
				Ordering:       channeltypes.UNORDERED,
				Counterparty:   counterparty,
				ConnectionHops: []string{suite.path.EndpointA.ConnectionID},
				Version:        tc.version,
			}

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			chanCap, err := suite.chainA.App.GetScopedIBCKeeper().NewCapability(suite.chainA.GetContext(), host.ChannelCapabilityPath(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID))
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			version, err := cbs.OnChanOpenInit(suite.chainA.GetContext(), channel.Ordering, channel.GetConnectionHops(),
				suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, chanCap, counterparty, channel.Version)

			if tc.expPass {
				// check if the channel is fee enabled. If so version string should include metaData
				if tc.isFeeEnabled {
					versionMetadata := types.Metadata{
						FeeVersion: types.Version,
						AppVersion: ibcmock.Version,
					}

					versionBytes, err := types.ModuleCdc.MarshalJSON(&versionMetadata)
					suite.Require().NoError(err)

					suite.Require().Equal(version, string(versionBytes))
				} else {
					suite.Require().Equal(ibcmock.Version, version)
				}

				suite.Require().NoError(err, "unexpected error from version: %s", tc.version)
			} else {
				suite.Require().Error(err, "error not returned for version: %s", tc.version)
				suite.Require().Equal("", version)
			}
		})
	}
}

// Tests OnChanOpenTry on ChainA
func (suite *FeeTestSuite) TestOnChanOpenTry() {
	testCases := []struct {
		name      string
		cpVersion string
		expPass   bool
	}{
		{
			"success - valid fee middleware version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version})),
			true,
		},
		{
			"success - valid mock version",
			ibcmock.Version,
			true,
		},
		{
			"invalid fee middleware version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: "invalid-ics29-1", AppVersion: ibcmock.Version})),
			false,
		},
		{
			"invalid mock version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: "invalid-mock-version"})),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			// reset suite
			suite.SetupTest()
			suite.coordinator.SetupConnections(suite.path)
			err := suite.path.EndpointB.ChanOpenInit()
			suite.Require().NoError(err)

			// setup mock callback
			suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanOpenTry = func(ctx context.Context, order channeltypes.Order, connectionHops []string,
				portID, channelID string, chanCap *capabilitytypes.Capability,
				counterparty channeltypes.Counterparty, counterpartyVersion string,
			) (string, error) {
				if counterpartyVersion != ibcmock.Version {
					return "", fmt.Errorf("incorrect mock version")
				}
				return ibcmock.Version, nil
			}

			var (
				chanCap *capabilitytypes.Capability
				ok      bool
			)

			chanCap, err = suite.chainA.App.GetScopedIBCKeeper().NewCapability(suite.chainA.GetContext(), host.ChannelCapabilityPath(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID))
			suite.Require().NoError(err)

			suite.path.EndpointA.ChannelID = ibctesting.FirstChannelID

			counterparty := channeltypes.NewCounterparty(suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID)
			channel := &channeltypes.Channel{
				State:          channeltypes.INIT,
				Ordering:       channeltypes.UNORDERED,
				Counterparty:   counterparty,
				ConnectionHops: []string{suite.path.EndpointA.ConnectionID},
				Version:        tc.cpVersion,
			}

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			_, err = cbs.OnChanOpenTry(suite.chainA.GetContext(), channel.Ordering, channel.GetConnectionHops(),
				suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, chanCap, counterparty, tc.cpVersion)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// Tests OnChanOpenAck on ChainA
func (suite *FeeTestSuite) TestOnChanOpenAck() {
	testCases := []struct {
		name      string
		cpVersion string
		malleate  func(suite *FeeTestSuite)
		expPass   bool
	}{
		{
			"success",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version})),
			func(suite *FeeTestSuite) {},
			true,
		},
		{
			"invalid fee version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: "invalid-ics29-1", AppVersion: ibcmock.Version})),
			func(suite *FeeTestSuite) {},
			false,
		},
		{
			"invalid mock version",
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: "invalid-mock-version"})),
			func(suite *FeeTestSuite) {},
			false,
		},
		{
			"invalid version fails to unmarshal metadata",
			ibctesting.InvalidID,
			func(suite *FeeTestSuite) {},
			false,
		},
		{
			"previous INIT set without fee, however counterparty set fee version", // note this can only happen with incompetent or malicious counterparty chain
			string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version})),
			func(suite *FeeTestSuite) {
				// do the first steps without fee version, then pass the fee version as counterparty version in ChanOpenACK
				suite.path.EndpointA.ChannelConfig.Version = ibcmock.Version
				suite.path.EndpointB.ChannelConfig.Version = ibcmock.Version
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			suite.coordinator.SetupConnections(suite.path)

			// setup mock callback
			suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanOpenAck = func(
				ctx context.Context, portID, channelID string, counterpartyChannelID string, counterpartyVersion string,
			) error {
				if counterpartyVersion != ibcmock.Version {
					return fmt.Errorf("incorrect mock version")
				}
				return nil
			}

			// malleate test case
			tc.malleate(suite)

			err := suite.path.EndpointA.ChanOpenInit()
			suite.Require().NoError(err)
			err = suite.path.EndpointB.ChanOpenTry()
			suite.Require().NoError(err)

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			err = cbs.OnChanOpenAck(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, suite.path.EndpointA.Counterparty.ChannelID, tc.cpVersion)
			if tc.expPass {
				suite.Require().NoError(err, "unexpected error for case: %s", tc.name)
			} else {
				suite.Require().Error(err, "%s expected error but returned none", tc.name)
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnChanCloseInit() {
	var (
		refundAcc sdk.AccAddress
		fee       types.Fee
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success", func() {}, true,
		},
		{
			"application callback fails", func() {
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanCloseInit = func(
					ctx context.Context, portID, channelID string,
				) error {
					return fmt.Errorf("application callback fails")
				}
			}, false,
		},
		{
			"RefundFeesOnChannelClosure continues - invalid refund address", func() {
				// store the fee in state & update escrow account balance
				packetID := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, uint64(1))
				packetFees := types.NewPacketFees([]types.PacketFee{types.NewPacketFee(fee, "invalid refund address", nil)})

				suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, packetFees)
				err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), refundAcc, types.ModuleName, fee.Total())
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"fee module locked", func() {
				lockFeeModule(suite.chainA)
			},
			false,
		},
		{
			"fee module is not enabled", func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeeEnabled(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			suite.coordinator.Setup(suite.path) // setup channel

			packetID := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 1)
			fee = types.Fee{
				RecvFee:    defaultRecvFee,
				AckFee:     defaultAckFee,
				TimeoutFee: defaultTimeoutFee,
			}

			refundAcc = suite.chainA.SenderAccount.GetAddress()
			packetFee := types.NewPacketFee(fee, refundAcc.String(), []string{})

			suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees([]types.PacketFee{packetFee}))
			err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), refundAcc, types.ModuleName, fee.Total())
			suite.Require().NoError(err)

			tc.malleate()

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			err = cbs.OnChanCloseInit(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// Tests OnChanCloseConfirm on chainA
func (suite *FeeTestSuite) TestOnChanCloseConfirm() {
	var (
		refundAcc sdk.AccAddress
		fee       types.Fee
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success", func() {}, true,
		},
		{
			"application callback fails", func() {
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanCloseConfirm = func(
					ctx context.Context, portID, channelID string,
				) error {
					return fmt.Errorf("application callback fails")
				}
			}, false,
		},
		{
			"RefundChannelFeesOnClosure continues - refund address is invalid", func() {
				// store the fee in state & update escrow account balance
				packetID := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, uint64(1))
				packetFees := types.NewPacketFees([]types.PacketFee{types.NewPacketFee(fee, "invalid refund address", nil)})

				suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, packetFees)
				err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), refundAcc, types.ModuleName, fee.Total())
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"fee module locked", func() {
				lockFeeModule(suite.chainA)
			},
			false,
		},
		{
			"fee module is not enabled", func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeeEnabled(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()
			suite.coordinator.Setup(suite.path) // setup channel

			packetID := channeltypes.NewPacketID(suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID, 1)
			fee = types.Fee{
				RecvFee:    defaultRecvFee,
				AckFee:     defaultAckFee,
				TimeoutFee: defaultTimeoutFee,
			}

			refundAcc = suite.chainA.SenderAccount.GetAddress()
			packetFee := types.NewPacketFee(fee, refundAcc.String(), []string{})

			suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees([]types.PacketFee{packetFee}))
			err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), refundAcc, types.ModuleName, fee.Total())
			suite.Require().NoError(err)

			tc.malleate()

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			err = cbs.OnChanCloseConfirm(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnRecvPacket() {
	testCases := []struct {
		name     string
		malleate func()
		// forwardRelayer bool indicates if there is a forwardRelayer address set
		forwardRelayer bool
		feeEnabled     bool
	}{
		{
			"success",
			func() {},
			true,
			true,
		},
		{
			"async write acknowledgement: ack is nil",
			func() {
				// setup mock callback
				suite.chainB.GetSimApp().FeeMockModule.IBCApp.OnRecvPacket = func(
					ctx context.Context,
					packet channeltypes.Packet,
					relayer sdk.AccAddress,
				) exported.Acknowledgement {
					return nil
				}
			},
			true,
			true,
		},
		{
			"fee not enabled",
			func() {
				suite.chainB.GetSimApp().IBCFeeKeeper.DeleteFeeEnabled(suite.chainB.GetContext(), suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID)
			},
			true,
			false,
		},
		{
			"forward address is not found",
			func() {
				suite.chainB.GetSimApp().IBCFeeKeeper.SetCounterpartyPayeeAddress(suite.chainB.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), "", suite.path.EndpointB.ChannelID)
			},
			false,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			// setup pathAToC (chainA -> chainC) first in order to have different channel IDs for chainA & chainB
			suite.coordinator.Setup(suite.pathAToC)
			// setup path for chainA -> chainB
			suite.coordinator.Setup(suite.path)

			suite.chainB.GetSimApp().IBCFeeKeeper.SetFeeEnabled(suite.chainB.GetContext(), suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID)

			packet := suite.CreateMockPacket()

			// set up module and callbacks
			module, _, err := suite.chainB.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainB.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainB.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			suite.chainB.GetSimApp().IBCFeeKeeper.SetCounterpartyPayeeAddress(suite.chainB.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), suite.path.EndpointB.ChannelID)

			// malleate test case
			tc.malleate()

			result := cbs.OnRecvPacket(suite.chainB.GetContext(), packet, suite.chainA.SenderAccount.GetAddress())

			switch {
			case tc.name == "success":
				forwardAddr, _ := suite.chainB.GetSimApp().IBCFeeKeeper.GetCounterpartyPayeeAddress(suite.chainB.GetContext(), suite.chainA.SenderAccount.GetAddress().String(), suite.path.EndpointB.ChannelID)

				expectedAck := types.IncentivizedAcknowledgement{
					AppAcknowledgement:    ibcmock.MockAcknowledgement.Acknowledgement(),
					ForwardRelayerAddress: forwardAddr,
					UnderlyingAppSuccess:  true,
				}
				suite.Require().Equal(expectedAck, result)

			case !tc.feeEnabled:
				suite.Require().Equal(ibcmock.MockAcknowledgement, result)

			case tc.forwardRelayer && result == nil:
				suite.Require().Equal(nil, result)
				packetID := channeltypes.NewPacketID(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

				// retrieve the forward relayer that was stored in `onRecvPacket`
				relayer, _ := suite.chainB.GetSimApp().IBCFeeKeeper.GetRelayerAddressForAsyncAck(suite.chainB.GetContext(), packetID)
				suite.Require().Equal(relayer, suite.chainA.SenderAccount.GetAddress().String())

			case !tc.forwardRelayer:
				expectedAck := types.IncentivizedAcknowledgement{
					AppAcknowledgement:    ibcmock.MockAcknowledgement.Acknowledgement(),
					ForwardRelayerAddress: "",
					UnderlyingAppSuccess:  true,
				}
				suite.Require().Equal(expectedAck, result)
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnAcknowledgementPacket() {
	var (
		ack                 []byte
		packetID            channeltypes.PacketId
		packetFee           types.PacketFee
		refundAddr          sdk.AccAddress
		relayerAddr         sdk.AccAddress
		escrowAmount        sdk.Coins
		initialRefundAccBal sdk.Coins
		expRefundAccBalance sdk.Coins
		expPayeeAccBalance  sdk.Coins
	)

	testCases := []struct {
		name      string
		malleate  func()
		expPass   bool
		expResult func()
	}{
		{
			"success",
			func() {
				// retrieve the relayer acc balance and add the expected recv and ack fees
				relayerAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom))
				expPayeeAccBalance = relayerAccBalance.Add(packetFee.Fee.RecvFee...).Add(packetFee.Fee.AckFee...)
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				relayerAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(relayerAccBalance))

				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(initialRefundAccBal, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"success: some refunds",
			func() {
				// set timeout_fee > recv_fee + ack_fee
				packetFee.Fee.TimeoutFee = packetFee.Fee.Total().Add(sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100)))...)

				escrowAmount = packetFee.Fee.Total()

				// retrieve the relayer acc balance and add the expected recv and ack fees
				relayerAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom))
				expPayeeAccBalance = relayerAccBalance.Add(packetFee.Fee.RecvFee...).Add(packetFee.Fee.AckFee...)
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				relayerAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(relayerAccBalance))

				// expect the correct refunds
				refundCoins := packetFee.Fee.Total().Sub(packetFee.Fee.RecvFee...).Sub(packetFee.Fee.AckFee...)
				expRefundAccBalance = initialRefundAccBal.Add(refundCoins...)
				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expRefundAccBalance, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"success: with registered payee address",
			func() {
				payeeAddr := suite.chainA.SenderAccounts[2].SenderAccount.GetAddress()
				suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
					suite.chainA.GetContext(),
					suite.chainA.SenderAccount.GetAddress().String(),
					payeeAddr.String(),
					suite.path.EndpointA.ChannelID,
				)

				// reassign ack.ForwardRelayerAddress to the registered payee address
				ack = types.NewIncentivizedAcknowledgement(payeeAddr.String(), ibcmock.MockAcknowledgement.Acknowledgement(), true).Acknowledgement()

				// retrieve the payee acc balance and add the expected recv and ack fees
				payeeAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), payeeAddr, sdk.DefaultBondDenom))
				expPayeeAccBalance = payeeAccBalance.Add(packetFee.Fee.RecvFee...).Add(packetFee.Fee.AckFee...)
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				payeeAddr := suite.chainA.SenderAccounts[2].SenderAccount.GetAddress()
				payeeAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), payeeAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(payeeAccBalance))

				// expect zero refunds
				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(initialRefundAccBal, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"success: no op without a packet fee",
			func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeesInEscrow(suite.chainA.GetContext(), packetID)

				ack = types.IncentivizedAcknowledgement{
					AppAcknowledgement:    ibcmock.MockAcknowledgement.Acknowledgement(),
					ForwardRelayerAddress: "",
				}.Acknowledgement()
			},
			true,
			func() {
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)
			},
		},
		{
			"success: channel is not fee enabled",
			func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeeEnabled(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)
				ack = ibcmock.MockAcknowledgement.Acknowledgement()
			},
			true,
			func() {},
		},
		{
			"success: fee module is disabled, skip fee logic",
			func() {
				lockFeeModule(suite.chainA)
			},
			true,
			func() {
				suite.Require().Equal(true, suite.chainA.GetSimApp().IBCFeeKeeper.IsLocked(suite.chainA.GetContext()))
			},
		},
		{
			"success: fail to distribute recv fee (blocked address), returned to refund account",
			func() {
				blockedAddr := suite.chainA.GetSimApp().AuthKeeper.GetModuleAccount(suite.chainA.GetContext(), transfertypes.ModuleName).GetAddress()

				// reassign ack.ForwardRelayerAddress to a blocked address
				ack = types.NewIncentivizedAcknowledgement(blockedAddr.String(), ibcmock.MockAcknowledgement.Acknowledgement(), true).Acknowledgement()

				// retrieve the relayer acc balance and add the expected ack fees
				relayerAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom))
				expPayeeAccBalance = relayerAccBalance.Add(packetFee.Fee.AckFee...)
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				relayerAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(relayerAccBalance))

				// expect only recv fee to be refunded
				expRefundAccBalance = initialRefundAccBal.Add(packetFee.Fee.RecvFee...)
				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expRefundAccBalance, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"fail: fee distribution fails and fee module is locked when escrow account does not have sufficient funds",
			func() {
				escrowAmount = sdk.NewCoins()
			},
			true,
			func() {
				suite.Require().Equal(true, suite.chainA.GetSimApp().IBCFeeKeeper.IsLocked(suite.chainA.GetContext()))
			},
		},
		{
			"ack wrong format",
			func() {
				ack = []byte("unsupported acknowledgement format")
			},
			false,
			func() {},
		},
		{
			"invalid registered payee address",
			func() {
				payeeAddr := "invalid-address"
				suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
					suite.chainA.GetContext(),
					suite.chainA.SenderAccount.GetAddress().String(),
					payeeAddr,
					suite.path.EndpointA.ChannelID,
				)
			},
			false,
			func() {},
		},
		{
			"application callback fails",
			func() {
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnAcknowledgementPacket = func(_ context.Context, _ channeltypes.Packet, _ []byte, _ sdk.AccAddress) error {
					return fmt.Errorf("mock fee app callback fails")
				}
			},
			false,
			func() {},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			suite.coordinator.Setup(suite.path)

			relayerAddr = suite.chainA.SenderAccounts[0].SenderAccount.GetAddress()
			refundAddr = suite.chainA.SenderAccounts[1].SenderAccount.GetAddress()

			packet := suite.CreateMockPacket()
			packetID = channeltypes.NewPacketID(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
			packetFee = types.NewPacketFee(types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee), refundAddr.String(), nil)
			escrowAmount = packetFee.Fee.Total()

			ack = types.NewIncentivizedAcknowledgement(relayerAddr.String(), ibcmock.MockAcknowledgement.Acknowledgement(), true).Acknowledgement()

			tc.malleate() // malleate mutates test data

			suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees([]types.PacketFee{packetFee}))

			err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), refundAddr, types.ModuleName, escrowAmount)
			suite.Require().NoError(err)

			initialRefundAccBal = sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom))

			// retrieve module callbacks
			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			err = cbs.OnAcknowledgementPacket(suite.chainA.GetContext(), packet, ack, relayerAddr)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}

			tc.expResult()
		})
	}
}

func (suite *FeeTestSuite) TestOnTimeoutPacket() {
	var (
		packetID             channeltypes.PacketId
		packetFee            types.PacketFee
		refundAddr           sdk.AccAddress
		relayerAddr          sdk.AccAddress
		escrowAmount         sdk.Coins
		initialRelayerAccBal sdk.Coins
		expRefundAccBalance  sdk.Coins
		expPayeeAccBalance   sdk.Coins
	)

	testCases := []struct {
		name      string
		malleate  func()
		expPass   bool
		expResult func()
	}{
		{
			"success: no refund",
			func() {
				// expect zero refunds
				refundAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom))
				expRefundAccBalance = refundAccBalance
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				expPayeeAccBalance = initialRelayerAccBal.Add(packetFee.Fee.TimeoutFee...)
				relayerAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(relayerAccBalance))

				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expRefundAccBalance, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"success: refund (recv_fee + ack_fee) - timeout_fee",
			func() {
				// set recv_fee + ack_fee > timeout_fee
				packetFee.Fee.RecvFee = packetFee.Fee.Total().Add(sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100)))...)

				escrowAmount = packetFee.Fee.Total()

				// retrieve the refund acc balance and add the expected recv and ack fees
				refundCoins := packetFee.Fee.Total().Sub(packetFee.Fee.TimeoutFee...)
				refundAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom))
				expRefundAccBalance = refundAccBalance.Add(refundCoins...)
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				expPayeeAccBalance = initialRelayerAccBal.Add(packetFee.Fee.TimeoutFee...)
				relayerAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(relayerAccBalance))

				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expRefundAccBalance, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"success: with registered payee address",
			func() {
				payeeAddr := suite.chainA.SenderAccounts[2].SenderAccount.GetAddress()
				suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
					suite.chainA.GetContext(),
					suite.chainA.SenderAccount.GetAddress().String(),
					payeeAddr.String(),
					suite.path.EndpointA.ChannelID,
				)

				// retrieve the relayer acc balance and add the expected timeout fees
				payeeAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), payeeAddr, sdk.DefaultBondDenom))
				expPayeeAccBalance = payeeAccBalance.Add(packetFee.Fee.TimeoutFee...)

				// expect zero refunds
				refundAccBalance := sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom))
				expRefundAccBalance = refundAccBalance
			},
			true,
			func() {
				// assert that the packet fees have been distributed
				found := suite.chainA.GetSimApp().IBCFeeKeeper.HasFeesInEscrow(suite.chainA.GetContext(), packetID)
				suite.Require().False(found)

				payeeAddr := suite.chainA.SenderAccounts[2].SenderAccount.GetAddress()
				payeeAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), payeeAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expPayeeAccBalance, sdk.NewCoins(payeeAccBalance))

				refundAccBalance := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), refundAddr, sdk.DefaultBondDenom)
				suite.Require().Equal(expRefundAccBalance, sdk.NewCoins(refundAccBalance))
			},
		},
		{
			"success: channel is not fee enabled",
			func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeeEnabled(suite.chainA.GetContext(), suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID)
			},
			true,
			func() {},
		},
		{
			"success: fee module is disabled, skip fee logic",
			func() {
				lockFeeModule(suite.chainA)
			},
			true,
			func() {
				suite.Require().Equal(true, suite.chainA.GetSimApp().IBCFeeKeeper.IsLocked(suite.chainA.GetContext()))
			},
		},
		{
			"success: no op if identified packet fee doesn't exist",
			func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeesInEscrow(suite.chainA.GetContext(), packetID)
			},
			true,
			func() {},
		},
		{
			"success: fail to distribute timeout fee (blocked address), returned to refund account",
			func() {
				relayerAddr = suite.chainA.GetSimApp().AuthKeeper.GetModuleAccount(suite.chainA.GetContext(), transfertypes.ModuleName).GetAddress()
			},
			true,
			func() {},
		},
		{
			"fee distribution fails and fee module is locked when escrow account does not have sufficient funds",
			func() {
				escrowAmount = sdk.NewCoins()
			},
			true,
			func() {
				suite.Require().Equal(true, suite.chainA.GetSimApp().IBCFeeKeeper.IsLocked(suite.chainA.GetContext()))
			},
		},
		{
			"invalid registered payee address",
			func() {
				payeeAddr := "invalid-address"
				suite.chainA.GetSimApp().IBCFeeKeeper.SetPayeeAddress(
					suite.chainA.GetContext(),
					suite.chainA.SenderAccount.GetAddress().String(),
					payeeAddr,
					suite.path.EndpointA.ChannelID,
				)
			},
			false,
			func() {},
		},
		{
			"application callback fails",
			func() {
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnTimeoutPacket = func(_ context.Context, _ channeltypes.Packet, _ sdk.AccAddress) error {
					return fmt.Errorf("mock fee app callback fails")
				}
			},
			false,
			func() {},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			suite.coordinator.Setup(suite.path)

			relayerAddr = suite.chainA.SenderAccounts[0].SenderAccount.GetAddress()
			refundAddr = suite.chainA.SenderAccounts[1].SenderAccount.GetAddress()

			packet := suite.CreateMockPacket()
			packetID = channeltypes.NewPacketID(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
			packetFee = types.NewPacketFee(types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee), refundAddr.String(), nil)
			escrowAmount = packetFee.Fee.Total()

			tc.malleate() // malleate mutates test data

			suite.chainA.GetSimApp().IBCFeeKeeper.SetFeesInEscrow(suite.chainA.GetContext(), packetID, types.NewPacketFees([]types.PacketFee{packetFee}))
			err := suite.chainA.GetSimApp().BankKeeper.SendCoinsFromAccountToModule(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), types.ModuleName, escrowAmount)
			suite.Require().NoError(err)

			initialRelayerAccBal = sdk.NewCoins(suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), relayerAddr, sdk.DefaultBondDenom))

			// retrieve module callbacks
			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			err = cbs.OnTimeoutPacket(suite.chainA.GetContext(), packet, relayerAddr)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}

			tc.expResult()
		})
	}
}

func (suite *FeeTestSuite) TestOnChanUpgradeInit() {
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
		{
			"success with downgraded version",
			func() {
				// create a new path using a fee enabled channel and downgrade it to disable fees
				path = ibctesting.NewPath(suite.chainA, suite.chainB)

				mockFeeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
				path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointA.ChannelConfig.Version = mockFeeVersion
				path.EndpointB.ChannelConfig.Version = mockFeeVersion

				upgradeVersion := ibcmock.Version
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

				suite.coordinator.Setup(path)
			},
			nil,
		},
		{
			"invalid upgrade version",
			func() {
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibctesting.InvalidID
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibctesting.InvalidID

				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeInit = func(_ context.Context, _, _ string, _ channeltypes.Order, _ []string, _ string) (string, error) {
					// intentionally force the error here so we can assert that a passthrough occurs when fees should not be enabled for this channel
					return "", ibcmock.MockApplicationCallbackError
				}
			},
			ibcmock.MockApplicationCallbackError,
		},
		{
			"invalid fee version",
			func() {
				upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: ibctesting.InvalidID, AppVersion: ibcmock.Version}))
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
			},
			types.ErrInvalidVersion,
		},
		{
			"underlying app callback returns error",
			func() {
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeInit = func(_ context.Context, _, _ string, _ channeltypes.Order, _ []string, _ string) (string, error) {
					return "", ibcmock.MockApplicationCallbackError
				}
			},
			ibcmock.MockApplicationCallbackError,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			// configure the initial path to create an unincentivized mock channel
			path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointA.ChannelConfig.Version = ibcmock.Version
			path.EndpointB.ChannelConfig.Version = ibcmock.Version

			suite.coordinator.Setup(path)

			// configure the channel upgrade version to enabled ics29 fee middleware
			upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

			tc.malleate()

			err := path.EndpointA.ChanUpgradeInit()

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expError.Error())
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnChanUpgradeTry() {
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
		{
			"success disable fees",
			func() {
				// create a new path using a fee enabled channel and downgrade it to disable fees
				path = ibctesting.NewPath(suite.chainA, suite.chainB)

				mockFeeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
				path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointA.ChannelConfig.Version = mockFeeVersion
				path.EndpointB.ChannelConfig.Version = mockFeeVersion

				upgradeVersion := ibcmock.Version
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

				suite.coordinator.Setup(path)
				err := path.EndpointA.ChanUpgradeInit()
				suite.Require().NoError(err)
			},
			nil,
		},
		{
			"invalid upgrade version",
			func() {
				counterpartyUpgrade := path.EndpointA.GetChannelUpgrade()
				counterpartyUpgrade.Fields.Version = ibctesting.InvalidID
				path.EndpointA.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainA)

				// intentionally force the error here so we can assert that a passthrough occurs when fees should not be enabled for this channel
				suite.chainB.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeTry = func(_ context.Context, _, _ string, _ channeltypes.Order, _ []string, _ string) (string, error) {
					return "", ibcmock.MockApplicationCallbackError
				}
			},
			ibcmock.MockApplicationCallbackError,
		},
		{
			"invalid fee version",
			func() {
				upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: ibctesting.InvalidID, AppVersion: ibcmock.Version}))

				counterpartyUpgrade := path.EndpointA.GetChannelUpgrade()
				counterpartyUpgrade.Fields.Version = upgradeVersion
				path.EndpointA.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainA)
			},
			types.ErrInvalidVersion,
		},
		{
			"underlying app callback returns error",
			func() {
				suite.chainB.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeTry = func(_ context.Context, _, _ string, _ channeltypes.Order, _ []string, _ string) (string, error) {
					return "", ibcmock.MockApplicationCallbackError
				}
			},
			ibcmock.MockApplicationCallbackError,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			// configure the initial path to create an unincentivized mock channel
			path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointA.ChannelConfig.Version = ibcmock.Version
			path.EndpointB.ChannelConfig.Version = ibcmock.Version

			suite.coordinator.Setup(path)

			// configure the channel upgrade version to enabled ics29 fee middleware
			upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			tc.malleate()

			err = path.EndpointB.ChanUpgradeTry()

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expError.Error())
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnChanUpgradeAck() {
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
		{
			"success with fee middleware disabled",
			func() {
				suite.chainA.GetSimApp().IBCFeeKeeper.DeleteFeeEnabled(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			},
			nil,
		},
		{
			"invalid upgrade version",
			func() {
				counterpartyUpgrade := path.EndpointB.GetChannelUpgrade()
				counterpartyUpgrade.Fields.Version = ibctesting.InvalidID
				path.EndpointB.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainB)

				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeAck = func(_ context.Context, _, _, _ string) error {
					return types.ErrInvalidVersion
				}
			},
			types.ErrInvalidVersion,
		},
		{
			"invalid fee version",
			func() {
				upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: ibctesting.InvalidID, AppVersion: ibcmock.Version}))

				counterpartyUpgrade := path.EndpointB.GetChannelUpgrade()
				counterpartyUpgrade.Fields.Version = upgradeVersion
				path.EndpointB.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainB)
			},
			types.ErrInvalidVersion,
		},
		{
			"underlying app callback returns error",
			func() {
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeAck = func(_ context.Context, _, _, _ string) error {
					return ibcmock.MockApplicationCallbackError
				}
			},
			ibcmock.MockApplicationCallbackError,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			// configure the initial path to create an unincentivized mock channel
			path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointA.ChannelConfig.Version = ibcmock.Version
			path.EndpointB.ChannelConfig.Version = ibcmock.Version

			suite.coordinator.Setup(path)

			// configure the channel upgrade version to enabled ics29 fee middleware
			upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			tc.malleate()

			counterpartyUpgrade := path.EndpointB.GetChannelUpgrade()

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			app, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			cbs, ok := app.(porttypes.UpgradableModule)
			suite.Require().True(ok)

			err = cbs.OnChanUpgradeAck(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, counterpartyUpgrade.Fields.Version)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *FeeTestSuite) TestOnChanUpgradeOpen() {
	var path *ibctesting.Path

	testCases := []struct {
		name          string
		malleate      func()
		expFeeEnabled bool
	}{
		{
			"success: enable fees",
			func() {
				// Assert in callback that correct upgrade information is passed
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeOpen = func(_ context.Context, portID, channelID string, order channeltypes.Order, connectionHops []string, version string) {
					suite.Require().Equal(path.EndpointA.ChannelConfig.PortID, portID)
					suite.Require().Equal(path.EndpointA.ChannelID, channelID)
					suite.Require().Equal(channeltypes.UNORDERED, order)
					suite.Require().Equal([]string{path.EndpointA.ConnectionID}, connectionHops)
					suite.Require().Equal(ibcmock.Version, version)
				}
			},
			true,
		},
		{
			"success: disable fees",
			func() {
				// create a new path using a fee enabled channel and downgrade it to disable fees
				path = ibctesting.NewPath(suite.chainA, suite.chainB)

				mockFeeVersion := &types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}
				mockFeeVersionBz := string(types.ModuleCdc.MustMarshalJSON(mockFeeVersion))
				path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointA.ChannelConfig.Version = mockFeeVersionBz
				path.EndpointB.ChannelConfig.Version = mockFeeVersionBz

				upgradeVersion := ibcmock.Version
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = upgradeVersion

				suite.coordinator.Setup(path)

				// Assert in callback that correct version is passed
				suite.chainA.GetSimApp().FeeMockModule.IBCApp.OnChanUpgradeOpen = func(_ context.Context, portID, channelID string, order channeltypes.Order, connectionHops []string, version string) {
					suite.Require().Equal(path.EndpointA.ChannelConfig.PortID, portID)
					suite.Require().Equal(path.EndpointA.ChannelID, channelID)
					suite.Require().Equal(channeltypes.UNORDERED, order)
					suite.Require().Equal([]string{path.EndpointA.ConnectionID}, connectionHops)
					suite.Require().Equal(mockFeeVersion.AppVersion, version)
				}
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			// configure the initial path to create an unincentivized mock channel
			path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
			path.EndpointA.ChannelConfig.Version = ibcmock.Version
			path.EndpointB.ChannelConfig.Version = ibcmock.Version

			suite.coordinator.Setup(path)

			// configure the channel upgrade version to enabled ics29 fee middleware
			upgradeVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: ibcmock.Version}))
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

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			app, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			cbs, ok := app.(porttypes.UpgradableModule)
			suite.Require().True(ok)

			upgrade := path.EndpointA.GetChannelUpgrade()
			cbs.OnChanUpgradeOpen(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgrade.Fields.Ordering, upgrade.Fields.ConnectionHops, upgrade.Fields.Version)

			isFeeEnabled := suite.chainA.GetSimApp().IBCFeeKeeper.IsFeeEnabled(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			if tc.expFeeEnabled {
				suite.Require().True(isFeeEnabled)
			} else {
				suite.Require().False(isFeeEnabled)
			}
		})
	}
}

func (suite *FeeTestSuite) TestGetAppVersion() {
	var (
		portID        string
		channelID     string
		expAppVersion string
	)
	testCases := []struct {
		name     string
		malleate func()
		expFound bool
	}{
		{
			"success for fee enabled channel",
			func() {
				expAppVersion = ibcmock.Version
			},
			true,
		},
		{
			"success for non fee enabled channel",
			func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				path.EndpointA.ChannelConfig.PortID = ibctesting.MockFeePort
				path.EndpointB.ChannelConfig.PortID = ibctesting.MockFeePort
				// by default a new path uses a non fee channel
				suite.coordinator.Setup(path)
				portID = path.EndpointA.ChannelConfig.PortID
				channelID = path.EndpointA.ChannelID

				expAppVersion = ibcmock.Version
			},
			true,
		},
		{
			"channel does not exist",
			func() {
				channelID = "does not exist"
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			suite.coordinator.Setup(suite.path)

			portID = suite.path.EndpointA.ChannelConfig.PortID
			channelID = suite.path.EndpointA.ChannelID

			// malleate test case
			tc.malleate()

			module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
			suite.Require().NoError(err)

			cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
			suite.Require().True(ok)

			feeModule := cbs.(porttypes.ICS4Wrapper)
			appVersion, found := feeModule.GetAppVersion(suite.chainA.GetContext(), portID, channelID)

			if tc.expFound {
				suite.Require().True(found)
				suite.Require().Equal(expAppVersion, appVersion)
			} else {
				suite.Require().False(found)
				suite.Require().Empty(appVersion)
			}
		})
	}
}

func (suite *FeeTestSuite) TestPacketDataUnmarshalerInterface() {
	module, _, err := suite.chainA.App.GetIBCKeeper().PortKeeper.LookupModuleByPort(suite.chainA.GetContext(), ibctesting.MockFeePort)
	suite.Require().NoError(err)

	cbs, ok := suite.chainA.App.GetIBCKeeper().Router.GetRoute(module)
	suite.Require().True(ok)

	feeModule, ok := cbs.(porttypes.PacketDataUnmarshaler)
	suite.Require().True(ok)

	packetData, err := feeModule.UnmarshalPacketData(ibcmock.MockPacketData)
	suite.Require().NoError(err)
	suite.Require().Equal(ibcmock.MockPacketData, packetData)
}

func (suite *FeeTestSuite) TestPacketDataUnmarshalerInterfaceError() {
	// test the case when the underlying application cannot be casted to a PacketDataUnmarshaler
	mockFeeMiddleware := ibcfee.NewIBCMiddleware(nil, feekeeper.Keeper{})

	_, err := mockFeeMiddleware.UnmarshalPacketData(ibcmock.MockPacketData)
	expError := errorsmod.Wrapf(types.ErrUnsupportedAction, "underlying app does not implement %T", (*porttypes.PacketDataUnmarshaler)(nil))
	suite.Require().ErrorIs(err, expError)
}
