package kv

import (
	"bytes"
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

// lookupValuesForIndices takes in a list of indices and looks up
// their corresponding values in the DB, returning a list of
// roots which can then be used for batch lookups of their corresponding
// objects from the DB. For example, if we are fetching
// attestations and we have an index `[]byte("5")` under the shard indices bucket,
// we might find roots `0x23` and `0x45` stored under that index. We can then
// do a batch read for attestations corresponding to those roots.
func lookupValuesForIndices(ctx context.Context, indicesByBucket map[string][]byte, tx *bolt.Tx) [][][]byte {
	_, span := trace.StartSpan(ctx, "BeaconDB.lookupValuesForIndices")
	defer span.End()
	values := make([][][]byte, 0, len(indicesByBucket))
	for k, v := range indicesByBucket {
		bkt := tx.Bucket([]byte(k))
		roots := bkt.Get(v)
		splitRoots := make([][]byte, 0, len(roots)/32)
		for i := 0; i < len(roots); i += 32 {
			splitRoots = append(splitRoots, roots[i:i+32])
		}
		values = append(values, splitRoots)
	}
	return values
}

// updateValueForIndices updates the value for each index by appending it to the previous
// values stored at said index. Typically, indices are roots of data that can then
// be used for reads or batch reads from the DB.
func updateValueForIndices(ctx context.Context, indicesByBucket map[string][]byte, root []byte, tx *bolt.Tx) error {
	_, span := trace.StartSpan(ctx, "BeaconDB.updateValueForIndices")
	defer span.End()
	for k, idx := range indicesByBucket {
		bkt := tx.Bucket([]byte(k))
		valuesAtIndex := bkt.Get(idx)
		if valuesAtIndex == nil {
			if err := bkt.Put(idx, root); err != nil {
				return err
			}
		} else {
			// Do not save duplication in indices bucket
			for i := 0; i < len(valuesAtIndex); i += 32 {
				if bytes.Equal(valuesAtIndex[i:i+32], root) {
					return nil
				}
			}
			if err := bkt.Put(idx, append(valuesAtIndex, root...)); err != nil {
				return err
			}
		}
	}
	return nil
}

// deleteValueForIndices clears a root stored at each index.
func deleteValueForIndices(ctx context.Context, indicesByBucket map[string][]byte, root []byte, tx *bolt.Tx) error {
	_, span := trace.StartSpan(ctx, "BeaconDB.deleteValueForIndices")
	defer span.End()
	for k, idx := range indicesByBucket {
		bkt := tx.Bucket([]byte(k))
		valuesAtIndex := bkt.Get(idx)
		if valuesAtIndex != nil {
			start := bytes.Index(valuesAtIndex, root)
			// If the root was not found inside the values at index slice, we continue.
			// Root must be correctly aligned to avoid matching to subsequences of adjacent values.
			if start == -1 || start%len(root) != 0 {
				continue
			}
			// We clear out the root from the values at index slice. For example,
			// If we had [0x32, 0x33, 0x45] and we wanted to clear out 0x33, the code below
			// updates the slice to [0x32, 0x45].

			valuesStart := make([]byte, len(valuesAtIndex[:start]))
			copy(valuesStart, valuesAtIndex[:start])

			valuesEnd := make([]byte, len(valuesAtIndex[start+len(root):]))
			copy(valuesEnd, valuesAtIndex[start+len(root):])

			valuesAtIndex = append(valuesStart, valuesEnd...)

			// If this removes the last value, delete the whole key/value entry.
			if len(valuesAtIndex) == 0 {
				if err := bkt.Delete(idx); err != nil {
					return err
				}
				continue
			}
			if err := bkt.Put(idx, valuesAtIndex); err != nil {
				return err
			}
		}
	}
	return nil
}

var errMisalignedRootList = errors.New("incorrectly packed root list, length is not a multiple of 32")

func splitRoots(b []byte) ([][32]byte, error) {
	rl := make([][32]byte, 0)
	if len(b)%32 != 0 {
		return nil, errors.Wrapf(errMisalignedRootList, "root list len=%d", len(b))
	}
	for s, f := 0, 32; f <= len(b); s, f = f, f+32 {
		rl = append(rl, bytesutil.ToBytes32(b[s:f]))
	}
	return rl, nil
}
