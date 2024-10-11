package types

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
)

func TestBlockMeta_ToProto(t *testing.T) {
	h := makeRandHeader()
	bi := BlockID{Hash: h.Hash(), PartSetHeader: PartSetHeader{Total: 123, Hash: cmtrand.Bytes(tmhash.Size)}}

	bm := &BlockMeta{
		BlockID:   bi,
		BlockSize: 200,
		Header:    h,
		NumTxs:    0,
	}

	tests := []struct {
		testName string
		bm       *BlockMeta
		expErr   bool
	}{
		{"success", bm, false},
		{"failure nil", nil, true},
	}

	for _, tt := range tests {
		t.Run(tt.testName, func(t *testing.T) {
			pb := tt.bm.ToProto()

			bm, err := BlockMetaFromTrustedProto(pb)

			if !tt.expErr {
				require.NoError(t, err, tt.testName)
				require.Equal(t, tt.bm, bm, tt.testName)
			} else {
				require.Error(t, err, tt.testName)
			}
		})
	}
}

func TestBlockMeta_ValidateBasic(t *testing.T) {
	h := makeRandHeader()
	bi := BlockID{Hash: h.Hash(), PartSetHeader: PartSetHeader{Total: 123, Hash: cmtrand.Bytes(tmhash.Size)}}
	bi2 := BlockID{
		Hash:          cmtrand.Bytes(tmhash.Size),
		PartSetHeader: PartSetHeader{Total: 123, Hash: cmtrand.Bytes(tmhash.Size)},
	}
	bi3 := BlockID{
		Hash:          []byte("incorrect hash"),
		PartSetHeader: PartSetHeader{Total: 123, Hash: []byte("incorrect hash")},
	}

	bm := &BlockMeta{
		BlockID:   bi,
		BlockSize: 200,
		Header:    h,
		NumTxs:    0,
	}

	bm2 := &BlockMeta{
		BlockID:   bi2,
		BlockSize: 200,
		Header:    h,
		NumTxs:    0,
	}

	bm3 := &BlockMeta{
		BlockID:   bi3,
		BlockSize: 200,
		Header:    h,
		NumTxs:    0,
	}

	tests := []struct {
		name    string
		bm      *BlockMeta
		wantErr bool
	}{
		{"success", bm, false},
		{"failure wrong blockID hash", bm2, true},
		{"failure wrong length blockID hash", bm3, true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if err := tt.bm.ValidateBasic(); (err != nil) != tt.wantErr {
				t.Errorf("BlockMeta.ValidateBasic() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
