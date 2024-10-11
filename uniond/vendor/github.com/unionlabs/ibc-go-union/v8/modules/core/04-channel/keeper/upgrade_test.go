package keeper_test

import (
	"fmt"
	"math"
	"testing"

	errorsmod "cosmossdk.io/errors"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/mock"
)

func (suite *KeeperTestSuite) TestChanUpgradeInit() {
	var (
		path          *ibctesting.Path
		expSequence   uint64
		upgradeFields types.UpgradeFields
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"success with later upgrade sequence",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.UpgradeSequence = 4
				path.EndpointA.SetChannel(channel)
				expSequence = 5
			},
			true,
		},
		{
			"upgrade fields are identical to channel end",
			func() {
				channel := path.EndpointA.GetChannel()
				upgradeFields = types.NewUpgradeFields(channel.Ordering, channel.ConnectionHops, channel.Version)
			},
			false,
		},
		{
			"channel not found",
			func() {
				path.EndpointA.ChannelID = "invalid-channel"
				path.EndpointA.ChannelConfig.PortID = "invalid-port"
			},
			false,
		},
		{
			"channel state is not in OPEN state",
			func() {
				suite.Require().NoError(path.EndpointA.SetChannelState(types.CLOSED))
			},
			false,
		},
		{
			"proposed channel connection not found",
			func() {
				upgradeFields.ConnectionHops = []string{"connection-100"}
			},
			false,
		},
		{
			"invalid proposed channel connection state",
			func() {
				connectionEnd := path.EndpointA.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED

				suite.chainA.GetSimApp().GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainA.GetContext(), "connection-100", connectionEnd)
				upgradeFields.ConnectionHops = []string{"connection-100"}
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			expSequence = 1

			upgradeFields = types.NewUpgradeFields(types.UNORDERED, []string{path.EndpointA.ConnectionID}, mock.UpgradeVersion)

			tc.malleate()

			upgrade, err := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeInit(
				suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgradeFields,
			)

			if tc.expPass {
				ctx := suite.chainA.GetContext()
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeInitChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgrade, upgrade.Fields.Version)
				channel := path.EndpointA.GetChannel()

				suite.Require().NoError(err)
				suite.Require().Equal(expSequence, channel.UpgradeSequence)
				suite.Require().Equal(mock.Version, channel.Version)
				suite.Require().Equal(types.OPEN, channel.State)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgradeTry() {
	var (
		path                *ibctesting.Path
		proposedUpgrade     types.Upgrade
		counterpartyUpgrade types.Upgrade
	)

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
			"success: crossing hellos",
			func() {
				err := path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)
			},
			nil,
		},
		{
			"success: upgrade sequence is fast forwarded to counterparty upgrade sequence",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.UpgradeSequence = 5
				path.EndpointA.SetChannel(channel)

				suite.coordinator.CommitBlock(suite.chainA)
			},
			nil,
		},
		{
			"channel not found",
			func() {
				path.EndpointB.ChannelID = ibctesting.InvalidID
			},
			types.ErrChannelNotFound,
		},
		{
			"channel state is not in OPEN state",
			func() {
				suite.Require().NoError(path.EndpointB.SetChannelState(types.CLOSED))
			},
			types.ErrInvalidChannelState,
		},
		{
			"connection not found",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.ConnectionHops = []string{"connection-100"}
				path.EndpointB.SetChannel(channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"invalid connection state",
			func() {
				connectionEnd := path.EndpointB.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED
				suite.chainB.GetSimApp().GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainB.GetContext(), path.EndpointB.ConnectionID, connectionEnd)
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"initializing handshake fails, proposed connection hops do not exist",
			func() {
				proposedUpgrade.Fields.ConnectionHops = []string{ibctesting.InvalidID}
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"fails due to proof verification failure, counterparty channel ordering does not match expected ordering",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.Ordering = types.ORDERED
				path.EndpointB.SetChannel(channel)
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"fails due to proof verification failure, counterparty upgrade connection hops are tampered with",
			func() {
				counterpartyUpgrade.Fields.ConnectionHops = []string{ibctesting.InvalidID}
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"fails due to incompatible upgrades, chainB proposes a new connection hop that does not match counterparty",
			func() {
				// reuse existing connection to create a new connection in a non OPEN state
				connection := path.EndpointB.GetConnection()
				// ensure counterparty connectionID does not match connectionID set in counterparty proposed upgrade
				connection.Counterparty.ConnectionId = "connection-50"

				// set proposed connection in state
				proposedConnectionID := "connection-100" //nolint:goconst
				suite.chainB.GetSimApp().GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainB.GetContext(), proposedConnectionID, connection)
				proposedUpgrade.Fields.ConnectionHops[0] = proposedConnectionID
			},
			types.ErrIncompatibleCounterpartyUpgrade,
		},
		{
			"fails due to mismatch in upgrade sequences",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.UpgradeSequence = 5
				path.EndpointB.SetChannel(channel)
			},
			// channel sequence will be returned so that counterparty inits on completely fresh sequence for both sides
			types.NewUpgradeError(5, types.ErrInvalidUpgradeSequence),
		},
		{
			"fails due to mismatch in upgrade sequences: chainB is on incremented sequence without an upgrade indicating it has already processed upgrade at this sequence.",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.UpgradeSequence = 1
				errorReceipt := types.NewUpgradeError(1, types.ErrInvalidUpgrade)
				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.WriteErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, errorReceipt)
				path.EndpointB.SetChannel(channel)
			},
			types.NewUpgradeError(1, types.ErrInvalidUpgradeSequence),
		},
		{
			"fails due to mismatch in upgrade sequences, crossing hello with the TRY chain having a higher sequence",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.UpgradeSequence = 4
				path.EndpointB.SetChannel(channel)

				// upgrade sequence is 5 after this call
				err := path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)
			},
			types.NewUpgradeError(4, types.ErrInvalidUpgradeSequence),
		},
		{
			// ChainA(Sequence: 0, mock-version-v2), ChainB(Sequence: 0, mock-version-v3)
			// ChainA.INIT(Sequence: 1)
			// ChainB.INIT(Sequence: 1)
			// ChainA.TRY => error (incompatible versions)
			// ChainB.TRY => error (incompatible versions)
			"crossing hellos: fails due to incompatible version",
			func() {
				// use incompatible version
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = fmt.Sprintf("%s-v3", mock.Version)
				proposedUpgrade = path.EndpointB.GetProposedUpgrade()

				err := path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointA.ChanUpgradeTry()
				suite.Require().Error(err)
				suite.Require().ErrorContains(err, "incompatible counterparty upgrade")
				suite.Require().Equal(uint64(1), path.EndpointA.GetChannel().UpgradeSequence)
			},
			types.ErrIncompatibleCounterpartyUpgrade,
		},
		{
			// ChainA(Sequence: 0, mock-version-v2), ChainB(Sequence: 4, mock-version-v3)
			// ChainA.INIT(Sequence: 1)
			// ChainB.INIT(Sequence: 5)
			// ChainA.TRY => error (incompatible versions)
			// ChainB.TRY(ErrorReceipt: 4)
			"crossing hellos: upgrade starts with mismatching upgrade sequences and try fails on counterparty due to incompatible version",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.UpgradeSequence = 4
				path.EndpointB.SetChannel(channel)

				// use incompatible version
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = fmt.Sprintf("%s-v3", mock.Version)
				proposedUpgrade = path.EndpointB.GetProposedUpgrade()

				err := path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointA.ChanUpgradeTry()
				suite.Require().Error(err)
				suite.Require().ErrorContains(err, "incompatible counterparty upgrade")
				suite.Require().Equal(uint64(1), path.EndpointA.GetChannel().UpgradeSequence)
			},
			types.NewUpgradeError(4, types.ErrInvalidUpgradeSequence),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			expPass := tc.expError == nil

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			proposedUpgrade = path.EndpointB.GetProposedUpgrade()

			var found bool
			counterpartyUpgrade, found = path.EndpointA.Chain.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(path.EndpointA.Chain.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().True(found)

			tc.malleate()

			// ensure clients are up to date to receive valid proofs
			suite.Require().NoError(path.EndpointB.UpdateClient())

			channelProof, upgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

			_, upgrade, err := suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeTry(
				suite.chainB.GetContext(),
				path.EndpointB.ChannelConfig.PortID,
				path.EndpointB.ChannelID,
				proposedUpgrade.Fields.ConnectionHops,
				counterpartyUpgrade.Fields,
				path.EndpointA.GetChannel().UpgradeSequence,
				channelProof,
				upgradeProof,
				proofHeight,
			)

			if expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(upgrade)
				suite.Require().Equal(proposedUpgrade.Fields, upgrade.Fields)

				nextSequenceSend, found := path.EndpointB.Chain.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceSend(path.EndpointB.Chain.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(nextSequenceSend, upgrade.NextSequenceSend)
			} else {
				suite.assertUpgradeError(err, tc.expError)
			}
		})
	}
}

