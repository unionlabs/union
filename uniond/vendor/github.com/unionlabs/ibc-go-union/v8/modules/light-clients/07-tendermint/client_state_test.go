package tendermint_test

import (
	"time"

	ics23 "github.com/cosmos/ics23/go"

	sdk "github.com/cosmos/cosmos-sdk/types"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

const (
	// Do not change the length of these variables
	fiftyCharChainID    = "12345678901234567890123456789012345678901234567890"
	fiftyOneCharChainID = "123456789012345678901234567890123456789012345678901"
)

var invalidProof = []byte("invalid proof")

func (suite *TendermintTestSuite) TestStatus() {
	var (
		path        *ibctesting.Path
		clientState *ibctm.ClientState
	)

	testCases := []struct {
		name      string
		malleate  func()
		expStatus exported.Status
	}{
		{"client is active", func() {}, exported.Active},
		{"client is frozen", func() {
			clientState.FrozenHeight = clienttypes.NewHeight(0, 1)
			path.EndpointA.SetClientState(clientState)
		}, exported.Frozen},
		{"client status without consensus state", func() {
			clientState.LatestHeight = clientState.LatestHeight.Increment().(clienttypes.Height)
			path.EndpointA.SetClientState(clientState)
		}, exported.Expired},
		{"client status is expired", func() {
			suite.coordinator.IncrementTimeBy(clientState.TrustingPeriod)
		}, exported.Expired},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), path.EndpointA.ClientID)
			clientState = path.EndpointA.GetClientState().(*ibctm.ClientState)

			tc.malleate()

			status := clientState.Status(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec())
			suite.Require().Equal(tc.expStatus, status)
		})

	}
}

func (suite *TendermintTestSuite) TestValidate() {
	testCases := []struct {
		name        string
		clientState *ibctm.ClientState
		expPass     bool
	}{
		{
			name:        "valid client",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     true,
		},
		{
			name:        "valid client with nil upgrade path",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), nil),
			expPass:     true,
		},
		{
			name:        "invalid chainID",
			clientState: ibctm.NewClientState("  ", ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			// NOTE: if this test fails, the code must account for the change in chainID length across tendermint versions!
			// Do not only fix the test, fix the code!
			// https://github.com/cosmos/ibc-go/issues/177
			name:        "valid chainID - chainID validation failed for chainID of length 50! ",
			clientState: ibctm.NewClientState(fiftyCharChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     true,
		},
		{
			// NOTE: if this test fails, the code must account for the change in chainID length across tendermint versions!
			// Do not only fix the test, fix the code!
			// https://github.com/cosmos/ibc-go/issues/177
			name:        "invalid chainID - chainID validation did not fail for chainID of length 51! ",
			clientState: ibctm.NewClientState(fiftyOneCharChainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid trust level",
			clientState: ibctm.NewClientState(chainID, ibctm.Fraction{Numerator: 0, Denominator: 1}, trustingPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid zero trusting period",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, 0, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid negative trusting period",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, -1, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid zero unbonding period",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, 0, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid negative unbonding period",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, -1, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid zero max clock drift",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, 0, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid negative max clock drift",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, -1, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid revision number",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, clienttypes.NewHeight(1, 1), commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "invalid revision height",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, trustingPeriod, ubdPeriod, maxClockDrift, clienttypes.ZeroHeight(), commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "trusting period not less than unbonding period",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, ubdPeriod, ubdPeriod, maxClockDrift, height, commitmenttypes.GetSDKSpecs(), upgradePath),
			expPass:     false,
		},
		{
			name:        "proof specs is nil",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, ubdPeriod, ubdPeriod, maxClockDrift, height, nil, upgradePath),
			expPass:     false,
		},
		{
			name:        "proof specs contains nil",
			clientState: ibctm.NewClientState(chainID, ibctm.DefaultTrustLevel, ubdPeriod, ubdPeriod, maxClockDrift, height, []*ics23.ProofSpec{ics23.TendermintSpec, nil}, upgradePath),
			expPass:     false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			err := tc.clientState.Validate()
			if tc.expPass {
				suite.Require().NoError(err, tc.name)
			} else {
				suite.Require().Error(err, tc.name)
			}
		})
	}
}

