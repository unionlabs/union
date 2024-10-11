package keeper_test

import (
	"fmt"
	"strings"

	sdkmath "cosmossdk.io/math"

	banktestutil "cosmossdk.io/x/bank/testutil"
	banktypes "cosmossdk.io/x/bank/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// test sending from chainA to chainB using both coin that orignate on
// chainA and coin that orignate on chainB
func (suite *KeeperTestSuite) TestSendTransfer() {
	var (
		coin            sdk.Coin
		path            *ibctesting.Path
		sender          sdk.AccAddress
		timeoutHeight   clienttypes.Height
		memo            string
		expEscrowAmount sdkmath.Int // total amount in escrow for denom on receiving chain
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"successful transfer with native token",
			func() {
				expEscrowAmount = sdkmath.NewInt(100)
			}, true,
		},
		{
			"successful transfer from source chain with memo",
			func() {
				memo = "memo" //nolint:goconst
				expEscrowAmount = sdkmath.NewInt(100)
			}, true,
		},
		{
			"successful transfer with IBC token",
			func() {
				// send IBC token back to chainB
				coin = types.GetTransferCoin(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, coin.Denom, coin.Amount)
			}, true,
		},
		{
			"successful transfer with IBC token and memo",
			func() {
				// send IBC token back to chainB
				coin = types.GetTransferCoin(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, coin.Denom, coin.Amount)
				memo = "memo"
			}, true,
		},
		{
			"successful transfer of entire balance",
			func() {
				coin.Amount = types.UnboundedSpendLimit()
				var ok bool
				expEscrowAmount, ok = sdkmath.NewIntFromString(ibctesting.DefaultGenesisAccBalance)
				suite.Require().True(ok)
			}, true,
		},
		{
			"source channel not found",
			func() {
				// channel references wrong ID
				path.EndpointA.ChannelID = ibctesting.InvalidID
			}, false,
		},
		{
			"transfer failed - sender account is blocked",
			func() {
				sender = suite.chainA.GetSimApp().AuthKeeper.GetModuleAddress(types.ModuleName)
			}, false,
		},
		{
			"send coin failed",
			func() {
				coin = sdk.NewCoin("randomdenom", sdkmath.NewInt(100))
			}, false,
		},
		{
			"failed to parse coin denom",
			func() {
				coin = types.GetTransferCoin(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, "randomdenom", coin.Amount)
			}, false,
		},
		{
			"send from module account failed, insufficient balance",
			func() {
				coin = types.GetTransferCoin(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, coin.Denom, coin.Amount.Add(sdkmath.NewInt(1)))
			}, false,
		},
		{
			"channel capability not found",
			func() {
				capability := suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				// Release channel capability
				suite.chainA.GetSimApp().ScopedTransferKeeper.ReleaseCapability(suite.chainA.GetContext(), capability) //nolint:errcheck // ignore error for testing
			}, false,
		},
		{
			"SendPacket fails, timeout height and timeout timestamp are zero",
			func() {
				timeoutHeight = clienttypes.ZeroHeight()
				expEscrowAmount = sdkmath.NewInt(100)
			}, false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = ibctesting.NewTransferPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			coin = sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
			sender = suite.chainA.SenderAccount.GetAddress()
			memo = ""
			timeoutHeight = suite.chainB.GetTimeoutHeight()
			expEscrowAmount = sdkmath.ZeroInt()

			// create IBC token on chainA
			transferMsg := types.NewMsgTransfer(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, coin, suite.chainB.SenderAccount.GetAddress().String(), suite.chainA.SenderAccount.GetAddress().String(), suite.chainA.GetTimeoutHeight(), 0, "")
			result, err := suite.chainB.SendMsgs(transferMsg)
			suite.Require().NoError(err) // message committed

			packet, err := ibctesting.ParsePacketFromEvents(result.Events)
			suite.Require().NoError(err)

			err = path.RelayPacket(packet)
			suite.Require().NoError(err)

			tc.malleate()

			msg := types.NewMsgTransfer(
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				coin, sender.String(), suite.chainB.SenderAccount.GetAddress().String(),
				timeoutHeight, 0, // only use timeout height
				memo,
			)

			res, err := suite.chainA.GetSimApp().TransferKeeper.Transfer(suite.chainA.GetContext(), msg)

			// check total amount in escrow of sent token denom on sending chain
			amount := suite.chainA.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainA.GetContext(), coin.GetDenom())
			suite.Require().Equal(expEscrowAmount, amount.Amount)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(res)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestSendTransferSetsTotalEscrowAmountForSourceIBCToken() {
	/*
		Given the following flow of tokens:

		chain A (channel 0) -> (channel-0) chain B (channel-1) -> (channel-1) chain A
		stake                  transfer/channel-0/stake           transfer/channel-1/transfer/channel-0/stake
		                                  ^
		                                  |
		                             SendTransfer

		This test will transfer vouchers of denom "transfer/channel-0/stake" from chain B
		to chain A over channel-1 to assert that total escrow amount is stored on chain B
		for vouchers of denom "transfer/channel-0/stake" because chain B acts as source
		in this case.

		Set up:
		- Two transfer channels between chain A and chain B (channel-0 and channel-1).
		- Tokens of native denom "stake" on chain A transferred to chain B over channel-0
		and vouchers minted with denom trace "tranfer/channel-0/stake".

		Execute:
		- Transfer vouchers of denom trace "tranfer/channel-0/stake" from chain B to chain A
		over channel-1.

		Assert:
		- The vouchers are not of a native denom (because they are of an IBC denom), but chain B
		is the source, then the value for total escrow amount should still be stored for the IBC
		denom that corresponds to the trace "tranfer/channel-0/stake".
	*/

	// set up
	// 2 transfer channels between chain A and chain B
	path1 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path1)

	path2 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path2)

	// create IBC token on chain B with denom trace "transfer/channel-0/stake"
	coin := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
	transferMsg := types.NewMsgTransfer(
		path1.EndpointA.ChannelConfig.PortID,
		path1.EndpointA.ChannelID,
		coin,
		suite.chainA.SenderAccount.GetAddress().String(),
		suite.chainB.SenderAccount.GetAddress().String(),
		suite.chainB.GetTimeoutHeight(), 0, "",
	)
	result, err := suite.chainA.SendMsgs(transferMsg)
	suite.Require().NoError(err) // message committed

	packet, err := ibctesting.ParsePacketFromEvents(result.Events)
	suite.Require().NoError(err)

	err = path1.RelayPacket(packet)
	suite.Require().NoError(err)

	// execute
	trace := types.ParseDenomTrace(types.GetPrefixedDenom(path1.EndpointB.ChannelConfig.PortID, path1.EndpointB.ChannelID, sdk.DefaultBondDenom))
	coin = sdk.NewCoin(trace.IBCDenom(), sdkmath.NewInt(100))
	msg := types.NewMsgTransfer(
		path2.EndpointB.ChannelConfig.PortID,
		path2.EndpointB.ChannelID,
		coin,
		suite.chainB.SenderAccount.GetAddress().String(),
		suite.chainA.SenderAccount.GetAddress().String(),
		suite.chainA.GetTimeoutHeight(), 0, "",
	)

	res, err := suite.chainB.GetSimApp().TransferKeeper.Transfer(suite.chainB.GetContext(), msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)

	// check total amount in escrow of sent token on sending chain
	totalEscrow := suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.NewInt(100), totalEscrow.Amount)
}

