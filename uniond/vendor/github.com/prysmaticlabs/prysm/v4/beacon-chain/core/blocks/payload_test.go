package blocks_test

import (
	"math/big"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func Test_IsMergeComplete(t *testing.T) {
	tests := []struct {
		name    string
		payload interfaces.ExecutionData
		want    bool
	}{
		{
			name: "empty payload header",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			want: false,
		},
		{
			name: "has parent hash",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has fee recipient",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.FeeRecipient = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has state root",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.StateRoot = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has receipt root",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ReceiptsRoot = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has logs bloom",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.LogsBloom = bytesutil.PadTo([]byte{'a'}, fieldparams.LogsBloomLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has random",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.PrevRandao = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has base fee",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.BaseFeePerGas = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has block hash",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.BlockHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has extra data",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ExtraData = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "has block number",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.BlockNumber = 1
				return h
			}(),
			want: true,
		},
		{
			name: "has gas limit",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.GasLimit = 1
				return h
			}(),
			want: true,
		},
		{
			name: "has gas used",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.GasUsed = 1
				return h
			}(),
			want: true,
		},
		{
			name: "has time stamp",
			payload: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.Timestamp = 1
				return h
			}(),
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
			require.NoError(t, st.SetLatestExecutionPayloadHeader(tt.payload))
			got, err := blocks.IsMergeTransitionComplete(st)
			require.NoError(t, err)
			if got != tt.want {
				t.Errorf("mergeComplete() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_IsMergeCompleteCapella(t *testing.T) {
	st, _ := util.DeterministicGenesisStateCapella(t, 1)
	got, err := blocks.IsMergeTransitionComplete(st)
	require.NoError(t, err)
	require.Equal(t, got, true)
}

func Test_IsExecutionBlock(t *testing.T) {
	tests := []struct {
		name    string
		payload *enginev1.ExecutionPayload
		want    bool
	}{
		{
			name:    "empty payload",
			payload: emptyPayload(),
			want:    false,
		},
		{
			name: "non-empty payload",
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return p
			}(),
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			blk := util.NewBeaconBlockBellatrix()
			blk.Block.Body.ExecutionPayload = tt.payload
			wrappedBlock, err := consensusblocks.NewBeaconBlock(blk.Block)
			require.NoError(t, err)
			got, err := blocks.IsExecutionBlock(wrappedBlock.Body())
			require.NoError(t, err)
			require.Equal(t, tt.want, got)
		})
	}
}

func Test_IsExecutionBlockCapella(t *testing.T) {
	blk := util.NewBeaconBlockCapella()
	blk.Block.Body.ExecutionPayload = emptyPayloadCapella()
	wrappedBlock, err := consensusblocks.NewBeaconBlock(blk.Block)
	require.NoError(t, err)
	got, err := blocks.IsExecutionBlock(wrappedBlock.Body())
	require.NoError(t, err)
	require.Equal(t, false, got)
}

