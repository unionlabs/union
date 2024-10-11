package v1_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	v1 "cosmossdk.io/x/gov/types/v1"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestVoteAlias(t *testing.T) {
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec

	testCases := []struct {
		name           string
		input          string
		expected       v1.MsgVote
		expectedErrMsg string
	}{
		{
			name:  "valid vote",
			input: `{"proposal_id":"1","voter":"cosmos1qperwt9wrnkg5k9e5gzfgjppzpqhyav5j24d66","option":"VOTE_OPTION_YES","metadata":"test"}`,
			expected: v1.MsgVote{
				ProposalId: 1,
				Voter:      "cosmos1qperwt9wrnkg5k9e5gzfgjppzpqhyav5j24d66",
				Option:     v1.VoteOption_VOTE_OPTION_YES,
				Metadata:   "test",
			},
		},
		{
			name:  "valid vote alias",
			input: `{"proposal_id":"1","voter":"cosmos1qperwt9wrnkg5k9e5gzfgjppzpqhyav5j24d66","option":"VOTE_OPTION_ONE","metadata":"test"}`,
			expected: v1.MsgVote{
				ProposalId: 1,
				Voter:      "cosmos1qperwt9wrnkg5k9e5gzfgjppzpqhyav5j24d66",
				Option:     v1.VoteOption_VOTE_OPTION_ONE,
				Metadata:   "test",
			},
		},
		{
			name:           "invalid vote",
			input:          `{"proposal_id":"1","voter":"cosmos1qperwt9wrnkg5k9e5gzfgjppzpqhyav5j24d66","option":"VOTE_OPTION_HELLO","metadata":"test"}`,
			expectedErrMsg: "unknown value \"VOTE_OPTION_HELLO\" for enum cosmos.gov.v1.VoteOption",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			vote := &v1.MsgVote{}
			err := cdc.UnmarshalJSON([]byte(tc.input), vote)
			if tc.expectedErrMsg != "" {
				require.ErrorContains(t, err, tc.expectedErrMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
