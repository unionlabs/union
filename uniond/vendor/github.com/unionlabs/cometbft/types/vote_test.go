package types

import (
	"fmt"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/libs/protoio"
	cmttime "github.com/cometbft/cometbft/types/time"
)

func examplePrevote() *Vote {
	return exampleVote(byte(PrevoteType))
}

func examplePrecommit() *Vote {
	vote := exampleVote(byte(PrecommitType))
	vote.Extension = []byte("extension")
	vote.ExtensionSignature = []byte("signature")
	return vote
}

func exampleVote(t byte) *Vote {
	stamp, err := time.Parse(TimeFormat, "2017-12-25T03:00:01.234Z")
	if err != nil {
		panic(err)
	}

	return &Vote{
		Type:      SignedMsgType(t),
		Height:    12345,
		Round:     2,
		Timestamp: stamp,
		BlockID: BlockID{
			Hash: tmhash.Sum([]byte("blockID_hash")),
			PartSetHeader: PartSetHeader{
				Total: 1000000,
				Hash:  tmhash.Sum([]byte("blockID_part_set_header_hash")),
			},
		},
		ValidatorAddress: crypto.AddressHash([]byte("validator_address")),
		ValidatorIndex:   56789,
	}
}

func TestVoteSignable(t *testing.T) {
	vote := examplePrecommit()
	v := vote.ToProto()
	signBytes := VoteSignBytes("test_chain_id", v)
	pb := CanonicalizeVote("test_chain_id", v)
	expected, err := protoio.MarshalDelimited(&pb)
	require.NoError(t, err)

	require.Equal(t, expected, signBytes, "Got unexpected sign bytes for Vote.")
}

func TestVoteSignBytesTestVectors(t *testing.T) {
	tests := []struct {
		chainID string
		vote    *Vote
		want    []byte
	}{
		0: {
			"", &Vote{},
			// NOTE: Height and Round are skipped here. This case needs to be considered while parsing.
			[]byte{0xd, 0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1},
		},
		// with proper (fixed size) height and round (PreCommit):
		1: {
			"", &Vote{Height: 1, Round: 1, Type: PrecommitType},
			[]byte{
				0x21,                                   // length
				0x8,                                    // (field_number << 3) | wire_type
				0x2,                                    // PrecommitType
				0x11,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // height
				0x19,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
				0x2a, // (field_number << 3) | wire_type
				// remaining fields (timestamp):
				0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
			},
		},
		// with proper (fixed size) height and round (PreVote):
		2: {
			"", &Vote{Height: 1, Round: 1, Type: PrevoteType},
			[]byte{
				0x21,                                   // length
				0x8,                                    // (field_number << 3) | wire_type
				0x1,                                    // PrevoteType
				0x11,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // height
				0x19,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
				0x2a, // (field_number << 3) | wire_type
				// remaining fields (timestamp):
				0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
			},
		},
		3: {
			"", &Vote{Height: 1, Round: 1},
			[]byte{
				0x1f,                                   // length
				0x11,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // height
				0x19,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
				// remaining fields (timestamp):
				0x2a,
				0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
			},
		},
		// containing non-empty chain_id:
		4: {
			"test_chain_id", &Vote{Height: 1, Round: 1},
			[]byte{
				0x2e,                                   // length
				0x11,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // height
				0x19,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
				// remaining fields:
				0x2a,                                                                // (field_number << 3) | wire_type
				0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1, // timestamp
				// (field_number << 3) | wire_type
				0x32,
				0xd, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x69, 0x6e, 0x5f, 0x69, 0x64,
			}, // chainID
		},
		// containing vote extension
		5: {
			"test_chain_id", &Vote{
				Height:    1,
				Round:     1,
				Extension: []byte("extension"),
			},
			[]byte{
				0x2e,                                   // length
				0x11,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // height
				0x19,                                   // (field_number << 3) | wire_type
				0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
				// remaining fields:
				0x2a,                                                                // (field_number << 3) | wire_type
				0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1, // timestamp
				// (field_number << 3) | wire_type
				0x32,
				0xd, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x69, 0x6e, 0x5f, 0x69, 0x64, // chainID
			}, // chainID
		},
	}
	for i, tc := range tests {
		v := tc.vote.ToProto()
		got := VoteSignBytes(tc.chainID, v)
		assert.Equal(t, len(tc.want), len(got), "test case #%v: got unexpected sign bytes length for Vote.", i)
		assert.Equal(t, tc.want, got, "test case #%v: got unexpected sign bytes for Vote.", i)
	}
}