// test receiving coin on chainB with coin that orignate on chainA and
// coin that originated on chainB (source). The bulk of the testing occurs
// in the test case for loop since setup is intensive for all cases. The
// malleate function allows for testing invalid cases.
func (suite *KeeperTestSuite) TestOnRecvPacket() {
	var (
		trace           types.DenomTrace
		amount          sdkmath.Int
		receiver        string
		memo            string
		expEscrowAmount sdkmath.Int // total amount in escrow for denom on receiving chain
	)

	testCases := []struct {
		msg          string
		malleate     func()
		recvIsSource bool // the receiving chain is the source of the coin originally
		expPass      bool
	}{
		{
			"success receive on source chain",
			func() {}, true, true,
		},
		{
			"success receive on source chain of half the amount",
			func() {
				amount = sdkmath.NewInt(50)
				expEscrowAmount = sdkmath.NewInt(50)
			}, true, true,
		},
		{
			"success receive on source chain with memo",
			func() {
				memo = "memo"
			}, true, true,
		},
		{
			"success receive with coin from another chain as source",
			func() {}, false, true,
		},
		{
			"success receive with coin from another chain as source with memo",
			func() {
				memo = "memo"
			}, false, true,
		},
		{
			"empty coin",
			func() {
				trace = types.DenomTrace{}
				amount = sdkmath.ZeroInt()
				expEscrowAmount = sdkmath.NewInt(100)
			}, true, false,
		},
		{
			"invalid receiver address",
			func() {
				receiver = "gaia1scqhwpgsmr6vmztaa7suurfl52my6nd2kmrudl"
				expEscrowAmount = sdkmath.NewInt(100)
			}, true, false,
		},

		// onRecvPacket
		// - coin from chain chainA
		{
			"failure: mint zero coin",
			func() {
				amount = sdkmath.ZeroInt()
			}, false, false,
		},

		// - coin being sent back to original chain (chainB)
		{
			"tries to unescrow more tokens than allowed",
			func() {
				amount = sdkmath.NewInt(1000000)
				expEscrowAmount = sdkmath.NewInt(100)
			}, true, false,
		},

		// - coin being sent to module address on chainA
		{
			"failure: receive on module account",
			func() {
				receiver = suite.chainB.GetSimApp().AuthKeeper.GetModuleAddress(types.ModuleName).String()
			}, false, false,
		},

		// - coin being sent back to original chain (chainB) to module address
		{
			"failure: receive on module account on source chain",
			func() {
				receiver = suite.chainB.GetSimApp().AuthKeeper.GetModuleAddress(types.ModuleName).String()
				expEscrowAmount = sdkmath.NewInt(100)
			}, true, false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			path := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)
			receiver = suite.chainB.SenderAccount.GetAddress().String() // must be explicitly changed in malleate

			memo = ""                           // can be explicitly changed in malleate
			amount = sdkmath.NewInt(100)        // must be explicitly changed in malleate
			expEscrowAmount = sdkmath.ZeroInt() // total amount in escrow of voucher denom on receiving chain

			// denom trace of tokens received on chain B and the associated expected metadata
			denomTraceOnB := types.ParseDenomTrace(types.GetPrefixedDenom(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, sdk.DefaultBondDenom))
			expDenomMetadataOnB := banktypes.Metadata{
				Description: fmt.Sprintf("IBC token from %s", denomTraceOnB.GetFullDenomPath()),
				DenomUnits: []*banktypes.DenomUnit{
					{
						Denom:    denomTraceOnB.GetBaseDenom(),
						Exponent: 0,
					},
				},
				Base:    denomTraceOnB.IBCDenom(),
				Display: denomTraceOnB.GetFullDenomPath(),
				Name:    fmt.Sprintf("%s IBC token", denomTraceOnB.GetFullDenomPath()),
				Symbol:  strings.ToUpper(denomTraceOnB.GetBaseDenom()),
			}

			seq := uint64(1)

			if tc.recvIsSource {
				// send coin from chainB to chainA, receive them, acknowledge them, and send back to chainB
				coinFromBToA := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
				transferMsg := types.NewMsgTransfer(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, coinFromBToA, suite.chainB.SenderAccount.GetAddress().String(), suite.chainA.SenderAccount.GetAddress().String(), clienttypes.NewHeight(1, 110), 0, memo)
				res, err := suite.chainB.SendMsgs(transferMsg)
				suite.Require().NoError(err) // message committed

				packet, err := ibctesting.ParsePacketFromEvents(res.Events)
				suite.Require().NoError(err)

				err = path.RelayPacket(packet)
				suite.Require().NoError(err) // relay committed

				seq++

				// NOTE: trace must be explicitly changed in malleate to test invalid cases
				trace = types.ParseDenomTrace(types.GetPrefixedDenom(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, sdk.DefaultBondDenom))
			} else {
				trace = types.ParseDenomTrace(sdk.DefaultBondDenom)
			}

			// send coin from chainA to chainB
			coin := sdk.NewCoin(trace.IBCDenom(), amount)
			transferMsg := types.NewMsgTransfer(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, coin, suite.chainA.SenderAccount.GetAddress().String(), receiver, clienttypes.NewHeight(1, 110), 0, memo)
			_, err := suite.chainA.SendMsgs(transferMsg)
			suite.Require().NoError(err) // message committed

			tc.malleate()

			data := types.NewFungibleTokenPacketData(trace.GetFullDenomPath(), amount.String(), suite.chainA.SenderAccount.GetAddress().String(), receiver, memo)
			packet := channeltypes.NewPacket(data.GetBytes(), seq, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.NewHeight(1, 100), 0)

			err = suite.chainB.GetSimApp().TransferKeeper.OnRecvPacket(suite.chainB.GetContext(), packet, data)

			// check total amount in escrow of received token denom on receiving chain
			totalEscrow := suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), sdk.DefaultBondDenom)
			suite.Require().Equal(expEscrowAmount, totalEscrow.Amount)

			if tc.expPass {
				suite.Require().NoError(err)

				if tc.recvIsSource {
					_, found := suite.chainB.GetSimApp().BankKeeper.GetDenomMetaData(suite.chainB.GetContext(), sdk.DefaultBondDenom)
					suite.Require().False(found)
				} else {
					denomMetadata, found := suite.chainB.GetSimApp().BankKeeper.GetDenomMetaData(suite.chainB.GetContext(), denomTraceOnB.IBCDenom())
					suite.Require().True(found)
					suite.Require().Equal(expDenomMetadataOnB, denomMetadata)
				}
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnRecvPacketSetsTotalEscrowAmountForSourceIBCToken() {
	/*
		Given the following flow of tokens:

		chain A (channel 0) -> (channel-0) chain B (channel-1) -> (channel-1) chain A (channel-1)             -> (channel-1) chain B
		stake                  transfer/channel-0/stake           transfer/channel-1/transfer/channel-0/stake    transfer/channel-0/stake
		                                                                                                                   ^
		                                                                                                                   |
		                                                                                                              OnRecvPacket

		This test will assert that on receiving vouchers of denom "transfer/channel-0/stake"
		on chain B the total escrow amount is updated on because chain B acted as source
		when vouchers were transferred to chain A over channel-1.

		Setup:
		- Two transfer channels between chain A and chain B.
		- Vouchers of denom trace "transfer/channel-0/stake" on chain B are in escrow
		account for port ID transfer and channel ID channel-1.

		Execute:
		- Receive vouchers of denom trace "transfer/channel-0/stake" from chain A to chain B
		over channel-1.

		Assert:
		- The vouchers are not of a native denom (because they are of an IBC denom), but chain B
		is the source, then the value for total escrow amount should still be updated for the IBC
		denom that corresponds to the trace "tranfer/channel-0/stake" when the vouchers are
		received back on chain B.
	*/

	seq := uint64(1)
	amount := sdkmath.NewInt(100)
	timeout := suite.chainA.GetTimeoutHeight()

	// setup
	// 2 transfer channels between chain A and chain B
	path1 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path1)

	path2 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path2)

	// denomTrace path: {transfer/channel-1/transfer/channel-0}
	denomTrace := types.DenomTrace{
		BaseDenom: sdk.DefaultBondDenom,
		Path:      fmt.Sprintf("%s/%s/%s/%s", path2.EndpointA.ChannelConfig.PortID, path2.EndpointA.ChannelID, path1.EndpointB.ChannelConfig.PortID, path1.EndpointB.ChannelID),
	}
	data := types.NewFungibleTokenPacketData(
		denomTrace.GetFullDenomPath(),
		amount.String(),
		suite.chainA.SenderAccount.GetAddress().String(),
		suite.chainB.SenderAccount.GetAddress().String(), "",
	)
	packet := channeltypes.NewPacket(
		data.GetBytes(),
		seq,
		path2.EndpointA.ChannelConfig.PortID,
		path2.EndpointA.ChannelID,
		path2.EndpointB.ChannelConfig.PortID,
		path2.EndpointB.ChannelID,
		timeout, 0,
	)

	// fund escrow account for transfer and channel-1 on chain B
	// denomTrace path: transfer/channel-0
	denomTrace = types.DenomTrace{
		BaseDenom: sdk.DefaultBondDenom,
		Path:      fmt.Sprintf("%s/%s", path1.EndpointB.ChannelConfig.PortID, path1.EndpointB.ChannelID),
	}
	escrowAddress := types.GetEscrowAddress(path2.EndpointB.ChannelConfig.PortID, path2.EndpointB.ChannelID)
	coin := sdk.NewCoin(denomTrace.IBCDenom(), amount)
	suite.Require().NoError(
		banktestutil.FundAccount(
			suite.chainB.GetContext(),
			suite.chainB.GetSimApp().BankKeeper,
			escrowAddress,
			sdk.NewCoins(coin),
		),
	)

	suite.chainB.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainB.GetContext(), coin)
	totalEscrowChainB := suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.NewInt(100), totalEscrowChainB.Amount)

	// execute onRecvPacket, when chaninB receives the source token the escrow amount should decrease
	err := suite.chainB.GetSimApp().TransferKeeper.OnRecvPacket(suite.chainB.GetContext(), packet, data)
	suite.Require().NoError(err)

	// check total amount in escrow of sent token on reveiving chain
	totalEscrowChainB = suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.ZeroInt(), totalEscrowChainB.Amount)
}

