package privval

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	privvalproto "github.com/cometbft/cometbft/api/cometbft/privval/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/types"
	cmterrors "github.com/cometbft/cometbft/types/errors"
	cmttime "github.com/cometbft/cometbft/types/time"
)

type signerTestCase struct {
	chainID      string
	mockPV       types.PrivValidator
	signerClient *SignerClient
	signerServer *SignerServer
}

func getSignerTestCases(t *testing.T) []signerTestCase {
	t.Helper()
	testCases := make([]signerTestCase, 0)

	// Get test cases for each possible dialer (DialTCP / DialUnix / etc)
	for _, dtc := range getDialerTestCases(t) {
		chainID := cmtrand.Str(12)
		mockPV := types.NewMockPV()

		// get a pair of signer listener, signer dialer endpoints
		sl, sd := getMockEndpoints(t, dtc.addr, dtc.dialer)
		sc, err := NewSignerClient(sl, chainID)
		require.NoError(t, err)
		ss := NewSignerServer(sd, chainID, mockPV)

		err = ss.Start()
		require.NoError(t, err)

		tc := signerTestCase{
			chainID:      chainID,
			mockPV:       mockPV,
			signerClient: sc,
			signerServer: ss,
		}

		testCases = append(testCases, tc)
	}

	return testCases
}

func TestSignerClose(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		err := tc.signerClient.Close()
		require.NoError(t, err)

		err = tc.signerServer.Stop()
		require.NoError(t, err)
	}
}

func TestSignerPing(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		err := tc.signerClient.Ping()
		require.NoError(t, err)
	}
}

func TestSignerGetPubKey(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		pubKey, err := tc.signerClient.GetPubKey()
		require.NoError(t, err)
		expectedPubKey, err := tc.mockPV.GetPubKey()
		require.NoError(t, err)

		assert.Equal(t, expectedPubKey, pubKey)

		pubKey, err = tc.signerClient.GetPubKey()
		require.NoError(t, err)
		expectedpk, err := tc.mockPV.GetPubKey()
		require.NoError(t, err)
		expectedAddr := expectedpk.Address()

		assert.Equal(t, expectedAddr, pubKey.Address())
	}
}

func TestSignerProposal(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		have := &types.Proposal{
			Type:      types.ProposalType,
			Height:    1,
			Round:     2,
			POLRound:  2,
			BlockID:   types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp: ts,
		}
		want := &types.Proposal{
			Type:      types.ProposalType,
			Height:    1,
			Round:     2,
			POLRound:  2,
			BlockID:   types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp: ts,
		}

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		require.NoError(t, tc.mockPV.SignProposal(tc.chainID, want.ToProto()))
		require.NoError(t, tc.signerClient.SignProposal(tc.chainID, have.ToProto()))

		assert.Equal(t, want.Signature, have.Signature)
	}
}

func TestSignerVote(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		valAddr := cmtrand.Bytes(crypto.AddressSize)
		want := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
		}

		have := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
		}

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		require.NoError(t, tc.mockPV.SignVote(tc.chainID, want.ToProto(), false))
		require.NoError(t, tc.signerClient.SignVote(tc.chainID, have.ToProto(), false))

		assert.Equal(t, want.Signature, have.Signature)
		assert.Nil(t, have.Signature)
		assert.Equal(t, want.ExtensionSignature, have.ExtensionSignature)
	}
}

func TestSignerVoteResetDeadline(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		valAddr := cmtrand.Bytes(crypto.AddressSize)
		want := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
		}

		have := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
		}

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		time.Sleep(testTimeoutReadWrite2o3)

		require.NoError(t, tc.mockPV.SignVote(tc.chainID, want.ToProto(), false))
		require.NoError(t, tc.signerClient.SignVote(tc.chainID, have.ToProto(), false))
		assert.Equal(t, want.Signature, have.Signature)
		assert.Nil(t, have.Signature)
		assert.Equal(t, want.ExtensionSignature, have.ExtensionSignature)

		// TODO(jleni): Clarify what is actually being tested

		// This would exceed the deadline if it was not extended by the previous message
		time.Sleep(testTimeoutReadWrite2o3)

		require.NoError(t, tc.mockPV.SignVote(tc.chainID, want.ToProto(), false))
		require.NoError(t, tc.signerClient.SignVote(tc.chainID, have.ToProto(), false))
		assert.Equal(t, want.Signature, have.Signature)
		assert.Nil(t, have.Signature)
		assert.Equal(t, want.ExtensionSignature, have.ExtensionSignature)
	}
}

func TestSignerVoteKeepAlive(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		valAddr := cmtrand.Bytes(crypto.AddressSize)
		want := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
		}

		have := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
		}

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		// Check that even if the client does not request a
		// signature for a long time. The service is still available

		// in this particular case, we use the dialer logger to ensure that
		// test messages are properly interleaved in the test logs
		tc.signerServer.Logger.Debug("TEST: Forced Wait -------------------------------------------------")
		time.Sleep(testTimeoutReadWrite * 3)
		tc.signerServer.Logger.Debug("TEST: Forced Wait DONE---------------------------------------------")

		require.NoError(t, tc.mockPV.SignVote(tc.chainID, want.ToProto(), false))
		require.NoError(t, tc.signerClient.SignVote(tc.chainID, have.ToProto(), false))

		assert.Equal(t, want.Signature, have.Signature)
		assert.Nil(t, have.Signature)
		assert.Equal(t, want.ExtensionSignature, have.ExtensionSignature)
	}
}