func TestVoteProposalNotEq(t *testing.T) {
	cv := CanonicalizeVote("", &cmtproto.Vote{Height: 1, Round: 1})
	p := CanonicalizeProposal("", &cmtproto.Proposal{Height: 1, Round: 1})
	vb, err := proto.Marshal(&cv)
	require.NoError(t, err)
	pb, err := proto.Marshal(&p)
	require.NoError(t, err)
	require.NotEqual(t, vb, pb)
}

func TestVoteVerifySignature(t *testing.T) {
	privVal := NewMockPV()
	pubkey, err := privVal.GetPubKey()
	require.NoError(t, err)

	vote := examplePrecommit()
	v := vote.ToProto()
	signBytes := VoteSignBytes("test_chain_id", v)

	// sign it
	err = privVal.SignVote("test_chain_id", v, false)
	require.NoError(t, err)

	// verify the same vote
	valid := pubkey.VerifySignature(VoteSignBytes("test_chain_id", v), v.Signature)
	require.True(t, valid)

	// serialize, deserialize and verify again....
	precommit := new(cmtproto.Vote)
	bs, err := proto.Marshal(v)
	require.NoError(t, err)
	err = proto.Unmarshal(bs, precommit)
	require.NoError(t, err)

	// verify the transmitted vote
	newSignBytes := VoteSignBytes("test_chain_id", precommit)
	require.Equal(t, string(signBytes), string(newSignBytes))
	valid = pubkey.VerifySignature(newSignBytes, precommit.Signature)
	require.True(t, valid)
}

// TestVoteExtension tests that the vote verification behaves correctly in each case
// of vote extension being set on the vote.
func TestVoteExtension(t *testing.T) {
	testCases := []struct {
		name             string
		extension        []byte
		includeSignature bool
		expectError      bool
	}{
		{
			name:             "all fields present",
			extension:        []byte("extension"),
			includeSignature: true,
			expectError:      false,
		},
		{
			name:             "no extension signature",
			extension:        []byte("extension"),
			includeSignature: false,
			expectError:      true,
		},
		{
			name:             "empty extension",
			includeSignature: true,
			expectError:      false,
		},
		{
			name:             "no extension and no signature",
			includeSignature: false,
			expectError:      true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			height, round := int64(1), int32(0)
			privVal := NewMockPV()
			pk, err := privVal.GetPubKey()
			require.NoError(t, err)
			vote := &Vote{
				ValidatorAddress: pk.Address(),
				ValidatorIndex:   0,
				Height:           height,
				Round:            round,
				Timestamp:        cmttime.Now(),
				Type:             PrecommitType,
				BlockID:          makeBlockIDRandom(),
			}

			v := vote.ToProto()
			err = privVal.SignVote("test_chain_id", v, true)
			require.NoError(t, err)
			vote.Signature = v.Signature
			if tc.includeSignature {
				vote.ExtensionSignature = v.ExtensionSignature
			}
			err = vote.VerifyExtension("test_chain_id", pk)
			if tc.expectError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestIsVoteTypeValid(t *testing.T) {
	tc := []struct {
		name string
		in   SignedMsgType
		out  bool
	}{
		{"Prevote", PrevoteType, true},
		{"Precommit", PrecommitType, true},
		{"InvalidType", SignedMsgType(0x3), false},
	}

	for _, tt := range tc {
		t.Run(tt.name, func(_ *testing.T) {
			if rs := IsVoteTypeValid(tt.in); rs != tt.out {
				t.Errorf("got unexpected Vote type. Expected:\n%v\nGot:\n%v", rs, tt.out)
			}
		})
	}
}

func TestVoteVerify(t *testing.T) {
	privVal := NewMockPV()
	pubkey, err := privVal.GetPubKey()
	require.NoError(t, err)

	vote := examplePrevote()
	vote.ValidatorAddress = pubkey.Address()

	err = vote.Verify("test_chain_id", ed25519.GenPrivKey().PubKey())
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, ErrVoteInvalidValidatorAddress, err)
	}

	err = vote.Verify("test_chain_id", pubkey)
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, ErrVoteInvalidSignature, err)
	}
}

func TestVoteString(t *testing.T) {
	str := examplePrecommit().String()
	expected := `Vote{56789:6AF1F4111082 12345/02/SIGNED_MSG_TYPE_PRECOMMIT(Precommit) 8B01023386C3 000000000000 657874656E73 @ 2017-12-25T03:00:01.234Z}` //nolint:lll //ignore line length for tests
	if str != expected {
		t.Errorf("got unexpected string for Vote. Expected:\n%v\nGot:\n%v", expected, str)
	}

	str2 := examplePrevote().String()
	expected = `Vote{56789:6AF1F4111082 12345/02/SIGNED_MSG_TYPE_PREVOTE(Prevote) 8B01023386C3 000000000000 000000000000 @ 2017-12-25T03:00:01.234Z}` //nolint:lll //ignore line length for tests
	if str2 != expected {
		t.Errorf("got unexpected string for Vote. Expected:\n%v\nGot:\n%v", expected, str2)
	}
}