// TestOnAcknowledgementPacket tests that successful acknowledgement is a no-op
// and failure acknowledment leads to refund when attempting to send from chainA
// to chainB. If sender is source then the denomination being refunded has no
// trace
func (suite *KeeperTestSuite) TestOnAcknowledgementPacket() {
	var (
		successAck      = channeltypes.NewResultAcknowledgement([]byte{byte(1)})
		failedAck       = channeltypes.NewErrorAcknowledgement(fmt.Errorf("failed packet transfer"))
		trace           types.DenomTrace
		amount          sdkmath.Int
		path            *ibctesting.Path
		expEscrowAmount sdkmath.Int
	)

	testCases := []struct {
		msg      string
		ack      channeltypes.Acknowledgement
		malleate func()
		success  bool // success of ack
		expPass  bool
	}{
		{
			"success ack causes no-op",
			successAck,
			func() {
				trace = types.ParseDenomTrace(types.GetPrefixedDenom(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, sdk.DefaultBondDenom))
			}, true, true,
		},
		{
			"successful refund from source chain",
			failedAck,
			func() {
				escrow := types.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				trace = types.ParseDenomTrace(sdk.DefaultBondDenom)
				coin := sdk.NewCoin(sdk.DefaultBondDenom, amount)

				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrow, sdk.NewCoins(coin)))

				// set escrow amount that would have been stored after successful execution of MsgTransfer
				suite.chainA.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainA.GetContext(), sdk.NewCoin(sdk.DefaultBondDenom, amount))
			}, false, true,
		},
		{
			"unsuccessful refund from source",
			failedAck,
			func() {
				trace = types.ParseDenomTrace(sdk.DefaultBondDenom)

				// set escrow amount that would have been stored after successful execution of MsgTransfer
				suite.chainA.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainA.GetContext(), sdk.NewCoin(sdk.DefaultBondDenom, amount))
				expEscrowAmount = sdkmath.NewInt(100)
			}, false, false,
		},
		{
			"successful refund with coin from external chain",
			failedAck,
			func() {
				escrow := types.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				trace = types.ParseDenomTrace(types.GetPrefixedDenom(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, sdk.DefaultBondDenom))
				coin := sdk.NewCoin(trace.IBCDenom(), amount)

				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrow, sdk.NewCoins(coin)))
			}, false, true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset
			path = ibctesting.NewTransferPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)
			amount = sdkmath.NewInt(100) // must be explicitly changed
			expEscrowAmount = sdkmath.ZeroInt()

			tc.malleate()

			data := types.NewFungibleTokenPacketData(trace.GetFullDenomPath(), amount.String(), suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(), "")
			packet := channeltypes.NewPacket(data.GetBytes(), 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.NewHeight(1, 100), 0)
			preCoin := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), trace.IBCDenom())

			err := suite.chainA.GetSimApp().TransferKeeper.OnAcknowledgementPacket(suite.chainA.GetContext(), packet, data, tc.ack)

			// check total amount in escrow of sent token denom on sending chain
			totalEscrow := suite.chainA.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainA.GetContext(), trace.IBCDenom())
			suite.Require().Equal(expEscrowAmount, totalEscrow.Amount)

			if tc.expPass {
				suite.Require().NoError(err)
				postCoin := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), trace.IBCDenom())
				deltaAmount := postCoin.Amount.Sub(preCoin.Amount)

				if tc.success {
					suite.Require().Equal(int64(0), deltaAmount.Int64(), "successful ack changed balance")
				} else {
					suite.Require().Equal(amount, deltaAmount, "failed ack did not trigger refund")
				}
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnAcknowledgementPacketSetsTotalEscrowAmountForSourceIBCToken() {
	/*
		This test is testing the following scenario. Given tokens travelling like this:

		chain A (channel 0) -> (channel-0) chain B (channel-1) -> (channel-1) chain A (channel-1)
		stake                  transfer/channel-0/stake           transfer/channel-1/transfer/channel-0/stake
		                                 ^
		                                 |
		                         OnAcknowledgePacket

		We want to assert that on failed acknowledgment of vouchers sent with denom trace
		"transfer/channel-0/stake" on chain B the total escrow amount is updated.

		Set up:
		- Two transfer channels between chain A and chain B.
		- Vouckers of denom "transfer/channel-0/stake" on chain B are in escrow
		account for port ID transfer and channel ID channel-1.

		Execute:
		- Acknowledge vouchers of denom trace "tranfer/channel-0/stake" sent from chain B
		to chain B over channel-1.

		Assert:
		- The vouchers are not of a native denom (because they are of an IBC denom), but chain B
		is the source, then the value for total escrow amount should still be updated for the IBC
		denom that corresponds to the trace "tranfer/channel-0/stake" when processing the failed
		acknowledgement.
	*/

	seq := uint64(1)
	amount := sdkmath.NewInt(100)
	ack := channeltypes.NewErrorAcknowledgement(fmt.Errorf("failed packet transfer"))

	// set up
	// 2 transfer channels between chain A and chain B
	path1 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path1)

	path2 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path2)

	// fund escrow account for transfer and channel-1 on chain B
	// denomTrace path = transfer/channel-0
	denomTrace := types.DenomTrace{
		BaseDenom: sdk.DefaultBondDenom,
		Path:      fmt.Sprintf("%s/%s", path1.EndpointB.ChannelConfig.PortID, path1.EndpointB.ChannelID),
	}
	escrowAddress := types.GetEscrowAddress(path2.EndpointB.ChannelConfig.PortID, path2.EndpointB.ChannelID)
	coin := sdk.NewCoin(denomTrace.IBCDenom(), amount)
	suite.Require().NoError(
		banktestutil.FundAccount(
			suite.chainB.GetContext(),
			suite.chainB.GetSimApp().BankKeeper,
			escrowAddress,
			sdk.NewCoins(coin),
		),
	)

	data := types.NewFungibleTokenPacketData(
		denomTrace.GetFullDenomPath(),
		amount.String(),
		suite.chainB.SenderAccount.GetAddress().String(),
		suite.chainA.SenderAccount.GetAddress().String(), "",
	)
	packet := channeltypes.NewPacket(
		data.GetBytes(),
		seq,
		path2.EndpointB.ChannelConfig.PortID,
		path2.EndpointB.ChannelID,
		path2.EndpointA.ChannelConfig.PortID,
		path2.EndpointA.ChannelID,
		suite.chainA.GetTimeoutHeight(), 0,
	)

	suite.chainB.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainB.GetContext(), coin)
	totalEscrowChainB := suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.NewInt(100), totalEscrowChainB.Amount)

	err := suite.chainB.GetSimApp().TransferKeeper.OnAcknowledgementPacket(suite.chainB.GetContext(), packet, data, ack)
	suite.Require().NoError(err)

	// check total amount in escrow of sent token on sending chain
	totalEscrowChainB = suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.ZeroInt(), totalEscrowChainB.Amount)
}