func Test_IsExecutionEnabled(t *testing.T) {
	tests := []struct {
		name        string
		payload     *enginev1.ExecutionPayload
		header      interfaces.ExecutionData
		useAltairSt bool
		want        bool
	}{
		{
			name:    "use older than bellatrix state",
			payload: emptyPayload(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			useAltairSt: true,
			want:        false,
		},
		{
			name:    "empty header, empty payload",
			payload: emptyPayload(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			want: false,
		},
		{
			name:    "non-empty header, empty payload",
			payload: emptyPayload(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "empty header, non-empty payload",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.Timestamp = 1
				return p
			}(),
			want: true,
		},
		{
			name: "non-empty header, non-empty payload",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.Timestamp = 1
				return p
			}(),
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
			require.NoError(t, st.SetLatestExecutionPayloadHeader(tt.header))
			blk := util.NewBeaconBlockBellatrix()
			blk.Block.Body.ExecutionPayload = tt.payload
			body, err := consensusblocks.NewBeaconBlockBody(blk.Block.Body)
			require.NoError(t, err)
			if tt.useAltairSt {
				st, _ = util.DeterministicGenesisStateAltair(t, 1)
			}
			got, err := blocks.IsExecutionEnabled(st, body)
			require.NoError(t, err)
			if got != tt.want {
				t.Errorf("IsExecutionEnabled() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_IsExecutionEnabledUsingHeader(t *testing.T) {
	tests := []struct {
		name    string
		payload *enginev1.ExecutionPayload
		header  interfaces.ExecutionData
		want    bool
	}{
		{
			name:    "empty header, empty payload",
			payload: emptyPayload(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			want: false,
		},
		{
			name:    "non-empty header, empty payload",
			payload: emptyPayload(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			want: true,
		},
		{
			name: "empty header, non-empty payload",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.Timestamp = 1
				return p
			}(),
			want: true,
		},
		{
			name: "non-empty header, non-empty payload",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.Timestamp = 1
				return p
			}(),
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			blk := util.NewBeaconBlockBellatrix()
			blk.Block.Body.ExecutionPayload = tt.payload
			body, err := consensusblocks.NewBeaconBlockBody(blk.Block.Body)
			require.NoError(t, err)
			got, err := blocks.IsExecutionEnabledUsingHeader(tt.header, body)
			require.NoError(t, err)
			if got != tt.want {
				t.Errorf("IsExecutionEnabled() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_ValidatePayloadWhenMergeCompletes(t *testing.T) {
	tests := []struct {
		name    string
		payload *enginev1.ExecutionPayload
		header  interfaces.ExecutionData
		err     error
	}{
		{
			name:    "merge incomplete",
			payload: emptyPayload(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			err: nil,
		},
		{
			name: "validate passes",
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return p
			}(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.BlockHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return h
			}(),
			err: nil,
		},
		{
			name: "incorrect blockhash",
			payload: func() *enginev1.ExecutionPayload {
				p := emptyPayload()
				p.ParentHash = bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength)
				return p
			}(),
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.BlockHash = bytesutil.PadTo([]byte{'b'}, fieldparams.RootLength)
				return h
			}(),
			err: blocks.ErrInvalidPayloadBlockHash,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
			require.NoError(t, st.SetLatestExecutionPayloadHeader(tt.header))
			wrappedPayload, err := consensusblocks.WrappedExecutionPayload(tt.payload)
			require.NoError(t, err)
			err = blocks.ValidatePayloadWhenMergeCompletes(st, wrappedPayload)
			if err != nil {
				require.Equal(t, tt.err.Error(), err.Error())
			} else {
				require.Equal(t, tt.err, err)
			}
		})
	}
}

func Test_ValidatePayload(t *testing.T) {
	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	random, err := helpers.RandaoMix(st, time.CurrentEpoch(st))
	require.NoError(t, err)
	ts, err := slots.ToTime(st.GenesisTime(), st.Slot())
	require.NoError(t, err)
	tests := []struct {
		name    string
		payload *enginev1.ExecutionPayload
		err     error
	}{
		{
			name: "validate passes",
			payload: func() *enginev1.ExecutionPayload {
				h := emptyPayload()
				h.PrevRandao = random
				h.Timestamp = uint64(ts.Unix())
				return h
			}(), err: nil,
		},
		{
			name:    "incorrect prev randao",
			payload: emptyPayload(),
			err:     blocks.ErrInvalidPayloadPrevRandao,
		},
		{
			name: "incorrect timestamp",
			payload: func() *enginev1.ExecutionPayload {
				h := emptyPayload()
				h.PrevRandao = random
				h.Timestamp = 1
				return h
			}(),
			err: blocks.ErrInvalidPayloadTimeStamp,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			wrappedPayload, err := consensusblocks.WrappedExecutionPayload(tt.payload)
			require.NoError(t, err)
			err = blocks.ValidatePayload(st, wrappedPayload)
			if err != nil {
				require.Equal(t, tt.err.Error(), err.Error())
			} else {
				require.Equal(t, tt.err, err)
			}
		})
	}
}

func Test_ProcessPayload(t *testing.T) {
	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	random, err := helpers.RandaoMix(st, time.CurrentEpoch(st))
	require.NoError(t, err)
	ts, err := slots.ToTime(st.GenesisTime(), st.Slot())
	require.NoError(t, err)
	tests := []struct {
		name    string
		payload *enginev1.ExecutionPayload
		err     error
	}{
		{
			name: "process passes",
			payload: func() *enginev1.ExecutionPayload {
				h := emptyPayload()
				h.PrevRandao = random
				h.Timestamp = uint64(ts.Unix())
				return h
			}(), err: nil,
		},
		{
			name:    "incorrect prev randao",
			payload: emptyPayload(),
			err:     blocks.ErrInvalidPayloadPrevRandao,
		},
		{
			name: "incorrect timestamp",
			payload: func() *enginev1.ExecutionPayload {
				h := emptyPayload()
				h.PrevRandao = random
				h.Timestamp = 1
				return h
			}(),
			err: blocks.ErrInvalidPayloadTimeStamp,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			wrappedPayload, err := consensusblocks.WrappedExecutionPayload(tt.payload)
			require.NoError(t, err)
			st, err := blocks.ProcessPayload(st, wrappedPayload)
			if err != nil {
				require.Equal(t, tt.err.Error(), err.Error())
			} else {
				require.Equal(t, tt.err, err)
				want, err := consensusblocks.PayloadToHeader(wrappedPayload)
				require.Equal(t, tt.err, err)
				h, err := st.LatestExecutionPayloadHeader()
				require.NoError(t, err)
				got, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				require.DeepSSZEqual(t, want, got)
			}
		})
	}
}

func Test_ProcessPayloadCapella(t *testing.T) {
	st, _ := util.DeterministicGenesisStateCapella(t, 1)
	header, err := emptyPayloadHeaderCapella()
	require.NoError(t, err)
	require.NoError(t, st.SetLatestExecutionPayloadHeader(header))
	payload := emptyPayloadCapella()
	random, err := helpers.RandaoMix(st, time.CurrentEpoch(st))
	require.NoError(t, err)
	payload.PrevRandao = random
	wrapped, err := consensusblocks.WrappedExecutionPayloadCapella(payload, big.NewInt(0))
	require.NoError(t, err)
	_, err = blocks.ProcessPayload(st, wrapped)
	require.NoError(t, err)
}

func Test_ProcessPayloadHeader(t *testing.T) {
	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	random, err := helpers.RandaoMix(st, time.CurrentEpoch(st))
	require.NoError(t, err)
	ts, err := slots.ToTime(st.GenesisTime(), st.Slot())
	require.NoError(t, err)
	tests := []struct {
		name   string
		header interfaces.ExecutionData
		err    error
	}{
		{
			name: "process passes",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.PrevRandao = random
				p.Timestamp = uint64(ts.Unix())
				return h
			}(), err: nil,
		},
		{
			name: "incorrect prev randao",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			err: blocks.ErrInvalidPayloadPrevRandao,
		},
		{
			name: "incorrect timestamp",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.PrevRandao = random
				p.Timestamp = 1
				return h
			}(),
			err: blocks.ErrInvalidPayloadTimeStamp,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			st, err := blocks.ProcessPayloadHeader(st, tt.header)
			if err != nil {
				require.Equal(t, tt.err.Error(), err.Error())
			} else {
				require.Equal(t, tt.err, err)
				want, ok := tt.header.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				h, err := st.LatestExecutionPayloadHeader()
				require.NoError(t, err)
				got, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				require.DeepSSZEqual(t, want, got)
			}
		})
	}
}

func Test_ValidatePayloadHeader(t *testing.T) {
	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	random, err := helpers.RandaoMix(st, time.CurrentEpoch(st))
	require.NoError(t, err)
	ts, err := slots.ToTime(st.GenesisTime(), st.Slot())
	require.NoError(t, err)
	tests := []struct {
		name   string
		header interfaces.ExecutionData
		err    error
	}{
		{
			name: "process passes",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.PrevRandao = random
				p.Timestamp = uint64(ts.Unix())
				return h
			}(), err: nil,
		},
		{
			name: "incorrect prev randao",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			err: blocks.ErrInvalidPayloadPrevRandao,
		},
		{
			name: "incorrect timestamp",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.PrevRandao = random
				p.Timestamp = 1
				return h
			}(),
			err: blocks.ErrInvalidPayloadTimeStamp,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err = blocks.ValidatePayloadHeader(st, tt.header)
			require.Equal(t, tt.err, err)
		})
	}
}

