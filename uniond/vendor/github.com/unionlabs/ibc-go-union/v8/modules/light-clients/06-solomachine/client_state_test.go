package solomachine_test

import (
	"bytes"

	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

const (
	counterpartyClientIdentifier = "chainA"
	testConnectionID             = "connectionid"
	testChannelID                = "testchannelid"
	testPortID                   = "testportid"
)

func (suite *SoloMachineTestSuite) TestStatus() {
	clientState := suite.solomachine.ClientState()
	// solo machine discards arguments
	status := clientState.Status(suite.chainA.GetContext(), nil, nil)
	suite.Require().Equal(exported.Active, status)

	// freeze solo machine
	clientState.IsFrozen = true
	status = clientState.Status(suite.chainA.GetContext(), nil, nil)
	suite.Require().Equal(exported.Frozen, status)
}

func (suite *SoloMachineTestSuite) TestClientStateValidateBasic() {
	// test singlesig and multisig public keys
	for _, sm := range []*ibctesting.Solomachine{suite.solomachine, suite.solomachineMulti} {

		testCases := []struct {
			name        string
			clientState *solomachine.ClientState
			expPass     bool
		}{
			{
				"valid client state",
				sm.ClientState(),
				true,
			},
			{
				"empty ClientState",
				&solomachine.ClientState{},
				false,
			},
			{
				"sequence is zero",
				solomachine.NewClientState(0, &solomachine.ConsensusState{sm.ConsensusState().PublicKey, sm.Diversifier, sm.Time}),
				false,
			},
			{
				"timestamp is zero",
				solomachine.NewClientState(1, &solomachine.ConsensusState{sm.ConsensusState().PublicKey, sm.Diversifier, 0}),
				false,
			},
			{
				"diversifier is blank",
				solomachine.NewClientState(1, &solomachine.ConsensusState{sm.ConsensusState().PublicKey, "  ", 1}),
				false,
			},
			{
				"pubkey is empty",
				solomachine.NewClientState(1, &solomachine.ConsensusState{nil, sm.Diversifier, sm.Time}),
				false,
			},
		}

		for _, tc := range testCases {
			tc := tc

			suite.Run(tc.name, func() {
				err := tc.clientState.Validate()

				if tc.expPass {
					suite.Require().NoError(err)
				} else {
					suite.Require().Error(err)
				}
			})
		}
	}
}

func (suite *SoloMachineTestSuite) TestInitialize() {
	// test singlesig and multisig public keys
	for _, sm := range []*ibctesting.Solomachine{suite.solomachine, suite.solomachineMulti} {
		malleatedConsensus := sm.ClientState().ConsensusState
		malleatedConsensus.Timestamp += 10

		testCases := []struct {
			name      string
			consState exported.ConsensusState
			expPass   bool
		}{
			{
				"valid consensus state",
				sm.ConsensusState(),
				true,
			},
			{
				"nil consensus state",
				nil,
				false,
			},
			{
				"invalid consensus state: Tendermint consensus state",
				&ibctm.ConsensusState{},
				false,
			},
			{
				"invalid consensus state: consensus state does not match consensus state in client",
				malleatedConsensus,
				false,
			},
		}

		for _, tc := range testCases {
			tc := tc

			suite.Run(tc.name, func() {
				suite.SetupTest()

				store := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), "solomachine")
				err := sm.ClientState().Initialize(
					suite.chainA.GetContext(), suite.chainA.Codec,
					store, tc.consState,
				)

				if tc.expPass {
					suite.Require().NoError(err, "valid testcase: %s failed", tc.name)
					suite.Require().True(store.Has(host.ClientStateKey()))
				} else {
					suite.Require().Error(err, "invalid testcase: %s passed", tc.name)
					suite.Require().False(store.Has(host.ClientStateKey()))
				}
			})
		}
	}
}