// TestOnTimeoutPacket test private refundPacket function since it is a simple
// wrapper over it. The actual timeout does not matter since IBC core logic
// is not being tested. The test is timing out a send from chainA to chainB
// so the refunds are occurring on chainA.
func (suite *KeeperTestSuite) TestOnTimeoutPacket() {
	var (
		trace           types.DenomTrace
		path            *ibctesting.Path
		amount          sdkmath.Int
		sender          string
		expEscrowAmount sdkmath.Int
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"successful timeout from sender as source chain",
			func() {
				escrow := types.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				trace = types.ParseDenomTrace(sdk.DefaultBondDenom)
				coin := sdk.NewCoin(trace.IBCDenom(), amount)
				expEscrowAmount = sdkmath.ZeroInt()

				// funds the escrow account to have balance
				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrow, sdk.NewCoins(coin)))
				// set escrow amount that would have been stored after successful execution of MsgTransfer
				suite.chainA.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainA.GetContext(), coin)
			}, true,
		},
		{
			"successful timeout from external chain",
			func() {
				escrow := types.GetEscrowAddress(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				trace = types.ParseDenomTrace(types.GetPrefixedDenom(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, sdk.DefaultBondDenom))
				coin := sdk.NewCoin(trace.IBCDenom(), amount)
				expEscrowAmount = sdkmath.ZeroInt()

				// funds the escrow account to have balance
				suite.Require().NoError(banktestutil.FundAccount(suite.chainA.GetContext(), suite.chainA.GetSimApp().BankKeeper, escrow, sdk.NewCoins(coin)))
			}, true,
		},
		{
			"no balance for coin denom",
			func() {
				trace = types.ParseDenomTrace("bitcoin")
				expEscrowAmount = amount

				// set escrow amount that would have been stored after successful execution of MsgTransfer
				suite.chainA.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainA.GetContext(), sdk.NewCoin(trace.IBCDenom(), amount))
			}, false,
		},
		{
			"unescrow failed",
			func() {
				trace = types.ParseDenomTrace(sdk.DefaultBondDenom)
				expEscrowAmount = amount

				// set escrow amount that would have been stored after successful execution of MsgTransfer
				suite.chainA.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainA.GetContext(), sdk.NewCoin(trace.IBCDenom(), amount))
			}, false,
		},
		{
			"mint failed",
			func() {
				trace = types.ParseDenomTrace(types.GetPrefixedDenom(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, sdk.DefaultBondDenom))
				amount = sdkmath.OneInt()
				sender = "invalid address"
			}, false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset

			path = ibctesting.NewTransferPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)
			amount = sdkmath.NewInt(100) // must be explicitly changed
			sender = suite.chainA.SenderAccount.GetAddress().String()
			expEscrowAmount = sdkmath.ZeroInt()

			tc.malleate()

			data := types.NewFungibleTokenPacketData(trace.GetFullDenomPath(), amount.String(), sender, suite.chainB.SenderAccount.GetAddress().String(), "")
			packet := channeltypes.NewPacket(data.GetBytes(), 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.NewHeight(1, 100), 0)
			preCoin := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), trace.IBCDenom())

			err := suite.chainA.GetSimApp().TransferKeeper.OnTimeoutPacket(suite.chainA.GetContext(), packet, data)

			postCoin := suite.chainA.GetSimApp().BankKeeper.GetBalance(suite.chainA.GetContext(), suite.chainA.SenderAccount.GetAddress(), trace.IBCDenom())
			deltaAmount := postCoin.Amount.Sub(preCoin.Amount)

			// check total amount in escrow of sent token denom on sending chain
			totalEscrow := suite.chainA.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainA.GetContext(), trace.IBCDenom())
			suite.Require().Equal(expEscrowAmount, totalEscrow.Amount)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(amount.Int64(), deltaAmount.Int64(), "successful timeout did not trigger refund")
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestOnTimeoutPacketSetsTotalEscrowAmountForSourceIBCToken() {
	/*
		Given the following flow of tokens:

		chain A (channel 0) -> (channel-0) chain B (channel-1) -> (channel-1) chain A (channel-1)
		stake                  transfer/channel-0/stake           transfer/channel-1/transfer/channel-0/stake
		                                 ^
		                                 |
		                           OnTimeoutPacket

		We want to assert that on timeout of vouchers sent with denom trace
		"transfer/channel-0/stake" on chain B the total escrow amount is updated.

		Set up:
		- Two transfer channels between chain A and chain B.
		- Vouckers of denom "transfer/channel-0/stake" on chain B are in escrow
		account for port ID transfer and channel ID channel-1.

		Execute:
		- Timeout vouchers of denom trace "tranfer/channel-0/stake" sent from chain B
		to chain B over channel-1.

		Assert:
		- The vouchers are not of a native denom (because they are of an IBC denom), but chain B
		is the source, then the value for total escrow amount should still be updated for the IBC
		denom that corresponds to the trace "tranfer/channel-0/stake" when processing the timeout.
	*/

	seq := uint64(1)
	amount := sdkmath.NewInt(100)

	// set up
	// 2 transfer channels between chain A and chain B
	path1 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path1)

	path2 := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path2)

	// fund escrow account for transfer and channel-1 on chain B
	denomTrace := types.DenomTrace{
		BaseDenom: sdk.DefaultBondDenom,
		Path:      fmt.Sprintf("%s/%s", path1.EndpointB.ChannelConfig.PortID, path1.EndpointB.ChannelID),
	}
	escrowAddress := types.GetEscrowAddress(path2.EndpointB.ChannelConfig.PortID, path2.EndpointB.ChannelID)
	coin := sdk.NewCoin(denomTrace.IBCDenom(), amount)
	suite.Require().NoError(
		banktestutil.FundAccount(
			suite.chainB.GetContext(),
			suite.chainB.GetSimApp().BankKeeper,
			escrowAddress,
			sdk.NewCoins(coin),
		),
	)

	data := types.NewFungibleTokenPacketData(
		denomTrace.GetFullDenomPath(),
		amount.String(),
		suite.chainB.SenderAccount.GetAddress().String(),
		suite.chainA.SenderAccount.GetAddress().String(), "",
	)
	packet := channeltypes.NewPacket(
		data.GetBytes(),
		seq,
		path2.EndpointB.ChannelConfig.PortID,
		path2.EndpointB.ChannelID,
		path2.EndpointA.ChannelConfig.PortID,
		path2.EndpointA.ChannelID,
		suite.chainA.GetTimeoutHeight(), 0,
	)

	suite.chainB.GetSimApp().TransferKeeper.SetTotalEscrowForDenom(suite.chainB.GetContext(), coin)
	totalEscrowChainB := suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.NewInt(100), totalEscrowChainB.Amount)

	err := suite.chainB.GetSimApp().TransferKeeper.OnTimeoutPacket(suite.chainB.GetContext(), packet, data)
	suite.Require().NoError(err)

	// check total amount in escrow of sent token on sending chain
	totalEscrowChainB = suite.chainB.GetSimApp().TransferKeeper.GetTotalEscrowForDenom(suite.chainB.GetContext(), coin.GetDenom())
	suite.Require().Equal(sdkmath.ZeroInt(), totalEscrowChainB.Amount)
}