func (suite *TendermintTestSuite) TestInitialize() {
	testCases := []struct {
		name           string
		consensusState exported.ConsensusState
		expPass        bool
	}{
		{
			name:           "valid consensus",
			consensusState: &ibctm.ConsensusState{},
			expPass:        true,
		},
		{
			name:           "invalid consensus: consensus state is solomachine consensus",
			consensusState: ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachine", "", 2).ConsensusState(),
			expPass:        false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			path := ibctesting.NewPath(suite.chainA, suite.chainB)

			tmConfig, ok := path.EndpointB.ClientConfig.(*ibctesting.TendermintConfig)
			suite.Require().True(ok)

			clientState := ibctm.NewClientState(
				path.EndpointB.Chain.ChainID,
				tmConfig.TrustLevel, tmConfig.TrustingPeriod, tmConfig.UnbondingPeriod, tmConfig.MaxClockDrift,
				suite.chainB.LastHeader.GetTrustedHeight(), commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath,
			)

			store := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), path.EndpointA.ClientID)
			err := clientState.Initialize(suite.chainA.GetContext(), suite.chainA.Codec, store, tc.consensusState)

			if tc.expPass {
				suite.Require().NoError(err, "valid case returned an error")
				suite.Require().True(store.Has(host.ClientStateKey()))
				suite.Require().True(store.Has(host.ConsensusStateKey(suite.chainB.LastHeader.GetTrustedHeight())))
			} else {
				suite.Require().Error(err, "invalid case didn't return an error")
				suite.Require().False(store.Has(host.ClientStateKey()))
				suite.Require().False(store.Has(host.ConsensusStateKey(suite.chainB.LastHeader.GetTrustedHeight())))
			}
		})
	}
}

