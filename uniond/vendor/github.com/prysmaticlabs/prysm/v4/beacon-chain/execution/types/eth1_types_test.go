package types

import (
	"math"
	"math/big"
	"reflect"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestRoundtrip_HeaderInfo(t *testing.T) {
	tests := []struct {
		name    string
		hInfo   HeaderInfo
		wantErr bool
	}{
		{
			name: "normal header object",
			hInfo: HeaderInfo{
				Number: big.NewInt(1000),
				Hash:   common.Hash{239, 10, 13, 71, 156, 192, 23, 93, 73, 154, 255, 209, 163, 204, 129, 12, 179, 183, 65, 70, 205, 200, 57, 12, 17, 211, 209, 4, 104, 133, 73, 86},
				Time:   1000,
			},
			wantErr: false,
		},
		{
			name: "large header object",
			hInfo: HeaderInfo{
				Number: big.NewInt(10023982389238920),
				Hash:   common.Hash{192, 19, 18, 71, 156, 239, 23, 93, 73, 17, 255, 209, 163, 204, 129, 12, 179, 129, 65, 70, 209, 200, 57, 12, 17, 211, 209, 4, 104, 57, 73, 86},
				Time:   math.MaxUint64,
			},
			wantErr: false,
		},
		{
			name: "missing number",
			hInfo: HeaderInfo{
				Hash: common.Hash{192, 19, 18, 71, 156, 239, 23, 93, 73, 17, 255, 209, 163, 204, 129, 12, 179, 129, 65, 70, 209, 200, 57, 12, 17, 211, 209, 4, 104, 57, 73, 86},
				Time: math.MaxUint64,
			},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			h := &HeaderInfo{
				Number: tt.hInfo.Number,
				Hash:   tt.hInfo.Hash,
				Time:   tt.hInfo.Time,
			}
			recv, err := h.MarshalJSON()
			assert.NoError(t, err)
			newH := &HeaderInfo{}
			err = newH.UnmarshalJSON(recv)
			if (err != nil) != tt.wantErr {
				t.Errorf("UnmarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if tt.wantErr {
				return
			}
			if !reflect.DeepEqual(*newH, tt.hInfo) {
				t.Errorf("MarshalJSON() got = %v, want %v", newH, tt.hInfo)
			}
		})
	}
}
