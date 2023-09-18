package types_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/evidence/exported"
	"github.com/cosmos/cosmos-sdk/x/evidence/types"
)

func testMsgSubmitEvidence(t *testing.T, e exported.Evidence, s sdk.AccAddress) exported.MsgSubmitEvidenceI {
	msg, err := types.NewMsgSubmitEvidence(s, e)
	require.NoError(t, err)
	return msg
}

func TestMsgSubmitEvidence(t *testing.T) {
	pk := ed25519.GenPrivKey()
	submitter := sdk.AccAddress("test________________")

	testCases := []struct {
		msg       sdk.Msg
		submitter sdk.AccAddress
		expectErr bool
	}{
		{
			testMsgSubmitEvidence(t, &types.Equivocation{
				Height:           0,
				Power:            100,
				Time:             time.Now().UTC(),
				ConsensusAddress: pk.PubKey().Address().String(),
			}, submitter),
			submitter,
			true,
		},
		{
			testMsgSubmitEvidence(t, &types.Equivocation{
				Height:           10,
				Power:            100,
				Time:             time.Now().UTC(),
				ConsensusAddress: pk.PubKey().Address().String(),
			}, submitter),
			submitter,
			false,
		},
	}

	for i, tc := range testCases {
		require.Equal(t, sdk.MsgTypeURL(&types.MsgSubmitEvidence{}), sdk.MsgTypeURL(tc.msg), "unexpected result for tc #%d", i)
		require.Equal(t, tc.expectErr, tc.msg.ValidateBasic() != nil, "unexpected result for tc #%d", i)

		if !tc.expectErr {
			require.Equal(t, tc.msg.GetSigners(), []sdk.AccAddress{tc.submitter}, "unexpected result for tc #%d", i)
		}
	}
}
