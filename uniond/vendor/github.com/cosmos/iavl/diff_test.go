package iavl

import (
	"encoding/binary"
	"fmt"
	"math"
	"math/rand"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

// TestDiffRoundTrip generate random change sets, build an iavl tree versions,
// then extract state changes from the versions and compare with the original change sets.
func TestDiffRoundTrip(t *testing.T) {
	changeSets := genChangeSets(rand.New(rand.NewSource(0)), 300)

	// apply changeSets to tree
	db := dbm.NewMemDB()
	tree := NewMutableTree(db, 0, true, NewNopLogger())
	for i := range changeSets {
		v, err := tree.SaveChangeSet(changeSets[i])
		require.NoError(t, err)
		require.Equal(t, int64(i+1), v)
	}

	// extract change sets from db
	var extractChangeSets []*ChangeSet
	tree2 := NewImmutableTree(db, 0, true, NewNopLogger())
	err := tree2.TraverseStateChanges(0, math.MaxInt64, func(_ int64, changeSet *ChangeSet) error {
		extractChangeSets = append(extractChangeSets, changeSet)
		return nil
	})
	require.NoError(t, err)
	require.Equal(t, changeSets, extractChangeSets)
}

func genChangeSets(r *rand.Rand, n int) []*ChangeSet {
	var changeSets []*ChangeSet

	for i := 0; i < n; i++ {
		items := make(map[string]*KVPair)
		start, count, step := r.Int63n(1000), r.Int63n(1000), r.Int63n(10)
		for i := start; i < start+count*step; i += step {
			value := make([]byte, 8)
			binary.LittleEndian.PutUint64(value, uint64(i))

			key := fmt.Sprintf("test-%d", i)
			items[key] = &KVPair{
				Key:   []byte(key),
				Value: value,
			}
		}
		if len(changeSets) > 0 {
			// pick some random keys to delete from the last version
			lastChangeSet := changeSets[len(changeSets)-1]
			count = r.Int63n(10)
			for _, pair := range lastChangeSet.Pairs {
				if count <= 0 {
					break
				}
				if pair.Delete {
					continue
				}
				items[string(pair.Key)] = &KVPair{
					Key:    pair.Key,
					Delete: true,
				}
				count--
			}

			// Special case, set to identical value
			if len(lastChangeSet.Pairs) > 0 {
				i := r.Int63n(int64(len(lastChangeSet.Pairs)))
				pair := lastChangeSet.Pairs[i]
				if !pair.Delete {
					items[string(pair.Key)] = &KVPair{
						Key:   pair.Key,
						Value: pair.Value,
					}
				}
			}
		}

		var keys []string
		for key := range items {
			keys = append(keys, key)
		}
		sort.Strings(keys)

		var cs ChangeSet
		for _, key := range keys {
			cs.Pairs = append(cs.Pairs, items[key])
		}

		changeSets = append(changeSets, &cs)
	}
	return changeSets
}
