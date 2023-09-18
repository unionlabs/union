package tx

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/types"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestDecodeMultisignatures(t *testing.T) {
	testSigs := [][]byte{
		[]byte("dummy1"),
		[]byte("dummy2"),
		[]byte("dummy3"),
	}

	badMultisig := testdata.BadMultiSignature{
		Signatures:     testSigs,
		MaliciousField: []byte("bad stuff..."),
	}
	bz, err := badMultisig.Marshal()
	require.NoError(t, err)

	_, err = decodeMultisignatures(bz)
	require.Error(t, err)

	goodMultisig := types.MultiSignature{
		Signatures: testSigs,
	}
	bz, err = goodMultisig.Marshal()
	require.NoError(t, err)

	decodedSigs, err := decodeMultisignatures(bz)
	require.NoError(t, err)

	require.Equal(t, testSigs, decodedSigs)
}
