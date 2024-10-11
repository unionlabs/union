package history

import (
	"fmt"
	"math"
	"reflect"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

func Test_uint64FromString(t *testing.T) {
	tests := []struct {
		name    string
		str     string
		want    uint64
		wantErr bool
	}{
		{
			name:    "Overflow uint64 gets MaxUint64",
			str:     "29348902839048290384902839048290384902938748278934789273984728934789273894798273498",
			want:    math.MaxUint64,
			wantErr: true,
		},
		{
			name:    "Max Uint64 works",
			str:     "18446744073709551615",
			want:    math.MaxUint64,
			wantErr: false,
		},
		{
			name:    "Negative number fails",
			str:     "-3",
			wantErr: true,
		},
		{
			name:    "Junk fails",
			str:     "alksjdkjasd",
			wantErr: true,
		},
		{
			name: "0 works",
			str:  "0",
			want: 0,
		},
		{
			name: "Normal uint64 works",
			str:  "23980",
			want: 23980,
		},
	}
	for _, tt := range tests {
		t.Run(fmt.Sprintf("Uint64/%s", tt.name), func(t *testing.T) {
			got, err := Uint64FromString(tt.str)
			if (err != nil) != tt.wantErr {
				t.Errorf("Uint64FromString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("Uint64FromString() got = %v, want %v", got, tt.want)
			}
		})
		t.Run(fmt.Sprintf("Epoch/%s", tt.name), func(t *testing.T) {
			got, err := EpochFromString(tt.str)
			if (err != nil) != tt.wantErr {
				t.Errorf("EpochFromString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != primitives.Epoch(tt.want) {
				t.Errorf("EpochFromString() got = %v, want %v", got, tt.want)
			}
		})
		t.Run(fmt.Sprintf("Slot/%s", tt.name), func(t *testing.T) {
			got, err := SlotFromString(tt.str)
			if (err != nil) != tt.wantErr {
				t.Errorf("SlotFromString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != primitives.Slot(tt.want) {
				t.Errorf("SlotFromString() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_pubKeyFromHex(t *testing.T) {
	tests := []struct {
		name    string
		str     string
		want    [fieldparams.BLSPubkeyLength]byte
		wantErr bool
	}{
		{
			name:    "Empty value fails due to wrong length",
			str:     "",
			wantErr: true,
		},
		{
			name:    "Junk fails",
			str:     "alksjdkjasd",
			wantErr: true,
		},
		{
			name:    "Empty value with 0x prefix fails due to wrong length",
			str:     "0x",
			wantErr: true,
		},
		{
			name: "Works with 0x prefix and good public key",
			str:  "0xb845089a1457f811bfc000588fbb4e713669be8ce060ea6be3c6ece09afc3794106c91ca73acda5e5457122d58723bed",
			want: [fieldparams.BLSPubkeyLength]byte{184, 69, 8, 154, 20, 87, 248, 17, 191, 192, 0, 88, 143, 187, 78, 113, 54, 105, 190, 140, 224, 96, 234, 107, 227, 198, 236, 224, 154, 252, 55, 148, 16, 108, 145, 202, 115, 172, 218, 94, 84, 87, 18, 45, 88, 114, 59, 237},
		},
		{
			name: "Works without 0x prefix and good public key",
			str:  "b845089a1457f811bfc000588fbb4e713669be8ce060ea6be3c6ece09afc3794106c91ca73acda5e5457122d58723bed",
			want: [fieldparams.BLSPubkeyLength]byte{184, 69, 8, 154, 20, 87, 248, 17, 191, 192, 0, 88, 143, 187, 78, 113, 54, 105, 190, 140, 224, 96, 234, 107, 227, 198, 236, 224, 154, 252, 55, 148, 16, 108, 145, 202, 115, 172, 218, 94, 84, 87, 18, 45, 88, 114, 59, 237},
		},
		{
			name:    "0x prefix and wrong length public key fails",
			str:     "0xb845089a1457f811bfc000588fbb4e713669be8",
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := PubKeyFromHex(tt.str)
			if (err != nil) != tt.wantErr {
				t.Errorf("PubKeyFromHex() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("PubKeyFromHex() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_rootFromHex(t *testing.T) {
	tests := []struct {
		name    string
		str     string
		want    [32]byte
		wantErr bool
	}{
		{
			name:    "Empty value fails due to wrong length",
			str:     "",
			wantErr: true,
		},
		{
			name:    "Junk fails",
			str:     "alksjdkjasd",
			wantErr: true,
		},
		{
			name:    "Empty value with 0x prefix fails due to wrong length",
			str:     "0x",
			wantErr: true,
		},
		{
			name: "Works with 0x prefix and good root",
			str:  "0x4ff6f743a43f3b4f95350831aeaf0a122a1a392922c45d804280284a69eb850b",
			want: [32]byte{79, 246, 247, 67, 164, 63, 59, 79, 149, 53, 8, 49, 174, 175, 10, 18, 42, 26, 57, 41, 34, 196, 93, 128, 66, 128, 40, 74, 105, 235, 133, 11},
		},
		{
			name: "Works without 0x prefix and good root",
			str:  "4ff6f743a43f3b4f95350831aeaf0a122a1a392922c45d804280284a69eb850b",
			want: [32]byte{79, 246, 247, 67, 164, 63, 59, 79, 149, 53, 8, 49, 174, 175, 10, 18, 42, 26, 57, 41, 34, 196, 93, 128, 66, 128, 40, 74, 105, 235, 133, 11},
		},
		{
			name:    "0x prefix and wrong length root fails",
			str:     "0xb845089a14",
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := RootFromHex(tt.str)
			if (err != nil) != tt.wantErr {
				t.Errorf("rootFromHex() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("rootFromHex() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_rootToHexString(t *testing.T) {
	mockRoot := [32]byte{1}
	tests := []struct {
		name    string
		root    []byte
		want    string
		wantErr bool
	}{
		{
			name:    "nil roots return empty string",
			root:    nil,
			want:    "",
			wantErr: false,
		},
		{
			name:    "len(root) == 0 returns empty string",
			root:    make([]byte, 0),
			want:    "",
			wantErr: false,
		},
		{
			name:    "non-empty root with incorrect size returns error",
			root:    make([]byte, 20),
			want:    "",
			wantErr: true,
		},
		{
			name:    "non-empty root with correct size returns expected value",
			root:    mockRoot[:],
			want:    "0x0100000000000000000000000000000000000000000000000000000000000000",
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := rootToHexString(tt.root)
			if (err != nil) != tt.wantErr {
				t.Errorf("rootToHexString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("rootToHexString() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_pubKeyToHexString(t *testing.T) {
	mockPubKey := [fieldparams.BLSPubkeyLength]byte{1}
	tests := []struct {
		name    string
		pubKey  []byte
		want    string
		wantErr bool
	}{
		{
			name:    "nil pubkey should return error",
			pubKey:  nil,
			want:    "",
			wantErr: true,
		},
		{
			name:    "empty pubkey should return error",
			pubKey:  make([]byte, 0),
			want:    "",
			wantErr: true,
		},
		{
			name:    "wrong length pubkey should return error",
			pubKey:  make([]byte, 3),
			want:    "",
			wantErr: true,
		},
		{
			name:    "non-empty pubkey with correct size returns expected value",
			pubKey:  mockPubKey[:],
			want:    "0x010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := pubKeyToHexString(tt.pubKey)
			if (err != nil) != tt.wantErr {
				t.Errorf("pubKeyToHexString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("pubKeyToHexString() got = %v, want %v", got, tt.want)
			}
		})
	}
}