func TestSignerSignProposalErrors(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		// Replace service with a mock that always fails
		tc.signerServer.privVal = types.NewErroringMockPV()
		tc.mockPV = types.NewErroringMockPV()

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		proposal := &types.Proposal{
			Type:      types.ProposalType,
			Height:    1,
			Round:     2,
			POLRound:  2,
			BlockID:   types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp: ts,
			Signature: []byte("signature"),
		}

		err := tc.signerClient.SignProposal(tc.chainID, proposal.ToProto())
		require.Equal(t, err.(*RemoteSignerError).Description, types.ErroringMockPVErr.Error())

		err = tc.mockPV.SignProposal(tc.chainID, proposal.ToProto())
		require.Error(t, err)

		err = tc.signerClient.SignProposal(tc.chainID, proposal.ToProto())
		require.Error(t, err)
	}
}

func TestSignerSignVoteErrors(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		valAddr := cmtrand.Bytes(crypto.AddressSize)
		vote := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
			Signature:        []byte("signature"),
		}

		// Replace signer service privval with one that always fails
		tc.signerServer.privVal = types.NewErroringMockPV()
		tc.mockPV = types.NewErroringMockPV()

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		err := tc.signerClient.SignVote(tc.chainID, vote.ToProto(), false)
		require.Equal(t, err.(*RemoteSignerError).Description, types.ErroringMockPVErr.Error())

		err = tc.mockPV.SignVote(tc.chainID, vote.ToProto(), false)
		require.Error(t, err)

		err = tc.signerClient.SignVote(tc.chainID, vote.ToProto(), false)
		require.Error(t, err)
	}
}

func brokenHandler(_ types.PrivValidator, request privvalproto.Message, _ string) (privvalproto.Message, error) {
	var res privvalproto.Message
	var err error

	switch r := request.Sum.(type) {
	// This is broken and will answer most requests with a pubkey response
	case *privvalproto.Message_PubKeyRequest:
		res = mustWrapMsg(&privvalproto.PubKeyResponse{PubKeyType: "", PubKeyBytes: []byte{}, Error: nil})
	case *privvalproto.Message_SignVoteRequest:
		res = mustWrapMsg(&privvalproto.PubKeyResponse{PubKeyType: "", PubKeyBytes: []byte{}, Error: nil})
	case *privvalproto.Message_SignProposalRequest:
		res = mustWrapMsg(&privvalproto.PubKeyResponse{PubKeyType: "", PubKeyBytes: []byte{}, Error: nil})
	case *privvalproto.Message_PingRequest:
		err, res = nil, mustWrapMsg(&privvalproto.PingResponse{})
	default:
		err = fmt.Errorf("unknown msg: %v", r)
	}

	return res, err
}

func TestSignerUnexpectedResponse(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		tc.signerServer.privVal = types.NewMockPV()
		tc.mockPV = types.NewMockPV()

		tc.signerServer.SetRequestHandler(brokenHandler)

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		ts := cmttime.Now()
		want := &types.Vote{Timestamp: ts, Type: types.PrecommitType}

		e := tc.signerClient.SignVote(tc.chainID, want.ToProto(), false)
		require.ErrorIs(t, e, cmterrors.ErrRequiredField{Field: "response"})
	}
}

func TestSignerVoteExtension(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		ts := cmttime.Now()
		hash := cmtrand.Bytes(tmhash.Size)
		valAddr := cmtrand.Bytes(crypto.AddressSize)
		want := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
			Extension:        []byte("hello"),
		}

		have := &types.Vote{
			Type:             types.PrecommitType,
			Height:           1,
			Round:            2,
			BlockID:          types.BlockID{Hash: hash, PartSetHeader: types.PartSetHeader{Hash: hash, Total: 2}},
			Timestamp:        ts,
			ValidatorAddress: valAddr,
			ValidatorIndex:   1,
			Extension:        []byte("world"),
		}

		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		require.NoError(t, tc.mockPV.SignVote(tc.chainID, want.ToProto(), true))
		require.NoError(t, tc.signerClient.SignVote(tc.chainID, have.ToProto(), true))

		assert.Equal(t, want.Signature, have.Signature)
		assert.Equal(t, want.ExtensionSignature, have.ExtensionSignature)
	}
}

func TestSignerSignBytes(t *testing.T) {
	for _, tc := range getSignerTestCases(t) {
		t.Cleanup(func() {
			if err := tc.signerServer.Stop(); err != nil {
				t.Error(err)
			}
		})
		t.Cleanup(func() {
			if err := tc.signerClient.Close(); err != nil {
				t.Error(err)
			}
		})

		bytes := cmtrand.Bytes(32)
		signature, err := tc.signerClient.SignBytes(bytes)
		require.NoError(t, err)

		pubKey, err := tc.mockPV.GetPubKey()
		require.NoError(t, err)
		require.True(t, pubKey.VerifySignature(bytes, signature))
	}
}