func Test_ValidatePayloadHeaderWhenMergeCompletes(t *testing.T) {
	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	emptySt := st.Copy()
	wrappedHeader, err := consensusblocks.WrappedExecutionPayloadHeader(&enginev1.ExecutionPayloadHeader{BlockHash: []byte{'a'}})
	require.NoError(t, err)
	require.NoError(t, st.SetLatestExecutionPayloadHeader(wrappedHeader))
	tests := []struct {
		name   string
		state  state.BeaconState
		header interfaces.ExecutionData
		err    error
	}{
		{
			name: "no merge",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				return h
			}(),
			state: emptySt,
			err:   nil,
		},
		{
			name: "process passes",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = []byte{'a'}
				return h
			}(),
			state: st,
			err:   nil,
		},
		{
			name: "invalid block hash",
			header: func() interfaces.ExecutionData {
				h, err := emptyPayloadHeader()
				require.NoError(t, err)
				p, ok := h.Proto().(*enginev1.ExecutionPayloadHeader)
				require.Equal(t, true, ok)
				p.ParentHash = []byte{'b'}
				return h
			}(),
			state: st,
			err:   blocks.ErrInvalidPayloadBlockHash,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err = blocks.ValidatePayloadHeaderWhenMergeCompletes(tt.state, tt.header)
			require.Equal(t, tt.err, err)
		})
	}
}

