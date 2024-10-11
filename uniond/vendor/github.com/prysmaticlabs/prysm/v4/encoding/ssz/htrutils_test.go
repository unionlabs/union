package ssz_test

import (
	"reflect"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestUint64Root(t *testing.T) {
	uintVal := uint64(1234567890)
	expected := [32]byte{210, 2, 150, 73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}

	result := ssz.Uint64Root(uintVal)
	assert.Equal(t, expected, result)
}

func TestForkRoot(t *testing.T) {
	testFork := ethpb.Fork{
		PreviousVersion: []byte{123},
		CurrentVersion:  []byte{124},
		Epoch:           1234567890,
	}
	expected := [32]byte{19, 46, 77, 103, 92, 175, 247, 33, 100, 64, 17, 111, 199, 145, 69, 38, 217, 112, 6, 16, 149, 201, 225, 144, 192, 228, 197, 172, 157, 78, 114, 140}

	result, err := ssz.ForkRoot(&testFork)
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

func TestCheckPointRoot(t *testing.T) {
	testCheckpoint := ethpb.Checkpoint{
		Epoch: 1234567890,
		Root:  []byte{222},
	}
	expected := [32]byte{228, 65, 39, 109, 183, 249, 167, 232, 125, 239, 25, 155, 207, 4, 84, 174, 176, 229, 175, 224, 62, 33, 215, 254, 170, 220, 132, 65, 246, 128, 68, 194}

	result, err := ssz.CheckpointRoot(&testCheckpoint)
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

func TestByteArrayRootWithLimit(t *testing.T) {
	testHistoricalRoots := [][]byte{{123}, {234}}
	expected := [32]byte{70, 204, 150, 196, 89, 138, 190, 205, 65, 207, 120, 166, 179, 247, 147, 20, 29, 133, 117, 116, 151, 234, 129, 32, 22, 15, 79, 178, 98, 73, 132, 152}

	result, err := ssz.ByteArrayRootWithLimit(testHistoricalRoots, 16777216)
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

func TestSlashingsRoot(t *testing.T) {
	testSlashingsRoot := []uint64{123, 234}
	expected := [32]byte{123, 0, 0, 0, 0, 0, 0, 0, 234, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}

	result, err := ssz.SlashingsRoot(testSlashingsRoot)
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

func TestTransactionsRoot(t *testing.T) {
	tests := []struct {
		name    string
		txs     [][]byte
		want    [32]byte
		wantErr bool
	}{
		{
			name: "nil",
			txs:  nil,
			want: [32]byte{127, 254, 36, 30, 166, 1, 135, 253, 176, 24, 123, 250, 34, 222, 53, 209, 249, 190, 215, 171, 6, 29, 148, 1, 253, 71, 227, 74, 84, 251, 237, 225},
		},
		{
			name: "empty",
			txs:  [][]byte{},
			want: [32]byte{127, 254, 36, 30, 166, 1, 135, 253, 176, 24, 123, 250, 34, 222, 53, 209, 249, 190, 215, 171, 6, 29, 148, 1, 253, 71, 227, 74, 84, 251, 237, 225},
		},
		{
			name: "one tx",
			txs:  [][]byte{{1, 2, 3}},
			want: [32]byte{102, 209, 140, 87, 217, 28, 68, 12, 133, 42, 77, 136, 191, 18, 234, 105, 166, 228, 216, 235, 230, 95, 200, 73, 85, 33, 134, 254, 219, 97, 82, 209},
		},
		{
			name: "max txs",
			txs: func() [][]byte {
				var txs [][]byte
				for i := 0; i < fieldparams.MaxTxsPerPayloadLength; i++ {
					txs = append(txs, []byte{})
				}
				return txs
			}(),
			want: [32]byte{13, 66, 254, 206, 203, 58, 48, 133, 78, 218, 48, 231, 120, 90, 38, 72, 73, 137, 86, 9, 31, 213, 185, 101, 103, 144, 0, 236, 225, 57, 47, 244},
		},
		{
			name: "exceed max txs",
			txs: func() [][]byte {
				var txs [][]byte
				for i := 0; i < fieldparams.MaxTxsPerPayloadLength+1; i++ {
					txs = append(txs, []byte{})
				}
				return txs
			}(),
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ssz.TransactionsRoot(tt.txs)
			if (err != nil) != tt.wantErr {
				t.Errorf("TransactionsRoot() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("TransactionsRoot() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestByteSliceRoot(t *testing.T) {
	tests := []struct {
		name      string
		slice     []byte
		maxLength uint64
		want      [32]byte
		wantErr   bool
	}{
		{
			name:  "nil",
			slice: nil,
			want:  [32]byte{245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67, 0, 61, 35, 32, 217, 240, 232, 234, 152, 49, 169, 39, 89, 251, 75},
		},
		{
			name:  "empty",
			slice: []byte{},
			want:  [32]byte{245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67, 0, 61, 35, 32, 217, 240, 232, 234, 152, 49, 169, 39, 89, 251, 75},
		},
		{
			name:  "byte slice 3 values",
			slice: []byte{1, 2, 3},
			want:  [32]byte{20, 159, 26, 252, 247, 204, 44, 159, 161, 135, 211, 195, 106, 59, 220, 149, 199, 163, 228, 155, 113, 118, 64, 126, 173, 223, 102, 1, 241, 158, 164, 185},
		},
		{
			name:  "byte slice 32 values",
			slice: []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32},
			want:  [32]byte{7, 30, 46, 77, 237, 240, 59, 126, 232, 232, 232, 6, 145, 210, 31, 18, 117, 12, 217, 40, 204, 141, 90, 236, 241, 128, 221, 45, 126, 39, 39, 202},
		},
		{
			name:    "over max length",
			slice:   make([]byte, fieldparams.RootLength+1),
			want:    [32]byte{},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.maxLength == 0 {
				tt.maxLength = fieldparams.RootLength
			}
			got, err := ssz.ByteSliceRoot(tt.slice, tt.maxLength)
			if (err != nil) != tt.wantErr {
				t.Errorf("ByteSliceRoot() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("ByteSliceRoot() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestPackByChunk_SingleList(t *testing.T) {
	tests := []struct {
		name  string
		input []byte
		want  [][32]byte
	}{
		{
			name:  "nil",
			input: nil,
			want:  [][32]byte{{}},
		},
		{
			name:  "empty",
			input: []byte{},
			want:  [][32]byte{{}},
		},
		{
			name:  "one",
			input: []byte{1},
			want:  [][32]byte{{1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		},
		{
			name:  "one, two",
			input: []byte{1, 2},
			want:  [][32]byte{{1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ssz.PackByChunk([][]byte{tt.input})
			require.NoError(t, err)
			require.DeepSSZEqual(t, tt.want, got)
		})
	}
}

func TestWithdrawalRoot(t *testing.T) {
	tests := []struct {
		name  string
		input *enginev1.Withdrawal
		want  [32]byte
	}{
		{
			name:  "nil",
			input: &enginev1.Withdrawal{},
			want:  [32]byte{0xdb, 0x56, 0x11, 0x4e, 0x0, 0xfd, 0xd4, 0xc1, 0xf8, 0x5c, 0x89, 0x2b, 0xf3, 0x5a, 0xc9, 0xa8, 0x92, 0x89, 0xaa, 0xec, 0xb1, 0xeb, 0xd0, 0xa9, 0x6c, 0xde, 0x60, 0x6a, 0x74, 0x8b, 0x5d, 0x71},
		},
		{
			name: "empty",
			input: &enginev1.Withdrawal{
				Address: make([]byte, 20),
			},
			want: [32]byte{0xdb, 0x56, 0x11, 0x4e, 0x0, 0xfd, 0xd4, 0xc1, 0xf8, 0x5c, 0x89, 0x2b, 0xf3, 0x5a, 0xc9, 0xa8, 0x92, 0x89, 0xaa, 0xec, 0xb1, 0xeb, 0xd0, 0xa9, 0x6c, 0xde, 0x60, 0x6a, 0x74, 0x8b, 0x5d, 0x71},
		},
		{
			name: "non-empty",
			input: &enginev1.Withdrawal{
				Index:          123,
				ValidatorIndex: 123123,
				Address:        []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0},
				Amount:         50,
			},
			want: [32]byte{0x4f, 0xca, 0x3a, 0x43, 0x6e, 0xcc, 0x34, 0xad, 0x33, 0xde, 0x3c, 0x22, 0xa3, 0x32, 0x27, 0xa, 0x8c, 0x4e, 0x75, 0xd8, 0x39, 0xc1, 0xd7, 0x55, 0x78, 0x77, 0xd7, 0x14, 0x6b, 0x34, 0x6a, 0xb6},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ssz.WithdrawalRoot(tt.input)
			require.NoError(t, err)
			require.DeepSSZEqual(t, tt.want, got)
		})
	}
}

func TestWithrawalSliceRoot(t *testing.T) {
	tests := []struct {
		name  string
		input []*enginev1.Withdrawal
		want  [32]byte
	}{
		{
			name:  "empty",
			input: make([]*enginev1.Withdrawal, 0),
			want:  [32]byte{0x79, 0x29, 0x30, 0xbb, 0xd5, 0xba, 0xac, 0x43, 0xbc, 0xc7, 0x98, 0xee, 0x49, 0xaa, 0x81, 0x85, 0xef, 0x76, 0xbb, 0x3b, 0x44, 0xba, 0x62, 0xb9, 0x1d, 0x86, 0xae, 0x56, 0x9e, 0x4b, 0xb5, 0x35},
		},
		{
			name: "non-empty",
			input: []*enginev1.Withdrawal{{
				Index:          123,
				ValidatorIndex: 123123,
				Address:        []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0},
				Amount:         50,
			},
			},
			want: [32]byte{0x10, 0x34, 0x29, 0xd1, 0x34, 0x30, 0xa0, 0x1c, 0x4, 0xdd, 0x3, 0xed, 0xe6, 0xa6, 0x33, 0xb2, 0xc9, 0x24, 0x23, 0x5c, 0x43, 0xca, 0xb2, 0x32, 0xaa, 0xed, 0xfe, 0xd5, 0x9, 0x78, 0xd1, 0x6f},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ssz.WithdrawalSliceRoot(tt.input, 16)
			require.NoError(t, err)
			require.DeepSSZEqual(t, tt.want, got)
		})
	}
}
