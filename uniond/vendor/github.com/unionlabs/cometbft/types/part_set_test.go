package types

import (
	"fmt"
	"io"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/merkle"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
)

const (
	testPartSize = 65536 // 64KB ...  4096 // 4KB
)

func TestBasicPartSet(t *testing.T) {
	// Construct random data of size partSize * 100
	nParts := 100
	data := cmtrand.Bytes(testPartSize * nParts)
	partSet := NewPartSetFromData(data, testPartSize)

	assert.NotEmpty(t, partSet.Hash())
	assert.EqualValues(t, nParts, partSet.Total())
	assert.Equal(t, nParts, partSet.BitArray().Size())
	assert.True(t, partSet.HashesTo(partSet.Hash()))
	assert.True(t, partSet.IsComplete())
	assert.EqualValues(t, nParts, partSet.Count())
	assert.EqualValues(t, testPartSize*nParts, partSet.ByteSize())

	// Test adding parts to a new partSet.
	partSet2 := NewPartSetFromHeader(partSet.Header())

	assert.True(t, partSet2.HasHeader(partSet.Header()))
	for i := 0; i < int(partSet.Total()); i++ {
		part := partSet.GetPart(i)
		// t.Logf("\n%v", part)
		added, err := partSet2.AddPart(part)
		if !added || err != nil {
			t.Errorf("failed to add part %v, error: %v", i, err)
		}
	}
	// adding part with invalid index
	added, err := partSet2.AddPart(&Part{Index: 10000})
	assert.False(t, added)
	require.Error(t, err)
	// adding existing part
	added, err = partSet2.AddPart(partSet2.GetPart(0))
	assert.False(t, added)
	require.NoError(t, err)

	assert.Equal(t, partSet.Hash(), partSet2.Hash())
	assert.EqualValues(t, nParts, partSet2.Total())
	assert.EqualValues(t, nParts*testPartSize, partSet.ByteSize())
	assert.True(t, partSet2.IsComplete())

	// Reconstruct data, assert that they are equal.
	data2Reader := partSet2.GetReader()
	data2, err := io.ReadAll(data2Reader)
	require.NoError(t, err)

	assert.Equal(t, data, data2)
}

func TestWrongProof(t *testing.T) {
	// Construct random data of size partSize * 100
	data := cmtrand.Bytes(testPartSize * 100)
	partSet := NewPartSetFromData(data, testPartSize)

	// Test adding a part with wrong data.
	partSet2 := NewPartSetFromHeader(partSet.Header())

	// Test adding a part with wrong trail.
	part := partSet.GetPart(0)
	part.Proof.Aunts[0][0] += byte(0x01)
	added, err := partSet2.AddPart(part)
	if added || err == nil {
		t.Errorf("expected to fail adding a part with bad trail.")
	}

	// Test adding a part with wrong bytes.
	part = partSet.GetPart(1)
	part.Bytes[0] += byte(0x01)
	added, err = partSet2.AddPart(part)
	if added || err == nil {
		t.Errorf("expected to fail adding a part with bad bytes.")
	}

	// Test adding a part with wrong proof index.
	part = partSet.GetPart(2)
	part.Proof.Index = 1
	added, err = partSet2.AddPart(part)
	if added || err == nil {
		t.Errorf("expected to fail adding a part with bad proof index.")
	}

	// Test adding a part with wrong proof total.
	part = partSet.GetPart(3)
	part.Proof.Total = int64(partSet.Total() - 1)
	added, err = partSet2.AddPart(part)
	if added || err == nil {
		t.Errorf("expected to fail adding a part with bad proof total.")
	}
}

func TestPartSetHeaderValidateBasic(t *testing.T) {
	testCases := []struct {
		testName              string
		malleatePartSetHeader func(*PartSetHeader)
		expectErr             bool
	}{
		{"Good PartSet", func(_ *PartSetHeader) {}, false},
		{"Invalid Hash", func(psHeader *PartSetHeader) { psHeader.Hash = make([]byte, 1) }, true},
	}
	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			data := cmtrand.Bytes(testPartSize * 100)
			ps := NewPartSetFromData(data, testPartSize)
			psHeader := ps.Header()
			tc.malleatePartSetHeader(&psHeader)
			assert.Equal(t, tc.expectErr, psHeader.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestPartValidateBasic(t *testing.T) {
	testCases := []struct {
		testName     string
		malleatePart func(*Part)
		expectErr    bool
	}{
		{"Good Part", func(_ *Part) {}, false},
		{"Too big part", func(pt *Part) { pt.Bytes = make([]byte, BlockPartSizeBytes+1) }, true},
		{"Good small last part", func(pt *Part) {
			pt.Index = 1
			pt.Bytes = make([]byte, BlockPartSizeBytes-1)
			pt.Proof.Total = 2
		}, false},
		{"Too small inner part", func(pt *Part) {
			pt.Index = 0
			pt.Bytes = make([]byte, BlockPartSizeBytes-1)
			pt.Proof.Total = 2
		}, true},
		{"Too big proof", func(pt *Part) {
			pt.Proof = merkle.Proof{
				Total:    2,
				Index:    1,
				LeafHash: make([]byte, 1024*1024),
			}
		}, true},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			data := cmtrand.Bytes(testPartSize * 100)
			ps := NewPartSetFromData(data, testPartSize)
			part := ps.GetPart(0)
			tc.malleatePart(part)
			assert.Equal(t, tc.expectErr, part.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestParSetHeaderProtoBuf(t *testing.T) {
	testCases := []struct {
		msg     string
		ps1     *PartSetHeader
		expPass bool
	}{
		{"success empty", &PartSetHeader{}, true},
		{
			"success",
			&PartSetHeader{Total: 1, Hash: []byte("hash")}, true,
		},
	}

	for _, tc := range testCases {
		protoBlockID := tc.ps1.ToProto()

		psh, err := PartSetHeaderFromProto(&protoBlockID)
		if tc.expPass {
			require.Equal(t, tc.ps1, psh, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

func TestPartProtoBuf(t *testing.T) {
	proof := merkle.Proof{
		Total:    1,
		Index:    1,
		LeafHash: cmtrand.Bytes(32),
	}
	testCases := []struct {
		msg     string
		ps1     *Part
		expPass bool
	}{
		{"failure empty", &Part{}, false},
		{"failure nil", nil, false},
		{
			"success",
			&Part{Index: 1, Bytes: cmtrand.Bytes(32), Proof: proof}, true,
		},
	}

	for _, tc := range testCases {
		proto, err := tc.ps1.ToProto()
		if tc.expPass {
			require.NoError(t, err, tc.msg)
		}

		p, err := PartFromProto(proto)
		if tc.expPass {
			require.NoError(t, err)
			require.Equal(t, tc.ps1, p, tc.msg)
		}
	}
}

func BenchmarkMakePartSet(b *testing.B) {
	for nParts := 1; nParts <= 5; nParts++ {
		b.Run(fmt.Sprintf("nParts=%d", nParts), func(b *testing.B) {
			data := cmtrand.Bytes(testPartSize * nParts)
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				NewPartSetFromData(data, testPartSize)
			}
		})
	}
}
