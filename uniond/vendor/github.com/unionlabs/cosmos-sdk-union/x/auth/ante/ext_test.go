package ante_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
)

func TestRejectExtensionOptionsDecorator(t *testing.T) {
	suite := SetupTestSuite(t, true)

	testCases := []struct {
		msg   string
		allow bool
	}{
		{"allow extension", true},
		{"reject extension", false},
	}
	for _, tc := range testCases {
		t.Run(tc.msg, func(t *testing.T) {
			txBuilder := suite.clientCtx.TxConfig.NewTxBuilder()

			reod := ante.NewExtensionOptionsDecorator(func(_ *codectypes.Any) bool {
				return tc.allow
			})
			antehandler := sdk.ChainAnteDecorators(reod)

			// no extension options should not trigger an error
			theTx := txBuilder.GetTx()
			_, err := antehandler(suite.ctx, theTx, false)
			require.NoError(t, err)

			extOptsTxBldr, ok := txBuilder.(tx.ExtensionOptionsTxBuilder)
			if !ok {
				// if we can't set extension options, this decorator doesn't apply and we're done
				return
			}

			// set an extension option and check
			any, err := codectypes.NewAnyWithValue(testdata.NewTestMsg())
			require.NoError(t, err)
			extOptsTxBldr.SetExtensionOptions(any)
			theTx = txBuilder.GetTx()
			_, err = antehandler(suite.ctx, theTx, false)
			if tc.allow {
				require.NoError(t, err)
			} else {
				require.EqualError(t, err, "unknown extension options")
			}
		})
	}
}
