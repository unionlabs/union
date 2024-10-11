package legacytx_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
)

func TestStdSignatureMarshalYAML(t *testing.T) {
	_, pk, _ := testdata.KeyTestPubAddr()
	pkStr := pk.String()

	testCases := []struct {
		sig      legacytx.StdSignature
		expected string
	}{
		{
			legacytx.StdSignature{},
			"pub_key: \"\"\nsignature: \"\"\n",
		},
		{
			legacytx.StdSignature{PubKey: pk, Signature: []byte("dummySig")},
			fmt.Sprintf("pub_key: %s\nsignature: 64756D6D79536967\n", pkStr),
		},
		{
			legacytx.StdSignature{PubKey: pk, Signature: nil},
			fmt.Sprintf("pub_key: %s\nsignature: \"\"\n", pkStr),
		},
	}

	for i, tc := range testCases {
		bz2, err := tc.sig.MarshalYAML()
		require.NoError(t, err)
		require.Equal(t, tc.expected, bz2.(string), "test case #%d", i)
	}
}