// TestChanUpgrade_CrossingHellos_UpgradeSucceeds_AfterCancel verifies that under crossing hellos if upgrade
// sequences become out of sync, the upgrade can still be performed successfully after the upgrade is cancelled.
// ChainA(Sequence: 0), ChainB(Sequence 4)
// ChainA.INIT(Sequence: 1)
// ChainB.INIT(Sequence: 5)
// ChainB.TRY(ErrorReceipt: 4)
// ChainA.Cancel(Sequence: 4)
// ChainA.TRY(Sequence: 5) // fastforward
// ChainB.ACK => Success
// ChainA.Confirm => Success
// ChainB.Open => Success
func (suite *KeeperTestSuite) TestChanUpgrade_CrossingHellos_UpgradeSucceeds_AfterCancel() {
	var path *ibctesting.Path

	suite.Run("setup path", func() {
		path = ibctesting.NewPath(suite.chainA, suite.chainB)
		suite.coordinator.Setup(path)

		path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
		path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
	})

	suite.Run("chainA upgrade init", func() {
		err := path.EndpointA.ChanUpgradeInit()
		suite.Require().NoError(err)
	})

	suite.Run("set chainB upgrade sequence ahead of counterparty", func() {
		channel := path.EndpointB.GetChannel()
		channel.UpgradeSequence = 4
		path.EndpointB.SetChannel(channel)
	})

	suite.Run("chainB upgrade init (crossing hello)", func() {
		err := path.EndpointB.ChanUpgradeInit()
		suite.Require().NoError(err)
	})

	suite.Run("chainB upgrade try fails with invalid sequence", func() {
		err := path.EndpointB.ChanUpgradeTry()
		suite.Require().NoError(err)

		errorReceipt, found := path.EndpointB.Chain.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
		suite.Require().True(found)
		suite.Require().Equal(uint64(4), errorReceipt.Sequence)
	})

	suite.Run("cancel upgrade on chainA and fast forward upgrade sequence", func() {
		err := path.EndpointA.ChanUpgradeCancel()
		suite.Require().NoError(err)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(types.OPEN, channel.State)
		suite.Require().Equal(uint64(4), channel.UpgradeSequence)
	})

	suite.Run("try chainA upgrade now succeeds with synchronized upgrade sequences", func() {
		err := path.EndpointA.ChanUpgradeTry()
		suite.Require().NoError(err)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(types.FLUSHING, channel.State)
		suite.Require().Equal(uint64(5), channel.UpgradeSequence)
	})

	suite.Run("upgrade handshake completes successfully", func() {
		err := path.EndpointB.ChanUpgradeAck()
		suite.Require().NoError(err)

		err = path.EndpointA.ChanUpgradeConfirm()
		suite.Require().NoError(err)

		err = path.EndpointB.ChanUpgradeOpen()
		suite.Require().NoError(err)
	})

	suite.Run("assert successful upgrade expected channel state", func() {
		channelA := path.EndpointA.GetChannel()
		suite.Require().Equal(types.OPEN, channelA.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channelA.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, path.EndpointB.GetChannel().Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(5), channelA.UpgradeSequence, "upgrade sequence should be incremented")

		channelB := path.EndpointB.GetChannel()
		suite.Require().Equal(types.OPEN, channelB.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channelB.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, channelB.Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(5), channelB.UpgradeSequence, "upgrade sequence should be incremented")
	})
}

// TestChanUpgrade_CrossingHellos_UpgradeSucceeds_AfterCancelErrors verifies that under crossing hellos if upgrade
// sequences become out of sync, the upgrade can still be performed successfully after the cancel fails.
// ChainA(Sequence: 0), ChainB(Sequence 4)
// ChainA.INIT(Sequence: 1)
// ChainB.INIT(Sequence: 5)
// ChainA.TRY(Sequence: 5) // fastforward
// ChainB.TRY(ErrorReceipt: 4)
// ChainA.Cancel => Error (errorReceipt.Sequence < channel.UpgradeSequence)
// ChainB.ACK => Success
// ChainA.Confirm => Success
// ChainB.Open => Success
func (suite *KeeperTestSuite) TestChanUpgrade_CrossingHellos_UpgradeSucceeds_AfterCancelErrors() {
	var (
		historicalChannelProof []byte
		historicalUpgradeProof []byte
		proofHeight            clienttypes.Height
		path                   *ibctesting.Path
	)

	suite.Run("setup path", func() {
		path = ibctesting.NewPath(suite.chainA, suite.chainB)
		suite.coordinator.Setup(path)

		path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
		path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
	})

	suite.Run("chainA upgrade init", func() {
		err := path.EndpointA.ChanUpgradeInit()
		suite.Require().NoError(err)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(uint64(1), channel.UpgradeSequence)
	})

	suite.Run("set chainB upgrade sequence ahead of counterparty", func() {
		channel := path.EndpointB.GetChannel()
		channel.UpgradeSequence = 4
		path.EndpointB.SetChannel(channel)
	})

	suite.Run("chainB upgrade init (crossing hello)", func() {
		err := path.EndpointB.ChanUpgradeInit()
		suite.Require().NoError(err)

		channel := path.EndpointB.GetChannel()
		suite.Require().Equal(uint64(5), channel.UpgradeSequence)
	})

	suite.Run("query proofs at chainA upgrade sequence 1", func() {
		// commit block and update client on chainB
		suite.coordinator.CommitBlock(suite.chainA, suite.chainB)
		suite.Require().NoError(path.EndpointB.UpdateClient())
		// use proofs when chain A has not executed TRY yet and use them when executing TRY on chain B
		historicalChannelProof, historicalUpgradeProof, proofHeight = path.EndpointA.QueryChannelUpgradeProof()
	})

	suite.Run("chainA upgrade try (fast-forwards sequence)", func() {
		err := path.EndpointA.ChanUpgradeTry()
		suite.Require().NoError(err)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(uint64(5), channel.UpgradeSequence)
	})

	suite.Run("chainB upgrade try fails with written error receipt (4)", func() {
		// NOTE: ante handlers are bypassed here and the handler is invoked directly.
		// Thus, we set the upgrade error receipt explicitly below
		_, _, err := suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.ChanUpgradeTry(
			suite.chainB.GetContext(),
			path.EndpointB.ChannelConfig.PortID,
			path.EndpointB.ChannelID,
			path.EndpointB.GetChannelUpgrade().Fields.ConnectionHops,
			path.EndpointA.GetChannelUpgrade().Fields,
			1, // proofs queried at chainA upgrade sequence 1
			historicalChannelProof,
			historicalUpgradeProof,
			proofHeight,
		)
		suite.Require().Error(err)
		suite.assertUpgradeError(err, types.NewUpgradeError(4, types.ErrInvalidUpgradeSequence))

		suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.WriteErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, err.(*types.UpgradeError))
		suite.coordinator.CommitBlock(suite.chainB)
	})

	suite.Run("chainA upgrade cancellation fails for invalid sequence", func() {
		err := path.EndpointA.ChanUpgradeCancel()
		suite.Require().Error(err)
		suite.Require().ErrorContains(err, "invalid upgrade sequence")

		// assert channel remains in flushing state at upgrade sequence 5
		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(types.FLUSHING, channel.State)
		suite.Require().Equal(uint64(5), channel.UpgradeSequence)
	})

	suite.Run("upgrade handshake completes successfully", func() {
		err := path.EndpointB.ChanUpgradeAck()
		suite.Require().NoError(err)

		err = path.EndpointA.ChanUpgradeConfirm()
		suite.Require().NoError(err)

		err = path.EndpointB.ChanUpgradeOpen()
		suite.Require().NoError(err)
	})

	suite.Run("assert successful upgrade expected channel state", func() {
		channelA := path.EndpointA.GetChannel()
		suite.Require().Equal(types.OPEN, channelA.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channelA.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, path.EndpointB.GetChannel().Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(5), channelA.UpgradeSequence, "upgrade sequence should be incremented")

		channelB := path.EndpointB.GetChannel()
		suite.Require().Equal(types.OPEN, channelB.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channelB.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, channelB.Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(5), channelB.UpgradeSequence, "upgrade sequence should be incremented")
	})
}