func Test_PayloadToHeader(t *testing.T) {
	p := emptyPayload()
	wrappedPayload, err := consensusblocks.WrappedExecutionPayload(p)
	require.NoError(t, err)
	h, err := consensusblocks.PayloadToHeader(wrappedPayload)
	require.NoError(t, err)
	txRoot, err := ssz.TransactionsRoot(p.Transactions)
	require.NoError(t, err)
	require.DeepSSZEqual(t, txRoot, bytesutil.ToBytes32(h.TransactionsRoot))

	// Verify copy works
	b := []byte{'a'}
	p.ParentHash = b
	p.FeeRecipient = b
	p.StateRoot = b
	p.ReceiptsRoot = b
	p.LogsBloom = b
	p.PrevRandao = b
	p.ExtraData = b
	p.BaseFeePerGas = b
	p.BlockHash = b
	p.BlockNumber = 1
	p.GasUsed = 1
	p.GasLimit = 1
	p.Timestamp = 1

	require.DeepSSZEqual(t, h.ParentHash, make([]byte, fieldparams.RootLength))
	require.DeepSSZEqual(t, h.FeeRecipient, make([]byte, fieldparams.FeeRecipientLength))
	require.DeepSSZEqual(t, h.StateRoot, make([]byte, fieldparams.RootLength))
	require.DeepSSZEqual(t, h.ReceiptsRoot, make([]byte, fieldparams.RootLength))
	require.DeepSSZEqual(t, h.LogsBloom, make([]byte, fieldparams.LogsBloomLength))
	require.DeepSSZEqual(t, h.PrevRandao, make([]byte, fieldparams.RootLength))
	require.DeepSSZEqual(t, h.ExtraData, make([]byte, 0))
	require.DeepSSZEqual(t, h.BaseFeePerGas, make([]byte, fieldparams.RootLength))
	require.DeepSSZEqual(t, h.BlockHash, make([]byte, fieldparams.RootLength))
	require.Equal(t, h.BlockNumber, uint64(0))
	require.Equal(t, h.GasUsed, uint64(0))
	require.Equal(t, h.GasLimit, uint64(0))
	require.Equal(t, h.Timestamp, uint64(0))
}

