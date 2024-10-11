package keeper_test

import (
	"github.com/cosmos/gogoproto/proto"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestSendTx() {
	var (
		path             *ibctesting.Path
		packetData       icatypes.InterchainAccountPacketData
		timeoutTimestamp uint64
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {
				interchainAccountAddr, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetInterchainAccountAddress(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				msg := &banktypes.MsgSend{
					FromAddress: interchainAccountAddr,
					ToAddress:   suite.chainB.SenderAccount.GetAddress().String(),
					Amount:      sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))),
				}

				data, err := icatypes.SerializeCosmosTx(suite.chainB.GetSimApp().AppCodec(), []proto.Message{msg}, icatypes.EncodingProtobuf)
				suite.Require().NoError(err)

				packetData = icatypes.InterchainAccountPacketData{
					Type: icatypes.EXECUTE_TX,
					Data: data,
				}
			},
			true,
		},
		{
			"success with multiple sdk.Msg",
			func() {
				interchainAccountAddr, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetInterchainAccountAddress(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				msgsBankSend := []proto.Message{
					&banktypes.MsgSend{
						FromAddress: interchainAccountAddr,
						ToAddress:   suite.chainB.SenderAccount.GetAddress().String(),
						Amount:      sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))),
					},
					&banktypes.MsgSend{
						FromAddress: interchainAccountAddr,
						ToAddress:   suite.chainB.SenderAccount.GetAddress().String(),
						Amount:      sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))),
					},
				}

				data, err := icatypes.SerializeCosmosTx(suite.chainB.GetSimApp().AppCodec(), msgsBankSend, icatypes.EncodingProtobuf)
				suite.Require().NoError(err)

				packetData = icatypes.InterchainAccountPacketData{
					Type: icatypes.EXECUTE_TX,
					Data: data,
				}
			},
			true,
		},
		{
			"data is nil",
			func() {
				packetData = icatypes.InterchainAccountPacketData{
					Type: icatypes.EXECUTE_TX,
					Data: nil,
				}
			},
			false,
		},
		{
			"active channel not found",
			func() {
				path.EndpointA.ChannelConfig.PortID = "invalid-port-id"
			},
			false,
		},
		{
			"channel in INIT state - optimistic packet sends fail",
			func() {
				channel, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)

				channel.State = channeltypes.INIT
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.SetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, channel)
			},
			false,
		},
		{
			"sendPacket fails - channel closed",
			func() {
				err := path.EndpointA.SetChannelState(channeltypes.CLOSED)
				suite.Require().NoError(err)
			},
			false,
		},
		{
			"controller submodule disabled",
			func() {
				suite.chainA.GetSimApp().ICAControllerKeeper.SetParams(suite.chainA.GetContext(), types.NewParams(false))
			},
			false,
		},
		{
			"timeout timestamp is not in the future",
			func() {
				interchainAccountAddr, found := suite.chainA.GetSimApp().ICAControllerKeeper.GetInterchainAccountAddress(suite.chainA.GetContext(), ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID)
				suite.Require().True(found)

				msg := &banktypes.MsgSend{
					FromAddress: interchainAccountAddr,
					ToAddress:   suite.chainB.SenderAccount.GetAddress().String(),
					Amount:      sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))),
				}

				data, err := icatypes.SerializeCosmosTx(suite.chainB.GetSimApp().AppCodec(), []proto.Message{msg}, icatypes.EncodingProtobuf)
				suite.Require().NoError(err)

				packetData = icatypes.InterchainAccountPacketData{
					Type: icatypes.EXECUTE_TX,
					Data: data,
				}

				timeoutTimestamp = uint64(suite.chainA.GetContext().BlockTime().UnixNano())
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.msg, func() {
			suite.SetupTest()             // reset
			timeoutTimestamp = ^uint64(0) // default

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			//nolint: staticcheck // SA1019: ibctesting.FirstConnectionID is deprecated: use path.EndpointA.ConnectionID instead. (staticcheck)
			_, err = suite.chainA.GetSimApp().ICAControllerKeeper.SendTx(suite.chainA.GetContext(), nil, ibctesting.FirstConnectionID, path.EndpointA.ChannelConfig.PortID, packetData, timeoutTimestamp)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnTimeoutPacket() {
	var path *ibctesting.Path

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.msg, func() {
			suite.SetupTest() // reset

			path = NewICAPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupConnections(path)

			err := SetupICAPath(path, TestOwnerAddress)
			suite.Require().NoError(err)

			tc.malleate() // malleate mutates test data

			packet := channeltypes.NewPacket(
				[]byte{},
				1,
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				path.EndpointB.ChannelConfig.PortID,
				path.EndpointB.ChannelID,
				clienttypes.NewHeight(0, 100),
				0,
			)

			err = suite.chainA.GetSimApp().ICAControllerKeeper.OnTimeoutPacket(suite.chainA.GetContext(), packet)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
