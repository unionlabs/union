package stateutil

import (
	"bytes"
	"encoding/binary"
	"fmt"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func HistoricalSummariesRoot(summaries []*ethpb.HistoricalSummary) ([32]byte, error) {
	max := uint64(fieldparams.HistoricalRootsLength)
	if uint64(len(summaries)) > max {
		return [32]byte{}, fmt.Errorf("historical summary exceeds max length %d", max)
	}

	roots := make([][32]byte, len(summaries))
	for i := 0; i < len(summaries); i++ {
		r, err := summaries[i].HashTreeRoot()
		if err != nil {
			return [32]byte{}, errors.Wrap(err, "could not merkleize historical summary")
		}
		roots[i] = r
	}

	summariesRoot, err := ssz.BitwiseMerkleize(roots, uint64(len(roots)), fieldparams.HistoricalRootsLength)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not compute historical summaries merkleization")
	}
	summariesLenBuf := new(bytes.Buffer)
	if err := binary.Write(summariesLenBuf, binary.LittleEndian, uint64(len(summaries))); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not marshal historical summary length")
	}
	// We need to mix in the length of the slice.
	summariesLenRoot := make([]byte, 32)
	copy(summariesLenRoot, summariesLenBuf.Bytes())
	res := ssz.MixInLength(summariesRoot, summariesLenRoot)
	return res, nil
}
