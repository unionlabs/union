package keeper_test

import (
	"encoding/hex"
	"fmt"
	"time"

	upgradetypes "cosmossdk.io/x/upgrade/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	localhost "github.com/cosmos/ibc-go/v8/modules/light-clients/09-localhost"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestCreateClient() {
	var (
		clientState    exported.ClientState
		consensusState exported.ConsensusState
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"success: 07-tendermint client type supported",
			func() {
				clientState = ibctm.NewClientState(testChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, testClientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath)
				consensusState = suite.consensusState
			},
			true,
		},
		{
			"failure: 07-tendermint client status is not active",
			func() {
				clientState = ibctm.NewClientState(testChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, testClientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath)
				tmcs, ok := clientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				tmcs.FrozenHeight = ibctm.FrozenHeight
				consensusState = suite.consensusState
			},
			false,
		},
		{
			"success: 06-solomachine client type supported",
			func() {
				clientState = solomachine.NewClientState(0, &solomachine.ConsensusState{PublicKey: suite.solomachine.ConsensusState().PublicKey, Diversifier: suite.solomachine.Diversifier, Timestamp: suite.solomachine.Time})
				consensusState = &solomachine.ConsensusState{PublicKey: suite.solomachine.ConsensusState().PublicKey, Diversifier: suite.solomachine.Diversifier, Timestamp: suite.solomachine.Time}
			},
			true,
		},
		{
			"failure: 09-localhost client type not supported",
			func() {
				clientState = localhost.NewClientState(clienttypes.GetSelfHeight(suite.chainA.GetContext()))
				consensusState = nil
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest() // reset
			tc.malleate()

			clientID, err := suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.CreateClient(suite.chainA.GetContext(), clientState, consensusState)

			// assert correct behaviour based on expected error
			clientState, found := suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.GetClientState(suite.chainA.GetContext(), clientID)
			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(clientID)
				suite.Require().True(found)
				suite.Require().NotEmpty(clientState)
			} else {
				suite.Require().Error(err)
				suite.Require().Empty(clientID)
				suite.Require().False(found)
				suite.Require().Empty(clientState)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestUpdateClientTendermint() {
	var (
		path         *ibctesting.Path
		updateHeader *ibctm.Header
	)

	// Must create header creation functions since suite.header gets recreated on each test case
	createFutureUpdateFn := func(trustedHeight clienttypes.Height) *ibctm.Header {
		header, err := suite.chainA.IBCClientHeader(path.EndpointB.Chain.LastHeader, trustedHeight)
		suite.Require().NoError(err)
		return header
	}
	createPastUpdateFn := func(fillHeight, trustedHeight clienttypes.Height) *ibctm.Header {
		consState, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientConsensusState(suite.chainA.GetContext(), path.EndpointA.ClientID, trustedHeight)
		suite.Require().True(found)

		return suite.chainB.CreateTMClientHeader(suite.chainB.ChainID, int64(fillHeight.RevisionHeight), trustedHeight, consState.(*ibctm.ConsensusState).Timestamp.Add(time.Second*5),
			suite.chainB.Vals, suite.chainB.Vals, suite.chainB.Vals, suite.chainB.Signers)
	}

	cases := []struct {
		name      string
		malleate  func()
		expPass   bool
		expFreeze bool
	}{
		{"valid update", func() {
			clientState := path.EndpointA.GetClientState().(*ibctm.ClientState)
			trustHeight := clientState.GetLatestHeight().(clienttypes.Height)

			// store intermediate consensus state to check that trustedHeight does not need to be highest consensus state before header height
			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			updateHeader = createFutureUpdateFn(trustHeight)
		}, true, false},
		{"valid past update", func() {
			clientState := path.EndpointA.GetClientState()
			trustedHeight := clientState.GetLatestHeight().(clienttypes.Height)

			currHeight := suite.chainB.CurrentHeader.Height
			fillHeight := clienttypes.NewHeight(clientState.GetLatestHeight().GetRevisionNumber(), uint64(currHeight))

			// commit a couple blocks to allow client to fill in gaps
			suite.coordinator.CommitBlock(suite.chainB) // this height is not filled in yet
			suite.coordinator.CommitBlock(suite.chainB) // this height is filled in by the update below

			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			// ensure fill height not set
			_, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientConsensusState(suite.chainA.GetContext(), path.EndpointA.ClientID, fillHeight)
			suite.Require().False(found)

			// updateHeader will fill in consensus state between prevConsState and suite.consState
			// clientState should not be updated
			updateHeader = createPastUpdateFn(fillHeight, trustedHeight)
		}, true, false},
		{"valid duplicate update", func() {
			height1 := clienttypes.NewHeight(1, 1)

			// store previous consensus state
			prevConsState := &ibctm.ConsensusState{
				Timestamp:          suite.past,
				NextValidatorsHash: suite.chainB.Vals.Hash(),
			}
			path.EndpointA.SetConsensusState(prevConsState, height1)

			height5 := clienttypes.NewHeight(1, 5)
			// store next consensus state to check that trustedHeight does not need to be highest consensus state before header height
			nextConsState := &ibctm.ConsensusState{
				Timestamp:          suite.past.Add(time.Minute),
				NextValidatorsHash: suite.chainB.Vals.Hash(),
			}
			path.EndpointA.SetConsensusState(nextConsState, height5)

			// update client state latest height
			clientState := path.EndpointA.GetClientState()
			clientState.(*ibctm.ClientState).LatestHeight = height5
			path.EndpointA.SetClientState(clientState)

			height3 := clienttypes.NewHeight(1, 3)
			// updateHeader will fill in consensus state between prevConsState and suite.consState
			// clientState should not be updated
			updateHeader = createPastUpdateFn(height3, height1)
			// set updateHeader's consensus state in store to create duplicate UpdateClient scenario
			path.EndpointA.SetConsensusState(updateHeader.ConsensusState(), updateHeader.GetHeight())
		}, true, false},
		{"misbehaviour detection: conflicting header", func() {
			clientID := path.EndpointA.ClientID

			height1 := clienttypes.NewHeight(1, 1)
			// store previous consensus state
			prevConsState := &ibctm.ConsensusState{
				Timestamp:          suite.past,
				NextValidatorsHash: suite.chainB.Vals.Hash(),
			}
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientConsensusState(suite.chainA.GetContext(), clientID, height1, prevConsState)

			height5 := clienttypes.NewHeight(1, 5)
			// store next consensus state to check that trustedHeight does not need to be hightest consensus state before header height
			nextConsState := &ibctm.ConsensusState{
				Timestamp:          suite.past.Add(time.Minute),
				NextValidatorsHash: suite.chainB.Vals.Hash(),
			}
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientConsensusState(suite.chainA.GetContext(), clientID, height5, nextConsState)

			height3 := clienttypes.NewHeight(1, 3)
			// updateHeader will fill in consensus state between prevConsState and suite.consState
			// clientState should not be updated
			updateHeader = createPastUpdateFn(height3, height1)
			// set conflicting consensus state in store to create misbehaviour scenario
			conflictConsState := updateHeader.ConsensusState()
			conflictConsState.Root = commitmenttypes.NewMerkleRoot([]byte("conflicting apphash"))
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientConsensusState(suite.chainA.GetContext(), clientID, updateHeader.GetHeight(), conflictConsState)
		}, true, true},
		{"misbehaviour detection: monotonic time violation", func() {
			clientState := path.EndpointA.GetClientState().(*ibctm.ClientState)
			clientID := path.EndpointA.ClientID
			trustedHeight := clientState.GetLatestHeight().(clienttypes.Height)

			// store intermediate consensus state at a time greater than updateHeader time
			// this will break time monotonicity
			incrementedClientHeight := clientState.GetLatestHeight().Increment().(clienttypes.Height)
			intermediateConsState := &ibctm.ConsensusState{
				Timestamp:          suite.coordinator.CurrentTime.Add(2 * time.Hour),
				NextValidatorsHash: suite.chainB.Vals.Hash(),
			}
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientConsensusState(suite.chainA.GetContext(), clientID, incrementedClientHeight, intermediateConsState)
			// set iteration key
			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), clientID)
			ibctm.SetIterationKey(clientStore, incrementedClientHeight)

			clientState.LatestHeight = incrementedClientHeight
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), clientID, clientState)

			updateHeader = createFutureUpdateFn(trustedHeight)
		}, true, true},
		{"client state not found", func() {
			updateHeader = createFutureUpdateFn(path.EndpointA.GetClientState().GetLatestHeight().(clienttypes.Height))

			path.EndpointA.ClientID = ibctesting.InvalidID
		}, false, false},
		{"consensus state not found", func() {
			clientState := path.EndpointA.GetClientState()
			tmClient, ok := clientState.(*ibctm.ClientState)
			suite.Require().True(ok)
			tmClient.LatestHeight = tmClient.LatestHeight.Increment().(clienttypes.Height)

			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID, clientState)
			updateHeader = createFutureUpdateFn(clientState.GetLatestHeight().(clienttypes.Height))
		}, false, false},
		{"client is not active", func() {
			clientState := path.EndpointA.GetClientState().(*ibctm.ClientState)
			clientState.FrozenHeight = clienttypes.NewHeight(1, 1)
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID, clientState)
			updateHeader = createFutureUpdateFn(clientState.GetLatestHeight().(clienttypes.Height))
		}, false, false},
		{"invalid header", func() {
			updateHeader = createFutureUpdateFn(path.EndpointA.GetClientState().GetLatestHeight().(clienttypes.Height))
			updateHeader.TrustedHeight = updateHeader.TrustedHeight.Increment().(clienttypes.Height)
		}, false, false},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(fmt.Sprintf("Case %s", tc.name), func() {
			suite.SetupTest()
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)

			tc.malleate()

			var clientState exported.ClientState
			if tc.expPass {
				clientState = path.EndpointA.GetClientState()
			}

			err := suite.chainA.App.GetIBCKeeper().ClientKeeper.UpdateClient(suite.chainA.GetContext(), path.EndpointA.ClientID, updateHeader)

			if tc.expPass {
				suite.Require().NoError(err, err)

				newClientState := path.EndpointA.GetClientState()

				if tc.expFreeze {
					suite.Require().True(!newClientState.(*ibctm.ClientState).FrozenHeight.IsZero(), "client did not freeze after conflicting header was submitted to UpdateClient")
				} else {
					expConsensusState := &ibctm.ConsensusState{
						Timestamp:          updateHeader.GetTime(),
						Root:               commitmenttypes.NewMerkleRoot(updateHeader.Header.GetAppHash()),
						NextValidatorsHash: updateHeader.Header.NextValidatorsHash,
					}

					consensusState, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientConsensusState(suite.chainA.GetContext(), path.EndpointA.ClientID, updateHeader.GetHeight())
					suite.Require().True(found)

					// Determine if clientState should be updated or not
					if updateHeader.GetHeight().GT(clientState.GetLatestHeight()) {
						// Header Height is greater than clientState latest Height, clientState should be updated with header.GetHeight()
						suite.Require().Equal(updateHeader.GetHeight(), newClientState.GetLatestHeight(), "clientstate height did not update")
					} else {
						// Update will add past consensus state, clientState should not be updated at all
						suite.Require().Equal(clientState.GetLatestHeight(), newClientState.GetLatestHeight(), "client state height updated for past header")
					}

					suite.Require().NoError(err)
					suite.Require().Equal(expConsensusState, consensusState, "consensus state should have been updated on case %s", tc.name)
				}
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestUpgradeClient() {
	var (
		path                                             *ibctesting.Path
		upgradedClient                                   exported.ClientState
		upgradedConsState                                exported.ConsensusState
		lastHeight                                       exported.Height
		upgradedClientProof, upgradedConsensusStateProof []byte
		upgradedClientBz, upgradedConsStateBz            []byte
	)

	testCases := []struct {
		name    string
		setup   func()
		expPass bool
	}{
		{
			name: "successful upgrade",
			setup: func() {
				// last Height is at next block
				lastHeight = clienttypes.NewHeight(1, uint64(suite.chainB.GetContext().BlockHeight()+1))

				// zero custom fields and store in upgrade store
				err := suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)
				suite.Require().NoError(err)
				err = suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz)
				suite.Require().NoError(err)

				// commit upgrade store changes and update clients
				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				cs, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
				suite.Require().True(found)

				upgradedClientProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedClientKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
				upgradedConsensusStateProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedConsStateKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
			},
			expPass: true,
		},
		{
			name: "client state not found",
			setup: func() {
				// last Height is at next block
				lastHeight = clienttypes.NewHeight(1, uint64(suite.chainB.GetContext().BlockHeight()+1))

				// zero custom fields and store in upgrade store
				err := suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)
				suite.Require().NoError(err)
				err = suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz)
				suite.Require().NoError(err)

				// commit upgrade store changes and update clients

				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				cs, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
				suite.Require().True(found)

				upgradedClientProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedClientKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
				upgradedConsensusStateProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedConsStateKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())

				path.EndpointA.ClientID = "wrongclientid"
			},
			expPass: false,
		},
		{
			name: "client state is not active",
			setup: func() {
				// client is frozen

				// last Height is at next block
				lastHeight = clienttypes.NewHeight(1, uint64(suite.chainB.GetContext().BlockHeight()+1))

				// zero custom fields and store in upgrade store
				err := suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)
				suite.Require().NoError(err)
				err = suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz)
				suite.Require().NoError(err)

				// commit upgrade store changes and update clients

				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				cs, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
				suite.Require().True(found)

				upgradedClientProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedClientKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
				upgradedConsensusStateProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedConsStateKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())

				// set frozen client in store
				tmClient, ok := cs.(*ibctm.ClientState)
				suite.Require().True(ok)
				tmClient.FrozenHeight = clienttypes.NewHeight(1, 1)
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID, tmClient)
			},
			expPass: false,
		},
		{
			name: "tendermint client VerifyUpgrade fails",
			setup: func() {
				// last Height is at next block
				lastHeight = clienttypes.NewHeight(1, uint64(suite.chainB.GetContext().BlockHeight()+1))

				// zero custom fields and store in upgrade store
				err := suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)
				suite.Require().NoError(err)
				err = suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz)
				suite.Require().NoError(err)

				// change upgradedClient client-specified parameters
				tmClient := upgradedClient.(*ibctm.ClientState)
				tmClient.ChainId = "wrongchainID"
				upgradedClient = tmClient

				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				cs, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
				suite.Require().True(found)

				upgradedClientProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedClientKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
				upgradedConsensusStateProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedConsStateKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
			},
			expPass: false,
		},
		{
			name: "unsuccessful upgrade: upgraded height is not greater than current height",
			setup: func() {
				// last Height is at next block
				lastHeight = clienttypes.NewHeight(1, uint64(suite.chainB.GetContext().BlockHeight()+1))

				// zero custom fields and store in upgrade store
				err := suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)
				suite.Require().NoError(err)
				err = suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz)
				suite.Require().NoError(err)

				// change upgradedClient height to be lower than current client state height
				tmClient := upgradedClient.(*ibctm.ClientState)
				tmClient.LatestHeight = clienttypes.NewHeight(0, 1)
				upgradedClient = tmClient

				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				cs, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
				suite.Require().True(found)

				upgradedClientProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedClientKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
				upgradedConsensusStateProof, _ = suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedConsStateKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
			},
			expPass: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		path = ibctesting.NewPath(suite.chainA, suite.chainB)
		suite.coordinator.SetupClients(path)

		clientState := path.EndpointA.GetClientState().(*ibctm.ClientState)
		revisionNumber := clienttypes.ParseChainID(clientState.ChainId)

		newChainID, err := clienttypes.SetRevisionNumber(clientState.ChainId, revisionNumber+1)
		suite.Require().NoError(err)

		upgradedClient = ibctm.NewClientState(newChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod+trustingPeriod, maxClockDrift, clienttypes.NewHeight(revisionNumber+1, clientState.GetLatestHeight().GetRevisionHeight()+1), commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath)
		upgradedClient = upgradedClient.ZeroCustomFields()
		upgradedClientBz, err = clienttypes.MarshalClientState(suite.chainA.App.AppCodec(), upgradedClient)
		suite.Require().NoError(err)

		upgradedConsState = &ibctm.ConsensusState{
			NextValidatorsHash: []byte("nextValsHash"),
		}
		upgradedConsStateBz, err = clienttypes.MarshalConsensusState(suite.chainA.App.AppCodec(), upgradedConsState)
		suite.Require().NoError(err)

		tc.setup()

		err = suite.chainA.App.GetIBCKeeper().ClientKeeper.UpgradeClient(suite.chainA.GetContext(), path.EndpointA.ClientID, upgradedClient, upgradedConsState, upgradedClientProof, upgradedConsensusStateProof)

		if tc.expPass {
			suite.Require().NoError(err, "verify upgrade failed on valid case: %s", tc.name)
		} else {
			suite.Require().Error(err, "verify upgrade passed on invalid case: %s", tc.name)
		}
	}
}

