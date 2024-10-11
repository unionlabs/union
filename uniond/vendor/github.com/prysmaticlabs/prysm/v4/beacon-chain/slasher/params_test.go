package slasher

import (
	"reflect"
	"testing"

	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestDefaultParams(t *testing.T) {
	def := DefaultParams()
	assert.Equal(t, true, def.chunkSize > 0)
	assert.Equal(t, true, def.validatorChunkSize > 0)
	assert.Equal(t, true, def.historyLength > 0)
}

func TestParams_cellIndex(t *testing.T) {
	type args struct {
		validatorIndex primitives.ValidatorIndex
		epoch          primitives.Epoch
	}
	tests := []struct {
		name   string
		fields *Parameters
		args   args
		want   uint64
	}{
		{
			name: "epoch 0 and validator index 0",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 0,
				epoch:          0,
			},
			want: 0,
		},
		{
			//     val0     val1     val2
			//      |        |        |
			//   {     }  {     }  {     }
			//  [2, 2, 2, 2, 2, 2, 2, 2, 2]
			//                        |-> epoch 1, validator 2
			name: "epoch < chunkSize and validatorIndex < validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 2,
				epoch:          1,
			},
			want: 7,
		},
		{
			//     val0     val1     val2
			//      |        |        |
			//   {     }  {     }  {     }
			//  [2, 2, 2, 2, 2, 2, 2, 2, 2]
			//                        |-> epoch 4, validator 2 (wrap around)
			name: "epoch > chunkSize and validatorIndex < validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 2,
				epoch:          4,
			},
			want: 7,
		},
		{
			//     val0     val1     val2
			//      |        |        |
			//   {     }  {     }  {     }
			//  [2, 2, 2, 2, 2, 2, 2, 2, 2]
			//                     |-> epoch 3, validator 2 (wrap around)
			name: "epoch = chunkSize and validatorIndex < validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 2,
				epoch:          3,
			},
			want: 6,
		},
		{
			//     val0     val1     val2
			//      |        |        |
			//   {     }  {     }  {     }
			//  [2, 2, 2, 2, 2, 2, 2, 2, 2]
			//   |-> epoch 0, validator 3 (wrap around)
			name: "epoch < chunkSize and validatorIndex = validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 3,
				epoch:          0,
			},
			want: 0,
		},
		{
			//     val0     val1     val2
			//      |        |        |
			//   {     }  {     }  {     }
			//  [2, 2, 2, 2, 2, 2, 2, 2, 2]
			//            |-> epoch 0, validator 4 (wrap around)
			name: "epoch < chunkSize and validatorIndex > validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 4,
				epoch:          0,
			},
			want: 3,
		},
		{
			//     val0     val1     val2
			//      |        |        |
			//   {     }  {     }  {     }
			//  [2, 2, 2, 2, 2, 2, 2, 2, 2]
			//   |-> epoch 3, validator 3 (wrap around)
			name: "epoch = chunkSize and validatorIndex = validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
			},
			args: args{
				validatorIndex: 3,
				epoch:          3,
			},
			want: 0,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				chunkSize:          tt.fields.chunkSize,
				validatorChunkSize: tt.fields.validatorChunkSize,
				historyLength:      tt.fields.historyLength,
			}
			if got := c.cellIndex(tt.args.validatorIndex, tt.args.epoch); got != tt.want {
				t.Errorf("cellIndex() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParams_chunkIndex(t *testing.T) {
	tests := []struct {
		name   string
		fields *Parameters
		epoch  primitives.Epoch
		want   uint64
	}{
		{
			name: "epoch 0",
			fields: &Parameters{
				chunkSize:     3,
				historyLength: 3,
			},
			epoch: 0,
			want:  0,
		},
		{
			name: "epoch < historyLength, epoch < chunkSize",
			fields: &Parameters{
				chunkSize:     3,
				historyLength: 3,
			},
			epoch: 2,
			want:  0,
		},
		{
			name: "epoch = historyLength, epoch < chunkSize",
			fields: &Parameters{
				chunkSize:     4,
				historyLength: 3,
			},
			epoch: 3,
			want:  0,
		},
		{
			name: "epoch > historyLength, epoch < chunkSize",
			fields: &Parameters{
				chunkSize:     5,
				historyLength: 3,
			},
			epoch: 4,
			want:  0,
		},
		{
			name: "epoch < historyLength, epoch < chunkSize",
			fields: &Parameters{
				chunkSize:     3,
				historyLength: 3,
			},
			epoch: 2,
			want:  0,
		},
		{
			name: "epoch = historyLength, epoch < chunkSize",
			fields: &Parameters{
				chunkSize:     4,
				historyLength: 3,
			},
			epoch: 3,
			want:  0,
		},
		{
			name: "epoch < historyLength, epoch = chunkSize",
			fields: &Parameters{
				chunkSize:     2,
				historyLength: 3,
			},
			epoch: 2,
			want:  1,
		},
		{
			name: "epoch < historyLength, epoch > chunkSize",
			fields: &Parameters{
				chunkSize:     2,
				historyLength: 4,
			},
			epoch: 3,
			want:  1,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				chunkSize:     tt.fields.chunkSize,
				historyLength: tt.fields.historyLength,
			}
			if got := c.chunkIndex(tt.epoch); got != tt.want {
				t.Errorf("chunkIndex() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParams_flatSliceID(t *testing.T) {
	tests := []struct {
		name                string
		fields              *Parameters
		validatorChunkIndex uint64
		chunkIndex          uint64
		want                uint64
	}{
		{
			name: "Proper disk key for 0, 0",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
				historyLength:      6,
			},
			chunkIndex:          0,
			validatorChunkIndex: 0,
			want:                0,
		},
		{
			name: "Proper disk key for epoch < historyLength, validator < validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
				historyLength:      6,
			},
			chunkIndex:          1,
			validatorChunkIndex: 1,
			want:                3,
		},
		{
			name: "Proper disk key for epoch > historyLength, validator > validatorChunkSize",
			fields: &Parameters{
				chunkSize:          3,
				validatorChunkSize: 3,
				historyLength:      6,
			},
			chunkIndex:          10,
			validatorChunkIndex: 10,
			want:                30,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				chunkSize:          tt.fields.chunkSize,
				validatorChunkSize: tt.fields.validatorChunkSize,
				historyLength:      tt.fields.historyLength,
			}
			got := c.flatSliceID(tt.validatorChunkIndex, tt.chunkIndex)
			decoded := ssz.UnmarshallUint64(got)
			if decoded != tt.want {
				t.Errorf("diskKey() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParams_validatorChunkIndex(t *testing.T) {
	tests := []struct {
		name           string
		fields         *Parameters
		validatorIndex primitives.ValidatorIndex
		want           uint64
	}{
		{
			name: "validator index < validatorChunkSize",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorIndex: 2,
			want:           0,
		},
		{
			name: "validator index = validatorChunkSize",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorIndex: 3,
			want:           1,
		},
		{
			name: "validator index > validatorChunkSize",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorIndex: 99,
			want:           33,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				validatorChunkSize: tt.fields.validatorChunkSize,
			}
			if got := c.validatorChunkIndex(tt.validatorIndex); got != tt.want {
				t.Errorf("validatorChunkIndex() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParams_chunkOffset(t *testing.T) {
	tests := []struct {
		name   string
		fields *Parameters
		epoch  primitives.Epoch
		want   uint64
	}{
		{
			name: "epoch < chunkSize",
			fields: &Parameters{
				chunkSize: 3,
			},
			epoch: 2,
			want:  2,
		},
		{
			name: "epoch = chunkSize",
			fields: &Parameters{
				chunkSize: 3,
			},
			epoch: 3,
			want:  0,
		},
		{
			name: "epoch > chunkSize",
			fields: &Parameters{
				chunkSize: 3,
			},
			epoch: 5,
			want:  2,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				chunkSize: tt.fields.chunkSize,
			}
			if got := c.chunkOffset(tt.epoch); got != tt.want {
				t.Errorf("chunkOffset() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParams_validatorOffset(t *testing.T) {
	tests := []struct {
		name           string
		fields         *Parameters
		validatorIndex primitives.ValidatorIndex
		want           uint64
	}{
		{
			name: "validatorIndex < validatorChunkSize",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorIndex: 2,
			want:           2,
		},
		{
			name: "validatorIndex = validatorChunkSize",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorIndex: 3,
			want:           0,
		},
		{
			name: "validatorIndex > validatorChunkSize",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorIndex: 5,
			want:           2,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				validatorChunkSize: tt.fields.validatorChunkSize,
			}
			if got := c.validatorOffset(tt.validatorIndex); got != tt.want {
				t.Errorf("validatorOffset() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParams_validatorIndicesInChunk(t *testing.T) {
	tests := []struct {
		name              string
		fields            *Parameters
		validatorChunkIdx uint64
		want              []primitives.ValidatorIndex
	}{
		{
			name: "Returns proper indices",
			fields: &Parameters{
				validatorChunkSize: 3,
			},
			validatorChunkIdx: 2,
			want:              []primitives.ValidatorIndex{6, 7, 8},
		},
		{
			name: "0 validator chunk size returns empty",
			fields: &Parameters{
				validatorChunkSize: 0,
			},
			validatorChunkIdx: 100,
			want:              []primitives.ValidatorIndex{},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Parameters{
				validatorChunkSize: tt.fields.validatorChunkSize,
			}
			if got := c.validatorIndicesInChunk(tt.validatorChunkIdx); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("validatorIndicesInChunk() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParameters_firstEpoch(t *testing.T) {
	tests := []struct {
		name       string
		params     *Parameters
		chunkIndex uint64
		want       primitives.Epoch
	}{
		{
			name:       "first epoch of chunk 0 is 0",
			params:     DefaultParams(),
			chunkIndex: 0,
			want:       0,
		},
		{
			name: "with chunk_size = 3, first epoch of chunk 1 is 3",
			params: &Parameters{
				chunkSize: 3,
			},
			chunkIndex: 1,
			want:       3,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.params.firstEpoch(tt.chunkIndex); got != tt.want {
				t.Errorf("firstEpoch() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestParameters_lastEpoch(t *testing.T) {
	tests := []struct {
		name       string
		params     *Parameters
		chunkIndex uint64
		want       primitives.Epoch
	}{
		{
			name: "with chunk_size = 3, last epoch of chunk 0 is 2",
			params: &Parameters{
				chunkSize: 3,
			},
			chunkIndex: 0,
			want:       2,
		},
		{
			name: "with chunk_size = 3, last epoch of chunk 1 is 5",
			params: &Parameters{
				chunkSize: 3,
			},
			chunkIndex: 1,
			want:       5,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.params.lastEpoch(tt.chunkIndex); got != tt.want {
				t.Errorf("lastEpoch() = %v, want %v", got, tt.want)
			}
		})
	}
}