func (suite *KeeperTestSuite) TestWriteUpgradeTry() {
	var (
		path            *ibctesting.Path
		proposedUpgrade types.Upgrade
	)

	testCases := []struct {
		name     string
		malleate func()
	}{
		{
			"success with no packet commitments",
			func() {},
		},
		{
			"success with packet commitments",
			func() {
				// manually set packet commitment
				sequence, err := path.EndpointB.SendPacket(suite.chainB.GetTimeoutHeight(), 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)
				suite.Require().Equal(uint64(1), sequence)
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			proposedUpgrade = path.EndpointB.GetProposedUpgrade()

			tc.malleate()

			ctx := suite.chainB.GetContext()
			upgradedChannelEnd, upgradeWithAppCallbackVersion := suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeTryChannel(
				ctx,
				path.EndpointB.ChannelConfig.PortID,
				path.EndpointB.ChannelID,
				proposedUpgrade,
				proposedUpgrade.Fields.Version,
			)

			channel := path.EndpointB.GetChannel()
			suite.Require().Equal(upgradedChannelEnd, channel)

			upgrade, found := suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			suite.Require().True(found)
			suite.Require().Equal(upgradeWithAppCallbackVersion, upgrade)
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgradeAck() {
	var (
		path                *ibctesting.Path
		counterpartyUpgrade types.Upgrade
	)

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
			"success with later upgrade sequence",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.UpgradeSequence = 10
				path.EndpointA.SetChannel(channel)

				channel = path.EndpointB.GetChannel()
				channel.UpgradeSequence = 10
				path.EndpointB.SetChannel(channel)

				suite.coordinator.CommitBlock(suite.chainA, suite.chainB)

				err := path.EndpointA.UpdateClient()
				suite.Require().NoError(err)
			},
			nil,
		},
		{
			"failure if initializing chain reinitializes before ACK",
			func() {
				err := path.EndpointA.ChanUpgradeInit()
				suite.Require().NoError(err)
			},
			commitmenttypes.ErrInvalidProof, // sequences are out of sync
		},
		{
			"channel not found",
			func() {
				path.EndpointA.ChannelID = ibctesting.InvalidID
				path.EndpointA.ChannelConfig.PortID = ibctesting.InvalidID
			},
			types.ErrChannelNotFound,
		},
		{
			"channel state is not in FLUSHING state",
			func() {
				suite.Require().NoError(path.EndpointA.SetChannelState(types.CLOSED))
			},
			types.ErrInvalidChannelState,
		},
		{
			"connection not found",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.ConnectionHops = []string{"connection-100"}
				path.EndpointA.SetChannel(channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"invalid connection state",
			func() {
				connectionEnd := path.EndpointA.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED
				path.EndpointA.SetConnection(connectionEnd)
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"upgrade not found",
			func() {
				store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(exported.ModuleName))
				store.Delete(host.ChannelUpgradeKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			types.ErrUpgradeNotFound,
		},
		{
			"fails due to upgrade incompatibility",
			func() {
				// Need to set counterparty upgrade in state and update clients to ensure
				// proofs submitted reflect the altered upgrade.
				counterpartyUpgrade.Fields.ConnectionHops = []string{ibctesting.InvalidID}
				path.EndpointB.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainB)

				err := path.EndpointA.UpdateClient()
				suite.Require().NoError(err)
			},
			types.NewUpgradeError(1, types.ErrIncompatibleCounterpartyUpgrade),
		},
		{
			"fails due to proof verification failure, counterparty channel ordering does not match expected ordering",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.Ordering = types.ORDERED
				path.EndpointA.SetChannel(channel)
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"fails due to proof verification failure, counterparty update has unexpected sequence",
			func() {
				// Decrementing NextSequenceSend is sufficient to cause the proof to fail.
				counterpartyUpgrade.NextSequenceSend--
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"fails due to mismatch in upgrade ordering",
			func() {
				upgrade := path.EndpointA.GetChannelUpgrade()
				upgrade.Fields.Ordering = types.NONE

				path.EndpointA.SetChannelUpgrade(upgrade)
			},
			types.NewUpgradeError(1, types.ErrIncompatibleCounterpartyUpgrade),
		},
		{
			"counterparty timeout has elapsed",
			func() {
				// Need to set counterparty upgrade in state and update clients to ensure
				// proofs submitted reflect the altered upgrade.
				counterpartyUpgrade.Timeout = types.NewTimeout(clienttypes.NewHeight(0, 1), 0)
				path.EndpointB.SetChannelUpgrade(counterpartyUpgrade)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)
			},
			types.NewUpgradeError(1, types.ErrTimeoutElapsed),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			// manually set packet commitment so that the chainB channel state is FLUSHING
			sequence, err := path.EndpointB.SendPacket(suite.chainB.GetTimeoutHeight(), 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			suite.Require().Equal(uint64(1), sequence)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			// ensure client is up to date to receive valid proofs
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			counterpartyUpgrade = path.EndpointB.GetChannelUpgrade()

			tc.malleate()

			channelProof, upgradeProof, proofHeight := path.EndpointB.QueryChannelUpgradeProof()

			err = suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeAck(
				suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, counterpartyUpgrade,
				channelProof, upgradeProof, proofHeight,
			)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)

				channel := path.EndpointA.GetChannel()
				// ChanUpgradeAck will set the channel state to FLUSHING
				// It will be set to FLUSHING_COMPLETE in the write function.
				suite.Require().Equal(types.FLUSHING, channel.State)
			} else {
				suite.assertUpgradeError(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestWriteChannelUpgradeAck() {
	var (
		path            *ibctesting.Path
		proposedUpgrade types.Upgrade
	)

	testCases := []struct {
		name                 string
		malleate             func()
		hasPacketCommitments bool
	}{
		{
			"success with no packet commitments",
			func() {},
			false,
		},
		{
			"success with packet commitments",
			func() {
				// manually set packet commitment
				sequence, err := path.EndpointA.SendPacket(suite.chainB.GetTimeoutHeight(), 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)
				suite.Require().Equal(uint64(1), sequence)
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			tc.malleate()

			// perform the upgrade handshake.
			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())

			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())

			ctx := suite.chainA.GetContext()
			proposedUpgrade = path.EndpointB.GetChannelUpgrade()

			suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeAckChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, proposedUpgrade)

			channel := path.EndpointA.GetChannel()
			upgrade := path.EndpointA.GetChannelUpgrade()
			suite.Require().Equal(mock.UpgradeVersion, upgrade.Fields.Version)

			if !tc.hasPacketCommitments {
				suite.Require().Equal(types.FLUSHCOMPLETE, channel.State)
			}
			counterpartyUpgrade, ok := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().True(ok)
			suite.Require().Equal(proposedUpgrade, counterpartyUpgrade)
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgrade_ReinitializedBeforeAck() {
	var path *ibctesting.Path
	suite.Run("setup path", func() {
		path = ibctesting.NewPath(suite.chainA, suite.chainB)
		suite.coordinator.Setup(path)

		path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
		path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
	})

	suite.Run("chainA upgrade init", func() {
		err := path.EndpointA.ChanUpgradeInit()
		suite.Require().NoError(err)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(uint64(1), channel.UpgradeSequence)
	})

	suite.Run("chainB upgrade try", func() {
		err := path.EndpointB.ChanUpgradeTry()
		suite.Require().NoError(err)
	})

	suite.Run("chainA upgrade init reinitialized after ack", func() {
		err := path.EndpointA.ChanUpgradeInit()
		suite.Require().NoError(err)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(uint64(2), channel.UpgradeSequence)
	})

	suite.Run("chan upgrade ack fails", func() {
		err := path.EndpointA.ChanUpgradeAck()
		suite.Require().Error(err)
	})

	suite.Run("chainB upgrade cancel", func() {
		err := path.EndpointB.ChanUpgradeCancel()
		suite.Require().NoError(err)
	})

	suite.Run("upgrade handshake succeeds on new upgrade attempt", func() {
		err := path.EndpointB.ChanUpgradeTry()
		suite.Require().NoError(err)

		err = path.EndpointA.ChanUpgradeAck()
		suite.Require().NoError(err)

		err = path.EndpointB.ChanUpgradeConfirm()
		suite.Require().NoError(err)

		err = path.EndpointA.ChanUpgradeOpen()
		suite.Require().NoError(err)
	})

	suite.Run("assert successful upgrade expected channel state", func() {
		channelA := path.EndpointA.GetChannel()
		suite.Require().Equal(types.OPEN, channelA.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channelA.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, path.EndpointB.GetChannel().Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(2), channelA.UpgradeSequence, "upgrade sequence should be incremented")

		channelB := path.EndpointB.GetChannel()
		suite.Require().Equal(types.OPEN, channelB.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channelB.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, channelB.Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(2), channelB.UpgradeSequence, "upgrade sequence should be incremented")
	})
}

func (suite *KeeperTestSuite) TestChanUpgradeConfirm() {
	var (
		path                     *ibctesting.Path
		counterpartyChannelState types.State
		counterpartyUpgrade      types.Upgrade
	)

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
			"success with later upgrade sequence",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.UpgradeSequence = 10
				path.EndpointA.SetChannel(channel)

				channel = path.EndpointB.GetChannel()
				channel.UpgradeSequence = 10
				path.EndpointB.SetChannel(channel)

				suite.coordinator.CommitBlock(suite.chainA, suite.chainB)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)
			},
			nil,
		},
		{
			"success with in-flight packets on init chain",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(path)

				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

				err := path.EndpointA.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointB.ChanUpgradeTry()
				suite.Require().NoError(err)

				seq, err := path.EndpointA.SendPacket(defaultTimeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().Equal(uint64(1), seq)
				suite.Require().NoError(err)

				err = path.EndpointA.ChanUpgradeAck()
				suite.Require().NoError(err)

				err = path.EndpointB.UpdateClient()
				suite.Require().NoError(err)

				counterpartyChannelState = path.EndpointA.GetChannel().State
				counterpartyUpgrade = path.EndpointA.GetChannelUpgrade()
			},
			nil,
		},
		{
			"success with in-flight packets on try chain",
			func() {
				portID, channelID := path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID
				suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.SetPacketCommitment(suite.chainB.GetContext(), portID, channelID, 1, []byte("hash"))
			},
			nil,
		},
		{
			"channel not found",
			func() {
				path.EndpointB.ChannelID = ibctesting.InvalidID
				path.EndpointB.ChannelConfig.PortID = ibctesting.InvalidID
			},
			types.ErrChannelNotFound,
		},
		{
			"channel is not in FLUSHING state",
			func() {
				err := path.EndpointB.SetChannelState(types.CLOSED)
				suite.Require().NoError(err)
			},
			types.ErrInvalidChannelState,
		},
		{
			"invalid counterparty channel state",
			func() {
				counterpartyChannelState = types.CLOSED
			},
			types.ErrInvalidCounterparty,
		},
		{
			"connection not found",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.ConnectionHops = []string{"connection-100"}
				path.EndpointB.SetChannel(channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"invalid connection state",
			func() {
				connectionEnd := path.EndpointB.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED
				path.EndpointB.SetConnection(connectionEnd)
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"fails due to proof verification failure, counterparty channel ordering does not match expected ordering",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.Ordering = types.ORDERED
				path.EndpointA.SetChannel(channel)

				suite.coordinator.CommitBlock(suite.chainA)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"fails due to mismatch in upgrade ordering",
			func() {
				upgrade := path.EndpointA.GetChannelUpgrade()
				upgrade.Fields.Ordering = types.NONE

				path.EndpointA.SetChannelUpgrade(upgrade)

				suite.coordinator.CommitBlock(suite.chainA)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"counterparty timeout has elapsed",
			func() {
				// Need to set counterparty upgrade in state and update clients to ensure
				// proofs submitted reflect the altered upgrade.
				counterpartyUpgrade.Timeout = types.NewTimeout(clienttypes.NewHeight(0, 1), 0)
				path.EndpointA.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainA)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)
			},
			types.NewUpgradeError(1, types.ErrTimeoutElapsed),
		},
		{
			"upgrade not found",
			func() {
				path.EndpointB.Chain.DeleteKey(host.ChannelUpgradeKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID))
			},
			types.ErrUpgradeNotFound,
		},
		{
			"upgrades are not compatible",
			func() {
				// the expected upgrade version is mock-version-v2
				counterpartyUpgrade.Fields.Version = fmt.Sprintf("%s-v3", mock.Version)
				path.EndpointA.SetChannelUpgrade(counterpartyUpgrade)

				suite.coordinator.CommitBlock(suite.chainA)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)
			},
			types.NewUpgradeError(1, types.ErrIncompatibleCounterpartyUpgrade),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			counterpartyChannelState = path.EndpointA.GetChannel().State
			counterpartyUpgrade = path.EndpointA.GetChannelUpgrade()

			tc.malleate()

			channelProof, upgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

			err = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeConfirm(
				suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, counterpartyChannelState, counterpartyUpgrade,
				channelProof, upgradeProof, proofHeight,
			)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
			} else {
				suite.assertUpgradeError(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestWriteUpgradeConfirm() {
	var (
		path            *ibctesting.Path
		proposedUpgrade types.Upgrade
	)

	testCases := []struct {
		name                 string
		malleate             func()
		hasPacketCommitments bool
	}{
		{
			"success with no packet commitments",
			func() {},
			false,
		},
		{
			"success with packet commitments",
			func() {
				// manually set packet commitment
				sequence, err := path.EndpointA.SendPacket(suite.chainB.GetTimeoutHeight(), 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)
				suite.Require().Equal(uint64(1), sequence)
			},
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			tc.malleate()

			// perform the upgrade handshake.
			suite.Require().NoError(path.EndpointB.ChanUpgradeInit())

			suite.Require().NoError(path.EndpointA.ChanUpgradeTry())

			suite.Require().NoError(path.EndpointB.ChanUpgradeAck())

			ctx := suite.chainA.GetContext()
			proposedUpgrade = path.EndpointB.GetChannelUpgrade()

			suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeConfirmChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, proposedUpgrade)

			channel := path.EndpointA.GetChannel()
			upgrade := path.EndpointA.GetChannelUpgrade()
			suite.Require().Equal(mock.UpgradeVersion, upgrade.Fields.Version)

			if !tc.hasPacketCommitments {
				suite.Require().Equal(types.FLUSHCOMPLETE, channel.State)
			} else {
				suite.Require().Equal(types.FLUSHING, channel.State)
			}
			counterpartyUpgrade, ok := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().True(ok, "counterparty upgrade should be present")
			suite.Require().Equal(proposedUpgrade, counterpartyUpgrade)
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgradeOpen() {
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
			"success: counterparty in flushcomplete",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(path)

				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

				// Need to create a packet commitment on A so as to keep it from going to FLUSHCOMPLETE if no inflight packets exist.
				sequence, err := path.EndpointA.SendPacket(defaultTimeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)
				packet := types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
				err = path.EndpointB.RecvPacket(packet)
				suite.Require().NoError(err)

				err = path.EndpointA.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointB.ChanUpgradeTry()
				suite.Require().NoError(err)

				err = path.EndpointA.ChanUpgradeAck()
				suite.Require().NoError(err)

				err = path.EndpointB.ChanUpgradeConfirm()
				suite.Require().NoError(err)

				err = path.EndpointA.AcknowledgePacket(packet, ibctesting.MockAcknowledgement)
				suite.Require().NoError(err)

				// cause the packet commitment on chain A to be deleted and the channel state to be updated to FLUSHCOMPLETE.
				suite.coordinator.CommitBlock(suite.chainA, suite.chainB)
				suite.Require().NoError(path.EndpointA.UpdateClient())
			},
			nil,
		},
		{
			"success: counterparty initiated new upgrade after opening",
			func() {
				// create reason to upgrade
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion + "additional upgrade"

				err := path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)
			},
			nil,
		},
		{
			"success: counterparty upgrade sequence is incorrect",
			func() {
				counterpartyCh := path.EndpointB.GetChannel()
				counterpartyCh.UpgradeSequence--
				path.EndpointB.SetChannel(counterpartyCh)
			},
			types.ErrInvalidUpgradeSequence,
		},
		{
			"channel not found",
			func() {
				path.EndpointA.ChannelConfig.PortID = ibctesting.InvalidID
			},
			types.ErrChannelNotFound,
		},
		{
			"channel state is not FLUSHCOMPLETE",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHING
				path.EndpointA.SetChannel(channel)
			},
			types.ErrInvalidChannelState,
		},
		{
			"connection not found",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.ConnectionHops = []string{"connection-100"}
				path.EndpointA.SetChannel(channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"invalid connection state",
			func() {
				connectionEnd := path.EndpointA.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED
				path.EndpointA.SetConnection(connectionEnd)
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"invalid counterparty channel state",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.State = types.CLOSED
				path.EndpointB.SetChannel(channel)
			},
			types.ErrInvalidCounterparty,
		},
	}

	// Create an initial path used only to invoke a ChanOpenInit handshake.
	// This bumps the channel identifier generated for chain A on the
	// next path used to run the upgrade handshake.
	// See issue 4062.
	path = ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path)
	suite.Require().NoError(path.EndpointA.ChanOpenInit())

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeConfirm()
			suite.Require().NoError(err)

			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			tc.malleate()

			channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			channelProof, proofHeight := path.EndpointB.QueryProof(channelKey)

			err = suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeOpen(
				suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID,
				path.EndpointB.GetChannel().State, path.EndpointB.GetChannel().UpgradeSequence, channelProof, proofHeight,
			)

			if tc.expError == nil {
				suite.Require().NoError(err)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestWriteUpgradeOpenChannel() {
	var path *ibctesting.Path

	testCases := []struct {
		name     string
		malleate func()
		expPanic bool
	}{
		{
			name:     "success",
			malleate: func() {},
			expPanic: false,
		},
		{
			name: "channel not found",
			malleate: func() {
				path.EndpointA.Chain.DeleteKey(host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			expPanic: true,
		},
		{
			name: "upgrade not found",
			malleate: func() {
				path.EndpointA.Chain.DeleteKey(host.ChannelUpgradeKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			expPanic: true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// Need to create a packet commitment on A so as to keep it from going to OPEN if no inflight packets exist.
			sequence, err := path.EndpointA.SendPacket(defaultTimeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet := types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())
			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())
			suite.Require().NoError(path.EndpointA.ChanUpgradeAck())
			suite.Require().NoError(path.EndpointB.ChanUpgradeConfirm())

			// Ack packet to delete packet commitment before calling WriteUpgradeOpenChannel
			err = path.EndpointA.AcknowledgePacket(packet, ibctesting.MockAcknowledgement)
			suite.Require().NoError(err)

			ctx := suite.chainA.GetContext()

			tc.malleate()

			if tc.expPanic {
				suite.Require().Panics(func() {
					suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeOpenChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				})
			} else {
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeOpenChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channel := path.EndpointA.GetChannel()

				// Assert that channel state has been updated
				suite.Require().Equal(types.OPEN, channel.State)
				suite.Require().Equal(mock.UpgradeVersion, channel.Version)

				// Assert that state stored for upgrade has been deleted
				upgrade, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().Equal(types.Upgrade{}, upgrade)
				suite.Require().False(found)

				counterpartyUpgrade, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().Equal(types.Upgrade{}, counterpartyUpgrade)
				suite.Require().False(found)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestWriteUpgradeOpenChannel_Ordering() {
	var path *ibctesting.Path

	testCases := []struct {
		name        string
		malleate    func()
		preUpgrade  func()
		postUpgrade func()
	}{
		{
			name: "success: ORDERED -> UNORDERED",
			malleate: func() {
				path.EndpointA.ChannelConfig.Order = types.ORDERED
				path.EndpointB.ChannelConfig.Order = types.ORDERED

				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Ordering = types.UNORDERED
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Ordering = types.UNORDERED
			},
			preUpgrade: func() {
				ctx := suite.chainA.GetContext()

				// assert that NextSeqAck is incremented to 2 because channel is still ordered
				seq, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceAck(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(2), seq)

				// assert that NextSeqRecv is incremented to 2 because channel is still ordered
				seq, found = suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceRecv(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(2), seq)

				// Assert that pruning sequence start has not been initialized.
				suite.Require().False(suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.HasPruningSequenceStart(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))

				// Assert that recv start sequence has not been set
				counterpartyNextSequenceSend, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetRecvStartSequence(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found)
				suite.Require().Equal(uint64(0), counterpartyNextSequenceSend)
			},
			postUpgrade: func() {
				channel := path.EndpointA.GetChannel()
				ctx := suite.chainA.GetContext()

				// Assert that channel state has been updated
				suite.Require().Equal(types.OPEN, channel.State)
				suite.Require().Equal(types.UNORDERED, channel.Ordering)

				// assert that NextSeqRecv is now 1, because channel is now UNORDERED
				seq, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceRecv(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), seq)

				// assert that NextSeqAck is now 1, because channel is now UNORDERED
				seq, found = suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceAck(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), seq)

				// Assert that pruning sequence start has been initialized (set to 1)
				suite.Require().True(suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.HasPruningSequenceStart(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
				pruningSeq, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetPruningSequenceStart(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), pruningSeq)

				// Assert that the recv start sequence has been set correctly
				counterpartySequenceSend, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetRecvStartSequence(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(2), counterpartySequenceSend)
			},
		},
		{
			name: "success: UNORDERED -> ORDERED",
			malleate: func() {
				path.EndpointA.ChannelConfig.Order = types.UNORDERED
				path.EndpointB.ChannelConfig.Order = types.UNORDERED

				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Ordering = types.ORDERED
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Ordering = types.ORDERED
			},
			preUpgrade: func() {
				ctx := suite.chainA.GetContext()

				// assert that NextSeqRecv  is 1 because channel is UNORDERED
				seq, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceRecv(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), seq)

				// assert that NextSeqAck is 1 because channel is UNORDERED
				seq, found = suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceAck(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), seq)

				// Assert that pruning sequence start has not been initialized.
				suite.Require().False(suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.HasPruningSequenceStart(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))

				// Assert that recv start sequence has not been set
				counterpartyNextSequenceSend, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetRecvStartSequence(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found)
				suite.Require().Equal(uint64(0), counterpartyNextSequenceSend)
			},
			postUpgrade: func() {
				channel := path.EndpointA.GetChannel()
				ctx := suite.chainA.GetContext()

				// Assert that channel state has been updated
				suite.Require().Equal(types.OPEN, channel.State)
				suite.Require().Equal(types.ORDERED, channel.Ordering)

				// assert that NextSeqRecv is incremented to 2, because channel is now ORDERED
				// NextSeqRecv updated in WriteUpgradeOpenChannel to latest sequence (one packet sent) + 1
				seq, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceRecv(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(2), seq)

				// assert that NextSeqAck is incremented to 2 because channel is now ORDERED
				seq, found = suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceAck(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(2), seq)

				// Assert that pruning sequence start has been initialized (set to 1)
				suite.Require().True(suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.HasPruningSequenceStart(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
				pruningSeq, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetPruningSequenceStart(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), pruningSeq)

				// Assert that the recv start sequence has been set correctly
				counterpartySequenceSend, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetRecvStartSequence(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(2), counterpartySequenceSend)
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			suite.coordinator.Setup(path)

			// Need to create a packet commitment on A so as to keep it from going to OPEN if no inflight packets exist.
			sequenceA, err := path.EndpointA.SendPacket(defaultTimeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packetA := types.NewPacket(ibctesting.MockPacketData, sequenceA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointB.RecvPacket(packetA)
			suite.Require().NoError(err)

			// send second packet from B to A
			sequenceB, err := path.EndpointB.SendPacket(defaultTimeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packetB := types.NewPacket(ibctesting.MockPacketData, sequenceB, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointA.RecvPacket(packetB)
			suite.Require().NoError(err)

			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())
			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())
			suite.Require().NoError(path.EndpointA.ChanUpgradeAck())
			suite.Require().NoError(path.EndpointB.ChanUpgradeConfirm())

			// Ack packets to delete packet commitments before calling WriteUpgradeOpenChannel
			err = path.EndpointA.AcknowledgePacket(packetA, ibctesting.MockAcknowledgement)
			suite.Require().NoError(err)

			err = path.EndpointB.AcknowledgePacket(packetB, ibctesting.MockAcknowledgement)
			suite.Require().NoError(err)

			// pre upgrade assertions
			tc.preUpgrade()

			tc.malleate()
			suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeOpenChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

			// post upgrade assertions
			tc.postUpgrade()

			// Assert that state stored for upgrade has been deleted
			upgrade, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().Equal(types.Upgrade{}, upgrade)
			suite.Require().False(found)

			counterpartyUpgrade, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().Equal(types.Upgrade{}, counterpartyUpgrade)
			suite.Require().False(found)
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgradeCancel() {
	var (
		path              *ibctesting.Path
		errorReceipt      types.ErrorReceipt
		errorReceiptProof []byte
		proofHeight       clienttypes.Height
	)

	tests := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			name: "success with flushing state",
			malleate: func() {
			},
			expError: nil,
		},
		{
			name: "success with flush complete state",
			malleate: func() {
				err := path.EndpointA.SetChannelState(types.FLUSHCOMPLETE)
				suite.Require().NoError(err)

				var ok bool
				errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(ok)

				// the error receipt upgrade sequence and the channel upgrade sequence must match
				errorReceipt.Sequence = path.EndpointA.GetChannel().UpgradeSequence

				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, errorReceipt)

				suite.coordinator.CommitBlock(suite.chainB)

				suite.Require().NoError(path.EndpointA.UpdateClient())

				upgradeErrorReceiptKey := host.ChannelUpgradeErrorKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				errorReceiptProof, proofHeight = suite.chainB.QueryProof(upgradeErrorReceiptKey)
			},
			expError: nil,
		},
		{
			name: "upgrade cannot be cancelled in FLUSHCOMPLETE with invalid error receipt",
			malleate: func() {
				err := path.EndpointA.SetChannelState(types.FLUSHCOMPLETE)
				suite.Require().NoError(err)

				errorReceiptProof = nil
			},
			expError: commitmenttypes.ErrInvalidProof,
		},
		{
			name: "channel not found",
			malleate: func() {
				path.EndpointA.Chain.DeleteKey(host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			expError: types.ErrChannelNotFound,
		},
		{
			name: "upgrade not found",
			malleate: func() {
				path.EndpointA.Chain.DeleteKey(host.ChannelUpgradeKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			expError: types.ErrUpgradeNotFound,
		},
		{
			name: "error receipt sequence less than channel upgrade sequence",
			malleate: func() {
				var ok bool
				errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(ok)

				errorReceipt.Sequence = path.EndpointA.GetChannel().UpgradeSequence - 1

				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, errorReceipt)

				suite.coordinator.CommitBlock(suite.chainB)

				suite.Require().NoError(path.EndpointA.UpdateClient())

				upgradeErrorReceiptKey := host.ChannelUpgradeErrorKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				errorReceiptProof, proofHeight = suite.chainB.QueryProof(upgradeErrorReceiptKey)
			},
			expError: types.ErrInvalidUpgradeSequence,
		},
		{
			name: "error receipt sequence greater than channel upgrade sequence when channel in FLUSHCOMPLETE",
			malleate: func() {
				err := path.EndpointA.SetChannelState(types.FLUSHCOMPLETE)
				suite.Require().NoError(err)
			},
			expError: types.ErrInvalidUpgradeSequence,
		},
		{
			name: "error receipt sequence smaller than channel upgrade sequence when channel in FLUSHCOMPLETE",
			malleate: func() {
				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHCOMPLETE
				path.EndpointA.SetChannel(channel)

				var ok bool
				errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(ok)

				errorReceipt.Sequence = path.EndpointA.GetChannel().UpgradeSequence - 1

				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, errorReceipt)

				suite.coordinator.CommitBlock(suite.chainB)

				suite.Require().NoError(path.EndpointA.UpdateClient())

				upgradeErrorReceiptKey := host.ChannelUpgradeErrorKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				errorReceiptProof, proofHeight = suite.chainB.QueryProof(upgradeErrorReceiptKey)
			},
			expError: types.ErrInvalidUpgradeSequence,
		},
		{
			name: "connection not found",
			malleate: func() {
				channel := path.EndpointA.GetChannel()
				channel.ConnectionHops = []string{"connection-100"}
				path.EndpointA.SetChannel(channel)
			},
			expError: connectiontypes.ErrConnectionNotFound,
		},
		{
			name: "channel is in flush complete, error verification failed",
			malleate: func() {
				var ok bool
				errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(ok)

				errorReceipt.Message = ibctesting.InvalidID

				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, errorReceipt)
				suite.coordinator.CommitBlock(suite.chainB)
			},
			expError: commitmenttypes.ErrInvalidProof,
		},
		{
			name: "error verification failed",
			malleate: func() {
				var ok bool
				errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(ok)

				errorReceipt.Message = ibctesting.InvalidID
				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, errorReceipt)
				suite.coordinator.CommitBlock(suite.chainB)
			},
			expError: commitmenttypes.ErrInvalidProof,
		},
		{
			name: "error verification failed with empty proof",
			malleate: func() {
				errorReceiptProof = nil
			},
			expError: commitmenttypes.ErrInvalidProof,
		},
	}

	for _, tc := range tests {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())

			// cause the upgrade to fail on chain b so an error receipt is written.
			// if the counterparty (chain A) upgrade sequence is less than the current sequence, (chain B)
			// an upgrade error will be returned by chain B during ChanUpgradeTry.
			channel := path.EndpointA.GetChannel()
			channel.UpgradeSequence = 1
			path.EndpointA.SetChannel(channel)

			channel = path.EndpointB.GetChannel()
			channel.UpgradeSequence = 2
			path.EndpointB.SetChannel(channel)

			suite.Require().NoError(path.EndpointA.UpdateClient())
			suite.Require().NoError(path.EndpointB.UpdateClient())

			// error receipt is written to chain B here.
			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())

			suite.Require().NoError(path.EndpointA.UpdateClient())

			upgradeErrorReceiptKey := host.ChannelUpgradeErrorKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			errorReceiptProof, proofHeight = suite.chainB.QueryProof(upgradeErrorReceiptKey)

			var ok bool
			errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			suite.Require().True(ok)

			channel = path.EndpointA.GetChannel()
			channel.State = types.FLUSHING
			path.EndpointA.SetChannel(channel)

			tc.malleate()

			err := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeCancel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, errorReceipt, errorReceiptProof, proofHeight)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

// TestChanUpgrade_UpgradeSucceeds_AfterCancel verifies that if upgrade sequences
// become out of sync, the upgrade can still be performed successfully after the upgrade is cancelled.
func (suite *KeeperTestSuite) TestChanUpgrade_UpgradeSucceeds_AfterCancel() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)

	path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
	path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

	suite.Require().NoError(path.EndpointA.ChanUpgradeInit())

	// cause the upgrade to fail on chain b so an error receipt is written.
	// if the counterparty (chain A) upgrade sequence is less than the current sequence, (chain B)
	// an upgrade error will be returned by chain B during ChanUpgradeTry.
	channel := path.EndpointA.GetChannel()
	channel.UpgradeSequence = 1
	path.EndpointA.SetChannel(channel)

	channel = path.EndpointB.GetChannel()
	channel.UpgradeSequence = 5
	path.EndpointB.SetChannel(channel)

	suite.Require().NoError(path.EndpointA.UpdateClient())
	suite.Require().NoError(path.EndpointB.UpdateClient())

	// error receipt is written to chain B here.
	suite.Require().NoError(path.EndpointB.ChanUpgradeTry())

	suite.Require().NoError(path.EndpointA.UpdateClient())

	var errorReceipt types.ErrorReceipt
	suite.T().Run("error receipt written", func(t *testing.T) {
		var ok bool
		errorReceipt, ok = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
		suite.Require().True(ok)
	})

	suite.T().Run("upgrade cancelled successfully", func(t *testing.T) {
		upgradeErrorReceiptKey := host.ChannelUpgradeErrorKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
		errorReceiptProof, proofHeight := suite.chainB.QueryProof(upgradeErrorReceiptKey)

		err := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeCancel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, errorReceipt, errorReceiptProof, proofHeight)
		suite.Require().NoError(err)

		// need to explicitly call WriteUpgradeOpenChannel as this usually would happen in the msg server layer.
		suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeCancelChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, errorReceipt.Sequence)

		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(types.OPEN, channel.State)

		suite.T().Run("verify upgrade sequence fastforwards to channelB sequence", func(t *testing.T) {
			suite.Require().Equal(uint64(5), channel.UpgradeSequence)
		})
	})

	suite.T().Run("successfully completes upgrade", func(t *testing.T) {
		suite.Require().NoError(path.EndpointA.ChanUpgradeInit())
		suite.Require().NoError(path.EndpointB.ChanUpgradeTry())
		suite.Require().NoError(path.EndpointA.ChanUpgradeAck())
		suite.Require().NoError(path.EndpointB.ChanUpgradeConfirm())
		suite.Require().NoError(path.EndpointA.ChanUpgradeOpen())
	})

	suite.T().Run("channel in expected state", func(t *testing.T) {
		channel := path.EndpointA.GetChannel()
		suite.Require().Equal(types.OPEN, channel.State, "channel should be in OPEN state")
		suite.Require().Equal(mock.UpgradeVersion, channel.Version, "version should be correctly upgraded")
		suite.Require().Equal(mock.UpgradeVersion, path.EndpointB.GetChannel().Version, "version should be correctly upgraded")
		suite.Require().Equal(uint64(6), channel.UpgradeSequence, "upgrade sequence should be incremented")
		suite.Require().Equal(uint64(6), path.EndpointB.GetChannel().UpgradeSequence, "upgrade sequence should be incremented on counterparty")
	})
}

func (suite *KeeperTestSuite) TestWriteUpgradeCancelChannel() {
	var path *ibctesting.Path

	testCases := []struct {
		name     string
		malleate func()
		expPanic bool
	}{
		{
			name:     "success",
			malleate: func() {},
			expPanic: false,
		},
		{
			name: "channel not found",
			malleate: func() {
				path.EndpointA.Chain.DeleteKey(host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			expPanic: true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())

			// cause the upgrade to fail on chain b so an error receipt is written.
			// if the counterparty (chain A) upgrade sequence is less than the current sequence, (chain B)
			// an upgrade error will be returned by chain B during ChanUpgradeTry.
			channel := path.EndpointA.GetChannel()
			channel.UpgradeSequence = 1
			path.EndpointA.SetChannel(channel)

			channel = path.EndpointB.GetChannel()
			channel.UpgradeSequence = 2
			path.EndpointB.SetChannel(channel)

			err := path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			errorReceipt, ok := suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			suite.Require().True(ok)

			ctx := suite.chainA.GetContext()
			tc.malleate()

			if tc.expPanic {
				suite.Require().Panics(func() {
					suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeCancelChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, errorReceipt.Sequence)
				})
			} else {
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteUpgradeCancelChannel(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, errorReceipt.Sequence)

				channel = path.EndpointA.GetChannel()

				// Verify that channel has been restored to previous state
				suite.Require().Equal(types.OPEN, channel.State)
				suite.Require().Equal(mock.Version, channel.Version)
				suite.Require().Equal(errorReceipt.Sequence, channel.UpgradeSequence)

				// Assert that state stored for upgrade has been deleted
				upgrade, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().Equal(types.Upgrade{}, upgrade)
				suite.Require().False(found)

				counterpartyUpgrade, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().Equal(types.Upgrade{}, counterpartyUpgrade)
				suite.Require().False(found)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgradeTimeout() {
	var (
		path         *ibctesting.Path
		channelProof []byte
		proofHeight  exported.Height
	)

	timeoutUpgrade := func() {
		upgrade := path.EndpointA.GetProposedUpgrade()
		upgrade.Timeout = types.NewTimeout(clienttypes.ZeroHeight(), 1)
		path.EndpointA.SetChannelUpgrade(upgrade)
		suite.Require().NoError(path.EndpointB.UpdateClient())
	}

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success: proof timestamp has passed",
			func() {
				timeoutUpgrade()

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)
			},
			nil,
		},
		{
			"channel not found",
			func() {
				path.EndpointA.ChannelID = ibctesting.InvalidID
			},
			types.ErrChannelNotFound,
		},
		{
			"channel state is not in FLUSHING or FLUSHINGCOMPLETE state",
			func() {
				suite.Require().NoError(path.EndpointA.SetChannelState(types.OPEN))
			},
			types.ErrInvalidChannelState,
		},
		{
			"current upgrade not found",
			func() {
				suite.chainA.DeleteKey(host.ChannelUpgradeKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			types.ErrUpgradeNotFound,
		},
		{
			"connection not found",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.ConnectionHops[0] = ibctesting.InvalidID
				path.EndpointA.SetChannel(channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"connection not open",
			func() {
				connectionEnd := path.EndpointA.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED
				path.EndpointA.SetConnection(connectionEnd)
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"unable to retrieve timestamp at proof height",
			func() {
				// TODO: revert this when the upgrade timeout is not hard coded to 1000
				proofHeight = clienttypes.NewHeight(clienttypes.ParseChainID(suite.chainA.ChainID), uint64(suite.chainA.GetContext().BlockHeight())+1000)
			},
			clienttypes.ErrConsensusStateNotFound,
		},
		{
			"invalid channel state proof",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.State = types.OPEN
				path.EndpointB.SetChannel(channel)

				timeoutUpgrade()

				suite.Require().NoError(path.EndpointA.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)

				// modify state so the proof becomes invalid.
				channel.State = types.FLUSHING
				path.EndpointB.SetChannel(channel)
				suite.coordinator.CommitNBlocks(suite.chainB, 1)
			},
			commitmenttypes.ErrInvalidProof,
		},
		{
			"invalid counterparty upgrade sequence",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.UpgradeSequence = path.EndpointA.GetChannel().UpgradeSequence - 1
				path.EndpointB.SetChannel(channel)

				timeoutUpgrade()

				suite.Require().NoError(path.EndpointA.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)
			},
			types.ErrInvalidUpgradeSequence,
		},
		{
			"timeout timestamp has not passed",
			func() {
				upgrade := path.EndpointA.GetProposedUpgrade()
				upgrade.Timeout.Timestamp = math.MaxUint64
				path.EndpointA.SetChannelUpgrade(upgrade)

				suite.Require().NoError(path.EndpointB.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)
			},
			types.ErrTimeoutNotReached,
		},
		{
			"counterparty channel state is not OPEN or FLUSHING (crossing hellos)",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.State = types.FLUSHCOMPLETE
				path.EndpointB.SetChannel(channel)

				timeoutUpgrade()

				suite.Require().NoError(path.EndpointA.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)
			},
			types.ErrInvalidCounterparty,
		},
		{
			"counterparty proposed connection invalid",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.State = types.OPEN
				path.EndpointB.SetChannel(channel)

				timeoutUpgrade()

				upgrade := path.EndpointA.GetChannelUpgrade()
				upgrade.Fields.ConnectionHops = []string{"connection-100"}
				path.EndpointA.SetChannelUpgrade(upgrade)

				suite.Require().NoError(path.EndpointA.UpdateClient())
				suite.Require().NoError(path.EndpointB.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"counterparty channel already upgraded",
			func() {
				// put chainA channel into OPEN state since both sides are in FLUSHCOMPLETE
				suite.Require().NoError(path.EndpointB.ChanUpgradeConfirm())

				timeoutUpgrade()

				suite.Require().NoError(path.EndpointA.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)
			},
			types.ErrUpgradeTimeoutFailed,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			expPass := tc.expError == nil

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())
			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())
			suite.Require().NoError(path.EndpointA.ChanUpgradeAck())

			channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			channelProof, proofHeight = path.EndpointB.QueryProof(channelKey)

			tc.malleate()

			err := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ChanUpgradeTimeout(
				suite.chainA.GetContext(),
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				path.EndpointB.GetChannel(),
				channelProof,
				proofHeight,
			)

			if expPass {
				suite.Require().NoError(err)
			} else {
				suite.assertUpgradeError(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestStartFlush() {
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
			"channel not found",
			func() {
				path.EndpointB.ChannelID = "invalid-channel"
				path.EndpointB.ChannelConfig.PortID = "invalid-port"
			},
			types.ErrChannelNotFound,
		},
		{
			"connection not found",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.ConnectionHops[0] = ibctesting.InvalidID
				path.EndpointB.SetChannel(channel)
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"connection state is not in OPEN state",
			func() {
				conn := path.EndpointB.GetConnection()
				conn.State = connectiontypes.INIT
				path.EndpointB.SetConnection(conn)
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"next sequence send not found",
			func() {
				// Delete next sequence send key from store
				store := suite.chainB.GetContext().KVStore(suite.chainB.GetSimApp().GetKey(exported.StoreKey))
				store.Delete(host.NextSequenceSendKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID))
			},
			types.ErrSequenceSendNotFound,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			// crossing hellos so that the upgrade is created on chain B.
			// the ChanUpgradeInit sub protocol is also called when it is not a crossing hello situation.
			err = path.EndpointB.ChanUpgradeInit()
			suite.Require().NoError(err)

			upgrade := path.EndpointB.GetChannelUpgrade()

			tc.malleate()

			err = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.StartFlushing(
				suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, &upgrade,
			)

			if tc.expError != nil {
				suite.assertUpgradeError(err, tc.expError)
			} else {
				channel := path.EndpointB.GetChannel()

				nextSequenceSend, ok := suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceSend(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(ok)

				suite.Require().Equal(types.FLUSHING, channel.State)
				suite.Require().Equal(nextSequenceSend, upgrade.NextSequenceSend)

				expectedTimeoutTimestamp := types.DefaultTimeout.Timestamp + uint64(suite.chainB.GetContext().BlockTime().UnixNano())
				suite.Require().Equal(expectedTimeoutTimestamp, upgrade.Timeout.Timestamp)
				suite.Require().Equal(clienttypes.ZeroHeight(), upgrade.Timeout.Height, "only timestamp should be set")
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestValidateUpgradeFields() {
	var (
		proposedUpgrade *types.UpgradeFields
		path            *ibctesting.Path
	)
	tests := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			name: "change channel version",
			malleate: func() {
				proposedUpgrade.Version = mock.UpgradeVersion
			},
			expPass: true,
		},
		{
			name: "change connection hops",
			malleate: func() {
				path := ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(path)
				proposedUpgrade.ConnectionHops = []string{path.EndpointA.ConnectionID}
			},
			expPass: true,
		},
		{
			name:     "fails with unmodified fields",
			malleate: func() {},
			expPass:  false,
		},
		{
			name: "fails when connection is not set",
			malleate: func() {
				storeKey := suite.chainA.GetSimApp().GetKey(exported.StoreKey)
				kvStore := suite.chainA.GetContext().KVStore(storeKey)
				kvStore.Delete(host.ConnectionKey(ibctesting.FirstConnectionID))
			},
			expPass: false,
		},
		{
			name: "fails when connection is not open",
			malleate: func() {
				connection := path.EndpointA.GetConnection()
				connection.State = connectiontypes.UNINITIALIZED
				path.EndpointA.SetConnection(connection)
			},
			expPass: false,
		},
		{
			name: "fails when connection versions do not exist",
			malleate: func() {
				// update channel version first so that existing channel end is not identical to proposed upgrade
				proposedUpgrade.Version = mock.UpgradeVersion

				connection := path.EndpointA.GetConnection()
				connection.Versions = []*connectiontypes.Version{}
				path.EndpointA.SetConnection(connection)
			},
			expPass: false,
		},
		{
			name: "fails when connection version does not support the new ordering",
			malleate: func() {
				// update channel version first so that existing channel end is not identical to proposed upgrade
				proposedUpgrade.Version = mock.UpgradeVersion

				connection := path.EndpointA.GetConnection()
				connection.Versions = []*connectiontypes.Version{
					connectiontypes.NewVersion("1", []string{"ORDER_ORDERED"}),
				}
				path.EndpointA.SetConnection(connection)
			},
			expPass: false,
		},
	}

	for _, tc := range tests {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			existingChannel := path.EndpointA.GetChannel()
			proposedUpgrade = &types.UpgradeFields{
				Ordering:       existingChannel.Ordering,
				ConnectionHops: existingChannel.ConnectionHops,
				Version:        existingChannel.Version,
			}

			tc.malleate()

			err := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.ValidateSelfUpgradeFields(suite.chainA.GetContext(), *proposedUpgrade, existingChannel)
			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) assertUpgradeError(actualError, expError error) {
	suite.Require().Error(actualError)

	if expUpgradeError, ok := expError.(*types.UpgradeError); ok {
		upgradeError, ok := actualError.(*types.UpgradeError)
		suite.Require().True(ok)
		suite.Require().Equal(expUpgradeError.GetErrorReceipt(), upgradeError.GetErrorReceipt())
	}

	suite.Require().True(errorsmod.IsOf(actualError, expError), fmt.Sprintf("expected error: %s, actual error: %s", expError, actualError))
}

// TestAbortUpgrade tests that when the channel handshake is aborted, the channel state
// is restored the previous state and that an error receipt is written, and upgrade state which
// is no longer required is deleted.
func (suite *KeeperTestSuite) TestAbortUpgrade() {
	var (
		path         *ibctesting.Path
		upgradeError error
	)

	tests := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			name:     "success",
			malleate: func() {},
			expPass:  true,
		},
		{
			name: "regular error",
			malleate: func() {
				// in app callbacks error receipts should still be written if a regular error is returned.
				// i.e. not an instance of `types.UpgradeError`
				upgradeError = types.ErrInvalidUpgrade
			},
			expPass: true,
		},
		{
			name: "channel does not exist",
			malleate: func() {
				suite.chainA.DeleteKey(host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			expPass: false,
		},
		{
			name: "fails with nil upgrade error",
			malleate: func() {
				upgradeError = nil
			},
			expPass: false,
		},
	}

	for _, tc := range tests {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			channelKeeper := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper

			path.EndpointA.ChannelConfig.Version = mock.UpgradeVersion
			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())

			// fetch the upgrade before abort for assertions later on.
			actualUpgrade, ok := channelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().True(ok, "upgrade should be found")

			upgradeError = types.NewUpgradeError(1, types.ErrInvalidChannel)

			tc.malleate()

			if tc.expPass {

				ctx := suite.chainA.GetContext()

				suite.Require().NotPanics(func() {
					channelKeeper.MustAbortUpgrade(ctx, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgradeError)
				})

				channel, found := channelKeeper.GetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "channel should be found")

				suite.Require().Equal(types.OPEN, channel.State, "channel state should be %s", types.OPEN.String())

				_, found = channelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "upgrade info should be deleted")

				errorReceipt, found := channelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "error receipt should be found")

				if ue, ok := upgradeError.(*types.UpgradeError); ok {
					suite.Require().Equal(ue.GetErrorReceipt(), errorReceipt, "error receipt does not match expected error receipt")
				}
			} else {

				suite.Require().Panics(func() {
					channelKeeper.MustAbortUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgradeError)
				})

				channel, found := channelKeeper.GetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				if found { // test cases uses a channel that exists
					suite.Require().Equal(types.OPEN, channel.State, "channel state should not be restored to %s", types.OPEN.String())
				}

				_, found = channelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "error receipt should not be found")

				upgrade, found := channelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				if found { // this should be all test cases except for when the upgrade is explicitly deleted.
					suite.Require().Equal(actualUpgrade, upgrade, "upgrade info should not be deleted")
				}
			}
		})
	}
}

func (suite *KeeperTestSuite) TestCheckForUpgradeCompatibility() {
	var (
		path                      *ibctesting.Path
		upgradeFields             types.UpgradeFields
		counterpartyUpgradeFields types.UpgradeFields
	)

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
			"upgrade ordering is not the same on both sides",
			func() {
				upgradeFields.Ordering = types.ORDERED
			},
			types.ErrIncompatibleCounterpartyUpgrade,
		},
		{
			"proposed connection is not found",
			func() {
				upgradeFields.ConnectionHops[0] = ibctesting.InvalidID
			},
			connectiontypes.ErrConnectionNotFound,
		},
		{
			"proposed connection is not in OPEN state",
			func() {
				// reuse existing connection to create a new connection in a non OPEN state
				connectionEnd := path.EndpointB.GetConnection()
				connectionEnd.State = connectiontypes.UNINITIALIZED
				connectionEnd.Counterparty.ConnectionId = counterpartyUpgradeFields.ConnectionHops[0] // both sides must be each other's counterparty

				// set proposed connection in state
				proposedConnectionID := "connection-100"
				suite.chainB.GetSimApp().GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainB.GetContext(), proposedConnectionID, connectionEnd)
				upgradeFields.ConnectionHops[0] = proposedConnectionID
			},
			connectiontypes.ErrInvalidConnectionState,
		},
		{
			"proposed connection ends are not each other's counterparty",
			func() {
				// reuse existing connection to create a new connection in a non OPEN state
				connectionEnd := path.EndpointB.GetConnection()
				// ensure counterparty connectionID does not match connectionID set in counterparty proposed upgrade
				connectionEnd.Counterparty.ConnectionId = "connection-50"

				// set proposed connection in state
				proposedConnectionID := "connection-100"
				suite.chainB.GetSimApp().GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chainB.GetContext(), proposedConnectionID, connectionEnd)
				upgradeFields.ConnectionHops[0] = proposedConnectionID
			},
			types.ErrIncompatibleCounterpartyUpgrade,
		},
		{
			"proposed upgrade version is not the same on both sides",
			func() {
				upgradeFields.Version = mock.Version
			},
			types.ErrIncompatibleCounterpartyUpgrade,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			upgradeFields = path.EndpointA.GetProposedUpgrade().Fields
			counterpartyUpgradeFields = path.EndpointB.GetProposedUpgrade().Fields

			tc.malleate()

			err = suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.CheckForUpgradeCompatibility(suite.chainB.GetContext(), upgradeFields, counterpartyUpgradeFields)
			if tc.expError != nil {
				suite.Require().ErrorIs(err, tc.expError)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestChanUpgradeCrossingHelloWithHistoricalProofs() {
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
			"counterparty (chain B) has already progressed to ACK step",
			func() {
				err := path.EndpointB.ChanUpgradeAck()
				suite.Require().NoError(err)
			},
			types.ErrInvalidChannelState,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeInit()
			suite.Require().NoError(err)

			suite.coordinator.CommitBlock(suite.chainA, suite.chainB)

			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			historicalChannelProof, historicalUpgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

			err = path.EndpointA.ChanUpgradeTry()
			suite.Require().NoError(err)

			tc.malleate()

			_, upgrade, err := suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.ChanUpgradeTry(
				suite.chainB.GetContext(),
				path.EndpointB.ChannelConfig.PortID,
				path.EndpointB.ChannelID,
				path.EndpointB.GetChannelUpgrade().Fields.ConnectionHops,
				path.EndpointA.GetChannelUpgrade().Fields,
				1,
				historicalChannelProof,
				historicalUpgradeProof,
				proofHeight,
			)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(upgrade)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestWriteErrorReceipt() {
	var path *ibctesting.Path
	var upgradeError *types.UpgradeError

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
			"success: existing error receipt found at a lower sequence",
			func() {
				// write an error sequence with a lower sequence number
				previousUpgradeError := types.NewUpgradeError(upgradeError.GetErrorReceipt().Sequence-1, types.ErrInvalidUpgrade)
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, previousUpgradeError)
			},
			nil,
		},
		{
			"failure: existing error receipt found at a higher sequence",
			func() {
				// write an error sequence with a higher sequence number
				previousUpgradeError := types.NewUpgradeError(upgradeError.GetErrorReceipt().Sequence+1, types.ErrInvalidUpgrade)
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.WriteErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, previousUpgradeError)
			},
			errorsmod.Wrap(types.ErrInvalidUpgradeSequence, "error receipt sequence (10) must be greater than existing error receipt sequence (11)"),
		},
		{
			"failure: upgrade exists for error receipt being written",
			func() {
				// attempt to write error receipt for existing upgrade without deleting upgrade info
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
				err := path.EndpointA.ChanUpgradeInit()
				suite.Require().NoError(err)
				ch := path.EndpointA.GetChannel()
				upgradeError = types.NewUpgradeError(ch.UpgradeSequence, types.ErrInvalidUpgrade)
			},
			errorsmod.Wrap(types.ErrInvalidUpgradeSequence, "attempting to write error receipt at sequence (1) while upgrade information exists at the same sequence"),
		},
		{
			"failure: channel not found",
			func() {
				suite.chainA.DeleteKey(host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			errorsmod.Wrap(types.ErrChannelNotFound, "port ID (mock) channel ID (channel-0)"),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			channelKeeper := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper

			upgradeError = types.NewUpgradeError(10, types.ErrInvalidUpgrade)

			tc.malleate()

			expPass := tc.expError == nil
			if expPass {
				suite.NotPanics(func() {
					channelKeeper.WriteErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgradeError)
				})
			} else {
				suite.PanicsWithError(tc.expError.Error(), func() {
					channelKeeper.WriteErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, upgradeError)
				})
			}
		})
	}
}
