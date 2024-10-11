package types

import (
	"encoding/hex"
	"testing"

	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func generateBlobIdentifiers(n int) []*eth.BlobIdentifier {
	r := make([]*eth.BlobIdentifier, n)
	for i := 0; i < n; i++ {
		r[i] = &eth.BlobIdentifier{
			BlockRoot: bytesutil.PadTo([]byte{byte(i)}, 32),
			Index:     0,
		}
	}
	return r
}

func TestBlobSidecarsByRootReq_MarshalSSZ(t *testing.T) {
	cases := []struct {
		name         string
		ids          []*eth.BlobIdentifier
		marshalErr   error
		unmarshalErr error
		unmarshalMod func([]byte) []byte
	}{
		{
			name: "empty list",
		},
		{
			name: "single item list",
			ids:  generateBlobIdentifiers(1),
		},
		{
			name: "10 item list",
			ids:  generateBlobIdentifiers(10),
		},
		{
			name: "wonky unmarshal size",
			ids:  generateBlobIdentifiers(10),
			unmarshalMod: func(in []byte) []byte {
				in = append(in, byte(0))
				return in
			},
			unmarshalErr: ssz.ErrIncorrectByteSize,
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			r := BlobSidecarsByRootReq(c.ids)
			by, err := r.MarshalSSZ()
			if c.marshalErr != nil {
				require.ErrorIs(t, err, c.marshalErr)
				return
			}
			require.NoError(t, err)
			if c.unmarshalMod != nil {
				by = c.unmarshalMod(by)
			}
			got := &BlobSidecarsByRootReq{}
			err = got.UnmarshalSSZ(by)
			if c.unmarshalErr != nil {
				require.ErrorIs(t, err, c.unmarshalErr)
				return
			}
			require.NoError(t, err)
			for i, gid := range *got {
				require.DeepEqual(t, c.ids[i], gid)
			}
		})
	}
}

func TestBeaconBlockByRootsReq_Limit(t *testing.T) {
	fixedRoots := make([][32]byte, 0)
	for i := uint64(0); i < params.BeaconConfig().MaxRequestBlocks+100; i++ {
		fixedRoots = append(fixedRoots, [32]byte{byte(i)})
	}
	req := BeaconBlockByRootsReq(fixedRoots)

	_, err := req.MarshalSSZ()
	require.ErrorContains(t, "beacon block by roots request exceeds max size", err)

	buf := make([]byte, 0)
	for _, rt := range fixedRoots {
		buf = append(buf, rt[:]...)
	}
	req2 := BeaconBlockByRootsReq(nil)
	require.ErrorContains(t, "expected buffer with length of up to", req2.UnmarshalSSZ(buf))
}

func TestErrorResponse_Limit(t *testing.T) {
	errorMessage := make([]byte, 0)
	// Provide a message of size 6400 bytes.
	for i := uint64(0); i < 200; i++ {
		byteArr := [32]byte{byte(i)}
		errorMessage = append(errorMessage, byteArr[:]...)
	}
	errMsg := ErrorMessage{}
	require.ErrorContains(t, "expected buffer with length of upto", errMsg.UnmarshalSSZ(errorMessage))
}

func TestRoundTripSerialization(t *testing.T) {
	roundTripTestBlocksByRootReq(t)
	roundTripTestErrorMessage(t)
}

func roundTripTestBlocksByRootReq(t *testing.T) {
	fixedRoots := make([][32]byte, 0)
	for i := 0; i < 200; i++ {
		fixedRoots = append(fixedRoots, [32]byte{byte(i)})
	}
	req := BeaconBlockByRootsReq(fixedRoots)

	marshalledObj, err := req.MarshalSSZ()
	require.NoError(t, err)
	newVal := BeaconBlockByRootsReq(nil)

	require.NoError(t, newVal.UnmarshalSSZ(marshalledObj))
	assert.DeepEqual(t, [][32]byte(newVal), fixedRoots)
}

func roundTripTestErrorMessage(t *testing.T) {
	errMsg := []byte{'e', 'r', 'r', 'o', 'r'}
	sszErr := make(ErrorMessage, len(errMsg))
	copy(sszErr, errMsg)

	marshalledObj, err := sszErr.MarshalSSZ()
	require.NoError(t, err)
	newVal := ErrorMessage(nil)

	require.NoError(t, newVal.UnmarshalSSZ(marshalledObj))
	assert.DeepEqual(t, []byte(newVal), errMsg)
}

func TestSSZBytes_HashTreeRoot(t *testing.T) {
	tests := []struct {
		name        string
		actualValue []byte
		root        []byte
		wantErr     bool
	}{
		{
			name:        "random1",
			actualValue: hexDecodeOrDie(t, "844e1063e0b396eed17be8eddb7eecd1fe3ea46542a4b72f7466e77325e5aa6d"),
			root:        hexDecodeOrDie(t, "844e1063e0b396eed17be8eddb7eecd1fe3ea46542a4b72f7466e77325e5aa6d"),
			wantErr:     false,
		},
		{
			name:        "random1",
			actualValue: hexDecodeOrDie(t, "7b16162ecd9a28fa80a475080b0e4fff4c27efe19ce5134ce3554b72274d59fd534400ba4c7f699aa1c307cd37c2b103"),
			root:        hexDecodeOrDie(t, "128ed34ee798b9f00716f9ba5c000df5c99443dabc4d3f2e9bb86c77c732e007"),
			wantErr:     false,
		},
		{
			name:        "random2",
			actualValue: []byte{},
			root:        hexDecodeOrDie(t, "0000000000000000000000000000000000000000000000000000000000000000"),
			wantErr:     false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := SSZBytes(tt.actualValue)
			htr, err := s.HashTreeRoot()
			require.NoError(t, err)
			require.DeepEqual(t, tt.root, htr[:])
		})
	}
}

func TestGoodbyeCodes(t *testing.T) {
	assert.Equal(t, primitives.SSZUint64(1), GoodbyeCodeClientShutdown)
	assert.Equal(t, primitives.SSZUint64(2), GoodbyeCodeWrongNetwork)
	assert.Equal(t, primitives.SSZUint64(3), GoodbyeCodeGenericError)
	assert.Equal(t, primitives.SSZUint64(128), GoodbyeCodeUnableToVerifyNetwork)
	assert.Equal(t, primitives.SSZUint64(129), GoodbyeCodeTooManyPeers)
	assert.Equal(t, primitives.SSZUint64(250), GoodbyeCodeBadScore)
	assert.Equal(t, primitives.SSZUint64(251), GoodbyeCodeBanned)

}

func hexDecodeOrDie(t *testing.T, str string) []byte {
	decoded, err := hex.DecodeString(str)
	require.NoError(t, err)
	return decoded
}
