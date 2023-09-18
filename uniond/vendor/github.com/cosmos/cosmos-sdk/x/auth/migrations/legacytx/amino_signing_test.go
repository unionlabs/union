package legacytx

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func TestLegacyAminoJSONHandler_GetSignBytes(t *testing.T) {
	priv1 := secp256k1.GenPrivKey()
	addr1 := sdk.AccAddress(priv1.PubKey().Address())
	priv2 := secp256k1.GenPrivKey()
	addr2 := sdk.AccAddress(priv2.PubKey().Address())

	coins := sdk.Coins{sdk.NewInt64Coin("foocoin", 10)}

	fee := StdFee{
		Amount: coins,
		Gas:    10000,
	}
	memo := "foo"
	msgs := []sdk.Msg{
		testdata.NewTestMsg(addr1, addr2),
	}

	var (
		chainId              = "test-chain"
		accNum        uint64 = 7
		seqNum        uint64 = 7
		timeoutHeight uint64 = 10
	)

	tx := StdTx{
		Msgs:          msgs,
		Fee:           fee,
		Signatures:    nil,
		Memo:          memo,
		TimeoutHeight: timeoutHeight,
	}

	handler := stdTxSignModeHandler{}
	signingData := signing.SignerData{
		Address:       addr1.String(),
		ChainID:       chainId,
		AccountNumber: accNum,
		Sequence:      seqNum,
		PubKey:        priv1.PubKey(),
	}
	signBz, err := handler.GetSignBytes(signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, signingData, tx)
	require.NoError(t, err)

	expectedSignBz := StdSignBytes(chainId, accNum, seqNum, timeoutHeight, fee, msgs, memo, nil)

	require.Equal(t, expectedSignBz, signBz)

	// expect error with wrong sign mode
	_, err = handler.GetSignBytes(signingtypes.SignMode_SIGN_MODE_DIRECT, signingData, tx)
	require.Error(t, err)
}

func TestLegacyAminoJSONHandler_DefaultMode(t *testing.T) {
	handler := stdTxSignModeHandler{}
	require.Equal(t, signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, handler.DefaultMode())
}

func TestLegacyAminoJSONHandler_Modes(t *testing.T) {
	handler := stdTxSignModeHandler{}
	require.Equal(t, []signingtypes.SignMode{signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON}, handler.Modes())
}