func signVote(t *testing.T, pv PrivValidator, vote *Vote) {
	t.Helper()
	chainID := "test_chain_id"

	v := vote.ToProto()
	require.NoError(t, pv.SignVote(chainID, v, true))
	vote.Signature = v.Signature
	vote.ExtensionSignature = v.ExtensionSignature
}

func TestValidVotes(t *testing.T) {
	privVal := NewMockPV()

	testCases := []struct {
		name         string
		vote         *Vote
		malleateVote func(*Vote)
	}{
		{"good prevote", examplePrevote(), func(_ *Vote) {}},
		{"good precommit without vote extension", examplePrecommit(), func(v *Vote) { v.Extension = nil }},
		{"good precommit with vote extension", examplePrecommit(), func(v *Vote) { v.Extension = []byte("extension") }},
	}
	for _, tc := range testCases {
		signVote(t, privVal, tc.vote)
		tc.malleateVote(tc.vote)
		require.NoError(t, tc.vote.ValidateBasic(), "ValidateBasic for %s", tc.name)
		require.NoError(t, tc.vote.EnsureExtension(), "EnsureExtension for %s", tc.name)
	}
}

func TestInvalidVotes(t *testing.T) {
	privVal := NewMockPV()

	testCases := []struct {
		name         string
		malleateVote func(*Vote)
	}{
		{"negative height", func(v *Vote) { v.Height = -1 }},
		{"negative round", func(v *Vote) { v.Round = -1 }},
		{"zero Height", func(v *Vote) { v.Height = 0 }},
		{"invalid block ID", func(v *Vote) { v.BlockID = BlockID{[]byte{1, 2, 3}, PartSetHeader{111, []byte("blockparts")}} }},
		{"invalid address", func(v *Vote) { v.ValidatorAddress = make([]byte, 1) }},
		{"invalid validator index", func(v *Vote) { v.ValidatorIndex = -1 }},
		{"invalid signature", func(v *Vote) { v.Signature = nil }},
		{"oversized signature", func(v *Vote) { v.Signature = make([]byte, MaxSignatureSize+1) }},
	}
	for _, tc := range testCases {
		prevote := examplePrevote()
		signVote(t, privVal, prevote)
		tc.malleateVote(prevote)
		require.Error(t, prevote.ValidateBasic(), "ValidateBasic for %s in invalid prevote", tc.name)
		require.NoError(t, prevote.EnsureExtension(), "EnsureExtension for %s in invalid prevote", tc.name)

		precommit := examplePrecommit()
		signVote(t, privVal, precommit)
		tc.malleateVote(precommit)
		require.Error(t, precommit.ValidateBasic(), "ValidateBasic for %s in invalid precommit", tc.name)
		require.NoError(t, precommit.EnsureExtension(), "EnsureExtension for %s in invalid precommit", tc.name)
	}
}

func TestInvalidPrevotes(t *testing.T) {
	privVal := NewMockPV()

	testCases := []struct {
		name         string
		malleateVote func(*Vote)
	}{
		{"vote extension present", func(v *Vote) { v.Extension = []byte("extension") }},
		{"vote extension signature present", func(v *Vote) { v.ExtensionSignature = []byte("signature") }},
	}
	for _, tc := range testCases {
		prevote := examplePrevote()
		signVote(t, privVal, prevote)
		tc.malleateVote(prevote)
		require.Error(t, prevote.ValidateBasic(), "ValidateBasic for %s", tc.name)
		require.NoError(t, prevote.EnsureExtension(), "EnsureExtension for %s", tc.name)
	}
}

func TestInvalidPrecommitExtensions(t *testing.T) {
	privVal := NewMockPV()

	testCases := []struct {
		name         string
		malleateVote func(*Vote)
	}{
		{"vote extension present without signature", func(v *Vote) {
			v.Extension = []byte("extension")
			v.ExtensionSignature = nil
		}},
		{"oversized vote extension signature", func(v *Vote) { v.ExtensionSignature = make([]byte, MaxSignatureSize+1) }},
	}
	for _, tc := range testCases {
		precommit := examplePrecommit()
		signVote(t, privVal, precommit)
		tc.malleateVote(precommit)
		// ValidateBasic ensures that vote extensions, if present, are well formed
		require.Error(t, precommit.ValidateBasic(), "ValidateBasic for %s", tc.name)
	}
}