func (suite *SoloMachineTestSuite) TestVerifyMembership() {
	// test singlesig and multisig public keys
	for _, sm := range []*ibctesting.Solomachine{suite.solomachine, suite.solomachineMulti} {

		var (
			clientState *solomachine.ClientState
			path        exported.Path
			proof       []byte
			testingPath *ibctesting.Path
			signBytes   solomachine.SignBytes
			err         error
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
				"success: client state verification",
				func() {
					clientState = sm.ClientState()
					clientStateBz, err := suite.chainA.Codec.Marshal(clientState)
					suite.Require().NoError(err)

					path = sm.GetClientStatePath(counterpartyClientIdentifier)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.GetHeight().GetRevisionHeight(),
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        clientStateBz,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: consensus state verification",
				func() {
					clientState = sm.ClientState()
					consensusState := clientState.ConsensusState
					consensusStateBz, err := suite.chainA.Codec.Marshal(consensusState)
					suite.Require().NoError(err)

					path = sm.GetConsensusStatePath(counterpartyClientIdentifier, clienttypes.NewHeight(0, 1))
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        consensusStateBz,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: connection state verification",
				func() {
					suite.coordinator.SetupConnections(testingPath)

					connectionEnd, found := suite.chainA.GetSimApp().IBCKeeper.ConnectionKeeper.GetConnection(suite.chainA.GetContext(), ibctesting.FirstConnectionID)
					suite.Require().True(found)

					connectionEndBz, err := suite.chainA.Codec.Marshal(&connectionEnd)
					suite.Require().NoError(err)

					path = sm.GetConnectionStatePath(ibctesting.FirstConnectionID)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        connectionEndBz,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: channel state verification",
				func() {
					suite.coordinator.SetupConnections(testingPath)
					suite.coordinator.CreateMockChannels(testingPath)

					channelEnd, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetChannel(suite.chainA.GetContext(), ibctesting.MockPort, ibctesting.FirstChannelID)
					suite.Require().True(found)

					channelEndBz, err := suite.chainA.Codec.Marshal(&channelEnd)
					suite.Require().NoError(err)

					path = sm.GetChannelStatePath(ibctesting.MockPort, ibctesting.FirstChannelID)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        channelEndBz,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: next sequence recv verification",
				func() {
					suite.coordinator.SetupConnections(testingPath)
					suite.coordinator.CreateMockChannels(testingPath)

					nextSeqRecv, found := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetNextSequenceRecv(suite.chainA.GetContext(), ibctesting.MockPort, ibctesting.FirstChannelID)
					suite.Require().True(found)

					path = sm.GetNextSequenceRecvPath(ibctesting.MockPort, ibctesting.FirstChannelID)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        sdk.Uint64ToBigEndian(nextSeqRecv),
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: packet commitment verification",
				func() {
					packet := channeltypes.NewPacket(
						ibctesting.MockPacketData,
						1,
						ibctesting.MockPort,
						ibctesting.FirstChannelID,
						ibctesting.MockPort,
						ibctesting.FirstChannelID,
						clienttypes.NewHeight(0, 10),
						0,
					)

					commitmentBz := channeltypes.CommitPacket(suite.chainA.Codec, packet)
					path = sm.GetPacketCommitmentPath(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        commitmentBz,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: packet acknowledgement verification",
				func() {
					path = sm.GetPacketAcknowledgementPath(ibctesting.MockPort, ibctesting.FirstChannelID, 1)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        ibctesting.MockAcknowledgement,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"success: packet receipt verification",
				func() {
					path = sm.GetPacketReceiptPath(ibctesting.MockPort, ibctesting.FirstChannelID, 1)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.Sequence,
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        []byte{byte(1)}, // packet receipt is stored as a single byte
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"invalid path type - empty",
				func() {
					path = ibcmock.KeyPath{}
				},
				false,
			},
			{
				"malformed proof fails to unmarshal",
				func() {
					path = sm.GetClientStatePath(counterpartyClientIdentifier)
					proof = []byte("invalid proof")
				},
				false,
			},
			{
				"consensus state timestamp is greater than signature",
				func() {
					consensusState := &solomachine.ConsensusState{
						Timestamp: sm.Time + 1,
						PublicKey: sm.ConsensusState().PublicKey,
					}

					clientState = solomachine.NewClientState(sm.Sequence, consensusState)
				},
				false,
			},
			{
				"signature data is nil",
				func() {
					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: nil,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				false,
			},
			{
				"consensus state public key is nil",
				func() {
					clientState.ConsensusState.PublicKey = nil
				},
				false,
			},
			{
				"malformed signature data fails to unmarshal",
				func() {
					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: []byte("invalid signature data"),
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				false,
			},
			{
				"proof is nil",
				func() {
					proof = nil
				},
				false,
			},
			{
				"proof verification failed",
				func() {
					signBytes.Data = []byte("invalid membership data value")
				},
				false,
			},
			{
				"empty path",
				func() {
					path = commitmenttypes.MerklePath{}
				},
				false,
			},
		}

		for _, tc := range testCases {
			tc := tc

			suite.Run(tc.name, func() {
				suite.SetupTest()
				testingPath = ibctesting.NewPath(suite.chainA, suite.chainB)

				clientState = sm.ClientState()

				path = commitmenttypes.NewMerklePath("ibc", "solomachine")
				merklePath, ok := path.(commitmenttypes.MerklePath)
				suite.Require().True(ok)
				key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
				suite.Require().NoError(err)
				signBytes = solomachine.SignBytes{
					Sequence:    sm.GetHeight().GetRevisionHeight(),
					Timestamp:   sm.Time,
					Diversifier: sm.Diversifier,
					Path:        key,
					Data:        []byte("solomachine"),
				}

				signBz, err := suite.chainA.Codec.Marshal(&signBytes)
				suite.Require().NoError(err)

				sig := sm.GenerateSignature(signBz)

				signatureDoc := &solomachine.TimestampedSignatureData{
					SignatureData: sig,
					Timestamp:     sm.Time,
				}

				proof, err = suite.chainA.Codec.Marshal(signatureDoc)
				suite.Require().NoError(err)

				tc.malleate()

				var expSeq uint64
				if clientState.ConsensusState != nil {
					expSeq = clientState.Sequence + 1
				}

				err = clientState.VerifyMembership(
					suite.chainA.GetContext(), suite.store, suite.chainA.Codec,
					clienttypes.ZeroHeight(), 0, 0, // solomachine does not check delay periods
					proof, path, signBytes.Data,
				)

				if tc.expPass {
					suite.Require().NoError(err)
					suite.Require().Equal(expSeq, clientState.Sequence)
					suite.Require().Equal(expSeq, suite.GetSequenceFromStore(), "sequence not updated in the store (%d) on valid test case %s", suite.GetSequenceFromStore(), tc.name)
				} else {
					suite.Require().Error(err)
				}
			})
		}
	}
}

func (suite *SoloMachineTestSuite) TestSignBytesMarshalling() {
	sm := suite.solomachine
	path := []byte("solomachine")
	signBytesNilData := solomachine.SignBytes{
		Sequence:    sm.GetHeight().GetRevisionHeight(),
		Timestamp:   sm.Time,
		Diversifier: sm.Diversifier,
		Path:        path,
		Data:        nil,
	}

	signBytesEmptyArray := solomachine.SignBytes{
		Sequence:    sm.GetHeight().GetRevisionHeight(),
		Timestamp:   sm.Time,
		Diversifier: sm.Diversifier,
		Path:        path,
		Data:        []byte{},
	}

	signBzNil, err := suite.chainA.Codec.Marshal(&signBytesNilData)
	suite.Require().NoError(err)

	signBzEmptyArray, err := suite.chainA.Codec.Marshal(&signBytesEmptyArray)
	suite.Require().NoError(err)

	suite.Require().True(bytes.Equal(signBzNil, signBzEmptyArray))
}

func (suite *SoloMachineTestSuite) TestVerifyNonMembership() {
	// test singlesig and multisig public keys
	for _, sm := range []*ibctesting.Solomachine{suite.solomachine, suite.solomachineMulti} {

		var (
			clientState *solomachine.ClientState
			path        exported.Path
			proof       []byte
			signBytes   solomachine.SignBytes
			err         error
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
				"success: packet receipt absence verification",
				func() {
					path = sm.GetPacketReceiptPath(ibctesting.MockPort, ibctesting.FirstChannelID, 1)
					merklePath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
					suite.Require().NoError(err)
					signBytes = solomachine.SignBytes{
						Sequence:    sm.GetHeight().GetRevisionHeight(),
						Timestamp:   sm.Time,
						Diversifier: sm.Diversifier,
						Path:        key,
						Data:        nil,
					}

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				true,
			},
			{
				"invalid path type",
				func() {
					path = ibcmock.KeyPath{}
				},
				false,
			},
			{
				"malformed proof fails to unmarshal",
				func() {
					path = sm.GetClientStatePath(counterpartyClientIdentifier)
					proof = []byte("invalid proof")
				},
				false,
			},
			{
				"consensus state timestamp is greater than signature",
				func() {
					consensusState := &solomachine.ConsensusState{
						Timestamp: sm.Time + 1,
						PublicKey: sm.ConsensusState().PublicKey,
					}

					clientState = solomachine.NewClientState(sm.Sequence, consensusState)
				},
				false,
			},
			{
				"signature data is nil",
				func() {
					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: nil,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				false,
			},
			{
				"consensus state public key is nil",
				func() {
					clientState.ConsensusState.PublicKey = nil
				},
				false,
			},
			{
				"malformed signature data fails to unmarshal",
				func() {
					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: []byte("invalid signature data"),
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				false,
			},
			{
				"proof is nil",
				func() {
					proof = nil
				},
				false,
			},
			{
				"proof verification failed",
				func() {
					signBytes.Data = []byte("invalid non-membership data value")

					signBz, err := suite.chainA.Codec.Marshal(&signBytes)
					suite.Require().NoError(err)

					sig := sm.GenerateSignature(signBz)

					signatureDoc := &solomachine.TimestampedSignatureData{
						SignatureData: sig,
						Timestamp:     sm.Time,
					}

					proof, err = suite.chainA.Codec.Marshal(signatureDoc)
					suite.Require().NoError(err)
				},
				false,
			},
		}

		for _, tc := range testCases {
			tc := tc

			suite.Run(tc.name, func() {
				clientState = sm.ClientState()

				path = commitmenttypes.NewMerklePath("ibc", "solomachine")
				merklePath, ok := path.(commitmenttypes.MerklePath)
				suite.Require().True(ok)
				key, err := merklePath.GetKey(1) // in a multistore context: index 0 is the key for the IBC store in the multistore, index 1 is the key in the IBC store
				suite.Require().NoError(err)
				signBytes = solomachine.SignBytes{
					Sequence:    sm.GetHeight().GetRevisionHeight(),
					Timestamp:   sm.Time,
					Diversifier: sm.Diversifier,
					Path:        key,
					Data:        nil,
				}

				signBz, err := suite.chainA.Codec.Marshal(&signBytes)
				suite.Require().NoError(err)

				sig := sm.GenerateSignature(signBz)

				signatureDoc := &solomachine.TimestampedSignatureData{
					SignatureData: sig,
					Timestamp:     sm.Time,
				}

				proof, err = suite.chainA.Codec.Marshal(signatureDoc)
				suite.Require().NoError(err)

				tc.malleate()

				var expSeq uint64
				if clientState.ConsensusState != nil {
					expSeq = clientState.Sequence + 1
				}

				err = clientState.VerifyNonMembership(
					suite.chainA.GetContext(), suite.store, suite.chainA.Codec,
					clienttypes.ZeroHeight(), 0, 0, // solomachine does not check delay periods
					proof, path,
				)

				if tc.expPass {
					suite.Require().NoError(err)
					suite.Require().Equal(expSeq, clientState.Sequence)
					suite.Require().Equal(expSeq, suite.GetSequenceFromStore(), "sequence not updated in the store (%d) on valid test case %s", suite.GetSequenceFromStore(), tc.name)
				} else {
					suite.Require().Error(err)
				}
			})
		}
	}
}

func (suite *SoloMachineTestSuite) TestGetTimestampAtHeight() {
	tmPath := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupClients(tmPath)
	// Single setup for all test cases.
	suite.SetupTest()

	testCases := []struct {
		name        string
		clientState *solomachine.ClientState
		height      exported.Height
		expValue    uint64
		expPass     bool
	}{
		{
			name:        "get timestamp at height exists",
			clientState: suite.solomachine.ClientState(),
			height:      suite.solomachine.ClientState().GetLatestHeight(),
			expValue:    suite.solomachine.ClientState().ConsensusState.Timestamp,
			expPass:     true,
		},
	}

	for i, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			ctx := suite.chainA.GetContext()

			ts, err := tc.clientState.GetTimestampAtHeight(
				ctx, suite.store, suite.chainA.Codec, tc.height,
			)

			suite.Require().Equal(tc.expValue, ts)

			if tc.expPass {
				suite.Require().NoError(err, "valid test case %d failed: %s", i, tc.name)
			} else {
				suite.Require().Error(err, "invalid test case %d passed: %s", i, tc.name)
			}
		})
	}
}
