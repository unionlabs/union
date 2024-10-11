package attestation_test

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestAttestingIndices(t *testing.T) {
	type args struct {
		bf        bitfield.Bitfield
		committee []primitives.ValidatorIndex
	}
	tests := []struct {
		name string
		args args
		want []uint64
		err  string
	}{
		{
			name: "Full committee attested",
			args: args{
				bf:        bitfield.Bitlist{0b1111},
				committee: []primitives.ValidatorIndex{0, 1, 2},
			},
			want: []uint64{0, 1, 2},
		},
		{
			name: "Partial committee attested",
			args: args{
				bf:        bitfield.Bitlist{0b1101},
				committee: []primitives.ValidatorIndex{0, 1, 2},
			},
			want: []uint64{0, 2},
		},
		{
			name: "Invalid bit length",
			args: args{
				bf:        bitfield.Bitlist{0b11111},
				committee: []primitives.ValidatorIndex{0, 1, 2},
			},
			err: "bitfield length 4 is not equal to committee length 3",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := attestation.AttestingIndices(tt.args.bf, tt.args.committee)
			if tt.err == "" {
				require.NoError(t, err)
				assert.DeepEqual(t, tt.want, got)
			} else {
				require.ErrorContains(t, tt.err, err)
			}
		})
	}
}

func TestIsValidAttestationIndices(t *testing.T) {
	tests := []struct {
		name      string
		att       *eth.IndexedAttestation
		wantedErr string
	}{
		{
			name: "Indices should not be nil",
			att: &eth.IndexedAttestation{
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
			wantedErr: "nil or missing indexed attestation data",
		},
		{
			name: "Indices should be non-empty",
			att: &eth.IndexedAttestation{
				AttestingIndices: []uint64{},
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
			wantedErr: "expected non-empty",
		},
		{
			name: "Greater than max validators per committee",
			att: &eth.IndexedAttestation{
				AttestingIndices: make([]uint64, params.BeaconConfig().MaxValidatorsPerCommittee+1),
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
			wantedErr: "indices count exceeds",
		},
		{
			name: "Needs to be sorted",
			att: &eth.IndexedAttestation{
				AttestingIndices: []uint64{3, 2, 1},
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
			wantedErr: "not uniquely sorted",
		},
		{
			name: "Valid indices",
			att: &eth.IndexedAttestation{
				AttestingIndices: []uint64{1, 2, 3},
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
		},
		{
			name: "Valid indices with length of 2",
			att: &eth.IndexedAttestation{
				AttestingIndices: []uint64{1, 2},
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
		},
		{
			name: "Valid indices with length of 1",
			att: &eth.IndexedAttestation{
				AttestingIndices: []uint64{1},
				Data: &eth.AttestationData{
					Target: &eth.Checkpoint{},
				},
				Signature: make([]byte, fieldparams.BLSSignatureLength),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := attestation.IsValidAttestationIndices(context.Background(), tt.att)
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func BenchmarkAttestingIndices_PartialCommittee(b *testing.B) {
	bf := bitfield.Bitlist{0b11111111, 0b11111111, 0b10000111, 0b11111111, 0b100}
	committee := []primitives.ValidatorIndex{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := attestation.AttestingIndices(bf, committee)
		require.NoError(b, err)
	}
}

func BenchmarkIsValidAttestationIndices(b *testing.B) {
	indices := make([]uint64, params.BeaconConfig().MaxValidatorsPerCommittee)
	for i := 0; i < len(indices); i++ {
		indices[i] = uint64(i)
	}
	att := &eth.IndexedAttestation{
		AttestingIndices: indices,
		Data: &eth.AttestationData{
			Target: &eth.Checkpoint{},
		},
		Signature: make([]byte, fieldparams.BLSSignatureLength),
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if err := attestation.IsValidAttestationIndices(context.Background(), att); err != nil {
			require.NoError(b, err)
		}
	}
}

func TestAttDataIsEqual(t *testing.T) {
	type test struct {
		name     string
		attData1 *eth.AttestationData
		attData2 *eth.AttestationData
		equal    bool
	}
	tests := []test{
		{
			name: "same",
			attData1: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
			attData2: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
			equal: true,
		},
		{
			name: "diff slot",
			attData1: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
			attData2: &eth.AttestationData{
				Slot:            4,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
		},
		{
			name: "diff block",
			attData1: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("good block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
			attData2: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
		},
		{
			name: "diff source root",
			attData1: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("good source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
			attData2: &eth.AttestationData{
				Slot:            5,
				CommitteeIndex:  2,
				BeaconBlockRoot: []byte("great block"),
				Source: &eth.Checkpoint{
					Epoch: 4,
					Root:  []byte("bad source"),
				},
				Target: &eth.Checkpoint{
					Epoch: 10,
					Root:  []byte("good target"),
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.equal, attestation.AttDataIsEqual(tt.attData1, tt.attData2))
		})
	}
}

func TestCheckPtIsEqual(t *testing.T) {
	type test struct {
		name     string
		checkPt1 *eth.Checkpoint
		checkPt2 *eth.Checkpoint
		equal    bool
	}
	tests := []test{
		{
			name: "same",
			checkPt1: &eth.Checkpoint{
				Epoch: 4,
				Root:  []byte("good source"),
			},
			checkPt2: &eth.Checkpoint{
				Epoch: 4,
				Root:  []byte("good source"),
			},
			equal: true,
		},
		{
			name: "diff epoch",
			checkPt1: &eth.Checkpoint{
				Epoch: 4,
				Root:  []byte("good source"),
			},
			checkPt2: &eth.Checkpoint{
				Epoch: 5,
				Root:  []byte("good source"),
			},
			equal: false,
		},
		{
			name: "diff root",
			checkPt1: &eth.Checkpoint{
				Epoch: 4,
				Root:  []byte("good source"),
			},
			checkPt2: &eth.Checkpoint{
				Epoch: 4,
				Root:  []byte("bad source"),
			},
			equal: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.equal, attestation.CheckPointIsEqual(tt.checkPt1, tt.checkPt2))
		})
	}
}

func BenchmarkAttDataIsEqual(b *testing.B) {
	attData1 := &eth.AttestationData{
		Slot:            5,
		CommitteeIndex:  2,
		BeaconBlockRoot: []byte("great block"),
		Source: &eth.Checkpoint{
			Epoch: 4,
			Root:  []byte("good source"),
		},
		Target: &eth.Checkpoint{
			Epoch: 10,
			Root:  []byte("good target"),
		},
	}
	attData2 := &eth.AttestationData{
		Slot:            5,
		CommitteeIndex:  2,
		BeaconBlockRoot: []byte("great block"),
		Source: &eth.Checkpoint{
			Epoch: 4,
			Root:  []byte("good source"),
		},
		Target: &eth.Checkpoint{
			Epoch: 10,
			Root:  []byte("good target"),
		},
	}

	b.Run("fast", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			assert.Equal(b, true, attestation.AttDataIsEqual(attData1, attData2))
		}
	})

	b.Run("proto.Equal", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			assert.Equal(b, true, attestation.AttDataIsEqual(attData1, attData2))
		}
	})
}