func TestEnsureVoteExtension(t *testing.T) {
	privVal := NewMockPV()

	testCases := []struct {
		name         string
		malleateVote func(*Vote)
		expectError  bool
	}{
		{"vote extension signature absent", func(v *Vote) {
			v.Extension = nil
			v.ExtensionSignature = nil
		}, true},
		{"vote extension signature present", func(v *Vote) {
			v.ExtensionSignature = []byte("extension signature")
		}, false},
	}
	for _, tc := range testCases {
		precommit := examplePrecommit()
		signVote(t, privVal, precommit)
		tc.malleateVote(precommit)
		if tc.expectError {
			require.Error(t, precommit.EnsureExtension(), "EnsureExtension for %s", tc.name)
		} else {
			require.NoError(t, precommit.EnsureExtension(), "EnsureExtension for %s", tc.name)
		}
	}
}

func TestVoteProtobuf(t *testing.T) {
	privVal := NewMockPV()
	vote := examplePrecommit()
	v := vote.ToProto()
	err := privVal.SignVote("test_chain_id", v, false)
	vote.Signature = v.Signature
	require.NoError(t, err)

	testCases := []struct {
		msg                 string
		vote                *Vote
		convertsOk          bool
		passesValidateBasic bool
	}{
		{"success", vote, true, true},
		{"fail vote validate basic", &Vote{}, true, false},
	}
	for _, tc := range testCases {
		protoProposal := tc.vote.ToProto()

		v, err := VoteFromProto(protoProposal)
		if tc.convertsOk {
			require.NoError(t, err)
		} else {
			require.Error(t, err)
		}

		err = v.ValidateBasic()
		if tc.passesValidateBasic {
			require.NoError(t, err)
			require.Equal(t, tc.vote, v, tc.msg)
		} else {
			require.Error(t, err)
		}
	}
}

func TestSignAndCheckVote(t *testing.T) {
	privVal := NewMockPV()

	testCases := []struct {
		name              string
		extensionsEnabled bool
		vote              *Vote
		expectError       bool
	}{
		{
			name:              "precommit with extension signature",
			extensionsEnabled: true,
			vote:              examplePrecommit(),
			expectError:       false,
		},
		{
			name:              "precommit with extension signature",
			extensionsEnabled: false,
			vote:              examplePrecommit(),
			expectError:       false,
		},
		{
			name:              "precommit with extension signature for a nil block",
			extensionsEnabled: true,
			vote: func() *Vote {
				v := examplePrecommit()
				v.BlockID = BlockID{make([]byte, 0), PartSetHeader{0, make([]byte, 0)}}
				return v
			}(),
			expectError: true,
		},
		{
			name:              "precommit with extension signature for a nil block",
			extensionsEnabled: false,
			vote: func() *Vote {
				v := examplePrecommit()
				v.BlockID = BlockID{make([]byte, 0), PartSetHeader{0, make([]byte, 0)}}
				return v
			}(),
			expectError: false,
		},
		{
			name:              "precommit without extension",
			extensionsEnabled: true,
			vote: func() *Vote {
				v := examplePrecommit()
				v.Extension = make([]byte, 0)
				return v
			}(),
			expectError: false,
		},
		{
			name:              "precommit without extension",
			extensionsEnabled: false,
			vote: func() *Vote {
				v := examplePrecommit()
				v.Extension = make([]byte, 0)
				return v
			}(),
			expectError: false,
		},
		{
			name:              "prevote",
			extensionsEnabled: true,
			vote:              examplePrevote(),
			expectError:       true,
		},
		{
			name:              "prevote",
			extensionsEnabled: false,
			vote:              examplePrevote(),
			expectError:       false,
		},
		{
			name:              "prevote with extension",
			extensionsEnabled: true,
			vote: func() *Vote {
				v := examplePrevote()
				v.Extension = []byte("extension")
				return v
			}(),
			expectError: true,
		},
		{
			name:              "prevote with extension",
			extensionsEnabled: false,
			vote: func() *Vote {
				v := examplePrevote()
				v.Extension = []byte("extension")
				return v
			}(),
			expectError: false,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("%s (extensionsEnabled: %t) ", tc.name, tc.extensionsEnabled), func(t *testing.T) {
			_, err := SignAndCheckVote(tc.vote, privVal, "test_chain_id", tc.extensionsEnabled)
			if tc.expectError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