func BenchmarkBellatrixComplete(b *testing.B) {
	st, _ := util.DeterministicGenesisStateBellatrix(b, 1)
	h, err := emptyPayloadHeader()
	require.NoError(b, err)
	require.NoError(b, st.SetLatestExecutionPayloadHeader(h))

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := blocks.IsMergeTransitionComplete(st)
		require.NoError(b, err)
	}
}

func emptyPayloadHeader() (interfaces.ExecutionData, error) {
	return consensusblocks.WrappedExecutionPayloadHeader(&enginev1.ExecutionPayloadHeader{
		ParentHash:       make([]byte, fieldparams.RootLength),
		FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:        make([]byte, fieldparams.RootLength),
		ReceiptsRoot:     make([]byte, fieldparams.RootLength),
		LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:       make([]byte, fieldparams.RootLength),
		ExtraData:        make([]byte, 0),
		BaseFeePerGas:    make([]byte, fieldparams.RootLength),
		BlockHash:        make([]byte, fieldparams.RootLength),
		TransactionsRoot: make([]byte, fieldparams.RootLength),
	})
}

func emptyPayloadHeaderCapella() (interfaces.ExecutionData, error) {
	return consensusblocks.WrappedExecutionPayloadHeaderCapella(&enginev1.ExecutionPayloadHeaderCapella{
		ParentHash:       make([]byte, fieldparams.RootLength),
		FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:        make([]byte, fieldparams.RootLength),
		ReceiptsRoot:     make([]byte, fieldparams.RootLength),
		LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:       make([]byte, fieldparams.RootLength),
		ExtraData:        make([]byte, 0),
		BaseFeePerGas:    make([]byte, fieldparams.RootLength),
		BlockHash:        make([]byte, fieldparams.RootLength),
		TransactionsRoot: make([]byte, fieldparams.RootLength),
		WithdrawalsRoot:  make([]byte, fieldparams.RootLength),
	}, big.NewInt(0))
}

func emptyPayload() *enginev1.ExecutionPayload {
	return &enginev1.ExecutionPayload{
		ParentHash:    make([]byte, fieldparams.RootLength),
		FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:     make([]byte, fieldparams.RootLength),
		ReceiptsRoot:  make([]byte, fieldparams.RootLength),
		LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:    make([]byte, fieldparams.RootLength),
		ExtraData:     make([]byte, 0),
		BaseFeePerGas: make([]byte, fieldparams.RootLength),
		BlockHash:     make([]byte, fieldparams.RootLength),
		Transactions:  make([][]byte, 0),
	}
}

func emptyPayloadCapella() *enginev1.ExecutionPayloadCapella {
	return &enginev1.ExecutionPayloadCapella{
		ParentHash:    make([]byte, fieldparams.RootLength),
		FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:     make([]byte, fieldparams.RootLength),
		ReceiptsRoot:  make([]byte, fieldparams.RootLength),
		LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
		PrevRandao:    make([]byte, fieldparams.RootLength),
		ExtraData:     make([]byte, 0),
		BaseFeePerGas: make([]byte, fieldparams.RootLength),
		BlockHash:     make([]byte, fieldparams.RootLength),
		Transactions:  make([][]byte, 0),
		Withdrawals:   make([]*enginev1.Withdrawal, 0),
	}
}
