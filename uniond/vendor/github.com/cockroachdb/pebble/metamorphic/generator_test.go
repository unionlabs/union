// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"testing"
	"time"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/pmezard/go-difflib/difflib"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestGenerator(t *testing.T) {
	rng := randvar.NewRand()
	g := newGenerator(rng, defaultConfig(), newKeyManager())

	g.newBatch()
	g.newBatch()
	g.newBatch()
	require.EqualValues(t, 3, len(g.liveBatches))
	require.EqualValues(t, 0, len(g.batches))

	g.newIndexedBatch()
	g.newIndexedBatch()
	g.newIndexedBatch()
	require.EqualValues(t, 6, len(g.liveBatches))
	require.EqualValues(t, 3, len(g.batches))
	require.EqualValues(t, 3, len(g.readers))

	g.newIter()
	g.newIter()
	g.newIter()
	require.EqualValues(t, 3, len(g.liveIters))

	g.batchAbort()
	g.batchAbort()
	g.batchAbort()
	g.batchAbort()
	g.batchAbort()
	g.batchAbort()

	require.EqualValues(t, 0, len(g.liveBatches))
	require.EqualValues(t, 0, len(g.batches))
	require.EqualValues(t, 0, len(g.iters))
	require.EqualValues(t, 1, len(g.liveReaders))
	require.EqualValues(t, 0, len(g.readers))
	require.EqualValues(t, 0, len(g.liveSnapshots))
	require.EqualValues(t, 0, len(g.snapshots))
	require.EqualValues(t, 1, len(g.liveWriters))

	g.randIter(g.iterClose)()
	g.randIter(g.iterClose)()
	g.randIter(g.iterClose)()
	require.EqualValues(t, 0, len(g.liveIters))

	if testing.Verbose() {
		t.Logf("\n%s", g)
	}

	g = newGenerator(rng, defaultConfig(), newKeyManager())

	g.newSnapshot()
	g.newSnapshot()
	g.newSnapshot()
	require.EqualValues(t, 3, len(g.liveSnapshots))
	require.EqualValues(t, 3, len(g.snapshots))

	g.newIter()
	g.newIter()
	g.newIter()
	g.snapshotClose()
	g.snapshotClose()
	g.snapshotClose()

	require.EqualValues(t, 0, len(g.liveBatches))
	require.EqualValues(t, 0, len(g.batches))
	require.EqualValues(t, 0, len(g.iters))
	require.EqualValues(t, 1, len(g.liveReaders))
	require.EqualValues(t, 0, len(g.readers))
	require.EqualValues(t, 0, len(g.liveSnapshots))
	require.EqualValues(t, 0, len(g.snapshots))
	require.EqualValues(t, 1, len(g.liveWriters))

	g.randIter(g.iterClose)()
	g.randIter(g.iterClose)()
	g.randIter(g.iterClose)()
	require.EqualValues(t, 0, len(g.liveIters))

	if testing.Verbose() {
		t.Logf("\n%s", g)
	}

	g = newGenerator(rng, defaultConfig(), newKeyManager())

	g.newIndexedBatch()
	g.newIndexedBatch()
	g.newIndexedBatch()
	g.newIter()
	g.newIter()
	g.newIter()
	g.writerApply()
	g.writerApply()
	g.writerApply()
	g.randIter(g.iterClose)()
	g.randIter(g.iterClose)()
	g.randIter(g.iterClose)()

	require.EqualValues(t, 0, len(g.liveBatches))
	require.EqualValues(t, 0, len(g.batches))
	require.EqualValues(t, 0, len(g.liveIters))
	require.EqualValues(t, 0, len(g.iters))
	require.EqualValues(t, 1, len(g.liveReaders))
	require.EqualValues(t, 0, len(g.readers))
	require.EqualValues(t, 0, len(g.liveSnapshots))
	require.EqualValues(t, 0, len(g.snapshots))
	require.EqualValues(t, 1, len(g.liveWriters))

	if testing.Verbose() {
		t.Logf("\n%s", g)
	}
}

func TestGeneratorRandom(t *testing.T) {
	seed := uint64(time.Now().UnixNano())
	ops := randvar.NewUniform(1000, 10000)
	generateFromSeed := func() string {
		rng := rand.New(rand.NewSource(seed))
		count := ops.Uint64(rng)
		return formatOps(generate(rng, count, defaultConfig(), newKeyManager()))
	}

	// Ensure that generate doesn't use any other source of randomness other
	// than rng.
	referenceOps := generateFromSeed()
	for i := 0; i < 10; i++ {
		regeneratedOps := generateFromSeed()
		diff, err := difflib.GetUnifiedDiffString(difflib.UnifiedDiff{
			A:       difflib.SplitLines(referenceOps),
			B:       difflib.SplitLines(regeneratedOps),
			Context: 1,
		})
		require.NoError(t, err)
		if len(diff) > 0 {
			t.Fatalf("Diff:\n%s", diff)
		}
	}
	if testing.Verbose() {
		t.Logf("\nOps:\n%s", referenceOps)
	}
}

func TestGenerateRandKeyToReadInRange(t *testing.T) {
	rng := randvar.NewRand()
	g := newGenerator(rng, defaultConfig(), newKeyManager())
	// Seed 100 initial keys.
	for i := 0; i < 100; i++ {
		_ = g.randKeyToWrite(1.0)
	}
	for i := 0; i < 100; i++ {
		a := g.randKeyToRead(0.01)
		b := g.randKeyToRead(0.01)
		// Ensure unique prefixes; required by randKeyToReadInRange.
		for g.equal(g.prefix(a), g.prefix(b)) {
			b = g.randKeyToRead(0.01)
		}
		if v := g.cmp(a, b); v > 0 {
			a, b = b, a
		}
		kr := pebble.KeyRange{Start: a, End: b}
		for j := 0; j < 10; j++ {
			k := g.randKeyToReadInRange(0.05, kr)
			if g.cmp(k, a) < 0 {
				t.Errorf("generated random key %q outside range %s", k, kr)
			} else if g.cmp(k, b) >= 0 {
				t.Errorf("generated random key %q outside range %s", k, kr)
			}
		}
	}
}

func TestGenerateDisjointKeyRanges(t *testing.T) {
	rng := randvar.NewRand()
	g := newGenerator(rng, defaultConfig(), newKeyManager())

	for i := 0; i < 10; i++ {
		keyRanges := g.generateDisjointKeyRanges(5)
		for j := range keyRanges {
			t.Logf("%d: %d: %s", i, j, keyRanges[j])
			for k := range keyRanges {
				if j == k {
					continue
				}
				if g.cmp(keyRanges[j].End, keyRanges[k].Start) <= 0 {
					require.Less(t, j, k)
					continue
				}
				if g.cmp(keyRanges[j].Start, keyRanges[k].End) >= 0 {
					require.Greater(t, j, k)
					continue
				}
				t.Fatalf("generated key ranges %q and %q overlap", keyRanges[j], keyRanges[k])
			}
		}
	}
}