func (suite *KeeperTestSuite) TestUpdateClientEventEmission() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupClients(path)

	tmClientState, ok := path.EndpointA.GetClientState().(*ibctm.ClientState)
	suite.Require().True(ok)
	trustedHeight := tmClientState.LatestHeight
	header, err := suite.chainA.IBCClientHeader(path.EndpointA.Counterparty.Chain.LastHeader, trustedHeight)
	suite.Require().NoError(err)

	msg, err := clienttypes.NewMsgUpdateClient(
		path.EndpointA.ClientID, header,
		suite.chainA.SenderAccount.GetAddress().String(),
	)
	suite.Require().NoError(err)

	result, err := suite.chainA.SendMsgs(msg)
	suite.Require().NoError(err)
	// first event type is "message", followed by 3 "tx" events in ante
	updateEvent := result.Events[4]
	suite.Require().Equal(clienttypes.EventTypeUpdateClient, updateEvent.Type)

	// use a boolean to ensure the update event contains the header
	contains := false
	for _, attr := range updateEvent.Attributes {
		if attr.Key == clienttypes.AttributeKeyHeader {
			contains = true

			bz, err := hex.DecodeString(attr.Value)
			suite.Require().NoError(err)

			emittedHeader, err := clienttypes.UnmarshalClientMessage(suite.chainA.App.AppCodec(), bz)
			suite.Require().NoError(err)
			suite.Require().Equal(header, emittedHeader)
		}
	}
	suite.Require().True(contains)
}