func (suite *TendermintTestSuite) TestVerifyMembership() {
	var (
		testingpath      *ibctesting.Path
		delayTimePeriod  uint64
		delayBlockPeriod uint64
		err              error
		proofHeight      exported.Height
		proof            []byte
		path             exported.Path
		value            []byte
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"successful ClientState verification",
			func() {
				// default proof construction uses ClientState
			},
			true,
		},
		{
			"successful ConsensusState verification", func() {
				key := host.FullConsensusStateKey(testingpath.EndpointB.ClientID, testingpath.EndpointB.GetClientState().GetLatestHeight())
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)

				consensusState := testingpath.EndpointB.GetConsensusState(testingpath.EndpointB.GetClientState().GetLatestHeight()).(*ibctm.ConsensusState)
				value, err = suite.chainB.Codec.MarshalInterface(consensusState)
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"successful Connection verification", func() {
				key := host.ConnectionKey(testingpath.EndpointB.ConnectionID)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)

				connection := testingpath.EndpointB.GetConnection()
				value, err = suite.chainB.Codec.Marshal(&connection)
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"successful Channel verification", func() {
				key := host.ChannelKey(testingpath.EndpointB.ChannelConfig.PortID, testingpath.EndpointB.ChannelID)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)

				channel := testingpath.EndpointB.GetChannel()
				value, err = suite.chainB.Codec.Marshal(&channel)
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"successful PacketCommitment verification", func() {
				// send from chainB to chainA since we are proving chainB sent a packet
				sequence, err := testingpath.EndpointB.SendPacket(clienttypes.NewHeight(1, 100), 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				// make packet commitment proof
				packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence, testingpath.EndpointB.ChannelConfig.PortID, testingpath.EndpointB.ChannelID, testingpath.EndpointA.ChannelConfig.PortID, testingpath.EndpointA.ChannelID, clienttypes.NewHeight(1, 100), 0)
				key := host.PacketCommitmentKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = testingpath.EndpointB.QueryProof(key)

				value = channeltypes.CommitPacket(suite.chainA.App.GetIBCKeeper().Codec(), packet)
			}, true,
		},
		{
			"successful Acknowledgement verification", func() {
				// send from chainA to chainB since we are proving chainB wrote an acknowledgement
				sequence, err := testingpath.EndpointA.SendPacket(clienttypes.NewHeight(1, 100), 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				// write receipt and ack
				packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence, testingpath.EndpointA.ChannelConfig.PortID, testingpath.EndpointA.ChannelID, testingpath.EndpointB.ChannelConfig.PortID, testingpath.EndpointB.ChannelID, clienttypes.NewHeight(1, 100), 0)
				err = testingpath.EndpointB.RecvPacket(packet)
				suite.Require().NoError(err)

				key := host.PacketAcknowledgementKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = testingpath.EndpointB.QueryProof(key)

				value = channeltypes.CommitAcknowledgement(ibcmock.MockAcknowledgement.Acknowledgement())
			},
			true,
		},
		{
			"successful NextSequenceRecv verification", func() {
				// send from chainA to chainB since we are proving chainB incremented the sequence recv

				// send packet
				sequence, err := testingpath.EndpointA.SendPacket(clienttypes.NewHeight(1, 100), 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				// next seq recv incremented
				packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence, testingpath.EndpointA.ChannelConfig.PortID, testingpath.EndpointA.ChannelID, testingpath.EndpointB.ChannelConfig.PortID, testingpath.EndpointB.ChannelID, clienttypes.NewHeight(1, 100), 0)
				err = testingpath.EndpointB.RecvPacket(packet)
				suite.Require().NoError(err)

				key := host.NextSequenceRecvKey(packet.GetSourcePort(), packet.GetSourceChannel())
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = testingpath.EndpointB.QueryProof(key)

				value = sdk.Uint64ToBigEndian(packet.GetSequence() + 1)
			},
			true,
		},
		{
			"successful verification outside IBC store", func() {
				key := transfertypes.PortKey
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(commitmenttypes.NewMerklePrefix([]byte(transfertypes.StoreKey)), merklePath)
				suite.Require().NoError(err)

				clientState := testingpath.EndpointA.GetClientState()
				proof, proofHeight = suite.chainB.QueryProofForStore(transfertypes.StoreKey, key, int64(clientState.GetLatestHeight().GetRevisionHeight()))

				value = []byte(suite.chainB.GetSimApp().TransferKeeper.GetPort(suite.chainB.GetContext()))
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"delay time period has passed", func() {
				delayTimePeriod = uint64(time.Second.Nanoseconds())
			},
			true,
		},
		{
			"delay time period has not passed", func() {
				delayTimePeriod = uint64(time.Hour.Nanoseconds())
			},
			false,
		},
		{
			"delay block period has passed", func() {
				delayBlockPeriod = 1
			},
			true,
		},
		{
			"delay block period has not passed", func() {
				delayBlockPeriod = 1000
			},
			false,
		},
		{
			"latest client height < height", func() {
				proofHeight = testingpath.EndpointA.GetClientState().GetLatestHeight().Increment()
			}, false,
		},
		{
			"invalid path type",
			func() {
				path = ibcmock.KeyPath{}
			},
			false,
		},
		{
			"failed to unmarshal merkle proof", func() {
				proof = invalidProof
			}, false,
		},
		{
			"consensus state not found", func() {
				proofHeight = clienttypes.ZeroHeight()
			}, false,
		},
		{
			"proof verification failed", func() {
				// change the value being proved
				value = []byte("invalid value")
			}, false,
		},
		{
			"proof is empty", func() {
				// change the inserted proof
				proof = []byte{}
			}, false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			testingpath = ibctesting.NewPath(suite.chainA, suite.chainB)
			testingpath.SetChannelOrdered()
			suite.coordinator.Setup(testingpath)

			// reset time and block delays to 0, malleate may change to a specific non-zero value.
			delayTimePeriod = 0
			delayBlockPeriod = 0

			// create default proof, merklePath, and value which passes
			// may be overwritten by malleate()
			key := host.FullClientStateKey(testingpath.EndpointB.ClientID)
			merklePath := commitmenttypes.NewMerklePath(string(key))
			path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
			suite.Require().NoError(err)

			proof, proofHeight = suite.chainB.QueryProof(key)

			clientState := testingpath.EndpointB.GetClientState().(*ibctm.ClientState)
			value, err = suite.chainB.Codec.MarshalInterface(clientState)
			suite.Require().NoError(err)

			tc.malleate() // make changes as necessary

			clientState = testingpath.EndpointA.GetClientState().(*ibctm.ClientState)

			ctx := suite.chainA.GetContext()
			store := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(ctx, testingpath.EndpointA.ClientID)

			err = clientState.VerifyMembership(
				ctx, store, suite.chainA.Codec, proofHeight, delayTimePeriod, delayBlockPeriod,
				proof, path, value,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *TendermintTestSuite) TestVerifyNonMembership() {
	var (
		testingpath         *ibctesting.Path
		delayTimePeriod     uint64
		delayBlockPeriod    uint64
		err                 error
		proofHeight         exported.Height
		path                exported.Path
		proof               []byte
		invalidClientID     = "09-tendermint"
		invalidConnectionID = "connection-100"
		invalidChannelID    = "channel-800"
		invalidPortID       = "invalid-port"
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"successful ClientState verification of non membership",
			func() {
				// default proof construction uses ClientState
			},
			true,
		},
		{
			"successful ConsensusState verification of non membership", func() {
				key := host.FullConsensusStateKey(invalidClientID, testingpath.EndpointB.GetClientState().GetLatestHeight())
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)
			},
			true,
		},
		{
			"successful Connection verification of non membership", func() {
				key := host.ConnectionKey(invalidConnectionID)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)
			},
			true,
		},
		{
			"successful Channel verification of non membership", func() {
				key := host.ChannelKey(testingpath.EndpointB.ChannelConfig.PortID, invalidChannelID)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)
			},
			true,
		},
		{
			"successful PacketCommitment verification of non membership", func() {
				// make packet commitment proof
				key := host.PacketCommitmentKey(invalidPortID, invalidChannelID, 1)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = testingpath.EndpointB.QueryProof(key)
			}, true,
		},
		{
			"successful Acknowledgement verification of non membership", func() {
				key := host.PacketAcknowledgementKey(invalidPortID, invalidChannelID, 1)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = testingpath.EndpointB.QueryProof(key)
			},
			true,
		},
		{
			"successful NextSequenceRecv verification of non membership", func() {
				key := host.NextSequenceRecvKey(invalidPortID, invalidChannelID)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = testingpath.EndpointB.QueryProof(key)
			},
			true,
		},
		{
			"successful verification of non membership outside IBC store", func() {
				key := []byte{0x08}
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(commitmenttypes.NewMerklePrefix([]byte(transfertypes.StoreKey)), merklePath)
				suite.Require().NoError(err)

				clientState := testingpath.EndpointA.GetClientState()
				proof, proofHeight = suite.chainB.QueryProofForStore(transfertypes.StoreKey, key, int64(clientState.GetLatestHeight().GetRevisionHeight()))
			},
			true,
		},
		{
			"delay time period has passed", func() {
				delayTimePeriod = uint64(time.Second.Nanoseconds())
			},
			true,
		},
		{
			"delay time period has not passed", func() {
				delayTimePeriod = uint64(time.Hour.Nanoseconds())
			},
			false,
		},
		{
			"delay block period has passed", func() {
				delayBlockPeriod = 1
			},
			true,
		},
		{
			"delay block period has not passed", func() {
				delayBlockPeriod = 1000
			},
			false,
		},
		{
			"latest client height < height", func() {
				proofHeight = testingpath.EndpointA.GetClientState().GetLatestHeight().Increment()
			}, false,
		},
		{
			"invalid path type",
			func() {
				path = ibcmock.KeyPath{}
			},
			false,
		},
		{
			"failed to unmarshal merkle proof", func() {
				proof = invalidProof
			}, false,
		},
		{
			"consensus state not found", func() {
				proofHeight = clienttypes.ZeroHeight()
			}, false,
		},
		{
			"verify non membership fails as path exists", func() {
				// change the value being proved
				key := host.FullClientStateKey(testingpath.EndpointB.ClientID)
				merklePath := commitmenttypes.NewMerklePath(string(key))
				path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				proof, proofHeight = suite.chainB.QueryProof(key)
			}, false,
		},
		{
			"proof is empty", func() {
				// change the inserted proof
				proof = []byte{}
			}, false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			testingpath = ibctesting.NewPath(suite.chainA, suite.chainB)
			testingpath.SetChannelOrdered()
			suite.coordinator.Setup(testingpath)

			// reset time and block delays to 0, malleate may change to a specific non-zero value.
			delayTimePeriod = 0
			delayBlockPeriod = 0

			// create default proof, merklePath, and value which passes
			// may be overwritten by malleate()
			key := host.FullClientStateKey("invalid-client-id")

			merklePath := commitmenttypes.NewMerklePath(string(key))
			path, err = commitmenttypes.ApplyPrefix(suite.chainB.GetPrefix(), merklePath)
			suite.Require().NoError(err)

			proof, proofHeight = suite.chainB.QueryProof(key)

			tc.malleate() // make changes as necessary

			clientState := testingpath.EndpointA.GetClientState().(*ibctm.ClientState)

			ctx := suite.chainA.GetContext()
			store := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(ctx, testingpath.EndpointA.ClientID)

			err = clientState.VerifyNonMembership(
				ctx, store, suite.chainA.Codec, proofHeight, delayTimePeriod, delayBlockPeriod,
				proof, path,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