func (suite *KeeperTestSuite) TestRecoverClient() {
	var (
		subject, substitute                       string
		subjectClientState, substituteClientState exported.ClientState
	)

	testCases := []struct {
		msg      string
		malleate func()
		expErr   error
	}{
		{
			"success",
			func() {},
			nil,
		},
		{
			"success, subject and substitute use different revision number",
			func() {
				tmClientState, ok := substituteClientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				consState, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientConsensusState(suite.chainA.GetContext(), substitute, tmClientState.LatestHeight)
				suite.Require().True(found)
				newRevisionNumber := tmClientState.GetLatestHeight().GetRevisionNumber() + 1

				tmClientState.LatestHeight = clienttypes.NewHeight(newRevisionNumber, tmClientState.GetLatestHeight().GetRevisionHeight())

				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientConsensusState(suite.chainA.GetContext(), substitute, tmClientState.LatestHeight, consState)
				clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), substitute)
				ibctm.SetProcessedTime(clientStore, tmClientState.LatestHeight, 100)
				ibctm.SetProcessedHeight(clientStore, tmClientState.LatestHeight, clienttypes.NewHeight(0, 1))
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), substitute, tmClientState)
			},
			nil,
		},
		{
			"subject client does not exist",
			func() {
				subject = ibctesting.InvalidID
			},
			clienttypes.ErrClientNotFound,
		},
		{
			"subject is Active",
			func() {
				tmClientState, ok := subjectClientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				// Set FrozenHeight to zero to ensure client is reported as Active
				tmClientState.FrozenHeight = clienttypes.ZeroHeight()
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), subject, tmClientState)
			},
			clienttypes.ErrInvalidRecoveryClient,
		},
		{
			"substitute client does not exist",
			func() {
				substitute = ibctesting.InvalidID
			},
			clienttypes.ErrClientNotFound,
		},
		{
			"subject and substitute have equal latest height",
			func() {
				tmClientState, ok := subjectClientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				tmClientState.LatestHeight = substituteClientState.GetLatestHeight().(clienttypes.Height)
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), subject, tmClientState)
			},
			clienttypes.ErrInvalidHeight,
		},
		{
			"subject height is greater than substitute height",
			func() {
				tmClientState, ok := subjectClientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				tmClientState.LatestHeight = substituteClientState.GetLatestHeight().Increment().(clienttypes.Height)
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), subject, tmClientState)
			},
			clienttypes.ErrInvalidHeight,
		},
		{
			"substitute is frozen",
			func() {
				tmClientState, ok := substituteClientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				tmClientState.FrozenHeight = clienttypes.NewHeight(0, 1)
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), substitute, tmClientState)
			},
			clienttypes.ErrClientNotActive,
		},
		{
			"CheckSubstituteAndUpdateState fails, substitute client trust level doesn't match subject client trust level",
			func() {
				tmClientState, ok := substituteClientState.(*ibctm.ClientState)
				suite.Require().True(ok)
				tmClientState.UnbondingPeriod += time.Minute
				suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), substitute, tmClientState)
			},
			clienttypes.ErrInvalidSubstitute,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.msg, func() {
			suite.SetupTest() // reset

			subjectPath := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(subjectPath)
			subject = subjectPath.EndpointA.ClientID
			subjectClientState = suite.chainA.GetClientState(subject)

			substitutePath := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(substitutePath)
			substitute = substitutePath.EndpointA.ClientID

			// update substitute twice
			err := substitutePath.EndpointA.UpdateClient()
			suite.Require().NoError(err)
			err = substitutePath.EndpointA.UpdateClient()
			suite.Require().NoError(err)
			substituteClientState = suite.chainA.GetClientState(substitute)

			tmClientState, ok := subjectClientState.(*ibctm.ClientState)
			suite.Require().True(ok)
			tmClientState.FrozenHeight = tmClientState.LatestHeight
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), subject, tmClientState)

			tc.malleate()

			err = suite.chainA.App.GetIBCKeeper().ClientKeeper.RecoverClient(suite.chainA.GetContext(), subject, substitute)

			expPass := tc.expErr == nil
			if expPass {
				suite.Require().NoError(err)

				// Assert that client status is now Active
				clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), subjectPath.EndpointA.ClientID)
				tmClientState := subjectPath.EndpointA.GetClientState().(*ibctm.ClientState)
				suite.Require().Equal(tmClientState.Status(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec()), exported.Active)
			} else {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, tc.expErr)
			}
		})
	}
}
